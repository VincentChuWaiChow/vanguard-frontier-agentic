//! Filesystem watcher integration for catalog, registry, and workspace paths.
//!
//! Uses `notify-debouncer-full` for intelligent rename/modify coalescing and
//! 500 ms debouncing (Req 1.6).  Debounced events are forwarded to a tokio
//! mpsc channel so the TUI event loop never blocks on notify callbacks (Req 1.7).
//!
//! # Re-establish logic (Req 1.4 / 25.5)
//!
//! When the underlying notify watcher signals an error (watch path deleted or
//! permission error) a warning is logged.  A background tokio task also checks
//! every 30 s whether any of the registered watch-roots still exist and, if so,
//! re-adds them to the watcher.  Callers can also invoke
//! [`WatcherHandle::reestablish`] explicitly.

use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};

use notify_debouncer_full::{
    new_debouncer, notify::RecursiveMode, DebounceEventResult, Debouncer, RecommendedCache,
};
use tokio::sync::mpsc;
use tracing::{debug, warn};

use crate::error::TuiError;

// ── Debounce window (Req 1.6) ─────────────────────────────────────────────────
const DEBOUNCE_TIMEOUT: Duration = Duration::from_millis(500);
/// How often the re-establish task retries failed watch paths (Req 1.4 / 25.5).
const REESTABLISH_INTERVAL: Duration = Duration::from_secs(30);

// ── Public types ──────────────────────────────────────────────────────────────

/// A filesystem change that needs to be acted upon by the catalog subsystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatcherEvent {
    /// A file under the catalog directory changed.
    Catalog(PathBuf),
    /// The workspace registry file changed.
    Registry,
    /// A file under one of the watched workspace paths changed.
    Workspace(PathBuf),
}

/// Configuration describing what to watch.
#[derive(Clone)]
pub struct WatchConfig {
    pub catalog_dir: PathBuf,
    pub registry_path: Option<PathBuf>,
    pub workspace_paths: Vec<PathBuf>,
}

/// Opaque handle that keeps the debouncer alive and exposes shutdown /
/// re-establish operations.
pub struct WatcherHandle {
    /// The debouncer **must** stay alive for the duration of watching.
    /// Wrapped in `Arc<Mutex<…>>` so the re-establish task can borrow it.
    debouncer:
        Arc<Mutex<Debouncer<notify_debouncer_full::notify::RecommendedWatcher, RecommendedCache>>>,
    config: WatchConfig,
}

impl WatcherHandle {
    /// Attempt to re-add all configured watch roots to the underlying watcher.
    ///
    /// Silently ignores paths that do not exist yet (they will be retried on
    /// the next periodic call).  Logs a warning for any path that exists but
    /// cannot be watched.
    pub fn reestablish(&self) {
        let mut guard = match self.debouncer.lock() {
            Ok(g) => g,
            Err(e) => {
                warn!("watcher mutex poisoned during reestablish: {e}");
                return;
            }
        };
        let cfg = &self.config;

        // Catalog dir
        if cfg.catalog_dir.exists() {
            if let Err(e) = guard.watch(&cfg.catalog_dir, RecursiveMode::Recursive) {
                warn!(
                    path = %cfg.catalog_dir.display(),
                    "failed to re-watch catalog dir: {e}"
                );
            } else {
                debug!(path = %cfg.catalog_dir.display(), "re-established catalog watch");
            }
        }

        // Registry file
        if let Some(reg) = &cfg.registry_path {
            if reg.exists() {
                // Watch the parent directory so we catch temp-then-rename writes
                if let Some(parent) = reg.parent() {
                    if let Err(e) = guard.watch(parent, RecursiveMode::NonRecursive) {
                        warn!(
                            path = %parent.display(),
                            "failed to re-watch registry parent: {e}"
                        );
                    } else {
                        debug!(path = %parent.display(), "re-established registry watch");
                    }
                }
            }
        }

        // Workspace paths
        for ws in &cfg.workspace_paths {
            if ws.exists() {
                if let Err(e) = guard.watch(ws, RecursiveMode::Recursive) {
                    warn!(
                        path = %ws.display(),
                        "failed to re-watch workspace path: {e}"
                    );
                } else {
                    debug!(path = %ws.display(), "re-established workspace watch");
                }
            }
        }
    }

    /// Consume the handle and stop the debouncer gracefully.
    pub fn shutdown(self) {
        match Arc::try_unwrap(self.debouncer) {
            Ok(mutex) => {
                if let Ok(debouncer) = mutex.into_inner() {
                    debouncer.stop_nonblocking();
                }
            }
            Err(_arc) => {
                // Re-establish task still holds a reference; the debouncer will
                // be stopped when all Arc clones are dropped.
                debug!("watcher shutdown deferred – re-establish task still active");
            }
        }
    }
}

// ── Path classification (pure, tested independently) ─────────────────────────

/// Classify a changed path into a [`WatcherEvent`].
///
/// This is a **pure function** with no filesystem I/O, making it easy to unit-
/// test independently of the notify backend (which can be timing-flaky in CI).
pub fn classify_path(
    changed: &Path,
    catalog_dir: &Path,
    registry_path: Option<&Path>,
    workspace_paths: &[PathBuf],
) -> Option<WatcherEvent> {
    // Registry file check (exact match or parent-dir match for temp-rename)
    if let Some(reg) = registry_path {
        if changed == reg {
            return Some(WatcherEvent::Registry);
        }
        // Handle parent-dir watch: notify may report the *final* path after a
        // temp-rename, which equals the registry path.
        if changed.parent() == reg.parent() && changed.file_name() == reg.file_name() {
            return Some(WatcherEvent::Registry);
        }
    }

    // Catalog directory
    if changed.starts_with(catalog_dir) {
        return Some(WatcherEvent::Catalog(changed.to_path_buf()));
    }

    // Workspace paths
    for ws in workspace_paths {
        if changed.starts_with(ws) {
            return Some(WatcherEvent::Workspace(changed.to_path_buf()));
        }
    }

    None
}

// ── Spawn function ────────────────────────────────────────────────────────────

/// Spawn a filesystem watcher and return a `(WatcherHandle, Receiver<WatcherEvent>)`.
///
/// The debouncer fires at most once per file per [`DEBOUNCE_TIMEOUT`] (500 ms).
/// All paths may be watched recursively; the registry parent directory is
/// watched non-recursively so that only the registry file's rename/modify events
/// are forwarded (avoiding noise from sibling files).
///
/// A background tokio task periodically calls [`WatcherHandle::reestablish`]
/// every [`REESTABLISH_INTERVAL`] (30 s) to recover from transient watch
/// failures such as watched-path deletion (Req 1.4 / 25.5).
pub fn spawn_watcher(
    catalog_dir: &Path,
    registry_path: Option<&Path>,
    workspace_paths: &[PathBuf],
) -> Result<(WatcherHandle, mpsc::Receiver<WatcherEvent>), TuiError> {
    // Tokio channel: the notify callback sends here; the TUI event loop reads.
    let (tx, rx) = mpsc::channel::<WatcherEvent>(256);

    let config = WatchConfig {
        catalog_dir: catalog_dir.to_path_buf(),
        registry_path: registry_path.map(|p| p.to_path_buf()),
        workspace_paths: workspace_paths.to_vec(),
    };

    // Clone config fields for use inside the notify callback closure.
    let cb_catalog_dir = config.catalog_dir.clone();
    let cb_registry = config.registry_path.clone();
    let cb_workspace_paths = config.workspace_paths.clone();
    let cb_tx = tx.clone();

    let debouncer = new_debouncer(
        DEBOUNCE_TIMEOUT,
        None, // tick_rate = None → 1/4 of timeout = 125 ms
        move |result: DebounceEventResult| {
            match result {
                Ok(events) => {
                    for event in events {
                        for path in &event.paths {
                            if let Some(watcher_event) = classify_path(
                                path,
                                &cb_catalog_dir,
                                cb_registry.as_deref(),
                                &cb_workspace_paths,
                            ) {
                                // Use try_send to avoid blocking the notify
                                // callback thread (Req 1.7).
                                if cb_tx.try_send(watcher_event).is_err() {
                                    debug!("watcher channel full or closed, dropping event");
                                }
                            }
                        }
                    }
                }
                Err(errors) => {
                    for e in errors {
                        warn!("filesystem watcher error: {e}");
                    }
                }
            }
        },
    )
    .map_err(|e| TuiError::LogDestination {
        path: catalog_dir.display().to_string(),
        reason: e.to_string(),
    })?;

    let debouncer_arc = Arc::new(Mutex::new(debouncer));

    // Establish initial watches.
    {
        let mut guard = debouncer_arc.lock().map_err(|e| TuiError::LogDestination {
            path: "watcher-mutex".to_string(),
            reason: e.to_string(),
        })?;
        if catalog_dir.exists() {
            guard
                .watch(catalog_dir, RecursiveMode::Recursive)
                .map_err(|e| TuiError::LogDestination {
                    path: catalog_dir.display().to_string(),
                    reason: e.to_string(),
                })?;
        } else {
            warn!(
                path = %catalog_dir.display(),
                "catalog dir does not exist at watcher startup; will retry"
            );
        }

        if let Some(reg) = registry_path {
            // Watch parent dir (non-recursive) to catch atomic/rename writes.
            if let Some(parent) = reg.parent() {
                if parent.exists() {
                    if let Err(e) = guard.watch(parent, RecursiveMode::NonRecursive) {
                        warn!(
                            path = %parent.display(),
                            "could not watch registry parent dir: {e}"
                        );
                    }
                }
            }
        }

        for ws in workspace_paths {
            if ws.exists() {
                if let Err(e) = guard.watch(ws, RecursiveMode::Recursive) {
                    warn!(path = %ws.display(), "could not watch workspace path: {e}");
                }
            } else {
                warn!(
                    path = %ws.display(),
                    "workspace path does not exist at watcher startup; will retry"
                );
            }
        }
    }

    let handle = WatcherHandle {
        debouncer: debouncer_arc.clone(),
        config: config.clone(),
    };

    // ── Background re-establish task (Req 1.4 / 25.5) ────────────────────────
    let reestablish_handle = WatcherHandle {
        debouncer: debouncer_arc,
        config,
    };
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(REESTABLISH_INTERVAL).await;
            reestablish_handle.reestablish();
        }
    });

    Ok((handle, rx))
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::TempDir;
    use tokio::time::timeout;

    // ── Pure classification tests (no filesystem, no timing) ─────────────────

    #[test]
    fn classify_catalog_file() {
        let cat = PathBuf::from("/tmp/catalog");
        let changed = cat.join("agents.json");
        let result = classify_path(&changed, &cat, None, &[]);
        assert_eq!(result, Some(WatcherEvent::Catalog(changed)));
    }

    #[test]
    fn classify_mods_catalog_without_a_filename_allowlist() {
        let catalog = PathBuf::from("/tmp/catalog");
        let changed = catalog.join("mods.json");
        assert_eq!(
            classify_path(&changed, &catalog, None, &[]),
            Some(WatcherEvent::Catalog(changed))
        );
    }

    #[test]
    fn classify_registry_path_exact() {
        let cat = PathBuf::from("/tmp/catalog");
        let reg = PathBuf::from("/home/user/.config/vfa/workspaces.toml");
        let result = classify_path(&reg, &cat, Some(&reg), &[]);
        assert_eq!(result, Some(WatcherEvent::Registry));
    }

    #[test]
    fn classify_workspace_path() {
        let cat = PathBuf::from("/tmp/catalog");
        let ws = PathBuf::from("/home/user/repos/my-app");
        let changed = ws.join(".claude").join("agents.md");
        let result = classify_path(&changed, &cat, None, &[ws]);
        assert_eq!(result, Some(WatcherEvent::Workspace(changed)));
    }

    #[test]
    fn classify_unrelated_path_is_none() {
        let cat = PathBuf::from("/tmp/catalog");
        let unrelated = PathBuf::from("/etc/hosts");
        let result = classify_path(&unrelated, &cat, None, &[]);
        assert!(result.is_none());
    }

    #[test]
    fn classify_registry_wins_over_workspace() {
        // If the registry file happens to live under a workspace path,
        // Registry classification should win.
        let cat = PathBuf::from("/tmp/catalog");
        let ws = PathBuf::from("/home/user");
        let reg = ws.join(".config/vfa/workspaces.toml");
        let result = classify_path(&reg, &cat, Some(&reg), &[ws]);
        assert_eq!(result, Some(WatcherEvent::Registry));
    }

    // ── End-to-end smoke tests (filesystem + tokio, timing-tolerant) ─────────
    //
    // NOTE: inotify on Linux should be reliable for these basic operations.
    // We use generous 3-second timeouts and test only the most fundamental
    // behavior to avoid CI flakiness from scheduling jitter.

    /// Helper: receive the next event with a timeout, draining duplicates.
    async fn recv_event(
        rx: &mut mpsc::Receiver<WatcherEvent>,
        timeout_dur: Duration,
    ) -> Option<WatcherEvent> {
        timeout(timeout_dur, rx.recv()).await.ok().flatten()
    }

    #[tokio::test]
    async fn e2e_catalog_file_modify_emits_event() {
        let dir = TempDir::new().unwrap();
        let catalog_dir = dir.path().join("catalog");
        std::fs::create_dir_all(&catalog_dir).unwrap();
        let file = catalog_dir.join("agents.json");
        std::fs::write(&file, b"[]").unwrap();

        let (_handle, mut rx) = spawn_watcher(&catalog_dir, None, &[]).unwrap();

        // Give notify time to set up the inotify watch before writing.
        tokio::time::sleep(Duration::from_millis(100)).await;

        std::fs::write(&file, b"[{}]").unwrap();

        // Wait up to 3 s for the debounced event (debounce = 500 ms, so allow
        // 500 ms + generous headroom for scheduling).
        let event = recv_event(&mut rx, Duration::from_secs(3)).await;
        assert!(
            matches!(event, Some(WatcherEvent::Catalog(_))),
            "expected Catalog event, got: {event:?}"
        );
    }

    #[tokio::test]
    async fn e2e_write_to_temp_then_rename_emits_event() {
        // Req 1.2: editors that use atomic write (write tmp → rename) must
        // still trigger an event.  notify-debouncer-full coalesces the
        // Rename(From) + Rename(To) pair into a single event for the final
        // path.
        let dir = TempDir::new().unwrap();
        let catalog_dir = dir.path().join("catalog");
        std::fs::create_dir_all(&catalog_dir).unwrap();
        let target = catalog_dir.join("agents.json");
        std::fs::write(&target, b"[]").unwrap();

        let (_handle, mut rx) = spawn_watcher(&catalog_dir, None, &[]).unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Atomic-write pattern: write to a sibling tmp file then rename over target.
        let tmp = catalog_dir.join("agents.json.tmp");
        std::fs::write(&tmp, b"[{}]").unwrap();
        std::fs::rename(&tmp, &target).unwrap();

        let event = recv_event(&mut rx, Duration::from_secs(3)).await;
        assert!(
            matches!(event, Some(WatcherEvent::Catalog(_))),
            "expected Catalog event after rename, got: {event:?}"
        );
    }

    #[tokio::test]
    async fn e2e_registry_file_change_emits_registry_event() {
        // Req 1.7 / path classification integration: a change to the registry
        // file must produce WatcherEvent::Registry.
        let dir = TempDir::new().unwrap();
        let catalog_dir = dir.path().join("catalog");
        std::fs::create_dir_all(&catalog_dir).unwrap();

        let reg_dir = dir.path().join("config");
        std::fs::create_dir_all(&reg_dir).unwrap();
        let reg_file = reg_dir.join("workspaces.toml");
        std::fs::write(&reg_file, b"").unwrap();

        let (_handle, mut rx) = spawn_watcher(&catalog_dir, Some(&reg_file), &[]).unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        std::fs::write(&reg_file, b"# updated").unwrap();

        let event = recv_event(&mut rx, Duration::from_secs(3)).await;
        assert!(
            matches!(event, Some(WatcherEvent::Registry)),
            "expected Registry event, got: {event:?}"
        );
    }

    #[tokio::test]
    async fn e2e_debounce_rapid_writes_do_not_flood() {
        // Req 1.6: at most one event per file per 500 ms.
        // We perform N rapid writes and assert we receive >= 1 but a
        // reasonable bound (not N) of events within the window.
        let dir = TempDir::new().unwrap();
        let catalog_dir = dir.path().join("catalog");
        std::fs::create_dir_all(&catalog_dir).unwrap();
        let file = catalog_dir.join("agents.json");
        std::fs::write(&file, b"[]").unwrap();

        let (_handle, mut rx) = spawn_watcher(&catalog_dir, None, &[]).unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Rapid-fire 10 writes well inside the 500 ms debounce window.
        for i in 0u8..10 {
            std::fs::write(&file, [i]).unwrap();
        }

        // Drain events for up to 2 s (one full debounce period + headroom).
        let mut count = 0usize;
        let drain_deadline = Duration::from_secs(2);
        let per_poll = Duration::from_millis(100);
        let polls = drain_deadline.as_millis() / per_poll.as_millis();
        for _ in 0..polls {
            if let Ok(Some(_)) = timeout(per_poll, rx.recv()).await {
                count += 1;
            }
        }

        // We should have received at least 1 event (debouncer did fire) and
        // strictly fewer than one-per-write (coalescing occurred). The exact
        // count is timing-dependent — on a loaded CI runner the write loop can
        // straddle multiple 500 ms windows — so the robust invariant is "fewer
        // than the 10 writes", not a tight magic number. A removed/broken
        // debouncer would deliver ~one event per write (≈10).
        assert!(count >= 1, "expected at least 1 debounced event, got 0");
        assert!(
            count < 10,
            "expected debouncing (coalesced < 10): got {count} events for 10 rapid writes"
        );
    }
}
