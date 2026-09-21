//! SQLite index manager — WAL mode, single-writer pattern, schema migrations.
//!
//! Implements Requirement 19 (all 9 criteria):
//! - WAL mode + `SQLITE_OPEN_NO_MUTEX` (Req 19.7, 19.9)
//! - Single-writer task via mpsc channel (Req 19.8)
//! - Sequential schema migrations preserving data (Req 19.4)
//! - In-memory fallback when file is inaccessible (Req 19.5)
//! - Stale-scan detection via `fs_mtime_at_scan` (Req 19.3)

use rusqlite::{Connection, OpenFlags};

use crate::error::TuiError;
use crate::persistence::schema::{BOOTSTRAP_SQL, MIGRATIONS};

// ---------------------------------------------------------------------------
// IndexManager
// ---------------------------------------------------------------------------

/// Manages the SQLite persistence layer.
///
/// Owns one write `Connection` (used by the writer task) and exposes
/// `read_connection()` to open additional per-thread read-only connections.
pub struct IndexManager {
    /// Path passed at open time — `None` for in-memory databases.
    db_path: Option<String>,
    /// Write connection (owned here; writer task borrows via the mpsc channel).
    write_conn: Connection,
    /// Current schema version after migration.
    pub schema_version: u32,
}

impl IndexManager {
    // -----------------------------------------------------------------------
    // Constructors
    // -----------------------------------------------------------------------

    /// Open (or create) the SQLite index at `path` with WAL mode.
    ///
    /// Uses `SQLITE_OPEN_NO_MUTEX` — thread safety is handled by Rust's
    /// ownership model (Req 19.9).  Runs schema migrations on every open
    /// (Req 19.4).
    pub fn open(path: &str) -> Result<Self, TuiError> {
        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX;
        let conn =
            Connection::open_with_flags(path, flags).map_err(|e| TuiError::PersistenceOpen {
                path: path.to_string(),
                detail: e.to_string(),
            })?;
        // Enable WAL mode (Req 19.7).
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| TuiError::PersistenceOpen {
                path: path.to_string(),
                detail: format!("WAL pragma failed: {e}"),
            })?;
        let mut mgr = Self {
            db_path: Some(path.to_string()),
            write_conn: conn,
            schema_version: 0,
        };
        mgr.schema_version = mgr.migrate()?;
        Ok(mgr)
    }

    /// Open an in-memory database — useful for tests and as a fallback when
    /// the on-disk file is corrupt or inaccessible (Req 19.5).
    pub fn open_in_memory() -> Result<Self, TuiError> {
        let conn = Connection::open_in_memory().map_err(|e| TuiError::PersistenceOpen {
            path: ":memory:".to_string(),
            detail: e.to_string(),
        })?;
        let mut mgr = Self {
            db_path: None,
            write_conn: conn,
            schema_version: 0,
        };
        mgr.schema_version = mgr.migrate()?;
        Ok(mgr)
    }

    // -----------------------------------------------------------------------
    // Migration
    // -----------------------------------------------------------------------

    /// Run pending schema migrations sequentially, preserving existing data.
    ///
    /// Detects the current version from `schema_meta` (0 if the table doesn't
    /// yet exist), applies each migration whose target version is higher, and
    /// updates `schema_version` afterward.  Each migration runs inside its own
    /// `SAVEPOINT` so a failure leaves the database at a clean prior version
    /// rather than in an unknown intermediate state (Req 19.4).
    pub fn migrate(&self) -> Result<u32, TuiError> {
        // Bootstrap: ensure schema_meta exists before querying it.
        self.write_conn.execute_batch(BOOTSTRAP_SQL).map_err(|e| {
            TuiError::PersistenceMigration {
                from: 0,
                to: 0,
                detail: format!("bootstrap failed: {e}"),
            }
        })?;

        // Read current version.  Absent key = fresh database (0).  A present
        // but unparsable value is NOT "fresh": silently treating it as 0
        // re-ran every migration against an unknown schema.  A version newer
        // than this build understands is rejected rather than operated on.
        let stored_version: Option<String> = match self.write_conn.query_row(
            "SELECT value FROM schema_meta WHERE key = 'schema_version'",
            [],
            |row| row.get::<_, String>(0),
        ) {
            Ok(s) => Some(s),
            Err(rusqlite::Error::QueryReturnedNoRows) => None,
            Err(e) => {
                return Err(TuiError::PersistenceMigration {
                    from: 0,
                    to: 0,
                    detail: format!("could not read schema_version: {e}"),
                })
            }
        };

        let current_version: u32 = match stored_version {
            None => 0,
            Some(raw) => raw
                .trim()
                .parse::<u32>()
                .map_err(|_| TuiError::PersistenceMigration {
                    from: 0,
                    to: 0,
                    detail: format!(
                        "unreadable schema_version {raw:?}; refusing to migrate a \
                             database with an unrecognised schema marker"
                    ),
                })?,
        };

        let latest_known = MIGRATIONS.iter().map(|(v, _)| *v).max().unwrap_or(0);
        if current_version > latest_known {
            return Err(TuiError::PersistenceMigration {
                from: current_version,
                to: latest_known,
                detail: format!(
                    "database schema version {current_version} is newer than the latest \
                     version this build supports ({latest_known}); upgrade vfa-tui rather \
                     than operating on a forward-versioned database"
                ),
            });
        }

        let mut version = current_version;

        for (target_version, sql) in MIGRATIONS {
            if *target_version <= version {
                continue;
            }
            // Apply this migration inside a savepoint for clean rollback.
            let sp_name = format!("migration_{target_version}");
            self.write_conn
                .execute_batch(&format!("SAVEPOINT {sp_name}"))
                .map_err(|e| TuiError::PersistenceMigration {
                    from: version,
                    to: *target_version,
                    detail: format!("savepoint failed: {e}"),
                })?;

            match self.write_conn.execute_batch(sql) {
                Ok(()) => {
                    // Update the stored version.
                    self.write_conn
                        .execute(
                            "INSERT OR REPLACE INTO schema_meta (key, value) \
                             VALUES ('schema_version', ?1)",
                            rusqlite::params![target_version.to_string()],
                        )
                        .map_err(|e| TuiError::PersistenceMigration {
                            from: version,
                            to: *target_version,
                            detail: format!("version update failed: {e}"),
                        })?;
                    self.write_conn
                        .execute_batch(&format!("RELEASE {sp_name}"))
                        .map_err(|e| TuiError::PersistenceMigration {
                            from: version,
                            to: *target_version,
                            detail: format!("release savepoint failed: {e}"),
                        })?;
                    version = *target_version;
                }
                Err(e) => {
                    let _ = self
                        .write_conn
                        .execute_batch(&format!("ROLLBACK TO {sp_name}"));
                    return Err(TuiError::PersistenceMigration {
                        from: version,
                        to: *target_version,
                        detail: e.to_string(),
                    });
                }
            }
        }

        Ok(version)
    }

    // -----------------------------------------------------------------------
    // Read connections (Req 19.8 — separate Connection per thread)
    // -----------------------------------------------------------------------

    /// Open a fresh **read-only** `Connection` for use in a caller thread.
    ///
    /// Each call returns an independent `Connection` instance, satisfying
    /// the requirement that every thread or task gets its own connection
    /// (Req 19.8, 19.9).
    ///
    /// In-memory managers share the write connection for reads (the in-memory
    /// database is not accessible from a second connection object).
    pub fn read_connection(&self) -> Result<Connection, TuiError> {
        match &self.db_path {
            Some(path) => {
                let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
                Connection::open_with_flags(path, flags).map_err(|e| TuiError::PersistenceQuery {
                    detail: format!("read connection failed: {e}"),
                })
            }
            None => {
                // For in-memory databases there is only one logical connection;
                // return an error so callers fall back to the write connection.
                Err(TuiError::PersistenceQuery {
                    detail: "in-memory database has no separate read connection".to_string(),
                })
            }
        }
    }

    /// Borrow the write connection for direct use (e.g. from tests or the
    /// writer task before the channel is set up).
    pub fn write_conn(&self) -> &Connection {
        &self.write_conn
    }

    // -----------------------------------------------------------------------
    // Stale-scan detection (Req 19.3)
    // -----------------------------------------------------------------------

    /// Returns `true` if the cached scan for `workspace_path` is older than
    /// `fs_mtime` (Unix seconds), meaning a re-scan is needed.
    ///
    /// Returns `true` when there is no cached entry (first scan).
    pub fn is_scan_stale(&self, workspace_path: &str, fs_mtime: i64) -> bool {
        let result: rusqlite::Result<i64> = self.write_conn.query_row(
            "SELECT fs_mtime_at_scan FROM workspace_scans WHERE workspace_path = ?1",
            rusqlite::params![workspace_path],
            |row| row.get(0),
        );
        match result {
            Ok(cached_mtime) => fs_mtime > cached_mtime,
            Err(_) => true, // No cached entry → stale by definition.
        }
    }

    // -----------------------------------------------------------------------
    // Cached scan loading (Req 19.2)
    // -----------------------------------------------------------------------

    /// Return all workspace paths that have cached scan entries.
    pub fn load_cached_scan_paths(&self) -> Vec<String> {
        let mut stmt = match self
            .write_conn
            .prepare("SELECT workspace_path FROM workspace_scans ORDER BY workspace_path")
        {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        stmt.query_map([], |row| row.get::<_, String>(0))
            .map(|rows| rows.filter_map(|r| r.ok()).collect())
            .unwrap_or_default()
    }

    // -----------------------------------------------------------------------
    // Drift history (Req 10.x)
    // -----------------------------------------------------------------------

    /// Load all drift-history rows as
    /// `(workspace_path, asset_id, drift_type, resolved_at)` tuples, ordered by
    /// id. `resolved_at` is `None` for still-active drift.
    pub fn load_drift_history(&self) -> Vec<(String, String, String, Option<String>)> {
        let mut stmt = match self.write_conn.prepare(
            "SELECT workspace_path, asset_id, drift_type, resolved_at \
             FROM drift_history ORDER BY id",
        ) {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        })
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default()
    }

    // -----------------------------------------------------------------------
    // Coverage cache (Req 3.6)
    // -----------------------------------------------------------------------

    /// Load cached per-workspace coverage scores as `(workspace_path, score)`
    /// tuples, ordered by workspace path.
    pub fn load_coverage_scores(&self) -> Vec<(String, f64)> {
        let mut stmt = match self.write_conn.prepare(
            "SELECT workspace_path, coverage_score FROM coverage_cache ORDER BY workspace_path",
        ) {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
        })
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Latest schema version this build knows about, derived from the
    /// migration table so adding a migration never breaks these assertions.
    fn latest_migration_version() -> u32 {
        MIGRATIONS
            .iter()
            .map(|(v, _)| *v)
            .max()
            .expect("at least one migration")
    }

    #[test]
    fn open_in_memory_migrates_to_latest() {
        let mgr = IndexManager::open_in_memory().expect("open_in_memory");
        assert_eq!(
            mgr.schema_version,
            latest_migration_version(),
            "should migrate to the latest known schema version"
        );
    }

    #[test]
    fn open_in_memory_tables_exist() {
        let mgr = IndexManager::open_in_memory().expect("open_in_memory");
        let conn = mgr.write_conn();

        // workspace_scans
        conn.execute(
            "INSERT INTO workspace_scans \
             (workspace_path, workspace_name, last_scan_ts, scan_duration_ms, asset_count, fs_mtime_at_scan) \
             VALUES ('/tmp/ws', 'ws', '2025-01-01T00:00:00Z', 100, 5, 9999)",
            [],
        )
        .expect("insert workspace_scans");

        // audit_log
        conn.execute(
            "INSERT INTO audit_log \
             (timestamp, event_type, subject, details, operator, entry_hash, prev_hash) \
             VALUES ('2025-01-01T00:00:00.000Z', 'policy_evaluation', 'ws', '{}', \
                     'system', 'h1', '')",
            [],
        )
        .expect("insert audit_log");

        // gate_history
        conn.execute(
            "INSERT INTO gate_history \
             (gate_name, status, exit_code, duration_ms, timestamp, catalog_hash) \
             VALUES ('lint', 'pass', 0, 800, '2025-01-01T00:00:00Z', 'ch')",
            [],
        )
        .expect("insert gate_history");
    }

    #[test]
    fn is_scan_stale_no_entry_returns_true() {
        let mgr = IndexManager::open_in_memory().expect("open_in_memory");
        assert!(mgr.is_scan_stale("/nonexistent/ws", 0));
    }

    #[test]
    fn is_scan_stale_old_mtime_returns_false() {
        let mgr = IndexManager::open_in_memory().expect("open_in_memory");
        mgr.write_conn()
            .execute(
                "INSERT INTO workspace_scans \
                 (workspace_path, workspace_name, last_scan_ts, scan_duration_ms, asset_count, fs_mtime_at_scan) \
                 VALUES ('/ws', 'ws', '2025-01-01T00:00:00Z', 100, 0, 1000)",
                [],
            )
            .unwrap();
        // fs_mtime == 1000, same as cached → not stale
        assert!(!mgr.is_scan_stale("/ws", 1000));
        // fs_mtime == 999, older than cached → not stale
        assert!(!mgr.is_scan_stale("/ws", 999));
    }

    #[test]
    fn is_scan_stale_newer_mtime_returns_true() {
        let mgr = IndexManager::open_in_memory().expect("open_in_memory");
        mgr.write_conn()
            .execute(
                "INSERT INTO workspace_scans \
                 (workspace_path, workspace_name, last_scan_ts, scan_duration_ms, asset_count, fs_mtime_at_scan) \
                 VALUES ('/ws', 'ws', '2025-01-01T00:00:00Z', 100, 0, 1000)",
                [],
            )
            .unwrap();
        // fs_mtime == 1001 > 1000 → stale
        assert!(mgr.is_scan_stale("/ws", 1001));
    }

    #[test]
    fn migrate_is_idempotent() {
        let mgr = IndexManager::open_in_memory().expect("open_in_memory");
        // Calling migrate() again should return the same version without error.
        let v2 = mgr.migrate().expect("second migrate");
        assert_eq!(v2, latest_migration_version());
    }

    #[test]
    fn open_file_based_migrates_and_is_queryable() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("test.db");
        let path_str = path.to_str().expect("path utf8");

        let mgr = IndexManager::open(path_str).expect("open file db");
        assert_eq!(mgr.schema_version, latest_migration_version());

        // Read connection should be openable.
        let rconn = mgr.read_connection().expect("read connection");
        let v: String = rconn
            .query_row(
                "SELECT value FROM schema_meta WHERE key = 'schema_version'",
                [],
                |row| row.get(0),
            )
            .expect("query schema_version");
        assert_eq!(v, latest_migration_version().to_string());
    }

    #[test]
    fn load_cached_scan_paths_empty_db() {
        let mgr = IndexManager::open_in_memory().expect("open_in_memory");
        let paths = mgr.load_cached_scan_paths();
        assert!(paths.is_empty());
    }

    #[test]
    fn load_cached_scan_paths_with_entries() {
        let mgr = IndexManager::open_in_memory().expect("open_in_memory");
        for ws in ["/ws/a", "/ws/b"] {
            mgr.write_conn()
                .execute(
                    "INSERT INTO workspace_scans \
                     (workspace_path, workspace_name, last_scan_ts, scan_duration_ms, asset_count, fs_mtime_at_scan) \
                     VALUES (?1, 'ws', '2025-01-01T00:00:00Z', 100, 0, 1000)",
                    rusqlite::params![ws],
                )
                .unwrap();
        }
        let paths = mgr.load_cached_scan_paths();
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&"/ws/a".to_string()));
        assert!(paths.contains(&"/ws/b".to_string()));
    }
}
