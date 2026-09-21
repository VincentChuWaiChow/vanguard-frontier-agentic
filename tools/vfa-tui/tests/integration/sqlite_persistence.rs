//! Integration test 13.6 — SQLite persistence: write → restart → read, schema
//! migration to the latest version, audit append-only enforcement, and
//! workspace-scan staleness round-trips.
//! Validates: Requirements 19.1–19.9, 14.x

use vfa_tui::models::audit::AuditEventType;
use vfa_tui::persistence::audit::AuditLogger;
use vfa_tui::persistence::index::IndexManager;

/// Latest schema version this build knows about, derived from the migration
/// table so adding a migration never breaks these assertions.
fn latest_schema_version() -> u32 {
    vfa_tui::persistence::schema::MIGRATIONS
        .iter()
        .map(|(v, _)| *v)
        .max()
        .expect("at least one migration")
}

fn db_path(dir: &tempfile::TempDir) -> String {
    dir.path()
        .join("index.sqlite")
        .to_string_lossy()
        .into_owned()
}

#[test]
fn fresh_open_migrates_to_latest_version() {
    let mgr = IndexManager::open_in_memory().expect("open in-memory");
    // Every migration in the table must have been applied.
    assert_eq!(
        mgr.schema_version,
        latest_schema_version(),
        "fresh db should migrate to the latest known version"
    );
    // Re-running migrate() is idempotent.
    assert_eq!(mgr.migrate().expect("re-migrate"), latest_schema_version());
}

#[test]
fn audit_entries_survive_restart() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = db_path(&dir);

    // Session 1: open, write two audit entries.
    {
        let mgr = IndexManager::open(&path).expect("open 1");
        assert_eq!(mgr.schema_version, latest_schema_version());
        let mut logger = AuditLogger::new(&mgr, String::new());
        logger
            .log(
                AuditEventType::OperatorAction,
                "subject-1",
                serde_json::json!({"n": 1}),
                "alice",
            )
            .expect("log 1");
        logger
            .log(
                AuditEventType::PolicyEvaluation,
                "subject-2",
                serde_json::json!({"n": 2}),
                "bob",
            )
            .expect("log 2");
    }

    // Session 2: reopen the same file; entries must still be there and the
    // hash chain must verify.
    {
        let mgr = IndexManager::open(&path).expect("open 2");
        assert_eq!(
            mgr.schema_version,
            latest_schema_version(),
            "version preserved across restart"
        );

        let conn = mgr.read_connection().expect("read conn");
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM audit_log", [], |r| r.get(0))
            .expect("count");
        assert_eq!(count, 2, "both audit entries persisted across restart");

        let logger = AuditLogger::from_manager(&mgr).expect("from_manager");
        logger.verify_chain().expect("chain valid after restart");
    }
}

#[test]
fn audit_log_is_append_only() {
    let mgr = IndexManager::open_in_memory().expect("open");
    let mut logger = AuditLogger::new(&mgr, String::new());
    logger
        .log(
            AuditEventType::OperatorAction,
            "subj",
            serde_json::json!({}),
            "op",
        )
        .expect("log");

    let conn = mgr.write_conn();

    // UPDATE must be rejected by the append-only trigger.
    let update = conn.execute("UPDATE audit_log SET operator = 'mallory' WHERE id = 1", []);
    assert!(update.is_err(), "UPDATE on audit_log must be rejected");

    // DELETE must be rejected by the append-only trigger.
    let delete = conn.execute("DELETE FROM audit_log WHERE id = 1", []);
    assert!(delete.is_err(), "DELETE on audit_log must be rejected");

    // The row is still intact.
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM audit_log", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn workspace_scan_staleness_round_trip() {
    let mgr = IndexManager::open_in_memory().expect("open");
    let conn = mgr.write_conn();
    conn.execute(
        "INSERT INTO workspace_scans \
         (workspace_path, workspace_name, last_scan_ts, scan_duration_ms, asset_count, fs_mtime_at_scan) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params!["/ws/a", "a", "2026-06-14T00:00:00Z", 10_i64, 3_i64, 1000_i64],
    )
    .expect("insert scan");

    // Same mtime → not stale; newer mtime → stale; unknown workspace → stale.
    assert!(!mgr.is_scan_stale("/ws/a", 1000), "same mtime is fresh");
    assert!(mgr.is_scan_stale("/ws/a", 2000), "newer mtime is stale");
    assert!(
        mgr.is_scan_stale("/ws/unknown", 1000),
        "never-scanned workspace is stale"
    );

    let cached = mgr.load_cached_scan_paths();
    assert!(
        cached.contains(&"/ws/a".to_string()),
        "cached scan path returned"
    );
}

// ---------------------------------------------------------------------------
// 7.3 drift persistence + 7.1 coverage cache (via the single-writer task)
// ---------------------------------------------------------------------------

use std::path::PathBuf;

use vfa_tui::federation::coverage::persist_coverage_scores;
use vfa_tui::federation::drift::{persist_drift, DriftKind, DriftRecord};
use vfa_tui::persistence::writer::{spawn_writer, DbCommand};

async fn flush(tx: &tokio::sync::mpsc::Sender<DbCommand>) {
    let (ack_tx, ack_rx) = tokio::sync::oneshot::channel();
    tx.send(DbCommand::Flush(ack_tx)).await.unwrap();
    ack_rx.await.unwrap();
}

#[tokio::test]
async fn drift_records_persist_to_drift_history() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = db_path(&dir);

    {
        let tx = spawn_writer(Some(&path), 64).expect("spawn writer");
        let records = vec![
            DriftRecord {
                workspace_path: PathBuf::from("/ws/a"),
                asset_id: "agents/x".to_string(),
                expected_hash: "aaaa".to_string(),
                actual_hash: "bbbb".to_string(),
                kind: DriftKind::ContentDrift,
            },
            // None-kind records must be skipped by persist_drift.
            DriftRecord {
                workspace_path: PathBuf::from("/ws/a"),
                asset_id: "agents/clean".to_string(),
                expected_hash: "cccc".to_string(),
                actual_hash: "cccc".to_string(),
                kind: DriftKind::None,
            },
        ];
        persist_drift(&tx, &records, "2026-06-15T00:00:00Z").await;
        flush(&tx).await;
        tx.send(DbCommand::Shutdown).await.unwrap();
    }

    // Reopen and read back: exactly one (non-None) drift row, still unresolved.
    let mgr = IndexManager::open(&path).expect("reopen");
    let rows = mgr.load_drift_history();
    assert_eq!(rows.len(), 1, "only the genuine drift is persisted");
    let (ws, asset, kind, resolved_at) = &rows[0];
    assert_eq!(ws, "/ws/a");
    assert_eq!(asset, "agents/x");
    assert_eq!(kind, "content_drift");
    assert!(resolved_at.is_none(), "newly recorded drift is unresolved");
}

#[tokio::test]
async fn coverage_scores_persist_to_cache() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = db_path(&dir);

    {
        let tx = spawn_writer(Some(&path), 64).expect("spawn writer");
        let scores = vec![
            ("/ws/a".to_string(), "team-a".to_string(), 87.5_f64),
            ("/ws/b".to_string(), "team-b".to_string(), 100.0_f64),
        ];
        persist_coverage_scores(&tx, &scores, "2026-06-15T00:00:00Z").await;
        // Upsert: re-recording the same workspace replaces the score.
        persist_coverage_scores(
            &tx,
            &[("/ws/a".to_string(), "team-a".to_string(), 90.0_f64)],
            "2026-06-15T01:00:00Z",
        )
        .await;
        flush(&tx).await;
        tx.send(DbCommand::Shutdown).await.unwrap();
    }

    let mgr = IndexManager::open(&path).expect("reopen");
    let scores = mgr.load_coverage_scores();
    assert_eq!(
        scores.len(),
        2,
        "two distinct workspaces (upsert, not duplicate)"
    );
    let a = scores.iter().find(|(p, _)| p == "/ws/a").unwrap();
    let b = scores.iter().find(|(p, _)| p == "/ws/b").unwrap();
    assert_eq!(a.1, 90.0, "upsert replaced the earlier score");
    assert_eq!(b.1, 100.0);
}
