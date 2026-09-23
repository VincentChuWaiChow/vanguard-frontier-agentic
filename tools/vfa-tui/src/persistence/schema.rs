//! SQLite schema migrations embedded as `const` strings.
//!
//! Each migration is a `(version, sql)` tuple. [`MIGRATIONS`] lists them in
//! ascending order so [`IndexManager::migrate`] can apply them sequentially.

/// Bootstrap: meta table that holds `schema_version` and `console_version`.
/// Must be created before any migration can read the version.
pub const BOOTSTRAP_SQL: &str = "
CREATE TABLE IF NOT EXISTS schema_meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
";

/// Migration 001: initial workspace-scan schema.
///
/// Creates:
/// - `workspace_scans` — one row per workspace, keyed by path
/// - `installed_assets` — per-asset detail rows referencing workspace_scans
/// - `content_hashes` — SHA-256 cache for catalog files
pub const MIGRATION_001_INITIAL_SCHEMA: &str = "
PRAGMA journal_mode = WAL;
PRAGMA busy_timeout = 5000;
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS workspace_scans (
    workspace_path     TEXT PRIMARY KEY,
    workspace_name     TEXT NOT NULL,
    last_scan_ts       TEXT NOT NULL,
    scan_duration_ms   INTEGER NOT NULL,
    asset_count        INTEGER NOT NULL,
    fs_mtime_at_scan   INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS installed_assets (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_path   TEXT NOT NULL REFERENCES workspace_scans(workspace_path) ON DELETE CASCADE,
    asset_id         TEXT NOT NULL,
    asset_type       TEXT NOT NULL,
    installed_path   TEXT NOT NULL,
    content_hash     TEXT NOT NULL,
    version          TEXT,
    detection_method TEXT NOT NULL,
    harness          TEXT NOT NULL,
    scan_ts          TEXT NOT NULL,
    UNIQUE(workspace_path, asset_id, harness)
);

CREATE INDEX IF NOT EXISTS idx_installed_assets_workspace
    ON installed_assets(workspace_path);
CREATE INDEX IF NOT EXISTS idx_installed_assets_asset_id
    ON installed_assets(asset_id);

CREATE TABLE IF NOT EXISTS content_hashes (
    path         TEXT PRIMARY KEY,
    hash         TEXT NOT NULL,
    last_checked TEXT NOT NULL
);
";

/// Migration 002: audit log with append-only triggers.
///
/// Creates:
/// - `audit_log` — tamper-evident hash-chain audit records
/// - `audit_log_no_update` trigger — rejects UPDATE
/// - `audit_log_no_delete` trigger — rejects DELETE
pub const MIGRATION_002_AUDIT_LOG: &str = "
CREATE TABLE IF NOT EXISTS audit_log (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp   TEXT    NOT NULL,
    event_type  TEXT    NOT NULL,
    subject     TEXT    NOT NULL,
    details     TEXT    NOT NULL,
    operator    TEXT    NOT NULL,
    entry_hash  TEXT    NOT NULL,
    prev_hash   TEXT    NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp
    ON audit_log(timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_log_event_type
    ON audit_log(event_type);
CREATE INDEX IF NOT EXISTS idx_audit_log_subject
    ON audit_log(subject);

CREATE TRIGGER IF NOT EXISTS audit_log_no_update
    BEFORE UPDATE ON audit_log
BEGIN
    SELECT RAISE(ABORT, 'audit_log is append-only: UPDATE not permitted');
END;

CREATE TRIGGER IF NOT EXISTS audit_log_no_delete
    BEFORE DELETE ON audit_log
BEGIN
    SELECT RAISE(ABORT, 'audit_log is append-only: DELETE not permitted');
END;
";

/// Migration 003: gate execution history and drift tracking.
///
/// Creates:
/// - `gate_history` — per-gate execution records
/// - `drift_history` — per-asset drift detection/resolution records
pub const MIGRATION_003_GATE_HISTORY: &str = "
CREATE TABLE IF NOT EXISTS gate_history (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    gate_name      TEXT    NOT NULL,
    status         TEXT    NOT NULL,
    exit_code      INTEGER,
    duration_ms    INTEGER NOT NULL,
    timestamp      TEXT    NOT NULL,
    catalog_hash   TEXT    NOT NULL,
    output_excerpt TEXT
);

CREATE INDEX IF NOT EXISTS idx_gate_history_name
    ON gate_history(gate_name);
CREATE INDEX IF NOT EXISTS idx_gate_history_timestamp
    ON gate_history(timestamp);

CREATE TABLE IF NOT EXISTS drift_history (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_path TEXT    NOT NULL,
    asset_id       TEXT    NOT NULL,
    drift_type     TEXT    NOT NULL,
    first_detected TEXT    NOT NULL,
    resolved_at    TEXT,
    expected_hash  TEXT    NOT NULL,
    actual_hash    TEXT    NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_drift_workspace
    ON drift_history(workspace_path);
CREATE INDEX IF NOT EXISTS idx_drift_unresolved
    ON drift_history(resolved_at)
    WHERE resolved_at IS NULL;
";

/// Migration 004: coverage score cache (Req 3.6).
///
/// Creates:
/// - `coverage_cache` — one row per workspace with its most recent coverage
///   score and the timestamp it was computed, so the TUI/headless paths can
///   display cached scores without recomputing on every launch.
pub const MIGRATION_004_COVERAGE_CACHE: &str = "
CREATE TABLE IF NOT EXISTS coverage_cache (
    workspace_path TEXT    PRIMARY KEY,
    workspace_name TEXT    NOT NULL,
    coverage_score REAL    NOT NULL,
    computed_at    TEXT    NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_coverage_cache_name
    ON coverage_cache(workspace_name);
";

/// Migration 5 — bind the operator into the audit hash chain.
///
/// Rows written before this migration hashed
/// `prev || timestamp || event_type || subject || details`, leaving `operator`
/// stored but unauthenticated: the actor on an entry could be changed without
/// breaking the chain.  New rows hash the operator too, with every field
/// length-prefixed so bytes cannot move across a field boundary.  `hash_version` records
/// which recipe produced a row so existing chains keep verifying.
pub const MIGRATION_005_AUDIT_HASH_VERSION: &str = "\
ALTER TABLE audit_log ADD COLUMN hash_version INTEGER NOT NULL DEFAULT 1;
";

/// All migrations in ascending order.
///
/// Each entry is `(target_schema_version, sql)`. The migration runner applies
/// every entry whose version is greater than the current stored version.
pub const MIGRATIONS: &[(u32, &str)] = &[
    (1, MIGRATION_001_INITIAL_SCHEMA),
    (2, MIGRATION_002_AUDIT_LOG),
    (3, MIGRATION_003_GATE_HISTORY),
    (4, MIGRATION_004_COVERAGE_CACHE),
    (5, MIGRATION_005_AUDIT_HASH_VERSION),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_are_in_ascending_order() {
        let versions: Vec<u32> = MIGRATIONS.iter().map(|(v, _)| *v).collect();
        for window in versions.windows(2) {
            assert!(
                window[0] < window[1],
                "migrations must be ordered: {versions:?}"
            );
        }
    }

    #[test]
    fn bootstrap_creates_schema_meta() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(BOOTSTRAP_SQL).unwrap();
        // Inserting a key must succeed.
        conn.execute(
            "INSERT OR REPLACE INTO schema_meta (key, value) VALUES ('schema_version', '0')",
            [],
        )
        .unwrap();
        let v: String = conn
            .query_row(
                "SELECT value FROM schema_meta WHERE key = 'schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(v, "0");
    }

    #[test]
    fn migration_001_creates_expected_tables() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(BOOTSTRAP_SQL).unwrap();
        conn.execute_batch(MIGRATION_001_INITIAL_SCHEMA).unwrap();
        // workspace_scans exists
        conn.execute(
            "INSERT INTO workspace_scans \
             (workspace_path, workspace_name, last_scan_ts, scan_duration_ms, asset_count, fs_mtime_at_scan) \
             VALUES ('/tmp/ws', 'ws', '2025-01-01T00:00:00Z', 100, 5, 1234567890)",
            [],
        )
        .unwrap();
        // installed_assets exists and references workspace_scans
        conn.execute(
            "INSERT INTO installed_assets \
             (workspace_path, asset_id, asset_type, installed_path, content_hash, \
              detection_method, harness, scan_ts) \
             VALUES ('/tmp/ws', 'agent-1', 'agent', '/tmp/ws/.claude/agents/a.md', \
                     'hash', 'directory_scan', 'claude', '2025-01-01T00:00:00Z')",
            [],
        )
        .unwrap();
    }

    #[test]
    fn migration_002_append_only_triggers() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(BOOTSTRAP_SQL).unwrap();
        conn.execute_batch(MIGRATION_001_INITIAL_SCHEMA).unwrap();
        conn.execute_batch(MIGRATION_002_AUDIT_LOG).unwrap();

        // Insert succeeds.
        conn.execute(
            "INSERT INTO audit_log \
             (timestamp, event_type, subject, details, operator, entry_hash, prev_hash) \
             VALUES ('2025-01-01T00:00:00.000Z', 'policy_evaluation', 'ws', '{}', \
                     'system', 'hash1', '')",
            [],
        )
        .unwrap();

        // UPDATE must be rejected.
        let update_result =
            conn.execute("UPDATE audit_log SET subject = 'tampered' WHERE id = 1", []);
        assert!(
            update_result.is_err(),
            "UPDATE should be rejected by trigger"
        );

        // DELETE must be rejected.
        let delete_result = conn.execute("DELETE FROM audit_log WHERE id = 1", []);
        assert!(
            delete_result.is_err(),
            "DELETE should be rejected by trigger"
        );
    }

    #[test]
    fn migration_003_creates_gate_history() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(BOOTSTRAP_SQL).unwrap();
        conn.execute_batch(MIGRATION_001_INITIAL_SCHEMA).unwrap();
        conn.execute_batch(MIGRATION_002_AUDIT_LOG).unwrap();
        conn.execute_batch(MIGRATION_003_GATE_HISTORY).unwrap();

        conn.execute(
            "INSERT INTO gate_history \
             (gate_name, status, exit_code, duration_ms, timestamp, catalog_hash) \
             VALUES ('lint', 'pass', 0, 1200, '2025-01-01T00:00:00Z', 'chash')",
            [],
        )
        .unwrap();
    }
}
