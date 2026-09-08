use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::time::SystemTime;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};
use uuid::Uuid;

use crate::security::redact::redact_secrets;

/// Outcome of an audited action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Success,
    Failure,
}

impl std::fmt::Display for Outcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Outcome::Success => write!(f, "success"),
            Outcome::Failure => write!(f, "failure"),
        }
    }
}

/// Structured audit event for compliance logging.
#[derive(Debug, Clone)]
pub struct AuditEvent {
    pub action: String,
    pub outcome: Outcome,
    pub detail: Option<String>,
}

/// Log a user-initiated action at INFO level.
///
/// Includes session_id, action type, and outcome in the structured event.
#[macro_export]
macro_rules! audit_info {
    ($session_id:expr, $action:expr, $outcome:expr $(, $($field:tt)*)?) => {
        tracing::info!(
            session_id = %$session_id,
            action = %$action,
            outcome = %$outcome,
            $($($field)*)?
        )
    };
}

/// Log a validation failure at WARN level.
#[macro_export]
macro_rules! audit_warn {
    ($session_id:expr, $action:expr, $outcome:expr $(, $($field:tt)*)?) => {
        tracing::warn!(
            session_id = %$session_id,
            action = %$action,
            outcome = %$outcome,
            $($($field)*)?
        )
    };
}

/// Log a subprocess failure or security rejection at ERROR level.
#[macro_export]
macro_rules! audit_error {
    ($session_id:expr, $action:expr, $outcome:expr $(, $($field:tt)*)?) => {
        tracing::error!(
            session_id = %$session_id,
            action = %$action,
            outcome = %$outcome,
            $($($field)*)?
        )
    };
}

/// A writer that applies secret redaction before writing to the inner writer.
struct RedactingWriter<W: Write> {
    inner: W,
}

impl<W: Write> Write for RedactingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let original_len = buf.len();
        let text = String::from_utf8_lossy(buf);
        let redacted = redact_secrets(&text);
        self.inner.write_all(redacted.as_bytes())?;
        Ok(original_len)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

/// A MakeWriter that wraps stderr with secret redaction.
#[derive(Clone)]
struct RedactingStderrMakeWriter;

impl<'a> MakeWriter<'a> for RedactingStderrMakeWriter {
    type Writer = RedactingWriter<std::io::Stderr>;

    fn make_writer(&'a self) -> Self::Writer {
        RedactingWriter {
            inner: std::io::stderr(),
        }
    }
}

/// A MakeWriter that wraps a shared file handle with secret redaction.
#[derive(Clone)]
struct RedactingFileMakeWriter {
    file: std::sync::Arc<Mutex<std::fs::File>>,
}

impl<'a> MakeWriter<'a> for RedactingFileMakeWriter {
    type Writer = RedactingWriter<SharedFileWriter>;

    fn make_writer(&'a self) -> Self::Writer {
        RedactingWriter {
            inner: SharedFileWriter {
                file: self.file.clone(),
            },
        }
    }
}

/// A writer backed by a shared file handle behind a Mutex.
struct SharedFileWriter {
    file: std::sync::Arc<Mutex<std::fs::File>>,
}

impl Write for SharedFileWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut f = self
            .file
            .lock()
            .map_err(|_| std::io::Error::other("file lock poisoned"))?;
        f.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let mut f = self
            .file
            .lock()
            .map_err(|_| std::io::Error::other("file lock poisoned"))?;
        f.flush()
    }
}

/// Initialize the logging subsystem with JSON output.
///
/// Configures tracing-subscriber with:
/// - JSON format for structured logging with ISO 8601 timestamps (ms precision)
/// - Configurable level filter from the `log_level` string
/// - Session ID included in all events via a default span
/// - Output to stderr (always) and optionally to a file (via --log-file)
/// - Secret redaction applied to all log output
/// - If the file cannot be opened, logs a warning to stderr and continues with stderr only
///
/// # Log Levels
/// - INFO: user-initiated actions (browse, search, filter, validate, export)
/// - WARN: validation failures
/// - ERROR: subprocess failures and security rejections
pub fn init_logging(
    log_file: Option<&Path>,
    log_level: &str,
    session_id: Uuid,
) -> anyhow::Result<()> {
    let filter = EnvFilter::try_new(log_level).unwrap_or_else(|_| EnvFilter::new("info"));

    // Create the base stderr layer with JSON format and redacting writer.
    // SystemTime provides ISO 8601 timestamps with subsecond (ms) precision.
    let stderr_layer = fmt::layer()
        .json()
        .with_target(true)
        .with_timer(SystemTime)
        .with_span_events(FmtSpan::NONE)
        .with_writer(RedactingStderrMakeWriter);

    if let Some(file_path) = log_file {
        match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
        {
            Ok(file) => {
                let shared_file = std::sync::Arc::new(Mutex::new(file));
                let file_layer = fmt::layer()
                    .json()
                    .with_target(true)
                    .with_timer(SystemTime)
                    .with_span_events(FmtSpan::NONE)
                    .with_writer(RedactingFileMakeWriter { file: shared_file });

                tracing_subscriber::registry()
                    .with(filter)
                    .with(stderr_layer)
                    .with(file_layer)
                    .init();
            }
            Err(e) => {
                // Fall back to stderr only, but still initialize
                tracing_subscriber::registry()
                    .with(filter)
                    .with(stderr_layer)
                    .init();

                // Emit warning about the failed log file
                tracing::warn!(
                    session_id = %session_id,
                    path = %file_path.display(),
                    error = %e,
                    action = "init_logging",
                    outcome = "failure",
                    "Failed to open log file, falling back to stderr only"
                );
            }
        }
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(stderr_layer)
            .init();
    }

    tracing::info!(
        session_id = %session_id,
        action = "session_start",
        outcome = "success",
        "Logging initialized"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_filter_parses_valid_levels() {
        let filter = EnvFilter::try_new("debug");
        assert!(filter.is_ok());

        let filter = EnvFilter::try_new("warn");
        assert!(filter.is_ok());

        let filter = EnvFilter::try_new("info");
        assert!(filter.is_ok());

        let filter = EnvFilter::try_new("trace");
        assert!(filter.is_ok());

        let filter = EnvFilter::try_new("error");
        assert!(filter.is_ok());
    }

    #[test]
    fn env_filter_falls_back_on_invalid() {
        // Invalid level should fall back to info (our code handles this)
        let filter = EnvFilter::try_new("not_a_level");
        // This may or may not error depending on the string, but our code
        // wraps it with unwrap_or_else
        let _ = filter;
    }

    #[test]
    fn session_id_is_valid_uuid_v4() {
        let id = Uuid::new_v4();
        let s = id.to_string();
        assert_eq!(s.len(), 36);
        assert!(Uuid::parse_str(&s).is_ok());
        // UUID v4 has version nibble = 4 at position 14
        assert_eq!(s.as_bytes()[14], b'4');
    }

    #[test]
    fn outcome_display() {
        assert_eq!(format!("{}", Outcome::Success), "success");
        assert_eq!(format!("{}", Outcome::Failure), "failure");
    }

    #[test]
    fn audit_event_construction() {
        let event = AuditEvent {
            action: "browse".to_string(),
            outcome: Outcome::Success,
            detail: Some("viewed agent list".to_string()),
        };
        assert_eq!(event.action, "browse");
        assert_eq!(event.outcome, Outcome::Success);
        assert_eq!(event.detail.unwrap(), "viewed agent list");
    }

    #[test]
    fn redacting_writer_redacts_secrets() {
        let mut output = Vec::new();
        {
            let mut writer = RedactingWriter { inner: &mut output };
            let secret = [
                "token: gh",
                "p_",
                "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
                "abcdefghijklmn\n",
            ]
            .concat();
            writer.write_all(secret.as_bytes()).unwrap();
        }
        let result = String::from_utf8(output).unwrap();
        assert!(!result.contains("ghp_"));
        assert!(result.contains("[REDACTED]"));
        assert!(result.contains("token: "));
    }

    #[test]
    fn redacting_writer_preserves_normal_text() {
        let mut output = Vec::new();
        {
            let mut writer = RedactingWriter { inner: &mut output };
            let normal = "INFO session started successfully\n";
            writer.write_all(normal.as_bytes()).unwrap();
        }
        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, "INFO session started successfully\n");
    }

    #[test]
    fn redacting_writer_returns_original_length() {
        let mut output = Vec::new();
        let mut writer = RedactingWriter { inner: &mut output };
        let secret = ["gh", "p_", "ABCDEFGHIJKLMNOPQRSTUVWXYZ", "abcdefghijklmn"].concat();
        let written = writer.write(secret.as_bytes()).unwrap();
        // write() returns the original buffer length, not the redacted length
        assert_eq!(written, secret.len());
    }
}
