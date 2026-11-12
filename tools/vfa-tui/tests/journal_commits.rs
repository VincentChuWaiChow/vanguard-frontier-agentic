//! Real SQLite receipts, rollback failures, and preservation of isolated fixtures.
//! No operator profile, helper execution, replay, or public SQL backdoor is used.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use rusqlite::{params, Connection};
use serde_json::json;
use sha2::{Digest, Sha256};
use tempfile::TempDir;
use vfa_tui::engine::profile::{ProfileError, ProfileOwner, ProfilePaths};
use vfa_tui::persistence::index::IndexManager;
use vfa_tui::persistence::operations::{
    JournalError, OperationEvent, OperationJournal, OperationStatus, MAX_PAYLOAD_BYTES,
};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

const OPERATION: &str = "834e9be9-a269-4e7c-a2f8-b613326d36ee";
const SESSION: &str = "27adc352-b625-4c43-9cc6-75b78b8a6159";
const OTHER_SESSION: &str = "e48c47d4-f4dd-4664-907a-700d455cc36e";
const APPLICATION_ID: i32 = 0x5646_4a31;

fn fixture() -> (TempDir, ProfilePaths, Arc<ProfileOwner>) {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture
        .path()
        .canonicalize()
        .unwrap()
        .join("journal Ω 空 spaces");
    let paths = ProfilePaths::resolve(Some(&root)).unwrap();
    let owner = Arc::new(ProfileOwner::acquire(paths.clone()).unwrap());
    (fixture, paths, owner)
}

fn event(sequence: u64, status: OperationStatus) -> OperationEvent {
    OperationEvent {
        operation_id: OPERATION.into(),
        session_id: SESSION.into(),
        operation_kind: "export".into(),
        event_kind: "observed".into(),
        status,
        sequence,
        observed_at: "2026-11-12T10:11:12Z".into(),
        payload: json!({"caller_redacted": "already redacted", "nested": {"z": 2, "a": 1}}),
    }
}

fn private_file(path: &Path) {
    #[cfg(unix)]
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
    #[cfg(not(unix))]
    assert!(path.is_file());
}

fn hashes(paths: &ProfilePaths) -> BTreeMap<OsString, Vec<u8>> {
    fs::read_dir(paths.root())
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.unwrap();
            // Windows exclusive byte-range locking denies reads through a
            // second handle. Preserve database/sidecar bytes, not lock bytes.
            if entry.path() == paths.lock_file() {
                return None;
            }
            assert!(entry.file_type().unwrap().is_file());
            Some((
                entry.file_name(),
                Sha256::digest(fs::read(entry.path()).unwrap()).to_vec(),
            ))
        })
        .collect()
}

fn count(connection: &Connection, table: &str) -> i64 {
    // Test-only SQL over fixed fixture table names, not a production API.
    assert!(["operations", "operation_events", "commit_children"].contains(&table));
    connection
        .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
            row.get(0)
        })
        .unwrap()
}

fn projection_sequence(connection: &Connection) -> i64 {
    connection
        .query_row(
            "SELECT sequence FROM operations WHERE operation_id = ?1",
            [OPERATION],
            |row| row.get(0),
        )
        .unwrap()
}

#[test]
fn actual_committed_receipts_projection_and_reopen_roundtrip() {
    let (_fixture, paths, owner) = fixture();
    let journal = OperationJournal::open(owner.clone()).unwrap();
    let first = event(1, OperationStatus::Running);
    let receipt = journal.append(first.clone()).unwrap();
    let database = Connection::open(paths.journal()).unwrap();
    let row: (i64, String, i64, String) = database
        .query_row(
            "SELECT id, operation_id, sequence, status FROM operation_events",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap();
    assert_eq!(
        row,
        (receipt.event_id, OPERATION.into(), 1, "RUNNING".into())
    );
    assert_eq!(
        journal.event(OPERATION, 1).unwrap().unwrap().receipt,
        receipt
    );
    let mut canonical_retry = first.clone();
    canonical_retry.observed_at = "2026-11-12T12:11:12+02:00".into();
    canonical_retry.operation_id = OPERATION.to_uppercase();
    canonical_retry.payload = serde_json::from_str(
        "{\"nested\":{\"a\":1,\"z\":2},\"caller_redacted\":\"already redacted\"}",
    )
    .unwrap();
    assert_eq!(journal.append(canonical_retry).unwrap(), receipt);
    let second_receipt = journal
        .append(event(2, OperationStatus::Succeeded))
        .unwrap();
    assert_ne!(receipt.event_id, second_receipt.event_id);
    assert_eq!(count(&database, "operation_events"), 2);
    assert_eq!(count(&database, "operations"), 1);
    assert_eq!(projection_sequence(&database), 2);
    let projection = journal.operation(OPERATION).unwrap().unwrap();
    assert_eq!(projection.status, OperationStatus::Succeeded);
    assert_eq!(projection.last_event_id, second_receipt.event_id);
    assert_eq!(projection.payload, first.payload);
    drop(database);
    journal.close().unwrap();
    let reopened = OperationJournal::open(owner).unwrap();
    assert_eq!(
        reopened.event(OPERATION, 2).unwrap().unwrap().receipt,
        second_receipt
    );
    assert_eq!(
        reopened.append(first).unwrap(),
        receipt,
        "old retries cannot advance/rewind projection"
    );
    assert_eq!(reopened.operation(OPERATION).unwrap().unwrap().sequence, 2);
    reopened.close().unwrap();
}

#[test]
fn duplicate_conflicts_and_sequence_gaps_are_errors_not_receipts() {
    let (_fixture, paths, owner) = fixture();
    let journal = OperationJournal::open(owner).unwrap();
    let first = event(1, OperationStatus::Queued);
    let receipt = journal.append(first.clone()).unwrap();
    let mut variants = Vec::new();
    let mut changed = first.clone();
    changed.status = OperationStatus::Failed;
    variants.push(changed);
    let mut changed = first.clone();
    changed.session_id = OTHER_SESSION.into();
    variants.push(changed);
    let mut changed = first.clone();
    changed.operation_kind = "gate".into();
    variants.push(changed);
    let mut changed = first.clone();
    changed.event_kind = "different".into();
    variants.push(changed);
    let mut changed = first.clone();
    changed.observed_at = "2026-11-12T10:11:13Z".into();
    variants.push(changed);
    let mut changed = first.clone();
    changed.payload = json!({"different": true});
    variants.push(changed);
    for changed in variants {
        assert!(matches!(
            journal.append(changed),
            Err(JournalError::Conflict { sequence: 1, .. })
        ));
    }
    assert!(matches!(
        journal.append(event(3, OperationStatus::Succeeded)),
        Err(JournalError::Sequence {
            expected: 2,
            actual: 3
        })
    ));
    assert_eq!(journal.append(first).unwrap(), receipt);
    let database = Connection::open(paths.journal()).unwrap();
    assert_eq!(count(&database, "operation_events"), 1);
    assert_eq!(projection_sequence(&database), 1);
    journal.close().unwrap();
}

#[test]
fn invalid_dtos_are_rejected_before_any_row_is_written() {
    let (_fixture, paths, owner) = fixture();
    let journal = OperationJournal::open(owner).unwrap();
    let first = event(1, OperationStatus::Queued);
    let mut invalid = Vec::new();
    let mut changed = first.clone();
    changed.operation_id = "not a UUID".into();
    invalid.push(changed);
    let mut changed = first.clone();
    changed.session_id = "00000000-0000-0000-0000-000000000000".into();
    invalid.push(changed);
    let mut changed = first.clone();
    changed.operation_kind = "bad label\n".into();
    invalid.push(changed);
    let mut changed = first.clone();
    changed.event_kind = "a".repeat(65);
    invalid.push(changed);
    let mut changed = first.clone();
    changed.sequence = 0;
    invalid.push(changed);
    let mut changed = first.clone();
    changed.sequence = u64::MAX;
    invalid.push(changed);
    let mut changed = first.clone();
    changed.observed_at = "yesterday".into();
    invalid.push(changed);
    let mut changed = first.clone();
    changed.payload = json!("x".repeat(MAX_PAYLOAD_BYTES));
    invalid.push(changed);
    let mut changed = first.clone();
    for _ in 0..34 {
        changed.payload = json!([changed.payload]);
    }
    invalid.push(changed);
    for changed in invalid {
        assert!(matches!(
            journal.append(changed),
            Err(JournalError::InvalidData(_))
        ));
    }
    let mut unknown = serde_json::to_value(first).unwrap();
    unknown["unknown_required_field"] = json!(true);
    assert!(serde_json::from_value::<OperationEvent>(unknown).is_err());
    let database = Connection::open(paths.journal()).unwrap();
    assert_eq!(count(&database, "operation_events"), 0);
    assert_eq!(count(&database, "operations"), 0);
    journal.close().unwrap();
}

#[test]
fn real_projection_sql_failure_rolls_back_the_inserted_event() {
    let (_fixture, paths, owner) = fixture();
    let journal = OperationJournal::open(owner).unwrap();
    let first = journal.append(event(1, OperationStatus::Running)).unwrap();
    let before = journal.operation(OPERATION).unwrap().unwrap();
    let database = Connection::open(paths.journal()).unwrap();
    database.execute_batch("CREATE TRIGGER fail_projection BEFORE UPDATE ON operations BEGIN SELECT RAISE(ABORT, 'fixture SQL failure'); END;").unwrap();
    let result = journal.append(event(2, OperationStatus::Succeeded));
    assert!(matches!(
        result,
        Err(JournalError::Sqlite {
            stage: "update operation projection",
            ..
        })
    ));
    assert_eq!(count(&database, "operation_events"), 1);
    assert_eq!(projection_sequence(&database), 1);
    assert!(journal.event(OPERATION, 2).unwrap().is_none());
    assert_eq!(journal.operation(OPERATION).unwrap().unwrap(), before);
    assert_eq!(journal.event(OPERATION, 1).unwrap().unwrap().receipt, first);
    database
        .execute_batch("DROP TRIGGER fail_projection;")
        .unwrap();
    let second = journal
        .append(event(2, OperationStatus::Succeeded))
        .unwrap();
    assert_eq!(
        journal.event(OPERATION, 2).unwrap().unwrap().receipt,
        second
    );
    assert_eq!(count(&database, "operation_events"), 2);
    journal.close().unwrap();
}

#[test]
fn actual_deferred_constraint_commit_failure_rolls_back_both_rows() {
    let (_fixture, paths, owner) = fixture();
    let journal = OperationJournal::open(owner).unwrap();
    journal.append(event(1, OperationStatus::Running)).unwrap();
    let before = journal.operation(OPERATION).unwrap().unwrap();
    let database = Connection::open(paths.journal()).unwrap();
    database.execute_batch("
        CREATE TABLE commit_parents (id INTEGER PRIMARY KEY);
        CREATE TABLE commit_children (parent_id INTEGER REFERENCES commit_parents(id) DEFERRABLE INITIALLY DEFERRED);
        CREATE TRIGGER fail_commit AFTER INSERT ON operation_events
        BEGIN INSERT INTO commit_children VALUES (NEW.id); END;
    ").unwrap();
    let result = journal.append(event(2, OperationStatus::Succeeded));
    assert!(
        matches!(
            result,
            Err(JournalError::Sqlite {
                stage: "commit event",
                ..
            })
        ),
        "must fail at actual COMMIT, not at statement execution"
    );
    assert_eq!(count(&database, "operation_events"), 1);
    assert_eq!(count(&database, "commit_children"), 0);
    assert_eq!(projection_sequence(&database), 1);
    assert_eq!(journal.operation(OPERATION).unwrap().unwrap(), before);
    database.execute_batch("DROP TRIGGER fail_commit;").unwrap();
    let second = journal
        .append(event(2, OperationStatus::Succeeded))
        .unwrap();
    assert_eq!(
        journal
            .append(event(2, OperationStatus::Succeeded))
            .unwrap(),
        second
    );
    assert_eq!(count(&database, "operation_events"), 2);
    journal.close().unwrap();
}

#[test]
fn finite_sqlite_busy_errors_do_not_acknowledge_or_fallback() {
    let (_fixture, paths, owner) = fixture();
    let journal = OperationJournal::open(owner.clone()).unwrap();
    let database = Connection::open(paths.journal()).unwrap();
    database.execute_batch("BEGIN IMMEDIATE;").unwrap();
    let start = Instant::now();
    assert!(matches!(
        journal.append(event(1, OperationStatus::Queued)),
        Err(JournalError::Sqlite {
            stage: "begin event",
            ..
        })
    ));
    assert!(start.elapsed() < Duration::from_secs(3));
    assert_eq!(count(&database, "operation_events"), 0);
    assert_eq!(count(&database, "operations"), 0);
    database.execute_batch("ROLLBACK;").unwrap();
    journal.append(event(1, OperationStatus::Queued)).unwrap();
    journal.close().unwrap();
    database.execute_batch("BEGIN EXCLUSIVE;").unwrap();
    let before = hashes(&paths);
    let start = Instant::now();
    assert!(OperationJournal::open(owner.clone()).is_err());
    assert!(start.elapsed() < Duration::from_secs(3));
    assert_eq!(hashes(&paths), before);
    assert_eq!(
        Arc::strong_count(&owner),
        1,
        "failed startup joins/drops its worker owner"
    );
    database.execute_batch("ROLLBACK;").unwrap();
    OperationJournal::open(owner).unwrap().close().unwrap();
}

#[test]
fn writer_retains_owner_until_connection_close_and_join() {
    for explicit_close in [true, false] {
        let (_fixture, paths, owner) = fixture();
        let journal = OperationJournal::open(owner.clone()).unwrap();
        drop(owner);
        assert!(matches!(
            ProfileOwner::acquire(paths.clone()),
            Err(ProfileError::Busy { .. })
        ));
        journal.append(event(1, OperationStatus::Queued)).unwrap();
        if explicit_close {
            journal.close().unwrap();
        } else {
            drop(journal);
        }
        let next_owner = ProfileOwner::acquire(paths.clone()).unwrap();
        let reopened = OperationJournal::open(Arc::new(next_owner)).unwrap();
        assert!(reopened.event(OPERATION, 1).unwrap().is_some());
        reopened.close().unwrap();
    }
}

#[test]
fn populated_legacy_v4_and_live_wal_sidecars_are_byte_preserved() {
    let (_fixture, paths, owner) = fixture();
    let legacy = IndexManager::open(paths.journal().to_str().unwrap()).unwrap();
    assert_eq!(legacy.schema_version, 4);
    let version: String = legacy
        .write_conn()
        .query_row(
            "SELECT value FROM schema_meta WHERE key='schema_version'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(version, "4");
    let application: i32 = legacy
        .write_conn()
        .pragma_query_value(None, "application_id", |row| row.get(0))
        .unwrap();
    assert_eq!(application, 0);
    legacy
        .write_conn()
        .execute(
            "INSERT INTO workspace_scans VALUES (?1, 'fixture', '2026-11-12T00:00:00Z', 1, 3, 42)",
            params![paths.root().to_str().unwrap()],
        )
        .unwrap();
    private_file(paths.journal());
    let before = hashes(&paths);
    assert!(before.contains_key(&OsString::from("operations.sqlite-wal")));
    assert!(before.contains_key(&OsString::from("operations.sqlite-shm")));
    assert!(matches!(
        OperationJournal::open(owner.clone()),
        Err(JournalError::ForeignStore { application: 0 })
    ));
    assert_eq!(
        hashes(&paths),
        before,
        "refusal cannot change DB or any live sidecar bytes"
    );
    assert_eq!(Arc::strong_count(&owner), 1);
    let rows: i64 = legacy
        .write_conn()
        .query_row("SELECT COUNT(*) FROM workspace_scans", [], |row| row.get(0))
        .unwrap();
    assert_eq!(rows, 1);
}

#[test]
fn foreign_corrupt_newer_and_recognized_wal_stores_are_not_relabelled() {
    // Foreign identity, no journal schema.
    let (_fixture, paths, owner) = fixture();
    let database = Connection::open(paths.journal()).unwrap();
    database.execute_batch("PRAGMA application_id=123; CREATE TABLE foreign_data (value TEXT); INSERT INTO foreign_data VALUES ('keep');").unwrap();
    drop(database);
    private_file(paths.journal());
    let before = hashes(&paths);
    assert!(matches!(
        OperationJournal::open(owner),
        Err(JournalError::ForeignStore { application: 123 })
    ));
    assert_eq!(hashes(&paths), before);

    // Corrupt bytes, not a missing optional fixture.
    let (_fixture, paths, owner) = fixture();
    fs::write(
        paths.journal(),
        b"not a SQLite database; preserve every byte",
    )
    .unwrap();
    private_file(paths.journal());
    let before = hashes(&paths);
    assert!(matches!(
        OperationJournal::open(owner),
        Err(JournalError::InvalidStore(_))
    ));
    assert_eq!(hashes(&paths), before);

    // Newer recognized identity, with real WAL/SHM files present.
    let (_fixture, paths, owner) = fixture();
    OperationJournal::open(owner.clone())
        .unwrap()
        .close()
        .unwrap();
    let database = Connection::open(paths.journal()).unwrap();
    database.execute_batch("PRAGMA user_version=3; UPDATE journal_meta SET schema_version=3; PRAGMA journal_mode=WAL; CREATE TABLE keep_wal (id INTEGER);").unwrap();
    let before = hashes(&paths);
    assert!(before.contains_key(&OsString::from("operations.sqlite-wal")));
    assert!(matches!(
        OperationJournal::open(owner),
        Err(JournalError::NewerSchema { version: 3 })
    ));
    assert_eq!(hashes(&paths), before);
    drop(database);

    // Even a recognized/current WAL store is explicitly unsupported, not reset.
    let (_fixture, paths, owner) = fixture();
    OperationJournal::open(owner.clone())
        .unwrap()
        .close()
        .unwrap();
    let database = Connection::open(paths.journal()).unwrap();
    database
        .execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE keep_wal (id INTEGER);")
        .unwrap();
    let before = hashes(&paths);
    assert!(matches!(
        OperationJournal::open(owner),
        Err(JournalError::InvalidStore(_))
    ));
    assert_eq!(hashes(&paths), before);
}

#[test]
fn recognized_but_wrong_schema_or_metadata_is_refused_read_only() {
    for mutation in [
        "UPDATE journal_meta SET application='foreign.application';",
        "DROP TABLE operations;",
    ] {
        let (_fixture, paths, owner) = fixture();
        OperationJournal::open(owner.clone())
            .unwrap()
            .close()
            .unwrap();
        let database = Connection::open(paths.journal()).unwrap();
        let id: i32 = database
            .pragma_query_value(None, "application_id", |row| row.get(0))
            .unwrap();
        assert_eq!(id, APPLICATION_ID);
        database.execute_batch(mutation).unwrap();
        drop(database);
        let before = hashes(&paths);
        assert!(matches!(
            OperationJournal::open(owner),
            Err(JournalError::InvalidStore(_))
        ));
        assert_eq!(hashes(&paths), before);
    }
}

#[test]
fn forward_index_migration_preserves_events_and_failed_migration_rolls_back() {
    let (_fixture, paths, owner) = fixture();
    let journal = OperationJournal::open(owner.clone()).unwrap();
    let receipt = journal.append(event(1, OperationStatus::Running)).unwrap();
    journal.close().unwrap();
    let database = Connection::open(paths.journal()).unwrap();
    database
        .execute_batch(
            "
        DROP INDEX operation_events_session;
        UPDATE journal_meta SET schema_version=1;
        PRAGMA user_version=1;
        CREATE TRIGGER fail_migration BEFORE UPDATE ON journal_meta
        BEGIN SELECT RAISE(ABORT, 'fixture migration failure'); END;
    ",
        )
        .unwrap();
    drop(database);
    let before = hashes(&paths);
    assert!(matches!(
        OperationJournal::open(owner.clone()),
        Err(JournalError::Sqlite {
            stage: "advance journal schema",
            ..
        })
    ));
    assert_eq!(
        hashes(&paths),
        before,
        "DDL and version update rollback together"
    );
    let database = Connection::open(paths.journal()).unwrap();
    let version: i32 = database
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    assert_eq!(version, 1);
    let indexes: i64 = database
        .query_row(
            "SELECT COUNT(*) FROM sqlite_schema WHERE name='operation_events_session'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(indexes, 0);
    assert_eq!(count(&database, "operation_events"), 1);
    database
        .execute_batch("DROP TRIGGER fail_migration;")
        .unwrap();
    drop(database);
    let migrated = OperationJournal::open(owner).unwrap();
    assert_eq!(
        migrated.event(OPERATION, 1).unwrap().unwrap().receipt,
        receipt
    );
    assert_eq!(
        migrated.append(event(1, OperationStatus::Running)).unwrap(),
        receipt
    );
    let database = Connection::open(paths.journal()).unwrap();
    let version: i32 = database
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    assert_eq!(version, 2);
    let indexes: i64 = database
        .query_row(
            "SELECT COUNT(*) FROM sqlite_schema WHERE name='operation_events_session'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(indexes, 1);
    assert_eq!(count(&database, "operation_events"), 1);
    migrated.close().unwrap();
}

#[test]
fn unknown_persisted_state_is_an_error_not_a_success_or_empty_result() {
    let (_fixture, paths, owner) = fixture();
    let journal = OperationJournal::open(owner).unwrap();
    journal.append(event(1, OperationStatus::Running)).unwrap();
    let database = Connection::open(paths.journal()).unwrap();
    database.execute_batch("PRAGMA ignore_check_constraints=ON; UPDATE operations SET status='UNKNOWN_FUTURE_STATE';").unwrap();
    assert!(matches!(
        journal.operation(OPERATION),
        Err(JournalError::InvalidData(_))
    ));
    assert!(matches!(
        journal.append(event(2, OperationStatus::Succeeded)),
        Err(JournalError::InvalidData(_))
    ));
    assert_eq!(count(&database, "operation_events"), 1);
    assert!(journal.event(OPERATION, 2).unwrap().is_none());
    journal.close().unwrap();
}

#[test]
fn inconsistent_projection_and_retry_are_rejected_without_repair() {
    for mutation in [
        "UPDATE operations SET status='SUCCEEDED' WHERE operation_id=?1",
        "UPDATE operations SET last_event_id=1 WHERE operation_id=?1",
        "UPDATE operations SET last_event_id=3 WHERE operation_id=?1",
        "UPDATE operations SET last_event_id=99999 WHERE operation_id=?1",
        "DELETE FROM operation_events WHERE operation_id=?1 AND sequence=2",
        "DELETE FROM operations WHERE operation_id=?1",
        "UPDATE operations SET payload='{}' WHERE operation_id=?1",
        "UPDATE operations SET sequence=1,last_event_id=1 WHERE operation_id=?1",
    ] {
        let (_fixture, paths, owner) = fixture();
        let journal = OperationJournal::open(owner).unwrap();
        let first = event(1, OperationStatus::Running);
        assert_eq!(journal.append(first.clone()).unwrap().event_id, 1);
        assert_eq!(
            journal
                .append(event(2, OperationStatus::Running))
                .unwrap()
                .event_id,
            2
        );
        let mut other = first.clone();
        other.operation_id = OTHER_SESSION.into();
        assert_eq!(journal.append(other).unwrap().event_id, 3);
        let database = Connection::open(paths.journal()).unwrap();
        database.pragma_update(None, "foreign_keys", false).unwrap();
        database.execute(mutation, [OPERATION]).unwrap();
        let before = hashes(&paths);
        assert!(
            matches!(
                journal.operation(OPERATION),
                Err(JournalError::InvalidStore(_))
            ),
            "{mutation}"
        );
        assert!(
            matches!(journal.append(first), Err(JournalError::InvalidStore(_))),
            "retry: {mutation}"
        );
        assert!(
            matches!(
                journal.append(event(3, OperationStatus::Succeeded)),
                Err(JournalError::InvalidStore(_))
            ),
            "advance: {mutation}"
        );
        assert_eq!(hashes(&paths), before, "never silently repair: {mutation}");
        journal.close().unwrap();
    }
}
