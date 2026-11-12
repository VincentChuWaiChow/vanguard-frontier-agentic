//! Separate Desktop journal identity and transactional, forward-only schema.
//!
//! Version 1 is the initial table layout; version 2 adds a session-event index.
//! These are initial implementation fixtures, not claims about deployed history.
//! This journal deliberately supports DELETE-format databases, not WAL stores.

use std::fs::{self, File, OpenOptions};
use std::io::Read;
use std::path::Path;
use std::time::Duration;

use rusqlite::{Connection, OpenFlags, OptionalExtension, TransactionBehavior};

use super::operations::JournalError;

#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};

pub(crate) const APPLICATION_ID: i32 = 0x5646_4a31;
pub(crate) const APPLICATION: &str = "vfa.desktop.operations";
pub(crate) const SCHEMA_VERSION: i32 = 2;
pub(crate) const BUSY_TIMEOUT: Duration = Duration::from_millis(250);

const META_TABLE: &str = "CREATE TABLE journal_meta (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    application TEXT NOT NULL,
    schema_version INTEGER NOT NULL,
    producer TEXT NOT NULL
)";

const EVENTS_TABLE: &str = "CREATE TABLE operation_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    operation_id TEXT NOT NULL,
    session_id TEXT NOT NULL,
    operation_kind TEXT NOT NULL,
    event_kind TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('QUEUED','RUNNING','CANCELLING','SUCCEEDED','FAILED','CANCELLED','TIMED_OUT','INTERRUPTED')),
    sequence INTEGER NOT NULL CHECK (sequence > 0),
    observed_at TEXT NOT NULL,
    payload TEXT NOT NULL CHECK (length(CAST(payload AS BLOB)) <= 16384),
    UNIQUE (operation_id, sequence)
)";

const OPERATIONS_TABLE: &str = "CREATE TABLE operations (
    operation_id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    operation_kind TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('QUEUED','RUNNING','CANCELLING','SUCCEEDED','FAILED','CANCELLED','TIMED_OUT','INTERRUPTED')),
    sequence INTEGER NOT NULL CHECK (sequence > 0),
    observed_at TEXT NOT NULL,
    payload TEXT NOT NULL CHECK (length(CAST(payload AS BLOB)) <= 16384),
    last_event_id INTEGER NOT NULL REFERENCES operation_events(id)
)";

const SESSION_INDEX: &str =
    "CREATE INDEX operation_events_session ON operation_events(session_id, id)";

/// The worker already owns the profile guard when this function is called.
pub(crate) fn open(path: &Path) -> Result<Connection, JournalError> {
    let existing = match fs::symlink_metadata(path) {
        Ok(_) => Some(preflight(path)?),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(JournalError::Io(error)),
    };
    if existing.is_none() {
        // Reserve a new private regular file. Never truncate an existing path.
        let mut options = OpenOptions::new();
        options.read(true).write(true).create_new(true);
        #[cfg(unix)]
        options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
        drop(options.open(path)?);
    }
    let mut connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|error| JournalError::sqlite("open writer", error))?;
    connection
        .busy_timeout(BUSY_TIMEOUT)
        .map_err(|error| JournalError::sqlite("set busy timeout", error))?;
    // Existing identity/schema was inspected read-only before these settings.
    // No WAL setup or fallback connection is used, including on failures.
    connection
        .pragma_update(None, "foreign_keys", "ON")
        .map_err(|error| JournalError::sqlite("enable foreign keys", error))?;
    connection
        .pragma_update(None, "synchronous", "FULL")
        .map_err(|error| JournalError::sqlite("set synchronous commits", error))?;
    if existing != Some(SCHEMA_VERSION) {
        migrate(&mut connection, existing.unwrap_or(0))?;
    }
    validate_schema(&connection, SCHEMA_VERSION)?;
    Ok(connection)
}

fn private_regular_file(path: &Path) -> Result<File, JournalError> {
    let named = fs::symlink_metadata(path)?;
    if !named.is_file() || named.file_type().is_symlink() {
        return Err(JournalError::InvalidStore(
            "journal must be a regular non-symlink file".into(),
        ));
    }
    #[cfg(unix)]
    {
        // SAFETY: geteuid has no preconditions and does not mutate state.
        if named.uid() != unsafe { libc::geteuid() }
            || named.permissions().mode() & 0o777 != 0o600
            || named.nlink() != 1
        {
            return Err(JournalError::InvalidStore(
                "journal must be user-owned, private (0600), and single-link".into(),
            ));
        }
    }
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    options.custom_flags(libc::O_NOFOLLOW);
    let file = options.open(path)?;
    #[cfg(unix)]
    {
        let opened = file.metadata()?;
        if named.dev() != opened.dev() || named.ino() != opened.ino() {
            return Err(JournalError::InvalidStore(
                "journal identity changed".into(),
            ));
        }
    }
    Ok(file)
}

fn preflight(path: &Path) -> Result<i32, JournalError> {
    // Header inspection cannot create/recover/modify database sidecars. Reject
    // foreign/newer/WAL stores before SQLite is allowed to open them at all.
    let mut file = private_regular_file(path)?;
    let mut header = [0u8; 100];
    file.read_exact(&mut header)
        .map_err(|_| JournalError::InvalidStore("missing SQLite header".into()))?;
    if &header[..16] != b"SQLite format 3\0" {
        return Err(JournalError::InvalidStore("invalid SQLite header".into()));
    }
    let application = i32::from_be_bytes(header[68..72].try_into().expect("fixed header slice"));
    let version = i32::from_be_bytes(header[60..64].try_into().expect("fixed header slice"));
    if application != APPLICATION_ID {
        return Err(JournalError::ForeignStore { application });
    }
    if version > SCHEMA_VERSION {
        return Err(JournalError::NewerSchema { version });
    }
    if !(1..=SCHEMA_VERSION).contains(&version) {
        return Err(JournalError::InvalidStore(
            "unsupported schema version".into(),
        ));
    }
    if header[18] != 1 || header[19] != 1 {
        return Err(JournalError::InvalidStore(
            "journal requires DELETE format; WAL stores are not admitted".into(),
        ));
    }
    let connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|error| JournalError::sqlite("read-only preflight", error))?;
    connection
        .busy_timeout(BUSY_TIMEOUT)
        .map_err(|error| JournalError::sqlite("preflight busy timeout", error))?;
    let check: String = connection
        .query_row("PRAGMA quick_check(1)", [], |row| row.get(0))
        .map_err(|error| JournalError::sqlite("check journal integrity", error))?;
    if check != "ok" {
        return Err(JournalError::InvalidStore(format!(
            "integrity check: {check}"
        )));
    }
    validate_schema(&connection, version)?;
    Ok(version)
}

fn normalized_sql(sql: &str) -> String {
    sql.trim_end_matches(';')
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn validate_schema(connection: &Connection, version: i32) -> Result<(), JournalError> {
    for (kind, name, expected) in [
        ("table", "journal_meta", META_TABLE),
        ("table", "operation_events", EVENTS_TABLE),
        ("table", "operations", OPERATIONS_TABLE),
    ] {
        let sql: Option<String> = connection
            .query_row(
                "SELECT sql FROM sqlite_schema WHERE type = ?1 AND name = ?2",
                [kind, name],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| JournalError::sqlite("inspect journal schema", error))?;
        if sql.as_deref().map(normalized_sql) != Some(normalized_sql(expected)) {
            return Err(JournalError::InvalidStore(format!(
                "unrecognized {name} schema"
            )));
        }
    }
    let index: Option<String> = connection
        .query_row(
            "SELECT sql FROM sqlite_schema WHERE type = 'index' AND name = 'operation_events_session'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| JournalError::sqlite("inspect session index", error))?;
    let valid_index = match version {
        1 => index.is_none(),
        2 => index.as_deref().map(normalized_sql) == Some(normalized_sql(SESSION_INDEX)),
        _ => false,
    };
    if !valid_index {
        return Err(JournalError::InvalidStore(
            "unexpected session index version".into(),
        ));
    }
    let count: i64 = connection
        .query_row("SELECT COUNT(*) FROM journal_meta", [], |row| row.get(0))
        .map_err(|error| JournalError::sqlite("count journal identity", error))?;
    let (application, stored_version, producer): (String, i32, String) = connection
        .query_row(
            "SELECT application, schema_version, producer FROM journal_meta WHERE singleton = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|error| JournalError::sqlite("read journal identity", error))?;
    if count != 1 || application != APPLICATION || stored_version != version || producer.is_empty()
    {
        return Err(JournalError::InvalidStore(
            "journal identity mismatch".into(),
        ));
    }
    Ok(())
}

fn migrate(connection: &mut Connection, from: i32) -> Result<(), JournalError> {
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| JournalError::sqlite("begin schema migration", error))?;
    if from == 0 {
        for sql in [META_TABLE, EVENTS_TABLE, OPERATIONS_TABLE] {
            transaction
                .execute_batch(sql)
                .map_err(|error| JournalError::sqlite("create journal tables", error))?;
        }
        transaction
            .execute(
                "INSERT INTO journal_meta VALUES (1, ?1, 1, ?2)",
                [APPLICATION, concat!("vfa-tui/", env!("CARGO_PKG_VERSION"))],
            )
            .map_err(|error| JournalError::sqlite("initialize journal identity", error))?;
        transaction
            .pragma_update(None, "application_id", APPLICATION_ID)
            .map_err(|error| JournalError::sqlite("initialize application identity", error))?;
    }
    if from <= 1 {
        transaction
            .execute_batch(SESSION_INDEX)
            .map_err(|error| JournalError::sqlite("migrate session event index", error))?;
        transaction
            .execute(
                "UPDATE journal_meta SET schema_version = 2 WHERE singleton = 1",
                [],
            )
            .map_err(|error| JournalError::sqlite("advance journal schema", error))?;
        transaction
            .pragma_update(None, "user_version", SCHEMA_VERSION)
            .map_err(|error| JournalError::sqlite("advance SQLite schema identity", error))?;
    }
    transaction
        .commit()
        .map_err(|error| JournalError::sqlite("commit schema migration", error))
}
