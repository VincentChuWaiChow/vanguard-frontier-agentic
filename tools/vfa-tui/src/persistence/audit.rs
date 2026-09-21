//! Audit logger with SHA-256 hash chain for tamper detection.
//!
//! Implements Requirement 14 (all 8 criteria):
//! - Append-only SQLite table (14.1, 14.2)
//! - JSON / CSV export (14.4)
//! - Console + schema version in meta (14.6)
//! - SHA-256 hash chain (14.8): entry_hash = SHA256(prev_hash || timestamp ||
//!   event_type || subject || details_json)

use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::Path;

use chrono::Utc;
use sha2::{Digest, Sha256};

use crate::error::TuiError;
use crate::models::audit::{AuditEntry, AuditEventType};
use crate::persistence::index::IndexManager;

// ---------------------------------------------------------------------------
// AuditLogger
// ---------------------------------------------------------------------------

/// Append-only audit logger backed by an in-process `IndexManager`.
///
/// Maintains `last_hash` so each call to [`AuditLogger::log`] automatically
/// threads the chain without additional DB round-trips.
/// Hash-recipe marker stored in `audit_log.hash_version`.
///
/// Rows predating schema version 5 carry `1` (operator not hashed); rows this
/// build writes carry `2`, binding the operator into the chain.
pub const HASH_VERSION_WITH_OPERATOR: i64 = 2;

pub struct AuditLogger<'a> {
    mgr: &'a IndexManager,
    /// Hash of the most-recently appended entry, or `""` for the first entry.
    last_hash: String,
}

impl<'a> AuditLogger<'a> {
    // -----------------------------------------------------------------------
    // Constructors
    // -----------------------------------------------------------------------

    /// Create a logger that will append to the database owned by `mgr`.
    ///
    /// `last_hash` should be the `entry_hash` of the last row already in the
    /// table (pass `""` for a fresh database).
    pub fn new(mgr: &'a IndexManager, last_hash: String) -> Self {
        Self { mgr, last_hash }
    }

    /// Initialise from a manager: reads the last `entry_hash` from the table
    /// so the chain is correct even if entries already exist.
    pub fn from_manager(mgr: &'a IndexManager) -> Result<Self, TuiError> {
        let last_hash: String = mgr
            .write_conn()
            .query_row(
                "SELECT entry_hash FROM audit_log ORDER BY id DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or_default();
        Ok(Self { mgr, last_hash })
    }

    // -----------------------------------------------------------------------
    // Logging
    // -----------------------------------------------------------------------

    /// Append one entry to the audit log.
    ///
    /// Computes the hash chain link automatically and stores both
    /// `entry_hash` and `prev_hash` in the database row.
    /// Returns the fully-populated [`AuditEntry`] (id assigned by SQLite).
    pub fn log(
        &mut self,
        event_type: AuditEventType,
        subject: &str,
        details: serde_json::Value,
        operator: &str,
    ) -> Result<AuditEntry, TuiError> {
        let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
        let details_json = serde_json::to_string(&details).unwrap_or_else(|_| "{}".to_string());

        let entry_hash = Self::compute_hash_v2(
            &self.last_hash,
            &timestamp,
            &event_type,
            subject,
            &details_json,
            operator,
        );

        let event_type_str = serde_json::to_string(&event_type)
            .unwrap_or_default()
            .trim_matches('"')
            .to_string();

        let conn = self.mgr.write_conn();
        conn.execute(
            "INSERT INTO audit_log \
             (timestamp, event_type, subject, details, operator, entry_hash, prev_hash, \
              hash_version) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                timestamp,
                event_type_str,
                subject,
                details_json,
                operator,
                entry_hash,
                self.last_hash,
                HASH_VERSION_WITH_OPERATOR,
            ],
        )?;

        let id = conn.last_insert_rowid();
        let prev_hash = self.last_hash.clone();
        self.last_hash = entry_hash.clone();

        Ok(AuditEntry {
            id,
            timestamp,
            event_type,
            subject: subject.to_string(),
            details,
            operator: operator.to_string(),
            entry_hash,
            prev_hash,
        })
    }

    // -----------------------------------------------------------------------
    // Hash chain
    // -----------------------------------------------------------------------

    /// Compute the **v1 (legacy)** entry hash:
    /// `SHA256(prev_hash || timestamp || event_type_str || subject || details_json)`.
    ///
    /// This recipe leaves `operator` unauthenticated.  It is retained only so
    /// rows written before schema version 5 keep verifying; new entries use
    /// [`compute_hash_v2`](Self::compute_hash_v2).
    pub fn compute_hash(
        prev_hash: &str,
        timestamp: &str,
        event_type: &AuditEventType,
        subject: &str,
        details_json: &str,
    ) -> String {
        let event_str = serde_json::to_string(event_type)
            .unwrap_or_default()
            .trim_matches('"')
            .to_string();

        let mut hasher = Sha256::new();
        hasher.update(prev_hash.as_bytes());
        hasher.update(timestamp.as_bytes());
        hasher.update(event_str.as_bytes());
        hasher.update(subject.as_bytes());
        hasher.update(details_json.as_bytes());

        let result = hasher.finalize();
        result.iter().fold(String::new(), |mut s, b| {
            let _ = write!(s, "{b:02x}");
            s
        })
    }

    /// Compute the **v2** entry hash, binding the acting operator into the
    /// chain: `SHA256(prev || timestamp || event_type || subject || details || operator)`.
    ///
    /// Without the trailing operator field the stored actor on an entry could be
    /// rewritten without breaking the chain, so the chain could not attest *who*
    /// performed a recorded action.
    pub fn compute_hash_v2(
        prev_hash: &str,
        timestamp: &str,
        event_type: &AuditEventType,
        subject: &str,
        details_json: &str,
        operator: &str,
    ) -> String {
        let event_str = serde_json::to_string(event_type)
            .unwrap_or_default()
            .trim_matches('"')
            .to_string();

        let mut hasher = Sha256::new();
        hasher.update(prev_hash.as_bytes());
        hasher.update(timestamp.as_bytes());
        hasher.update(event_str.as_bytes());
        hasher.update(subject.as_bytes());
        hasher.update(details_json.as_bytes());
        hasher.update(operator.as_bytes());

        let result = hasher.finalize();
        result.iter().fold(String::new(), |mut s, b| {
            let _ = write!(s, "{b:02x}");
            s
        })
    }

    // -----------------------------------------------------------------------
    // Chain verification (Req 14.8)
    // -----------------------------------------------------------------------

    /// Walk every entry in insertion order and recompute each `entry_hash`.
    ///
    /// Returns `Ok(())` if the chain is intact, or
    /// `Err(TuiError::AuditChainBroken { entry_id })` at the first mismatch.
    pub fn verify_chain(&self) -> Result<(), TuiError> {
        let conn = self.mgr.write_conn();
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, event_type, subject, details, operator, \
                    entry_hash, prev_hash, hash_version \
             FROM audit_log ORDER BY id ASC",
        )?;

        let mut expected_prev = String::new();

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,    // id
                row.get::<_, String>(1)?, // timestamp
                row.get::<_, String>(2)?, // event_type
                row.get::<_, String>(3)?, // subject
                row.get::<_, String>(4)?, // details
                row.get::<_, String>(5)?, // operator
                row.get::<_, String>(6)?, // entry_hash
                row.get::<_, String>(7)?, // prev_hash
                row.get::<_, i64>(8)?,    // hash_version
            ))
        })?;

        for row_result in rows {
            let (
                id,
                timestamp,
                event_type_str,
                subject,
                details_json,
                operator,
                stored_hash,
                stored_prev,
                hash_version,
            ) = row_result?;

            // prev_hash in the row must match what we expect.
            if stored_prev != expected_prev {
                return Err(TuiError::AuditChainBroken { entry_id: id });
            }

            // Recompute entry_hash from stored fields.
            let event_type: AuditEventType = serde_json::from_str(&format!("\"{event_type_str}\""))
                .map_err(|_| TuiError::AuditChainBroken { entry_id: id })?;

            // Dispatch on the recipe that produced the row so pre-migration
            // entries keep verifying while new ones authenticate the operator.
            let recomputed = if hash_version >= HASH_VERSION_WITH_OPERATOR {
                Self::compute_hash_v2(
                    &expected_prev,
                    &timestamp,
                    &event_type,
                    &subject,
                    &details_json,
                    &operator,
                )
            } else {
                Self::compute_hash(
                    &expected_prev,
                    &timestamp,
                    &event_type,
                    &subject,
                    &details_json,
                )
            };

            if recomputed != stored_hash {
                return Err(TuiError::AuditChainBroken { entry_id: id });
            }

            expected_prev = stored_hash;
        }

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Export (Req 14.4)
    // -----------------------------------------------------------------------

    /// Export the audit log to `out_path` in `format` (`"json"` or `"csv"`).
    pub fn export_audit(&self, format: &str, out_path: &Path) -> Result<(), TuiError> {
        let conn = self.mgr.write_conn();
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, event_type, subject, details, operator, \
                    entry_hash, prev_hash \
             FROM audit_log ORDER BY id ASC",
        )?;

        match format {
            "json" => {
                let rows: Vec<serde_json::Value> = stmt
                    .query_map([], |row| {
                        let details_str: String = row.get(4)?;
                        Ok(serde_json::json!({
                            "id":         row.get::<_, i64>(0)?,
                            "timestamp":  row.get::<_, String>(1)?,
                            "event_type": row.get::<_, String>(2)?,
                            "subject":    row.get::<_, String>(3)?,
                            "details":    details_str,
                            "operator":   row.get::<_, String>(5)?,
                            "entry_hash": row.get::<_, String>(6)?,
                            "prev_hash":  row.get::<_, String>(7)?,
                        }))
                    })?
                    .filter_map(|r| r.ok())
                    .collect();

                let json = serde_json::to_string_pretty(&rows).map_err(|e| {
                    TuiError::PersistenceQuery {
                        detail: e.to_string(),
                    }
                })?;
                fs::write(out_path, json).map_err(|e| TuiError::LogDestination {
                    path: out_path.display().to_string(),
                    reason: e.to_string(),
                })
            }

            "csv" => {
                let mut csv = String::from(
                    "id,timestamp,event_type,subject,details,operator,entry_hash,prev_hash\n",
                );
                let rows: Vec<_> = stmt
                    .query_map([], |row| {
                        Ok((
                            row.get::<_, i64>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, String>(3)?,
                            row.get::<_, String>(4)?,
                            row.get::<_, String>(5)?,
                            row.get::<_, String>(6)?,
                            row.get::<_, String>(7)?,
                        ))
                    })?
                    .filter_map(|r| r.ok())
                    .collect();

                for (id, ts, et, subj, det, op, eh, ph) in rows {
                    let _ = writeln!(
                        csv,
                        "{id},{ts},{et},{},{},{},{eh},{ph}",
                        escape_csv(&subj),
                        escape_csv(&det),
                        escape_csv(&op),
                    );
                }

                fs::write(out_path, csv).map_err(|e| TuiError::LogDestination {
                    path: out_path.display().to_string(),
                    reason: e.to_string(),
                })
            }

            other => Err(TuiError::ConfigInvalid {
                flag: "--export-audit".to_string(),
                detail: format!("unknown format '{other}'; expected 'json' or 'csv'"),
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Minimal CSV field escape: wrap in double-quotes if the value contains
/// a comma, double-quote, or newline.
fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

// ---------------------------------------------------------------------------
// Record console version in meta (Req 14.6)
// ---------------------------------------------------------------------------

/// Store the Console version and schema version in `schema_meta`.
///
/// Called once during startup so audit exports can include version context.
pub fn record_version_meta(mgr: &IndexManager, console_version: &str) -> Result<(), TuiError> {
    let conn = mgr.write_conn();
    conn.execute(
        "INSERT OR REPLACE INTO schema_meta (key, value) VALUES ('console_version', ?1)",
        rusqlite::params![console_version],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO schema_meta (key, value) \
         VALUES ('schema_version', ?1)",
        rusqlite::params![mgr.schema_version.to_string()],
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::audit::AuditEventType;
    use crate::persistence::index::IndexManager;

    fn fresh_logger() -> (IndexManager,) {
        let mgr = IndexManager::open_in_memory().expect("open_in_memory");
        (mgr,)
    }

    // -----------------------------------------------------------------------
    // Basic logging
    // -----------------------------------------------------------------------

    #[test]
    fn log_first_entry_has_empty_prev_hash() {
        let (mgr,) = fresh_logger();
        let mut logger = AuditLogger::new(&mgr, String::new());
        let entry = logger
            .log(
                AuditEventType::PolicyEvaluation,
                "ws-a",
                serde_json::json!({"passed": true}),
                "system",
            )
            .expect("log");
        assert_eq!(entry.prev_hash, "");
        assert!(!entry.entry_hash.is_empty());
        assert_eq!(entry.id, 1);
    }

    #[test]
    fn log_second_entry_chains_to_first() {
        let (mgr,) = fresh_logger();
        let mut logger = AuditLogger::new(&mgr, String::new());

        let e1 = logger
            .log(
                AuditEventType::Promotion,
                "asset-1",
                serde_json::json!({}),
                "system",
            )
            .expect("log e1");
        let e2 = logger
            .log(
                AuditEventType::GateExecution,
                "gate-lint",
                serde_json::json!({}),
                "system",
            )
            .expect("log e2");

        assert_eq!(e2.prev_hash, e1.entry_hash, "chain must link");
    }

    // -----------------------------------------------------------------------
    // Append-only enforcement (Req 14.2)
    // -----------------------------------------------------------------------

    #[test]
    fn update_on_audit_log_rejected() {
        let mgr = IndexManager::open_in_memory().expect("open_in_memory");
        let mut logger = AuditLogger::new(&mgr, String::new());
        logger
            .log(
                AuditEventType::DriftDetected,
                "ws",
                serde_json::json!({}),
                "system",
            )
            .expect("log");

        let result = mgr
            .write_conn()
            .execute("UPDATE audit_log SET subject = 'tampered' WHERE id = 1", []);
        assert!(result.is_err(), "UPDATE should be rejected by trigger");
    }

    #[test]
    fn delete_on_audit_log_rejected() {
        let mgr = IndexManager::open_in_memory().expect("open_in_memory");
        let mut logger = AuditLogger::new(&mgr, String::new());
        logger
            .log(
                AuditEventType::OperatorAction,
                "ws",
                serde_json::json!({}),
                "operator",
            )
            .expect("log");

        let result = mgr
            .write_conn()
            .execute("DELETE FROM audit_log WHERE id = 1", []);
        assert!(result.is_err(), "DELETE should be rejected by trigger");
    }

    // -----------------------------------------------------------------------
    // Hash chain verification
    // -----------------------------------------------------------------------

    #[test]
    fn verify_chain_empty_db_ok() {
        let (mgr,) = fresh_logger();
        let logger = AuditLogger::new(&mgr, String::new());
        logger.verify_chain().expect("empty chain should be valid");
    }

    #[test]
    fn verify_chain_single_entry_ok() {
        let (mgr,) = fresh_logger();
        let mut logger = AuditLogger::new(&mgr, String::new());
        logger
            .log(
                AuditEventType::ConfigChange,
                "cfg",
                serde_json::json!({}),
                "system",
            )
            .expect("log");
        logger.verify_chain().expect("single-entry chain valid");
    }

    #[test]
    fn verify_chain_multiple_entries_ok() {
        let (mgr,) = fresh_logger();
        let mut logger = AuditLogger::new(&mgr, String::new());
        for i in 0..5 {
            logger
                .log(
                    AuditEventType::GateExecution,
                    &format!("gate-{i}"),
                    serde_json::json!({"i": i}),
                    "system",
                )
                .expect("log");
        }
        logger.verify_chain().expect("multi-entry chain valid");
    }

    #[test]
    fn verify_chain_detects_tamper() {
        // Build a two-entry chain correctly via the logger so both entries
        // have real, consistent hashes.
        let mgr = IndexManager::open_in_memory().expect("open_in_memory");
        let mut logger = AuditLogger::new(&mgr, String::new());
        let _e1 = logger
            .log(
                AuditEventType::PolicyEvaluation,
                "ws-0",
                serde_json::json!({}),
                "system",
            )
            .expect("log e1");
        let _e2 = logger
            .log(
                AuditEventType::PolicyEvaluation,
                "ws-1",
                serde_json::json!({}),
                "system",
            )
            .expect("log e2");

        // Append a third entry whose prev_hash is intentionally wrong.
        // Since UPDATE/DELETE are blocked by the trigger, we INSERT the broken
        // row directly — INSERT is not restricted.
        mgr.write_conn()
            .execute(
                "INSERT INTO audit_log \
                 (timestamp, event_type, subject, details, operator, entry_hash, prev_hash) \
                 VALUES ('2025-01-01T00:00:02.000Z', 'policy_evaluation', 'ws-2', '{}', \
                         'system', 'any_hash', 'WRONG_PREV_HASH_INTENTIONALLY')",
                [],
            )
            .expect("insert broken entry");

        // verify_chain must detect the break at entry id 3.
        let err = logger
            .verify_chain()
            .expect_err("should detect chain break");
        match err {
            TuiError::AuditChainBroken { entry_id } => {
                assert_eq!(entry_id, 3, "should flag entry 3 (the tampered row)");
            }
            other => panic!("expected AuditChainBroken, got {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // compute_hash determinism
    // -----------------------------------------------------------------------

    #[test]
    fn compute_hash_is_deterministic() {
        let h1 = AuditLogger::compute_hash(
            "",
            "2025-01-01T00:00:00.000Z",
            &AuditEventType::PolicyEvaluation,
            "ws",
            "{}",
        );
        let h2 = AuditLogger::compute_hash(
            "",
            "2025-01-01T00:00:00.000Z",
            &AuditEventType::PolicyEvaluation,
            "ws",
            "{}",
        );
        assert_eq!(h1, h2, "same inputs must yield same hash");
    }

    #[test]
    fn compute_hash_changes_with_prev_hash() {
        let h1 = AuditLogger::compute_hash("prev1", "ts", &AuditEventType::Promotion, "s", "{}");
        let h2 = AuditLogger::compute_hash("prev2", "ts", &AuditEventType::Promotion, "s", "{}");
        assert_ne!(h1, h2);
    }

    // -----------------------------------------------------------------------
    // Export
    // -----------------------------------------------------------------------

    #[test]
    fn export_json_creates_file() {
        let (mgr,) = fresh_logger();
        let mut logger = AuditLogger::new(&mgr, String::new());
        logger
            .log(
                AuditEventType::Promotion,
                "asset-x",
                serde_json::json!({}),
                "headless",
            )
            .expect("log");

        let dir = tempfile::tempdir().expect("tempdir");
        let out = dir.path().join("audit.json");
        logger.export_audit("json", &out).expect("export json");

        let content = fs::read_to_string(&out).expect("read file");
        assert!(
            content.contains("promotion"),
            "JSON should contain event type"
        );
        assert!(content.contains("asset-x"), "JSON should contain subject");
    }

    #[test]
    fn export_csv_creates_file() {
        let (mgr,) = fresh_logger();
        let mut logger = AuditLogger::new(&mgr, String::new());
        logger
            .log(
                AuditEventType::DriftDetected,
                "ws-csv",
                serde_json::json!({}),
                "system",
            )
            .expect("log");

        let dir = tempfile::tempdir().expect("tempdir");
        let out = dir.path().join("audit.csv");
        logger.export_audit("csv", &out).expect("export csv");

        let content = fs::read_to_string(&out).expect("read file");
        assert!(content.starts_with("id,timestamp"), "CSV header missing");
        assert!(content.contains("drift_detected"), "CSV missing event_type");
        assert!(content.contains("ws-csv"), "CSV missing subject");
    }

    #[test]
    fn export_unknown_format_returns_error() {
        let (mgr,) = fresh_logger();
        let logger = AuditLogger::new(&mgr, String::new());
        let dir = tempfile::tempdir().expect("tempdir");
        let out = dir.path().join("audit.xml");
        let err = logger.export_audit("xml", &out).expect_err("should fail");
        match err {
            TuiError::ConfigInvalid { .. } => {}
            other => panic!("expected ConfigInvalid, got {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // from_manager reads existing last_hash
    // -----------------------------------------------------------------------

    #[test]
    fn from_manager_reads_last_hash() {
        let (mgr,) = fresh_logger();
        let mut logger = AuditLogger::new(&mgr, String::new());
        let e1 = logger
            .log(
                AuditEventType::PolicyEvaluation,
                "ws",
                serde_json::json!({}),
                "system",
            )
            .expect("log");

        // Re-create logger from manager — should pick up e1's hash.
        let mut logger2 = AuditLogger::from_manager(&mgr).expect("from_manager");
        let e2 = logger2
            .log(
                AuditEventType::GateExecution,
                "gate",
                serde_json::json!({}),
                "system",
            )
            .expect("log e2");
        assert_eq!(e2.prev_hash, e1.entry_hash);
    }

    // -----------------------------------------------------------------------
    // record_version_meta
    // -----------------------------------------------------------------------

    #[test]
    fn record_version_meta_stores_versions() {
        let (mgr,) = fresh_logger();
        record_version_meta(&mgr, "0.2.0").expect("record_version_meta");

        let v: String = mgr
            .write_conn()
            .query_row(
                "SELECT value FROM schema_meta WHERE key = 'console_version'",
                [],
                |row| row.get(0),
            )
            .expect("query console_version");
        assert_eq!(v, "0.2.0");
    }
}
