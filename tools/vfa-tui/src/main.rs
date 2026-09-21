//! VFA-TUI entry point — mode dispatch and wiring (Tasks 11.1–11.5).
//!
//! # Modes
//!
//! After parsing CLI args and detecting the workspace root the binary enters
//! one of four modes:
//!
//! 1. **ValidateConfig** (`--validate-config`): parse all config files, report
//!    errors to stderr, exit 0 if valid or 2 if invalid (Req 31.2).
//! 2. **ExportAudit** (`--export-audit <fmt> <path>`): open the SQLite index,
//!    export the audit log, exit 0/2.
//! 3. **Headless** (`--report <type>`): run the HeadlessReporter pipeline —
//!    single scan, no terminal manipulation, no filesystem watchers (Req 1.5,
//!    17.1).  Exits with the reporter's exit code (0–3, Req 18).
//! 4. **TUI** (default): set up raw mode / alternate screen, run the existing
//!    synchronous event loop, exit 0 on quit (Req 18.7).
//!
//! # Accessibility (Req 29)
//!
//! `--no-color` / `NO_COLOR` is detected via `Cli::is_no_color()`.  The flag
//! is forwarded to the headless reporter and app state.  Status text
//! indicators (`[PASS]`, `[FAIL]`, etc.) are always present in headless output
//! regardless of color mode (Req 29.2).
//!
//! # Cross-platform paths (Req 28)
//!
//! Default registry/policy/index paths are resolved via `crate::paths`.  The
//! clap defaults in `Cli` already embed the Linux XDG defaults; this module
//! does *not* override them at runtime — the `paths` module is available for
//! callers that need the helpers independently.

#![deny(warnings)]

use vfa_tui::{app, catalog, federation, headless, logging, persistence, policy, ui, workspace};

use std::path::PathBuf;

use clap::Parser;

use vfa_tui::cli::Cli;
use vfa_tui::headless::reporter::HeadlessReporter;

// ---------------------------------------------------------------------------
// Mode selection
// ---------------------------------------------------------------------------

/// The four execution modes the binary can enter.
///
/// Extracted into a pure enum so `select_mode` can be unit-tested independently
/// of I/O (Task 11.1 requirement for testable dispatch).
#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    /// `--validate-config` was passed.
    ValidateConfig,
    /// `--export-audit <fmt> <path>` was passed.
    ExportAudit,
    /// `--report <type>` was passed — headless pipeline, no TUI.
    Headless,
    /// No special flags — launch the interactive TUI.
    Tui,
}

/// Determine which mode to run based on the parsed CLI args.
///
/// Priority order (first match wins):
/// 1. `--validate-config`
/// 2. `--export-audit`
/// 3. `--report` present → Headless
/// 4. Default → TUI
pub fn select_mode(cli: &Cli) -> Mode {
    if cli.validate_config {
        return Mode::ValidateConfig;
    }
    if cli.export_audit.is_some() {
        return Mode::ExportAudit;
    }
    if cli.report.is_some() {
        return Mode::Headless;
    }
    Mode::Tui
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut cli = Cli::parse();
    cli.expand_home_paths();
    let mode = select_mode(&cli);

    // ── Workspace detection ───────────────────────────────────────────────────
    // Detect workspace root; on error print to stderr and exit 2 (operational).
    let workspace_root = match workspace::detect::detect_workspace(cli.workspace.as_deref()) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[vfa-tui] workspace detection failed: {e}");
            std::process::exit(2);
        }
    };

    // ── Logging init ──────────────────────────────────────────────────────────
    // For headless/validate-config modes use stderr subscriber (no file).
    // For TUI mode use file-based subscriber when --log-file is set.
    let session_id = uuid::Uuid::new_v4();
    let log_file = match mode {
        Mode::Headless | Mode::ValidateConfig | Mode::ExportAudit => {
            // In headless modes we suppress file logging unless explicitly
            // requested, to avoid interfering with structured stdout output.
            if cli.quiet {
                None
            } else {
                cli.log_file.as_deref()
            }
        }
        Mode::Tui => cli.log_file.as_deref(),
    };

    let log_level_str = format!("{:?}", cli.log_level).to_lowercase();
    logging::audit::init_logging(log_file, &log_level_str, session_id)?;

    // ── Mode dispatch ─────────────────────────────────────────────────────────
    match mode {
        // ── ValidateConfig ────────────────────────────────────────────────────
        Mode::ValidateConfig => {
            run_validate_config(&cli, &workspace_root);
        }

        // ── ExportAudit ───────────────────────────────────────────────────────
        Mode::ExportAudit => {
            run_export_audit(&cli, &workspace_root);
        }

        // ── Headless ──────────────────────────────────────────────────────────
        // Req 17.1: no raw mode, no alternate screen, no panic hook.
        // Req 1.5: single scan, no filesystem watchers.
        Mode::Headless => {
            let format = cli.output_format();
            let quiet = cli.quiet;
            let reporter = HeadlessReporter::new(format, quiet);

            // Pass no-color flag through to formatting context.
            // HeadlessReporter.run() writes formatted output to stdout directly.
            let (_value, exit_code) = reporter.run(&cli, &workspace_root);

            // Best-effort audit record of the headless run (Req 14.7 / Task 11.2).
            // Never block report output on audit-log availability. Ensure the
            // index's parent dir exists (clean installs lack ~/.local/share/vfa).
            if let Some(parent) = std::path::Path::new(&cli.index_path).parent() {
                if !parent.as_os_str().is_empty() {
                    let _ = std::fs::create_dir_all(parent);
                }
            }
            // Audit persistence stays non-blocking (Req 14.7) but must not be
            // silent: a failure here previously left no trace at all, so an
            // operator could believe a run was recorded when nothing was.
            match persistence::index::IndexManager::open(&cli.index_path) {
                Ok(mgr) => {
                    if let Err(e) = headless::reporter::record_headless_audit(
                        &mgr,
                        &cli.report_types(),
                        exit_code,
                    ) {
                        eprintln!(
                            "[vfa-tui] WARNING: report completed but the audit record was \
                             not written to {}: {e}",
                            cli.index_path
                        );
                    }
                }
                Err(e) => {
                    eprintln!(
                        "[vfa-tui] WARNING: report completed but the audit index at {} \
                         could not be opened: {e}",
                        cli.index_path
                    );
                }
            }

            std::process::exit(exit_code as i32);
        }

        // ── TUI ───────────────────────────────────────────────────────────────
        // Install the terminal panic hook only in TUI mode (Req 17.1 — headless
        // must never enter raw mode / install terminal hooks).
        Mode::Tui => {
            ui::install_panic_hook();
            run_tui_async(&cli, &workspace_root).await?;
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// ValidateConfig runner (Req 31.2)
// ---------------------------------------------------------------------------

/// Parse all configuration files and report errors.  Exits 0 if valid, 2 if
/// any config is invalid.
fn run_validate_config(cli: &Cli, workspace_root: &std::path::Path) {
    let mut errors: Vec<String> = Vec::new();

    // Validate registry TOML.
    let registry_path = PathBuf::from(&cli.registry);
    if registry_path.exists() {
        match federation::registry::WorkspaceRegistry::load(&registry_path) {
            Ok(federation::registry::LoadResult::Loaded(reg)) => {
                let validation_errors = reg.validate();
                if !validation_errors.is_empty() {
                    for err in validation_errors {
                        errors.push(format!(
                            "registry {}: semantic validation error: {err}",
                            registry_path.display()
                        ));
                    }
                }
            }
            Ok(federation::registry::LoadResult::NotFound(_)) => {
                // Registry was not found but that's ok for validation.
            }
            Err(e) => {
                errors.push(format!(
                    "registry {}: parse/load error: {e}",
                    registry_path.display()
                ));
            }
        }
    } else if !cli.quiet {
        eprintln!(
            "[vfa-tui] registry not found at {} (skipping validation)",
            registry_path.display()
        );
    }

    // Validate policies TOML.
    let policy_path = PathBuf::from(&cli.policies);
    if policy_path.exists() {
        match policy::parser::load(&policy_path) {
            Ok(_) => {}
            Err(e) => {
                errors.push(format!("policies {}: {e}", policy_path.display()));
            }
        }
    } else if !cli.quiet {
        eprintln!(
            "[vfa-tui] policies not found at {} (skipping validation)",
            policy_path.display()
        );
    }

    // Validate catalog directory.
    let catalog_dir = workspace_root.join("catalog");
    if !catalog_dir.exists() {
        errors.push(format!(
            "catalog directory not found at {}",
            catalog_dir.display()
        ));
    }

    if errors.is_empty() {
        if !cli.quiet {
            eprintln!("[vfa-tui] configuration valid");
        }
        std::process::exit(0);
    } else {
        for e in &errors {
            eprintln!("[vfa-tui] config error: {e}");
        }
        std::process::exit(2);
    }
}

// ---------------------------------------------------------------------------
// ExportAudit runner
// ---------------------------------------------------------------------------

/// Open the SQLite index and export the audit log to the requested format/path.
fn run_export_audit(cli: &Cli, _workspace_root: &std::path::Path) {
    let parts = cli
        .export_audit
        .as_ref()
        .expect("ExportAudit mode implies export_audit is Some");
    if parts.len() < 2 {
        eprintln!("[vfa-tui] --export-audit requires FORMAT and PATH");
        std::process::exit(2);
    }
    let format = parts[0].as_str();
    let out_path = std::path::Path::new(&parts[1]);

    // Open (or create) the SQLite index.
    let index_path = &cli.index_path;
    let mgr = match persistence::index::IndexManager::open(index_path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("[vfa-tui] failed to open index at {index_path}: {e}");
            std::process::exit(2);
        }
    };

    let logger = match persistence::audit::AuditLogger::from_manager(&mgr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[vfa-tui] failed to initialise audit logger: {e}");
            std::process::exit(2);
        }
    };

    // Verify before export: an export that silently emits a broken chain is
    // worse than no export, because the output looks like attested evidence.
    if let Err(e) = logger.verify_chain() {
        eprintln!("[vfa-tui] refusing to export: audit chain integrity check failed: {e}");
        std::process::exit(2);
    }

    match logger.export_audit(format, out_path) {
        Ok(()) => {
            if !cli.quiet {
                eprintln!(
                    "[vfa-tui] audit log exported ({format}) to {}",
                    out_path.display()
                );
            }
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("[vfa-tui] export failed: {e}");
            std::process::exit(2);
        }
    }
}

// ---------------------------------------------------------------------------
// TUI runner — async event loop with tokio::select!
// ---------------------------------------------------------------------------

use tokio::sync::mpsc;
use tokio::time::interval;

/// Launch the interactive TUI with async event loop.
///
/// Multiplexes event sources via tokio::select!:
/// 1. Crossterm events (key press, resize) — polling in blocking task
/// 2. 250ms tick timer — triggers state updates and coalesces input
///
/// Rendering is optimized via dirty flag: only renders when state changed.
/// Event coalescing: rapid input within 250ms window is coalesced into one render.
/// (Req 9.1, 18.7)
async fn run_tui_async(cli: &Cli, workspace_root: &std::path::Path) -> anyhow::Result<()> {
    // Load catalog.
    let catalog = catalog::store::CatalogStore::load(workspace_root);
    if !catalog.load_errors.is_empty() {
        for err in &catalog.load_errors {
            tracing::warn!(%err, "catalog load warning");
        }
    }

    // Setup terminal (raw mode + alternate screen).
    let mut terminal_mgr = ui::TerminalManager::new()?;

    let session_id = uuid::Uuid::new_v4();
    let no_color = cli.is_no_color();

    // Resolve the light/dark theme: `--theme dark|light` forces a palette,
    // `--theme auto` (default) detects the terminal background (Req 35.1–35.3).
    // This is the interactive (TUI) path, so detection is permitted.
    let theme_mode = ui::theme::resolve_theme(cli.theme, false);

    // Create app.
    let mut app = app::App::new(catalog, workspace_root.to_path_buf(), session_id, no_color);
    app.theme_mode = theme_mode;

    // Channel: crossterm events sent from blocking task.
    let (tx_event, mut rx_event) = mpsc::channel(256);

    // Spawn crossterm event reader in a blocking task (non-async I/O).
    let _event_task = tokio::task::spawn_blocking({
        let tx = tx_event.clone();
        move || loop {
            if let Ok(true) = crossterm::event::poll(std::time::Duration::from_millis(50)) {
                if let Ok(evt) = crossterm::event::read() {
                    if tx.blocking_send(evt).is_err() {
                        break;
                    }
                }
            }
        }
    });

    // Timer: 250ms tick for event coalescing and state cleanup.
    let mut tick_interval = interval(std::time::Duration::from_millis(250));

    // Filesystem watcher: live-reload the catalog when files change (Req 4.x /
    // Task 9.1). Best-effort — if the watcher cannot be established (e.g. the
    // catalog dir is missing), fall back to a tick-only loop.
    let catalog_dir = workspace_root.join("catalog");
    let registry_pb = std::path::PathBuf::from(&cli.registry);
    let registry_opt = if registry_pb.exists() {
        Some(registry_pb.as_path())
    } else {
        None
    };
    let (_watch_handle, mut rx_watch) =
        match catalog::watcher::spawn_watcher(&catalog_dir, registry_opt, &[]) {
            Ok((handle, rx)) => (Some(handle), Some(rx)),
            Err(e) => {
                tracing::warn!(%e, "filesystem watcher unavailable; live reload disabled");
                (None, None)
            }
        };

    loop {
        // Drain all pending events within this tick cycle (event coalescing).
        let mut event_occurred = false;
        while let Ok(evt) = rx_event.try_recv() {
            use crossterm::event::Event;
            match evt {
                Event::Key(key) => {
                    app.handle_key_event(key);
                    event_occurred = true;
                }
                Event::Resize(_width, _height) => {
                    event_occurred = true;
                }
                _ => {}
            }
        }

        // If any event occurred, mark dirty.
        if event_occurred {
            app.mark_dirty();
        }

        // Render only if dirty flag is set.
        if app.dirty {
            terminal_mgr.draw(|frame| app.render(frame))?;
            app.dirty = false;
        }

        if app.should_quit {
            break;
        }

        // Wait for the 250ms tick or a filesystem-watcher event.
        tokio::select! {
            _ = tick_interval.tick() => {
                app.tick();
                app.mark_dirty();
            }
            maybe_ev = async {
                match rx_watch.as_mut() {
                    Some(rx) => rx.recv().await,
                    None => std::future::pending().await,
                }
            } => {
                if let Some(event) = maybe_ev {
                    match event {
                        catalog::watcher::WatcherEvent::Catalog(path) => {
                            // A deleted file would surface as a read error that
                            // `reload_catalog_file` treats as RetainedPrevious, so
                            // removals would never reflect. Full-reload when the
                            // path no longer exists.
                            if path.exists() {
                                let _ = app.reload_catalog_file(&path);
                            } else {
                                app.reload_catalog();
                            }
                        }
                        catalog::watcher::WatcherEvent::Registry
                        | catalog::watcher::WatcherEvent::Workspace(_) => {
                            app.reload_catalog();
                        }
                    }
                }
            }
        }
    }

    drop(tx_event);
    terminal_mgr.restore()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Unit tests — select_mode dispatch
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn parse(args: &[&str]) -> Cli {
        Cli::try_parse_from(args).expect("parse")
    }

    #[test]
    fn default_flags_select_tui() {
        let cli = parse(&["vfa-tui"]);
        assert_eq!(select_mode(&cli), Mode::Tui);
    }

    #[test]
    fn validate_config_flag_selects_validate_config() {
        let cli = parse(&["vfa-tui", "--validate-config"]);
        assert_eq!(select_mode(&cli), Mode::ValidateConfig);
    }

    #[test]
    fn export_audit_flag_selects_export_audit() {
        let cli = parse(&["vfa-tui", "--export-audit", "json", "/tmp/a.json"]);
        assert_eq!(select_mode(&cli), Mode::ExportAudit);
    }

    #[test]
    fn report_flag_selects_headless() {
        let cli = parse(&["vfa-tui", "--report", "summary"]);
        assert_eq!(select_mode(&cli), Mode::Headless);
    }

    #[test]
    fn report_coverage_selects_headless() {
        let cli = parse(&["vfa-tui", "--report", "coverage"]);
        assert_eq!(select_mode(&cli), Mode::Headless);
    }

    #[test]
    fn report_all_selects_headless() {
        let cli = parse(&["vfa-tui", "--report", "all"]);
        assert_eq!(select_mode(&cli), Mode::Headless);
    }

    #[test]
    fn validate_config_has_priority_over_report() {
        // --validate-config takes priority even when --report is also set.
        let cli = parse(&["vfa-tui", "--validate-config", "--report", "summary"]);
        assert_eq!(select_mode(&cli), Mode::ValidateConfig);
    }

    #[test]
    fn export_audit_has_priority_over_report() {
        let cli = parse(&[
            "vfa-tui",
            "--export-audit",
            "json",
            "/tmp/a.json",
            "--report",
            "summary",
        ]);
        assert_eq!(select_mode(&cli), Mode::ExportAudit);
    }

    #[test]
    fn no_color_accessible_via_is_no_color() {
        let cli = parse(&["vfa-tui", "--no-color"]);
        assert!(cli.is_no_color());
    }

    #[test]
    fn web_flag_still_selects_tui() {
        // --web is a stretch goal; without --report it should still be TUI mode.
        let cli = parse(&["vfa-tui", "--web"]);
        assert_eq!(select_mode(&cli), Mode::Tui);
    }
}
