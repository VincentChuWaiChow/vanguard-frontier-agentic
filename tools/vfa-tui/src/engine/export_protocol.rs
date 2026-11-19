//! Incremental validator for the authoritative Node `vfa-export/1` apply stream.
//!
//! This module performs no I/O or authorization. The controller must supply the
//! current expected plan, raw stdout before display loss/redaction, and actual
//! child exit status. Validated effects survive subsequent protocol failures;
//! absence of an outcome never proves absence of a write. Do not expose these
//! raw protocol values as display/audit projections without their own redaction.
//!
//! Limits here bound wire input and framing, not the future shared in-flight
//! stdout/stderr budget, journal persistence, or process cleanup.

use std::collections::HashSet;

use serde::{Deserialize, Deserializer};
use thiserror::Error;

pub const MAX_RECORD_BYTES: usize = 64 * 1024;
pub const MAX_PROTOCOL_BYTES: usize = 8 * 1024 * 1024;

/// Final artifact metadata supplied by the authoritative plan, not re-rendered.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FinalArtifact {
    pub sha256: String,
    pub size: u64,
    pub mode: u32,
}

/// Projection of an already validated plan. Its digest is not authorization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedFile {
    pub path: String,
    pub final_artifact: FinalArtifact,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileOutcome {
    pub file_index: usize,
    pub file: ExpectedFile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApplyStatus {
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FailureCode {
    WriteFailed,
    OutputFailed,
    ProtocolLimit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplySummary {
    pub status: ApplyStatus,
    pub planned: usize,
    pub written: usize,
    pub failed_index: Option<usize>,
    pub unattempted: usize,
    pub uncertain: bool,
    pub code: Option<FailureCode>,
}

/// Fixed diagnostics deliberately exclude untrusted protocol text and paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ProtocolError {
    #[error("invalid expected export plan projection")]
    InvalidPlan,
    #[error("export protocol record exceeds 64 KiB including newline")]
    RecordLimit,
    #[error("export protocol exceeds 8 MiB")]
    ProtocolLimit,
    #[error("invalid export protocol JSON or schema")]
    InvalidRecord,
    #[error("export protocol record is out of order")]
    OutOfOrder,
    #[error("export protocol differs from the expected plan")]
    PlanMismatch,
    #[error("export summary contradicts observed outcomes")]
    InvalidSummary,
    #[error("export protocol is incomplete")]
    Incomplete,
    #[error("export child exit is unsuccessful or unknown")]
    ChildExit,
    #[error("export producer reported failure")]
    ApplyFailed,
    #[error("data follows the terminal export summary")]
    TrailingData,
    #[error("export protocol consumer is closed")]
    Closed,
}

// Unlike a plain Option field, this requires the key even when its value is null.
fn required_nullable<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

#[derive(Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
enum Record {
    #[serde(rename = "apply-start")]
    Start {
        protocol: String,
        plan_digest: String,
        seq: usize,
        file_count: usize,
    },
    #[serde(rename = "file-outcome")]
    File {
        protocol: String,
        plan_digest: String,
        seq: usize,
        file_index: usize,
        path: String,
        status: String,
        #[serde(rename = "final")]
        final_artifact: FinalArtifact,
    },
    #[serde(rename = "apply-summary")]
    Summary {
        protocol: String,
        plan_digest: String,
        seq: usize,
        status: ApplyStatus,
        planned: usize,
        written: usize,
        #[serde(deserialize_with = "required_nullable")]
        failed_index: Option<usize>,
        unattempted: usize,
        uncertain: bool,
        #[serde(deserialize_with = "required_nullable")]
        code: Option<FailureCode>,
    },
}

/// One apply attempt, with latched failures and retained validated observations.
pub struct ExportProtocol {
    digest: String,
    expected: Vec<ExpectedFile>,
    frame: Vec<u8>,
    received: usize,
    started: bool,
    finished: bool,
    failure: Option<ProtocolError>,
    outcomes: Vec<FileOutcome>,
    summary: Option<ApplySummary>,
}

impl ExportProtocol {
    pub fn new(digest: String, expected: Vec<ExpectedFile>) -> Result<Self, ProtocolError> {
        let mut paths = HashSet::new();
        if !valid_hash(&digest) || expected.is_empty() || expected.len() > 10000 {
            return Err(ProtocolError::InvalidPlan);
        }
        for file in &expected {
            if !valid_path(&file.path)
                || !paths.insert(&file.path)
                || !valid_hash(&file.final_artifact.sha256)
                || file.final_artifact.size > 256 * 1024 * 1024
                || file.final_artifact.mode > 0o1777
            {
                return Err(ProtocolError::InvalidPlan);
            }
        }
        Ok(Self {
            digest,
            expected,
            frame: Vec::new(),
            received: 0,
            started: false,
            finished: false,
            failure: None,
            outcomes: Vec::new(),
            summary: None,
        })
    }

    /// Consume bounded records without copying a caller's entire chunk. On any
    /// error the caller must stop apply admission and arrange safe child cleanup.
    pub fn feed(&mut self, mut bytes: &[u8]) -> Result<(), ProtocolError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        if self.finished {
            return self.reject(ProtocolError::Closed);
        }
        while !bytes.is_empty() {
            if self.summary.is_some() {
                return self.reject(ProtocolError::TrailingData);
            }
            let newline = bytes.iter().position(|byte| *byte == b'\n');
            let count = newline.map_or(bytes.len(), |index| index + 1);
            // An unterminated frame must still leave room for its mandatory LF.
            let available = MAX_RECORD_BYTES - self.frame.len();
            if count > available || (newline.is_none() && count == available) {
                return self.reject(ProtocolError::RecordLimit);
            }
            if count > MAX_PROTOCOL_BYTES - self.received {
                return self.reject(ProtocolError::ProtocolLimit);
            }
            self.received += count;
            self.frame.extend_from_slice(&bytes[..count]);
            bytes = &bytes[count..];
            if newline.is_some() {
                let result = self.accept_record();
                self.frame.clear();
                if let Err(error) = result {
                    return self.reject(error);
                }
            }
        }
        Ok(())
    }

    /// Call only at stdout EOF with the actual reaped child's exit code. This
    /// local verdict is not a journal receipt or descendant-cleanup guarantee.
    pub fn finish(&mut self, exit_code: Option<i32>) -> Result<&ApplySummary, ProtocolError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        if self.finished {
            return self.reject(ProtocolError::Closed);
        }
        self.finished = true;
        if !self.frame.is_empty() || self.summary.is_none() {
            return self.reject(ProtocolError::Incomplete);
        }
        if exit_code != Some(0) {
            return self.reject(ProtocolError::ChildExit);
        }
        if self.summary.as_ref().map(|s| s.status) != Some(ApplyStatus::Succeeded) {
            return self.reject(ProtocolError::ApplyFailed);
        }
        // Established above; avoid a panic even if this method is later changed.
        self.summary.as_ref().ok_or(ProtocolError::Incomplete)
    }

    pub fn outcomes(&self) -> &[FileOutcome] {
        &self.outcomes
    }

    pub fn summary(&self) -> Option<&ApplySummary> {
        self.summary.as_ref()
    }

    pub fn buffered_bytes(&self) -> usize {
        self.frame.len()
    }

    pub fn bytes_received(&self) -> usize {
        self.received
    }

    fn reject<T>(&mut self, error: ProtocolError) -> Result<T, ProtocolError> {
        self.failure = Some(error);
        self.frame.clear();
        Err(error)
    }

    fn accept_record(&mut self) -> Result<(), ProtocolError> {
        let record: Record =
            serde_json::from_slice(&self.frame).map_err(|_| ProtocolError::InvalidRecord)?;
        let (protocol, digest, seq) = match &record {
            Record::Start {
                protocol,
                plan_digest,
                seq,
                ..
            }
            | Record::File {
                protocol,
                plan_digest,
                seq,
                ..
            }
            | Record::Summary {
                protocol,
                plan_digest,
                seq,
                ..
            } => (protocol, plan_digest, seq),
        };
        if protocol != "vfa-export/1" || digest != &self.digest {
            return Err(ProtocolError::PlanMismatch);
        }
        let next = if self.started {
            self.outcomes.len() + 2
        } else {
            1
        };
        if *seq != next {
            return Err(ProtocolError::OutOfOrder);
        }
        match record {
            Record::Start { file_count, .. } if !self.started => {
                if file_count != self.expected.len() {
                    return Err(ProtocolError::PlanMismatch);
                }
                self.started = true;
            }
            Record::File {
                file_index,
                path,
                status,
                final_artifact,
                ..
            } if self.started => {
                let expected = self
                    .expected
                    .get(self.outcomes.len())
                    .ok_or(ProtocolError::OutOfOrder)?;
                if file_index != self.outcomes.len()
                    || path != expected.path
                    || final_artifact != expected.final_artifact
                    || status != "WRITTEN"
                {
                    return Err(ProtocolError::PlanMismatch);
                }
                self.outcomes.push(FileOutcome {
                    file_index,
                    file: expected.clone(),
                });
            }
            Record::Summary {
                status,
                planned,
                written,
                failed_index,
                unattempted,
                uncertain,
                code,
                ..
            } if self.started => {
                if planned != self.expected.len() || written != self.outcomes.len() {
                    return Err(ProtocolError::InvalidSummary);
                }
                let remaining = planned - written;
                let consistent = match status {
                    ApplyStatus::Succeeded => {
                        remaining == 0
                            && failed_index.is_none()
                            && unattempted == 0
                            && !uncertain
                            && code.is_none()
                    }
                    ApplyStatus::Failed => {
                        code.is_some()
                            && match failed_index {
                                Some(index) => {
                                    remaining > 0
                                        && index == written
                                        && uncertain
                                        && unattempted == remaining - 1
                                }
                                None => !uncertain && unattempted == remaining,
                            }
                    }
                };
                if !consistent {
                    return Err(ProtocolError::InvalidSummary);
                }
                self.summary = Some(ApplySummary {
                    status,
                    planned,
                    written,
                    failed_index,
                    unattempted,
                    uncertain,
                    code,
                });
            }
            _ => return Err(ProtocolError::OutOfOrder),
        }
        Ok(())
    }
}

fn valid_hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_path(value: &str) -> bool {
    // Match the producer's slash-separated relative structure, not a display
    // policy. Interior POSIX filenames may contain colons, backslashes or LF.
    let drive_prefix = value
        .as_bytes()
        .first()
        .is_some_and(u8::is_ascii_alphabetic)
        && value.as_bytes().get(1) == Some(&b':');
    !value.is_empty()
        && !value.starts_with('\\')
        && !drive_prefix
        && !value.contains('\0')
        && value
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}
