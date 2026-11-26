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
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use notify_debouncer_full::{
    new_debouncer,
    notify::{RecursiveMode, Watcher},
    DebounceEventResult, Debouncer, FileIdMap,
};
use tokio::sync::{mpsc, watch, Notify};
use tokio::task::JoinHandle;
use tracing::warn;

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
    /// A watched workspace changed, or a whole-root rescan is required after
    /// overflow/backend error. Consumers must refresh the complete observation.
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
type NativeDebouncer = Debouncer<notify_debouncer_full::notify::RecommendedWatcher, FileIdMap>;

pub struct WatcherHandle {
    /// The debouncer **must** stay alive for the duration of watching.
    /// Wrapped in `Arc<Mutex<…>>` so the re-establish task can borrow it.
    debouncer: Arc<Mutex<Option<NativeDebouncer>>>,
    cancel: watch::Sender<bool>,
    worker: Option<JoinHandle<Result<(), String>>>,
    completion: Option<Result<(), String>>,
    dirty: Arc<DirtySignal>,
    #[cfg(all(test, unix))]
    ingress: mpsc::Sender<WatcherEvent>,
    config: WatchConfig,
}

impl WatcherHandle {
    /// Attempt to re-add all configured watch roots to the underlying watcher.
    ///
    /// Silently ignores paths that do not exist yet (they will be retried on
    /// the next periodic call).  Logs a warning for any path that exists but
    /// cannot be watched.
    pub fn reestablish(&self) {
        reestablish_roots(&self.debouncer, &self.config);
        self.dirty.mark();
    }

    /// Request shutdown without waiting. Kept for legacy callers; this does not
    /// acknowledge cleanup. Use `shutdown_async` when completion matters.
    pub fn shutdown(self) {
        self.cancel.send_replace(true);
    }

    /// Cancel and observe retry-worker exit and notification resource shutdown.
    /// The join stays in `self` while awaited: cancelling this future cannot
    /// lose ownership of cleanup, and a later call can observe its completion.
    pub async fn shutdown_async(&mut self) -> Result<(), String> {
        self.cancel.send_replace(true);
        self.wait_stopped().await
    }

    /// Observe completion after receiver closure or a shutdown request.
    pub async fn wait_stopped(&mut self) -> Result<(), String> {
        let Some(worker) = self.worker.as_mut() else {
            return self.completion.clone().unwrap_or(Ok(()));
        };
        let result = worker
            .await
            .map_err(|error| error.to_string())
            .and_then(|result| result);
        self.worker = None;
        self.completion = Some(result.clone());
        result
    }
}

impl Drop for WatcherHandle {
    fn drop(&mut self) {
        // Dropping a legacy owner requests finite cleanup rather than leaving
        // an unowned infinite retry task. Async callers must observe completion.
        self.cancel.send_replace(true);
    }
}

/// One bit retains all overflow, independent of the number of changed paths.
/// Notify is only a wakeup; the atomic bit is the authoritative pending state.
#[derive(Default)]
struct DirtySignal {
    pending: AtomicBool,
    wake: Notify,
}

impl DirtySignal {
    fn mark(&self) {
        self.pending.store(true, Ordering::Release);
        self.wake.notify_one();
    }
}

fn deliver(tx: &mpsc::Sender<WatcherEvent>, dirty: &DirtySignal, event: WatcherEvent) {
    if matches!(tx.try_send(event), Err(mpsc::error::TrySendError::Full(_))) {
        dirty.mark();
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
    let dirty = Arc::new(DirtySignal::default());
    let cb_dirty = Arc::clone(&dirty);

    let debouncer = new_debouncer(
        DEBOUNCE_TIMEOUT,
        None, // tick_rate = None → 1/4 of timeout = 125 ms
        move |result: DebounceEventResult| {
            match result {
                Ok(events) => {
                    for event in events {
                        if event.need_rescan() {
                            cb_dirty.mark();
                        }
                        for path in &event.paths {
                            if let Some(watcher_event) = classify_path(
                                path,
                                &cb_catalog_dir,
                                cb_registry.as_deref(),
                                &cb_workspace_paths,
                            ) {
                                // Use try_send to avoid blocking the notify
                                // callback thread (Req 1.7).
                                deliver(&cb_tx, &cb_dirty, watcher_event);
                            }
                        }
                    }
                }
                Err(errors) => {
                    // An error invalidates the prior observation as surely as a
                    // change. Retain a rescan even when the event queue is full.
                    cb_dirty.mark();
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

    let debouncer_arc = Arc::new(Mutex::new(Some(debouncer)));

    // Establish initial watches.
    {
        let mut guard = debouncer_arc.lock().map_err(|e| TuiError::LogDestination {
            path: "watcher-mutex".to_string(),
            reason: e.to_string(),
        })?;
        let watcher = guard
            .as_mut()
            .ok_or_else(|| TuiError::LogDestination {
                path: "watcher-mutex".into(),
                reason: "watcher already stopped".into(),
            })?
            .watcher();

        if catalog_dir.exists() {
            watcher
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
                    if let Err(e) = watcher.watch(parent, RecursiveMode::NonRecursive) {
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
                if let Err(e) = watcher.watch(ws, RecursiveMode::Recursive) {
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

    let (cancel, mut cancelled) = watch::channel(false);
    let resources = Arc::clone(&debouncer_arc);
    let worker_config = config.clone();
    let cleanup_config = config.clone();
    let worker_dirty = Arc::clone(&dirty);
    #[cfg(all(test, unix))]
    let ingress = tx.clone();
    let worker = tokio::spawn(async move {
        let dirty = worker_dirty;
        let mut retry = tokio::time::interval(REESTABLISH_INTERVAL);
        // interval's first tick is immediate; initial watches already exist.
        retry.tick().await;
        loop {
            tokio::select! {
                biased;
                _ = cancelled.changed() => break,
                _ = tx.closed() => break,
                _ = dirty.wake.notified() => {
                    if !dirty.pending.swap(false, Ordering::AcqRel) { continue; }
                    // Overflow loses path detail, never the final invalidation.
                    // One full-source invalidation is bounded independently of event rate.
                    // Workspace is already the legacy full-refresh signal.
                    // Catalog(directory) would instead attempt a single-file
                    // reload in existing consumers and silently miss the rescan.
                    tokio::select! {
                        biased;
                        _ = cancelled.changed() => {},
                        result = tx.send(WatcherEvent::Workspace(worker_config.catalog_dir.clone())) => {
                            if result.is_err() { break; }
                        }
                    }
                    if *cancelled.borrow() || tx.is_closed() { break; }
                }
                _ = retry.tick() => {
                    reestablish_roots(&resources, &worker_config);
                    dirty.mark();
                }
            }
        }
        // stop() joins the debouncer thread. Never perform this blocking join
        // on the async executor, and do not report completion before it returns.
        tokio::task::spawn_blocking(move || {
            let debouncer = resources.lock().unwrap_or_else(|e| e.into_inner()).take();
            let mut failure = None;
            if let Some(mut debouncer) = debouncer {
                // Linux notify 6.1.1 unwatch waits for the event-loop response:
                // acknowledge removal of subscriptions before joining callbacks.
                let mut roots = vec![cleanup_config.catalog_dir];
                if let Some(parent) = cleanup_config
                    .registry_path
                    .as_deref()
                    .and_then(Path::parent)
                {
                    roots.push(parent.to_path_buf());
                }
                roots.extend(cleanup_config.workspace_paths);
                for root in roots {
                    if let Err(error) = debouncer.watcher().unwatch(&root) {
                        if !matches!(
                            error.kind,
                            notify_debouncer_full::notify::ErrorKind::WatchNotFound
                        ) {
                            failure = Some(error.to_string());
                        }
                    }
                }
                debouncer.stop();
            }
            failure.map_or(Ok(()), Err)
        })
        .await
        .map_err(|error| error.to_string())??;
        Ok(())
    });

    Ok((
        WatcherHandle {
            debouncer: debouncer_arc,
            config,
            cancel,
            worker: Some(worker),
            completion: None,
            dirty,
            #[cfg(all(test, unix))]
            ingress,
        },
        rx,
    ))
}

fn reestablish_roots(resources: &Mutex<Option<NativeDebouncer>>, config: &WatchConfig) {
    let mut guard = resources.lock().unwrap_or_else(|e| e.into_inner());
    let Some(debouncer) = guard.as_mut() else {
        return;
    };
    let watcher = debouncer.watcher();
    let mut roots = vec![(config.catalog_dir.as_path(), RecursiveMode::Recursive)];
    if let Some(parent) = config.registry_path.as_deref().and_then(Path::parent) {
        roots.push((parent, RecursiveMode::NonRecursive));
    }
    roots.extend(
        config
            .workspace_paths
            .iter()
            .map(|p| (p.as_path(), RecursiveMode::Recursive)),
    );
    for (path, mode) in roots {
        if path.exists() {
            if let Err(error) = watcher.watch(path, mode) {
                warn!(path = %path.display(), "failed to re-establish watch: {error}");
            }
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::TempDir;
    use tokio::time::timeout;

    #[tokio::test]
    async fn closed_receiver_releases_retry_owned_resources() {
        let dir = TempDir::new().unwrap();
        let (handle, rx) = spawn_watcher(dir.path(), None, &[]).unwrap();
        let resources = Arc::downgrade(&handle.debouncer);
        drop(rx);
        handle.shutdown();
        timeout(Duration::from_secs(3), async {
            while resources.upgrade().is_some() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("closed receiver/shutdown retained retry-owned resources");
    }

    #[cfg(unix)] // SourceSession native identity oracle is intentionally Unix-only.
    #[tokio::test]
    async fn overflow_coalesces_final_change_into_bounded_root_rescan() {
        let dir = TempDir::new().unwrap();
        std::fs::create_dir(dir.path().join("catalog")).unwrap();
        std::fs::write(
            dir.path().join("package.json"),
            r#"{"name":"@raishin/vanguard-frontier-agentic"}"#,
        )
        .unwrap();
        let agents = dir.path().join("catalog/agents.json");
        std::fs::write(
            &agents,
            include_bytes!("../../tests/fixtures/compatibility/core/catalog/agents.json"),
        )
        .unwrap();
        let mut session = crate::engine::session::SourceSession::new();
        let first = session.select(dir.path()).unwrap();
        assert_eq!(first.catalog().agents.len(), 3);
        // A separate watched root isolates the deterministic callback-injection
        // grader from incidental native event scheduling during fixture writes.
        let notifications = TempDir::new().unwrap();
        let (mut handle, mut events) = spawn_watcher(notifications.path(), None, &[]).unwrap();
        // No await: on this current-thread executor the worker cannot drain the
        // channel while the callback delivery path is deterministically flooded.
        for index in 0..10_000 {
            deliver(
                &handle.ingress,
                &handle.dirty,
                WatcherEvent::Catalog(dir.path().join(format!("{index}.json"))),
            );
        }
        // Final bytes differ from the acquired observation. The final callback
        // was dropped from the full queue; only its retained dirty signal remains.
        std::fs::write(&agents, "[]").unwrap();
        assert_eq!(events.len(), 256);
        assert!(handle.dirty.pending.load(Ordering::Acquire));
        // Handshake: let the supervisor take the first dirty bit while its
        // rescan send is blocked by the full queue. A later final invalidation
        // must survive this in-flight send (a wake-only design loses it).
        timeout(Duration::from_secs(3), async {
            while handle.dirty.pending.load(Ordering::Acquire) {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        deliver(
            &handle.ingress,
            &handle.dirty,
            WatcherEvent::Catalog(agents.clone()),
        );
        assert!(handle.dirty.pending.load(Ordering::Acquire));
        for _ in 0..256 {
            events.recv().await.unwrap();
        }
        assert_eq!(
            timeout(Duration::from_secs(3), events.recv())
                .await
                .unwrap(),
            Some(WatcherEvent::Workspace(notifications.path().to_path_buf()))
        );
        assert_eq!(
            timeout(Duration::from_secs(3), events.recv())
                .await
                .unwrap(),
            Some(WatcherEvent::Workspace(notifications.path().to_path_buf()))
        );
        assert!(
            events.try_recv().is_err(),
            "overflow must not retain an unbounded path queue"
        );
        let refreshed = session.refresh_external(first.context()).unwrap().unwrap();
        assert!(refreshed.catalog().agents.is_empty());
        assert_eq!(refreshed.revision(), first.revision() + 1);
        handle.shutdown_async().await.unwrap();
    }

    // ── Pure classification tests (no filesystem, no timing) ─────────────────

    #[test]
    fn classify_catalog_file() {
        let cat = PathBuf::from("/tmp/catalog");
        let changed = cat.join("agents.json");
        let result = classify_path(&changed, &cat, None, &[]);
        assert_eq!(result, Some(WatcherEvent::Catalog(changed)));
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
