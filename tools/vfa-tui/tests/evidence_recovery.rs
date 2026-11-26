//! Real temporary SQLite admission evidence; inert probes only.
use rusqlite::Connection;
use std::sync::Arc;
use tempfile::TempDir;
use vfa_tui::engine::evidence::EvidenceController;
use vfa_tui::engine::profile::{ProfileOwner, ProfilePaths};
use vfa_tui::persistence::operations::OperationJournal;
const OP: &str = "834e9be9-a269-4e7c-a2f8-b613326d36ee";
const SESSION: &str = "27adc352-b625-4c43-9cc6-75b78b8a6159";
const AT: &str = "2026-11-26T10:00:00Z";
fn fixture() -> (TempDir, ProfilePaths, OperationJournal) {
    let temp = tempfile::tempdir().unwrap();
    let paths = ProfilePaths::resolve(Some(&temp.path().join("profile"))).unwrap();
    let owner = Arc::new(ProfileOwner::acquire(paths.clone()).unwrap());
    let journal = OperationJournal::open(owner).unwrap();
    (temp, paths, journal)
}
#[test]
fn start_sql_failure_never_invokes_probe() {
    let (_temp, paths, journal) = fixture();
    let db = Connection::open(paths.journal()).unwrap();
    db.execute_batch("CREATE TRIGGER fail_start BEFORE INSERT ON operation_events BEGIN SELECT RAISE(ABORT, 'fixture'); END;").unwrap();
    let mut controller = EvidenceController::new(journal, SESSION).unwrap();
    assert!(controller.recover_step(AT).unwrap());
    let mut calls = 0;
    let result = controller.run_probe(OP, AT, || {
        calls += 1;
        true
    });
    assert_eq!(calls, 0, "actual SQL failure must prevent action");
    assert!(result.is_err());
}

use serde_json::json;
use vfa_tui::engine::evidence::{
    AuditStatus, RetryClass, MAX_EVIDENCE_RETRIES, MAX_PENDING_EVENTS,
};
use vfa_tui::persistence::operations::{
    JournalError, OperationEvent, OperationStatus, RECOVERY_PAGE_SIZE,
};

fn ready(journal: OperationJournal) -> EvidenceController {
    let mut controller = EvidenceController::new(journal, SESSION).unwrap();
    while !controller.recover_step(AT).unwrap() {}
    assert!(controller.admission_ready());
    controller
}
fn event(id: &str, status: OperationStatus) -> OperationEvent {
    OperationEvent {
        operation_id: id.into(),
        session_id: SESSION.into(),
        operation_kind: "fixture".into(),
        event_kind: "fixture.observed".into(),
        status,
        sequence: 1,
        observed_at: AT.into(),
        payload: json!({"preserved":true}),
    }
}
fn rows(db: &Connection) -> i64 {
    db.query_row("SELECT COUNT(*) FROM operation_events", [], |r| r.get(0))
        .unwrap()
}
fn deferred_failure(db: &Connection) {
    db.execute_batch("CREATE TABLE parents(id INTEGER PRIMARY KEY); CREATE TABLE children(id INTEGER REFERENCES parents(id) DEFERRABLE INITIALLY DEFERRED); CREATE TRIGGER fail_commit AFTER INSERT ON operation_events BEGIN INSERT INTO children VALUES(NEW.id); END;").unwrap();
}
#[test]
fn start_commit_failure_never_invokes_probe_or_returns_receipt() {
    let (_temp, paths, journal) = fixture();
    let db = Connection::open(paths.journal()).unwrap();
    deferred_failure(&db);
    let mut controller = ready(journal);
    let mut calls = 0;
    assert!(matches!(
        controller.run_probe(OP, AT, || {
            calls += 1;
            true
        }),
        Err(JournalError::Sqlite {
            stage: "commit event",
            ..
        })
    ));
    assert_eq!(calls, 0);
    assert_eq!(rows(&db), 0);
    assert!(!controller.admission_ready());
}
#[test]
fn committed_start_visible_before_probe_and_duplicate_id_never_replays() {
    let (_temp, paths, journal) = fixture();
    let db = Connection::open(paths.journal()).unwrap();
    let mut controller = ready(journal);
    let mut calls = 0;
    let observed = controller
        .run_probe(OP, AT, || {
            calls += 1;
            assert_eq!(rows(&db), 1);
            true
        })
        .unwrap();
    assert!(observed.succeeded);
    assert!(matches!(observed.audit, AuditStatus::Committed(_)));
    assert_eq!(calls, 1);
    assert!(controller
        .run_probe(OP, AT, || {
            calls += 1;
            true
        })
        .is_err());
    assert_eq!(calls, 1);
    assert_eq!(rows(&db), 2);
}
#[test]
fn failed_callback_result_is_not_success() {
    let (_temp, paths, journal) = fixture();
    let mut controller = ready(journal);
    let observed = controller.run_probe(OP, AT, || false).unwrap();
    assert!(!observed.succeeded);
    let db = Connection::open(paths.journal()).unwrap();
    let status: String = db
        .query_row("SELECT status FROM operations", [], |r| r.get(0))
        .unwrap();
    assert_eq!(status, "FAILED");
}
#[test]
fn success_and_failed_audit_are_independent_retry_is_identical_and_action_free() {
    let (_temp, paths, journal) = fixture();
    let db = Connection::open(paths.journal()).unwrap();
    let mut controller = ready(journal);
    let mut calls = 0;
    let observed = controller
        .run_probe(OP, AT, || {
            calls += 1;
            db.execute_batch("BEGIN IMMEDIATE;").unwrap();
            true
        })
        .unwrap();
    assert!(observed.succeeded);
    assert_eq!(observed.audit, AuditStatus::Failed(RetryClass::Transient));
    assert!(!controller.admission_ready());
    assert_eq!(MAX_PENDING_EVENTS, 1);
    let retained = controller.pending().unwrap().event.clone();
    assert!(controller
        .run_probe("00000000-0000-0000-0000-000000000001", AT, || {
            calls += 1;
            true
        })
        .is_err());
    db.execute_batch("ROLLBACK;").unwrap();
    let receipt = controller.retry_pending().unwrap();
    assert_eq!(calls, 1);
    assert!(controller.pending().is_none());
    assert!(controller.admission_ready());
    controller.close().unwrap();
    let owner = Arc::new(ProfileOwner::acquire(paths.clone()).unwrap());
    let journal = OperationJournal::open(owner).unwrap();
    assert_eq!(journal.event(OP, 2).unwrap().unwrap().event, retained);
    assert_eq!(journal.append(retained).unwrap(), receipt);
    assert_eq!(rows(&db), 2);
}
#[test]
fn terminal_sql_and_actual_commit_failures_retain_success_and_stop_permanent_retry() {
    for commit in [false, true] {
        let (_temp, paths, journal) = fixture();
        let db = Connection::open(paths.journal()).unwrap();
        let mut controller = ready(journal);
        let observed=controller.run_probe(OP,AT,||{if commit {deferred_failure(&db)}else{db.execute_batch("CREATE TRIGGER fail_terminal BEFORE INSERT ON operation_events WHEN NEW.sequence=2 BEGIN SELECT RAISE(ABORT,'terminal'); END;").unwrap();}true}).unwrap();
        assert!(observed.succeeded);
        assert_eq!(observed.audit, AuditStatus::Failed(RetryClass::Permanent));
        assert_eq!(rows(&db), 1);
        let retained = controller.pending().unwrap().event.clone();
        assert!(controller.retry_pending().is_err());
        assert_eq!(controller.pending().unwrap().retries, 0);
        assert_eq!(controller.pending().unwrap().event, retained);
    }
}
#[test]
fn retries_are_bounded_and_never_clear_uncommitted_evidence() {
    let (_temp, paths, journal) = fixture();
    let db = Connection::open(paths.journal()).unwrap();
    let mut controller = ready(journal);
    controller
        .run_probe(OP, AT, || {
            db.execute_batch("BEGIN IMMEDIATE;").unwrap();
            true
        })
        .unwrap();
    let retained = controller.pending().unwrap().event.clone();
    for _ in 0..MAX_EVIDENCE_RETRIES {
        assert!(controller.retry_pending().is_err());
    }
    assert_eq!(controller.pending().unwrap().retries, MAX_EVIDENCE_RETRIES);
    db.execute_batch("ROLLBACK;").unwrap();
    assert!(controller.retry_pending().is_err());
    assert_eq!(controller.pending().unwrap().event, retained);
    assert_eq!(rows(&db), 1);
    assert!(!controller.admission_ready());
}
#[test]
fn callback_unwind_leaves_unknown_effects_pending_without_fabricated_success() {
    let (_temp, paths, journal) = fixture();
    let mut controller = ready(journal);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        controller.run_probe(OP, AT, || panic!("inert probe panic"))
    }));
    assert!(result.is_err());
    assert!(!controller.admission_ready());
    let pending = controller.pending().unwrap();
    assert_eq!(pending.event.status, OperationStatus::Interrupted);
    assert_eq!(pending.event.payload["effects"], "unknown");
    controller.retry_pending().unwrap();
    let db = Connection::open(paths.journal()).unwrap();
    assert_eq!(rows(&db), 2);
}
#[test]
fn conflicting_retry_is_permanent_and_retains_original_event() {
    let (_temp, paths, journal) = fixture();
    let db = Connection::open(paths.journal()).unwrap();
    let mut controller = ready(journal);
    controller
        .run_probe(OP, AT, || {
            db.execute_batch("BEGIN IMMEDIATE;").unwrap();
            true
        })
        .unwrap();
    let retained = controller.pending().unwrap().event.clone();
    db.execute_batch("ROLLBACK;").unwrap();
    // A separate fixture connection injects a competing event and consistent projection.
    db.execute("INSERT INTO operation_events(operation_id,session_id,operation_kind,event_kind,status,sequence,observed_at,payload) VALUES(?1,?2,'inert_probe','probe.observed','FAILED',2,?3,'{}')",rusqlite::params![OP,SESSION,AT]).unwrap();
    db.execute("UPDATE operations SET status='FAILED',sequence=2,payload='{}',last_event_id=2 WHERE operation_id=?1",[OP]).unwrap();
    assert!(matches!(
        controller.retry_pending(),
        Err(JournalError::Conflict { .. })
    ));
    assert_eq!(
        controller.pending().unwrap().retry_class,
        RetryClass::Permanent
    );
    assert_eq!(controller.pending().unwrap().event, retained);
    assert!(controller.retry_pending().is_err());
    assert_eq!(rows(&db), 2);
}
#[test]
fn recovery_keyset_is_bounded_and_preserves_terminal_and_prior_events() {
    let (_temp, paths, journal) = fixture();
    for i in 1..=RECOVERY_PAGE_SIZE + 3 {
        let id = format!("00000000-0000-0000-0000-{i:012x}");
        journal
            .append(event(
                &id,
                if i == 2 {
                    OperationStatus::Succeeded
                } else {
                    OperationStatus::Running
                },
            ))
            .unwrap();
    }
    assert_eq!(
        journal.recovery_page(None).unwrap().len(),
        RECOVERY_PAGE_SIZE
    );
    let mut controller = EvidenceController::new(journal, SESSION).unwrap();
    assert!(!controller.admission_ready());
    assert!(!controller.recover_step(AT).unwrap());
    assert!(!controller.recover_step(AT).unwrap());
    assert!(controller.recover_step(AT).unwrap());
    assert!(controller.recover_step(AT).unwrap());
    let db = Connection::open(paths.journal()).unwrap();
    assert_eq!(rows(&db), (2 * (RECOVERY_PAGE_SIZE + 3) - 1) as i64);
    let interrupted: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM operations WHERE status='INTERRUPTED'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(interrupted, (RECOVERY_PAGE_SIZE + 2) as i64);
    let preserved: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM operation_events WHERE sequence=1 AND payload=?1",
            [r#"{"preserved":true}"#],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(preserved, (RECOVERY_PAGE_SIZE + 3) as i64);
}
#[test]
fn corrupt_unknown_and_orphaned_rows_fail_recovery_closed() {
    for mutation in [
        "PRAGMA ignore_check_constraints=ON; UPDATE operations SET status='FUTURE';",
        "DELETE FROM operations;",
        "UPDATE operations SET payload='{}';",
    ] {
        let (_temp, paths, journal) = fixture();
        journal.append(event(OP, OperationStatus::Running)).unwrap();
        let db = Connection::open(paths.journal()).unwrap();
        db.execute_batch(mutation).unwrap();
        let mut controller = EvidenceController::new(journal, SESSION).unwrap();
        assert!(controller.recover_step(AT).is_err());
        assert!(!controller.admission_ready());
        assert_eq!(rows(&db), 1);
    }
}

#[test]
fn shared_profile_owner_cannot_create_two_live_controllers() {
    let temp = tempfile::tempdir().unwrap();
    let paths = ProfilePaths::resolve(Some(&temp.path().join("profile"))).unwrap();
    let owner = Arc::new(ProfileOwner::acquire(paths).unwrap());
    let first = OperationJournal::open(owner.clone()).unwrap();
    let mut first = ready(first);
    let panic_once = || panic!("leave first controller live with pending evidence");
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        first.run_probe(OP, AT, panic_once)
    }));
    assert!(result.is_err());
    let second = OperationJournal::open(owner.clone()).unwrap();
    assert!(EvidenceController::new(second, SESSION).is_err());
    assert!(first.pending().is_some());
    first.close().unwrap();
    let third = OperationJournal::open(owner).unwrap();
    let third = ready(third);
    assert!(third.admission_ready());
}

#[test]
fn recovery_child() {
    let Some(root) = std::env::var_os("VFA_EVIDENCE_CHILD_ROOT") else {
        return;
    };
    let paths = ProfilePaths::resolve(Some(std::path::Path::new(&root))).unwrap();
    let owner = Arc::new(ProfileOwner::acquire(paths).unwrap());
    let journal = OperationJournal::open(owner).unwrap();
    journal.append(event(OP, OperationStatus::Running)).unwrap();
    journal
        .append(event(
            "00000000-0000-0000-0000-000000000001",
            OperationStatus::Succeeded,
        ))
        .unwrap();
    std::fs::write(
        std::env::var_os("VFA_EVIDENCE_CHILD_READY").unwrap(),
        b"committed",
    )
    .unwrap();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    journal.close().unwrap();
}

#[test]
fn live_process_cannot_be_recovered_and_owner_death_allows_unknown_recovery() {
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("profile");
    let ready_file = temp.path().join("ready");
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "recovery_child",
            "--nocapture",
            "--test-threads=1",
        ])
        .env("VFA_EVIDENCE_CHILD_ROOT", &root)
        .env("VFA_EVIDENCE_CHILD_READY", &ready_file)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    struct Reap(std::process::Child);
    impl Drop for Reap {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let deadline = Instant::now() + Duration::from_secs(15);
    while !ready_file.exists() {
        if child.try_wait().unwrap().is_some() || Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("child did not commit fixture");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let mut child = Reap(child);
    let paths = ProfilePaths::resolve(Some(&root)).unwrap();
    assert!(matches!(
        ProfileOwner::acquire(paths.clone()),
        Err(vfa_tui::engine::profile::ProfileError::Busy { .. })
    ));
    let db = Connection::open(paths.journal()).unwrap();
    assert_eq!(rows(&db), 2);
    child.0.kill().unwrap();
    child.0.wait().unwrap();
    let owner = Arc::new(ProfileOwner::acquire(paths.clone()).unwrap());
    let journal = OperationJournal::open(owner).unwrap();
    let controller = ready(journal);
    let (status, payload): (String, String) = db
        .query_row(
            "SELECT status,payload FROM operations WHERE operation_id=?1",
            [OP],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(status, "INTERRUPTED");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&payload).unwrap()["effects"],
        "unknown"
    );
    assert_eq!(rows(&db), 3);
    controller.close().unwrap();
    let owner = Arc::new(ProfileOwner::acquire(paths).unwrap());
    let controller = ready(OperationJournal::open(owner).unwrap());
    assert_eq!(rows(&db), 3);
    controller.close().unwrap();
}

#[test]
fn failed_recovery_is_retained_and_identical_retry_does_not_recover_twice() {
    let (_temp, paths, journal) = fixture();
    journal.append(event(OP, OperationStatus::Running)).unwrap();
    let db = Connection::open(paths.journal()).unwrap();
    db.execute_batch("BEGIN IMMEDIATE;").unwrap();
    let mut controller = EvidenceController::new(journal, SESSION).unwrap();
    assert!(controller.recover_step(AT).is_err());
    let retained = controller.pending().unwrap().event.clone();
    assert_eq!(retained.status, OperationStatus::Interrupted);
    assert!(!controller.admission_ready());
    db.execute_batch("ROLLBACK;").unwrap();
    let receipt = controller.retry_pending().unwrap();
    while !controller.recover_step(AT).unwrap() {}
    assert_eq!(rows(&db), 2);
    controller.close().unwrap();
    let owner = Arc::new(ProfileOwner::acquire(paths).unwrap());
    let journal = OperationJournal::open(owner).unwrap();
    assert_eq!(journal.append(retained).unwrap(), receipt);
    assert_eq!(rows(&db), 2);
}

#[test]
fn unrecovered_controller_and_invalid_inputs_do_not_call_probe() {
    let (_temp, paths, journal) = fixture();
    let mut controller = EvidenceController::new(journal, SESSION).unwrap();
    let mut calls = 0;
    assert!(controller
        .run_probe(OP, AT, || {
            calls += 1;
            true
        })
        .is_err());
    assert!(controller.recover_step(AT).unwrap());
    for (id, at) in [("bad-id", AT), (OP, "bad-time")] {
        assert!(controller
            .run_probe(id, at, || {
                calls += 1;
                true
            })
            .is_err());
    }
    assert_eq!(calls, 0);
    let db = Connection::open(paths.journal()).unwrap();
    assert_eq!(rows(&db), 0);
}
