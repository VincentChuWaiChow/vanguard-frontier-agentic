//! Serial, evidence-only admission for **inert caller probes**, not helpers.
//!
//! Owns the real journal and its retained profile lock. Complete bounded startup
//! recovery before admission. Never use this preparation API to launch a real
//! helper: authorization, cancellation and process-tree cleanup are not integrated.
//! A callback panic unwinds normally; its pre-retained unknown-effects evidence
//! blocks admission. No catch_unwind or fabricated successful receipt is used.

use chrono::{DateTime, SecondsFormat, Utc};
use rusqlite::ErrorCode;
use serde_json::json;
use uuid::Uuid;

use crate::persistence::operations::{
    CommitReceipt, JournalError, OperationEvent, OperationJournal, OperationStatus,
};

pub const MAX_PENDING_EVENTS: usize = 1;
pub const MAX_EVIDENCE_RETRIES: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryClass {
    Transient,
    Permanent,
}

pub fn retry_class(error: &JournalError) -> RetryClass {
    match error {
        JournalError::QueueFull => RetryClass::Transient,
        JournalError::Sqlite {
            source: rusqlite::Error::SqliteFailure(error, _),
            ..
        } if matches!(
            error.code,
            ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked
        ) =>
        {
            RetryClass::Transient
        }
        _ => RetryClass::Permanent,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditStatus {
    Committed(CommitReceipt),
    Failed(RetryClass),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeObservation {
    pub start: CommitReceipt,
    /// Only the retained callback return value determines this known outcome.
    pub succeeded: bool,
    pub audit: AuditStatus,
}

#[derive(Debug)]
pub struct PendingEvidence {
    pub event: OperationEvent,
    pub retry_class: RetryClass,
    pub retries: usize,
}

/// One admitted probe at a time, one pending terminal slot, no action retry API.
/// All blocking methods belong on an engine worker, never a UI callback.
pub struct EvidenceController {
    journal: OperationJournal,
    session: String,
    recovered: bool,
    cursor: Option<String>,
    blocked: bool,
    pending: Option<PendingEvidence>,
}

impl EvidenceController {
    pub fn new(mut journal: OperationJournal, session: &str) -> Result<Self, JournalError> {
        let session = identity(session)?;
        journal.claim_evidence_authority()?;
        Ok(Self {
            journal,
            session,
            recovered: false,
            cursor: None,
            blocked: false,
            pending: None,
        })
    }

    pub fn pending(&self) -> Option<&PendingEvidence> {
        self.pending.as_ref()
    }

    pub fn admission_ready(&self) -> bool {
        self.recovered && !self.blocked && self.pending.is_none()
    }

    /// Scan at most one fixed-size page. Returns true only after an empty page
    /// proves completion. Keyset paging includes terminals and does not skip
    /// rows as recovery changes their status. Existing evidence is never edited.
    pub fn recover_step(&mut self, at: &str) -> Result<bool, JournalError> {
        if self.pending.is_some() || self.blocked {
            return Err(rejected("unresolved evidence blocks recovery"));
        }
        if self.recovered {
            return Ok(true);
        }
        let at = timestamp(at)?;
        let page = match self.journal.recovery_page(self.cursor.as_deref()) {
            Ok(page) => page,
            Err(error) => {
                self.blocked = true;
                return Err(error);
            }
        };
        if page.is_empty() {
            self.recovered = true;
            return Ok(true);
        }
        for previous in page {
            if matches!(
                previous.status,
                OperationStatus::Queued | OperationStatus::Running | OperationStatus::Cancelling
            ) {
                let sequence = previous
                    .sequence
                    .checked_add(1)
                    .filter(|sequence| *sequence <= i64::MAX as u64)
                    .ok_or_else(|| rejected("recovery sequence exhausted"))?;
                let event = OperationEvent {
                    operation_id: previous.operation_id.clone(),
                    session_id: self.session.clone(),
                    operation_kind: previous.operation_kind,
                    event_kind: "restart.interrupted".into(),
                    status: OperationStatus::Interrupted,
                    sequence,
                    observed_at: at.clone(),
                    payload: json!({"effects":"unknown", "previous_session":previous.session_id,
                        "previous_event_id":previous.last_event_id}),
                };
                self.pending = Some(PendingEvidence {
                    event,
                    retry_class: RetryClass::Transient,
                    retries: 0,
                });
                self.persist_pending()?;
            }
            self.cursor = Some(previous.operation_id);
        }
        Ok(false)
    }

    /// Consumes FnOnce only after a real committed start. Reusing any persisted
    /// operation identity is rejected, even when an identical start would dedup.
    /// Fixed bounded payloads avoid caller-produced terminal builders or errors.
    pub fn run_probe<F: FnOnce() -> bool>(
        &mut self,
        id: &str,
        at: &str,
        probe: F,
    ) -> Result<ProbeObservation, JournalError> {
        if !self.admission_ready() {
            return Err(rejected(
                "admission blocked by recovery or unresolved evidence",
            ));
        }
        let id = identity(id)?;
        let at = timestamp(at)?;
        match self.journal.operation(&id) {
            Ok(None) => {}
            Ok(Some(_)) => return Err(rejected("operation identity already used; never replay")),
            Err(error) => {
                self.blocked = true;
                return Err(error);
            }
        }
        let start = OperationEvent {
            operation_id: id,
            session_id: self.session.clone(),
            operation_kind: "inert_probe".into(),
            event_kind: "probe.started".into(),
            status: OperationStatus::Running,
            sequence: 1,
            observed_at: at,
            payload: json!({"effects":"none_verified"}),
        };
        let mut terminal = start.clone();
        terminal.sequence = 2;
        terminal.event_kind = "probe.observed".into();
        terminal.status = OperationStatus::Interrupted;
        terminal.payload = json!({"effects":"unknown", "observation":"callback_did_not_return"});
        // Allocate the fallback before the callback and before the start commit.
        // No user-provided builder runs between observing a result and retaining it.
        let pending = PendingEvidence {
            event: terminal,
            retry_class: RetryClass::Transient,
            retries: 0,
        };
        let receipt = match self.journal.append(start) {
            Ok(receipt) => receipt,
            Err(error) => {
                self.blocked = true;
                return Err(error);
            }
        };
        self.pending = Some(pending);
        let succeeded = probe();
        if let Some(pending) = self.pending.as_mut() {
            pending.event.status = if succeeded {
                OperationStatus::Succeeded
            } else {
                OperationStatus::Failed
            };
            pending.event.payload = json!({"effects":"inert_probe_only", "succeeded":succeeded});
        }
        let audit = match self.persist_pending() {
            Ok(receipt) => AuditStatus::Committed(receipt),
            Err(error) => AuditStatus::Failed(retry_class(&error)),
        };
        Ok(ProbeObservation {
            start: receipt,
            succeeded,
            audit,
        })
    }

    /// Exactly one evidence append, never a callback or action. Permanent errors
    /// and exhausted retry budgets retain the original event and stop here.
    pub fn retry_pending(&mut self) -> Result<CommitReceipt, JournalError> {
        let pending = self
            .pending
            .as_mut()
            .ok_or_else(|| rejected("no pending evidence"))?;
        if pending.retry_class == RetryClass::Permanent || pending.retries >= MAX_EVIDENCE_RETRIES {
            return Err(rejected(
                "evidence retry stopped; operator correction required",
            ));
        }
        pending.retries += 1;
        self.persist_pending()
    }

    fn persist_pending(&mut self) -> Result<CommitReceipt, JournalError> {
        let pending = self
            .pending
            .as_ref()
            .ok_or_else(|| rejected("no pending evidence"))?;
        match self.journal.append(pending.event.clone()) {
            Ok(receipt) => {
                self.pending = None;
                Ok(receipt)
            }
            Err(error) => {
                if let Some(pending) = self.pending.as_mut() {
                    pending.retry_class = retry_class(&error);
                }
                Err(error)
            }
        }
    }

    /// Close/join the journal worker, NOT an evidence flush or full shutdown.
    /// Pending events are never silently retried. Choosing to close with pending
    /// evidence discards its in-memory observation; restart can only classify
    /// persisted nonterminal records interrupted with unknown effects. Inspect
    /// `pending()` before making that operator decision. Ok proves worker closure
    /// only, not audit completion or preservation of an uncommitted outcome.
    pub fn close(self) -> Result<(), JournalError> {
        self.journal.close()
    }
}

fn rejected(message: &str) -> JournalError {
    JournalError::InvalidData(message.into())
}

fn identity(value: &str) -> Result<String, JournalError> {
    let id = Uuid::parse_str(value).map_err(|_| rejected("invalid evidence identity"))?;
    if id.is_nil() {
        return Err(rejected("nil evidence identity"));
    }
    Ok(id.to_string())
}

fn timestamp(value: &str) -> Result<String, JournalError> {
    DateTime::parse_from_rfc3339(value)
        .map(|at| {
            at.with_timezone(&Utc)
                .to_rfc3339_opts(SecondsFormat::AutoSi, true)
        })
        .map_err(|_| rejected("invalid evidence timestamp"))
}
