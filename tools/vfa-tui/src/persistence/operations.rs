//! Desktop operation evidence with real transactional commit acknowledgments.
//!
//! A dedicated thread owns its connection and the profile owner until closure.
//! This journal does not execute/replay actions, redact payloads, enforce the
//! later lifecycle controller, or claim immutable/tamper-proof history.
//! Blocking calls belong on an engine worker, not a UI callback. No runtime is
//! constructed per request. Use `close` to observe orderly connection errors.

use std::sync::{mpsc, Arc};
use std::thread::{self, JoinHandle};

use chrono::{DateTime, SecondsFormat, Utc};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::engine::profile::ProfileOwner;

use super::operation_schema;

pub const MAX_PAYLOAD_BYTES: usize = 16_384;
const QUEUE_CAPACITY: usize = 32;

#[derive(Debug, thiserror::Error)]
pub enum JournalError {
    #[error("invalid event or persisted data: {0}")]
    InvalidData(String),
    #[error("unrecognized or unusable journal: {0}")]
    InvalidStore(String),
    #[error("foreign database application ID: {application}")]
    ForeignStore { application: i32 },
    #[error("journal schema {version} is newer than supported")]
    NewerSchema { version: i32 },
    #[error("conflicting event for operation {operation_id}, sequence {sequence}")]
    Conflict { operation_id: String, sequence: u64 },
    #[error("expected sequence {expected}, got {actual}")]
    Sequence { expected: u64, actual: u64 },
    #[error("journal request queue is full")]
    QueueFull,
    #[error("journal writer stopped before replying")]
    WorkerStopped,
    #[error("journal writer panicked")]
    WorkerPanicked,
    #[error("journal filesystem/thread error: {0}")]
    Io(#[from] std::io::Error),
    #[error("journal SQLite failure during {stage}: {source}")]
    Sqlite {
        stage: &'static str,
        #[source]
        source: rusqlite::Error,
    },
}

impl JournalError {
    pub(crate) fn sqlite(stage: &'static str, source: rusqlite::Error) -> Self {
        Self::Sqlite { stage, source }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OperationStatus {
    Queued,
    Running,
    Cancelling,
    Succeeded,
    Failed,
    Cancelled,
    TimedOut,
    Interrupted,
}

impl OperationStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "QUEUED",
            Self::Running => "RUNNING",
            Self::Cancelling => "CANCELLING",
            Self::Succeeded => "SUCCEEDED",
            Self::Failed => "FAILED",
            Self::Cancelled => "CANCELLED",
            Self::TimedOut => "TIMED_OUT",
            Self::Interrupted => "INTERRUPTED",
        }
    }

    fn parse(value: &str) -> Result<Self, JournalError> {
        match value {
            "QUEUED" => Ok(Self::Queued),
            "RUNNING" => Ok(Self::Running),
            "CANCELLING" => Ok(Self::Cancelling),
            "SUCCEEDED" => Ok(Self::Succeeded),
            "FAILED" => Ok(Self::Failed),
            "CANCELLED" => Ok(Self::Cancelled),
            "TIMED_OUT" => Ok(Self::TimedOut),
            "INTERRUPTED" => Ok(Self::Interrupted),
            _ => Err(JournalError::InvalidData(format!(
                "unknown persisted status: {value}"
            ))),
        }
    }
}

/// Validated again at the persistence boundary. Payload redaction is the
/// caller's responsibility; canonicalization only sorts JSON object keys.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationEvent {
    pub operation_id: String,
    pub session_id: String,
    pub operation_kind: String,
    pub event_kind: String,
    pub status: OperationStatus,
    pub sequence: u64,
    pub observed_at: String,
    pub payload: Value,
}

/// Stable identity read from the committed event row, including on retries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitReceipt {
    pub event_id: i64,
    pub operation_id: String,
    pub sequence: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PersistedEvent {
    pub receipt: CommitReceipt,
    pub event: OperationEvent,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OperationProjection {
    pub operation_id: String,
    pub session_id: String,
    pub operation_kind: String,
    pub status: OperationStatus,
    pub sequence: u64,
    pub observed_at: String,
    pub payload: Value,
    pub last_event_id: i64,
}

struct CanonicalEvent {
    event: OperationEvent,
    payload: String,
}

impl CanonicalEvent {
    fn new(mut event: OperationEvent) -> Result<Self, JournalError> {
        event.operation_id = canonical_uuid(&event.operation_id)?;
        event.session_id = canonical_uuid(&event.session_id)?;
        validate_label(&event.operation_kind)?;
        validate_label(&event.event_kind)?;
        if event.sequence == 0 || event.sequence > i64::MAX as u64 {
            return Err(JournalError::InvalidData(
                "sequence must be in 1..=i64::MAX".into(),
            ));
        }
        if event.observed_at.len() > 64 {
            return Err(JournalError::InvalidData(
                "observed time is too long".into(),
            ));
        }
        event.observed_at = DateTime::parse_from_rfc3339(&event.observed_at)
            .map_err(|_| JournalError::InvalidData("observed time must be RFC3339".into()))?
            .with_timezone(&Utc)
            .to_rfc3339_opts(SecondsFormat::AutoSi, true);
        check_payload_shape(&event.payload)?;
        event.payload = sorted_json(&event.payload);
        let payload = serde_json::to_string(&event.payload)
            .map_err(|error| JournalError::InvalidData(error.to_string()))?;
        if payload.len() > MAX_PAYLOAD_BYTES {
            return Err(JournalError::InvalidData(
                "payload exceeds 16384 bytes".into(),
            ));
        }
        Ok(Self { event, payload })
    }
}

fn canonical_uuid(value: &str) -> Result<String, JournalError> {
    let id = Uuid::parse_str(value)
        .map_err(|_| JournalError::InvalidData("operation/session ID must be a UUID".into()))?;
    if id.is_nil() {
        return Err(JournalError::InvalidData(
            "operation/session ID cannot be nil".into(),
        ));
    }
    Ok(id.to_string())
}

fn validate_label(value: &str) -> Result<(), JournalError> {
    if value.is_empty()
        || value.len() > 64
        || !value.as_bytes()[0].is_ascii_lowercase()
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_.-".contains(&byte)
        })
    {
        return Err(JournalError::InvalidData(
            "kind must be a bounded lowercase label".into(),
        ));
    }
    Ok(())
}

fn check_payload_shape(payload: &Value) -> Result<(), JournalError> {
    let mut stack = vec![(payload, 0usize)];
    let mut lower_bound = 0usize;
    while let Some((value, depth)) = stack.pop() {
        if depth > 32 || lower_bound > MAX_PAYLOAD_BYTES {
            return Err(JournalError::InvalidData(
                "payload exceeds size/depth bound".into(),
            ));
        }
        lower_bound = lower_bound.saturating_add(1);
        match value {
            Value::String(text) => lower_bound = lower_bound.saturating_add(text.len()),
            Value::Array(values) => {
                if values.len() > MAX_PAYLOAD_BYTES {
                    return Err(JournalError::InvalidData(
                        "payload array is too large".into(),
                    ));
                }
                stack.extend(values.iter().map(|value| (value, depth + 1)));
            }
            Value::Object(values) => {
                if values.len() > MAX_PAYLOAD_BYTES {
                    return Err(JournalError::InvalidData(
                        "payload object is too large".into(),
                    ));
                }
                for (key, value) in values {
                    lower_bound = lower_bound.saturating_add(key.len());
                    stack.push((value, depth + 1));
                }
            }
            _ => {}
        }
    }
    if lower_bound > MAX_PAYLOAD_BYTES {
        return Err(JournalError::InvalidData(
            "payload exceeds size bound".into(),
        ));
    }
    Ok(())
}

fn sorted_json(value: &Value) -> Value {
    match value {
        Value::Object(values) => {
            let mut keys: Vec<_> = values.keys().collect();
            keys.sort();
            Value::Object(
                keys.into_iter()
                    .map(|key| (key.clone(), sorted_json(&values[key])))
                    .collect(),
            )
        }
        Value::Array(values) => Value::Array(values.iter().map(sorted_json).collect()),
        value => value.clone(),
    }
}

enum Request {
    Append(
        Box<CanonicalEvent>,
        mpsc::Sender<Result<CommitReceipt, JournalError>>,
    ),
    Event(
        String,
        u64,
        mpsc::Sender<Result<Option<PersistedEvent>, JournalError>>,
    ),
    Operation(
        String,
        mpsc::Sender<Result<Option<OperationProjection>, JournalError>>,
    ),
}

/// One bounded request queue and one dedicated writer per journal handle.
/// Share this handle, rather than opening per-request journal instances.
pub struct OperationJournal {
    sender: Option<mpsc::SyncSender<Request>>,
    worker: Option<JoinHandle<Result<(), JournalError>>>,
}

impl OperationJournal {
    pub fn open(owner: Arc<ProfileOwner>) -> Result<Self, JournalError> {
        let (sender, receiver) = mpsc::sync_channel(QUEUE_CAPACITY);
        let (ready_sender, ready_receiver) = mpsc::sync_channel(1);
        let worker = thread::Builder::new()
            .name("vfa-operations".into())
            .spawn(move || run_worker(owner, receiver, ready_sender))?;
        match ready_receiver.recv() {
            Ok(Ok(())) => Ok(Self {
                sender: Some(sender),
                worker: Some(worker),
            }),
            result => {
                drop(sender);
                worker.join().map_err(|_| JournalError::WorkerPanicked)??;
                match result {
                    Ok(Err(error)) => Err(error),
                    _ => Err(JournalError::WorkerStopped),
                }
            }
        }
    }

    /// Returns only after SQL and COMMIT succeed. Queue admission is not a receipt.
    pub fn append(&self, event: OperationEvent) -> Result<CommitReceipt, JournalError> {
        let event = Box::new(CanonicalEvent::new(event)?);
        let (reply, response) = mpsc::channel();
        self.send(Request::Append(event, reply))?;
        response.recv().map_err(|_| JournalError::WorkerStopped)?
    }

    pub fn event(
        &self,
        operation_id: &str,
        sequence: u64,
    ) -> Result<Option<PersistedEvent>, JournalError> {
        if sequence == 0 || sequence > i64::MAX as u64 {
            return Err(JournalError::InvalidData("invalid event sequence".into()));
        }
        let (reply, response) = mpsc::channel();
        self.send(Request::Event(
            canonical_uuid(operation_id)?,
            sequence,
            reply,
        ))?;
        response.recv().map_err(|_| JournalError::WorkerStopped)?
    }

    pub fn operation(
        &self,
        operation_id: &str,
    ) -> Result<Option<OperationProjection>, JournalError> {
        let (reply, response) = mpsc::channel();
        self.send(Request::Operation(canonical_uuid(operation_id)?, reply))?;
        response.recv().map_err(|_| JournalError::WorkerStopped)?
    }

    fn send(&self, request: Request) -> Result<(), JournalError> {
        self.sender
            .as_ref()
            .ok_or(JournalError::WorkerStopped)?
            .try_send(request)
            .map_err(|error| match error {
                mpsc::TrySendError::Full(_) => JournalError::QueueFull,
                mpsc::TrySendError::Disconnected(_) => JournalError::WorkerStopped,
            })
    }

    /// Disconnect the bounded queue, drain admitted requests, close, and join.
    /// SQLite busy waits are finite; underlying OS disk stalls are not timed out.
    pub fn close(mut self) -> Result<(), JournalError> {
        self.finish()
    }

    fn finish(&mut self) -> Result<(), JournalError> {
        self.sender.take();
        match self.worker.take() {
            Some(worker) => worker.join().map_err(|_| JournalError::WorkerPanicked)?,
            None => Ok(()),
        }
    }
}

impl Drop for OperationJournal {
    fn drop(&mut self) {
        // Callers needing close-error evidence must use explicit `close`.
        let _ = self.finish();
    }
}

fn run_worker(
    owner: Arc<ProfileOwner>,
    receiver: mpsc::Receiver<Request>,
    ready: mpsc::SyncSender<Result<(), JournalError>>,
) -> Result<(), JournalError> {
    let mut connection = match operation_schema::open(owner.paths().journal()) {
        Ok(connection) => connection,
        Err(error) => {
            let _ = ready.send(Err(error));
            return Ok(());
        }
    };
    if ready.send(Ok(())).is_ok() {
        while let Ok(request) = receiver.recv() {
            match request {
                Request::Append(event, reply) => {
                    let _ = reply.send(commit_event(&mut connection, &event));
                }
                Request::Event(id, sequence, reply) => {
                    let _ = reply.send(read_event(&connection, &id, sequence));
                }
                Request::Operation(id, reply) => {
                    let _ = reply.send(read_operation(&connection, &id));
                }
            }
        }
    }
    let closed = match connection.close() {
        Ok(()) => Ok(()),
        Err((connection, error)) => {
            drop(connection);
            Err(JournalError::sqlite("close writer", error))
        }
    };
    // Explicit order: the profile cannot be reacquired before SQLite is closed.
    drop(owner);
    closed
}

fn projection(event: &OperationEvent, event_id: i64) -> OperationProjection {
    OperationProjection {
        operation_id: event.operation_id.clone(),
        session_id: event.session_id.clone(),
        operation_kind: event.operation_kind.clone(),
        status: event.status,
        sequence: event.sequence,
        observed_at: event.observed_at.clone(),
        payload: event.payload.clone(),
        last_event_id: event_id,
    }
}

fn commit_event(
    connection: &mut Connection,
    canonical: &CanonicalEvent,
) -> Result<CommitReceipt, JournalError> {
    let event = &canonical.event;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| JournalError::sqlite("begin event", error))?;
    // Even old, idempotent retries must not acknowledge inconsistent evidence.
    // A consistent newer projection is valid and is never rewound by a retry.
    let current = read_operation(&transaction, &event.operation_id)?;
    if let Some(existing) = read_event(&transaction, &event.operation_id, event.sequence)? {
        if existing.event != *event {
            return Err(JournalError::Conflict {
                operation_id: event.operation_id.clone(),
                sequence: event.sequence,
            });
        }
        transaction
            .commit()
            .map_err(|error| JournalError::sqlite("commit retry", error))?;
        return Ok(existing.receipt);
    }
    let expected = match current {
        Some(ref current) => {
            if current.operation_kind != event.operation_kind {
                return Err(JournalError::InvalidData(
                    "operation kind cannot change".into(),
                ));
            }
            current
                .sequence
                .checked_add(1)
                .ok_or_else(|| JournalError::InvalidData("sequence exhausted".into()))?
        }
        None => 1,
    };
    if event.sequence != expected {
        return Err(JournalError::Sequence {
            expected,
            actual: event.sequence,
        });
    }
    transaction.execute(
        "INSERT INTO operation_events (operation_id, session_id, operation_kind, event_kind, status, sequence, observed_at, payload)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![event.operation_id, event.session_id, event.operation_kind, event.event_kind, event.status.as_str(), event.sequence as i64, event.observed_at, canonical.payload],
    ).map_err(|error| JournalError::sqlite("insert event", error))?;
    let event_id = transaction.last_insert_rowid();
    transaction.execute(
        "INSERT INTO operations (operation_id, session_id, operation_kind, status, sequence, observed_at, payload, last_event_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(operation_id) DO UPDATE SET session_id=excluded.session_id, operation_kind=excluded.operation_kind,
         status=excluded.status, sequence=excluded.sequence, observed_at=excluded.observed_at, payload=excluded.payload, last_event_id=excluded.last_event_id",
        params![event.operation_id, event.session_id, event.operation_kind, event.status.as_str(), event.sequence as i64, event.observed_at, canonical.payload, event_id],
    ).map_err(|error| JournalError::sqlite("update operation projection", error))?;
    let persisted = read_event(&transaction, &event.operation_id, event.sequence)?
        .ok_or_else(|| JournalError::InvalidStore("inserted event is missing".into()))?;
    if persisted.event != *event
        || read_operation(&transaction, &event.operation_id)? != Some(projection(event, event_id))
    {
        return Err(JournalError::InvalidStore(
            "event/projection readback mismatch".into(),
        ));
    }
    transaction
        .commit()
        .map_err(|error| JournalError::sqlite("commit event", error))?;
    Ok(persisted.receipt)
}

fn read_event(
    connection: &Connection,
    operation_id: &str,
    sequence: u64,
) -> Result<Option<PersistedEvent>, JournalError> {
    let row = connection.query_row(
        "SELECT id, operation_id, session_id, operation_kind, event_kind, status, sequence, observed_at, payload
         FROM operation_events WHERE operation_id = ?1 AND sequence = ?2",
        params![operation_id, sequence as i64],
        |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?, row.get::<_, String>(4)?, row.get::<_, String>(5)?, row.get::<_, i64>(6)?, row.get::<_, String>(7)?, row.get::<_, String>(8)?)),
    ).optional().map_err(|error| JournalError::sqlite("read event", error))?;
    let Some((
        id,
        operation_id,
        session_id,
        operation_kind,
        event_kind,
        status,
        sequence,
        observed_at,
        payload,
    )) = row
    else {
        return Ok(None);
    };
    if id <= 0 || sequence <= 0 {
        return Err(JournalError::InvalidData(
            "invalid persisted event identity".into(),
        ));
    }
    let canonical = CanonicalEvent::new(OperationEvent {
        operation_id,
        session_id,
        operation_kind,
        event_kind,
        status: OperationStatus::parse(&status)?,
        sequence: sequence as u64,
        observed_at,
        payload: serde_json::from_str(&payload)
            .map_err(|error| JournalError::InvalidData(error.to_string()))?,
    })?;
    if canonical.payload != payload {
        return Err(JournalError::InvalidData(
            "noncanonical persisted payload".into(),
        ));
    }
    Ok(Some(PersistedEvent {
        receipt: CommitReceipt {
            event_id: id,
            operation_id: canonical.event.operation_id.clone(),
            sequence: canonical.event.sequence,
        },
        event: canonical.event,
    }))
}

fn read_operation(
    connection: &Connection,
    operation_id: &str,
) -> Result<Option<OperationProjection>, JournalError> {
    let latest_sequence: Option<i64> = connection
        .query_row(
            "SELECT MAX(sequence) FROM operation_events WHERE operation_id = ?1",
            [operation_id],
            |row| row.get(0),
        )
        .map_err(|error| JournalError::sqlite("read latest operation event", error))?;
    let row = connection.query_row(
        "SELECT last_event_id, operation_id, session_id, operation_kind, status, sequence, observed_at, payload
         FROM operations WHERE operation_id = ?1",
        [operation_id],
        |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?, row.get::<_, String>(4)?, row.get::<_, i64>(5)?, row.get::<_, String>(6)?, row.get::<_, String>(7)?)),
    ).optional().map_err(|error| JournalError::sqlite("read operation projection", error))?;
    let Some((
        id,
        operation_id,
        session_id,
        operation_kind,
        status,
        sequence,
        observed_at,
        payload,
    )) = row
    else {
        if latest_sequence.is_some() {
            return Err(JournalError::InvalidStore(
                "operation events have no projection".into(),
            ));
        }
        return Ok(None);
    };
    if id <= 0 || sequence <= 0 {
        return Err(JournalError::InvalidData(
            "invalid persisted projection identity".into(),
        ));
    }
    let canonical = CanonicalEvent::new(OperationEvent {
        operation_id,
        session_id,
        operation_kind,
        event_kind: "projection".into(),
        status: OperationStatus::parse(&status)?,
        sequence: sequence as u64,
        observed_at,
        payload: serde_json::from_str(&payload)
            .map_err(|error| JournalError::InvalidData(error.to_string()))?,
    })?;
    if canonical.payload != payload {
        return Err(JournalError::InvalidData(
            "noncanonical persisted projection payload".into(),
        ));
    }
    let current = projection(&canonical.event, id);
    let linked = read_event(connection, &canonical.event.operation_id, sequence as u64)?
        .ok_or_else(|| JournalError::InvalidStore("projection event is missing".into()))?;
    if latest_sequence != Some(sequence)
        || current != projection(&linked.event, linked.receipt.event_id)
    {
        return Err(JournalError::InvalidStore(
            "projection does not match the latest operation event".into(),
        ));
    }
    Ok(Some(current))
}
