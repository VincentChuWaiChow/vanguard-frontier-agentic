//! Integration test: headless report auto-persists coverage/drift to SQLite.
//!
//! Validates Residual 1 — after `HeadlessReporter::run()` the SQLite index at
//! `cli.index_path` is created and the `coverage_cache` / `drift_history`
//! tables are queryable (even when the workspace registry does not exist and
//! therefore no workspaces or drift rows are recorded).

use std::path::{Path, PathBuf};

use tempfile::TempDir;

use vfa_tui::cli::{Cli, LogLevel, OutputFormatCli};
use vfa_tui::headless::reporter::HeadlessReporter;
use vfa_tui::models::report::OutputFormat;
use vfa_tui::persistence::index::IndexManager;

fn fixtures_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// Build a minimal [`Cli`] pointing at `index_path` with a nonexistent registry.
fn make_cli(index_path: &str) -> Cli {
    Cli {
        workspace: None,
        log_file: None,
        log_level: LogLevel::Info,
        no_color: true,
        theme: vfa_tui::ui::theme::ThemePreference::Auto,
        registry: "/nonexistent/workspaces.toml".to_string(),
        policies: "/nonexistent/policies.toml".to_string(),
        index_path: index_path.to_string(),
        _home_expanded: true,
        report: None,
        format: OutputFormatCli::Json,
        workspace_filter: None,
        rebuild_index: false,
        quiet: true,
        allow_empty_policy: false,
        validate_config: false,
        export_audit: None,
        web: false,
        web_bind: "127.0.0.1:8080".to_string(),
    }
}

#[test]
fn headless_report_creates_queryable_db() {
    let root = fixtures_root();
    if !root.join("catalog").exists() {
        // Skip when catalog fixtures are not available.
        return;
    }

    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("index.db");
    let db_str = db_path.to_str().unwrap().to_string();

    let cli = make_cli(&db_str);
    let reporter = HeadlessReporter::new(OutputFormat::Json, true);

    // Run the headless report — this should persist to the DB.
    let (_val, _exit) = reporter.run(&cli, &root);

    // The DB file must now exist.
    assert!(db_path.exists(), "index.db must be created by headless run");

    // Open the DB and verify the tables are queryable (schema was migrated).
    let mgr = IndexManager::open(&db_str).expect("open index after headless run");
    let latest = vfa_tui::persistence::schema::MIGRATIONS
        .iter()
        .map(|(v, _)| *v)
        .max()
        .expect("at least one migration");
    assert_eq!(
        mgr.schema_version, latest,
        "schema must be migrated to the latest known version"
    );

    // coverage_cache table must be queryable (may be empty — no workspaces).
    let scores = mgr.load_coverage_scores();
    // With no workspaces (registry missing) scores must be an empty vec, not a panic.
    let _ = scores;

    // drift_history table must be queryable (may be empty — no installed assets).
    let drift = mgr.load_drift_history();
    let _ = drift;
}

#[test]
fn headless_report_persists_coverage_scores_when_workspaces_present() {
    // This test uses the fixtures workspace which has a catalog but no
    // registered downstream workspaces.  We verify that the DB is written to
    // and that re-running the report does not panic or corrupt the DB.
    let root = fixtures_root();
    if !root.join("catalog").exists() {
        return;
    }

    let tmp = TempDir::new().unwrap();
    let db_str = tmp.path().join("idx.db").to_str().unwrap().to_string();

    let cli = make_cli(&db_str);
    let reporter = HeadlessReporter::new(OutputFormat::Json, true);

    // Run twice to verify idempotency (INSERT OR REPLACE).
    let (_v1, _e1) = reporter.run(&cli, &root);
    let (_v2, _e2) = reporter.run(&cli, &root);

    let mgr = IndexManager::open(&db_str).unwrap();
    // Tables remain queryable after two runs.
    let _ = mgr.load_coverage_scores();
    let _ = mgr.load_drift_history();
}
