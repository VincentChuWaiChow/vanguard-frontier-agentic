//! Minimal owned contracts for the installation-free report service host.
//!
//! Request IDs are local to one host, not Desktop operation/session/context IDs.
//! Native foreground delivery and full Desktop lifecycle remain adapter work.

use std::fmt;

use crate::headless::ReportResult;

/// Pending, active and unacknowledged terminal requests share this same budget.
pub const MAX_RETAINED_REQUESTS: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RequestId(pub(crate) u64);

impl RequestId {
    pub fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for RequestId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EngineError {
    #[error("engine admission/retention budget is busy")]
    Busy,
    #[error("engine shutdown has been requested")]
    ShuttingDown,
    #[error("engine is stopped or unavailable")]
    Stopped,
    #[error("unknown or explicitly released request {0}")]
    UnknownRequest(RequestId),
    #[error("request {0} has no terminal outcome to release")]
    NotTerminal(RequestId),
    #[error("request ID space exhausted")]
    IdExhausted,
    #[error("engine shared state is poisoned; refusing reuse")]
    StatePoisoned,
    #[error("engine host startup failed: {0}")]
    Startup(String),
    #[error("engine work failed; host will not be reused: {0}")]
    WorkFailed(String),
    #[error("engine host thread panicked")]
    HostPanicked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownChoice {
    Wait,
    Cancel,
}

/// Cancellation is admission/observation control, not evidence of cleanup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cancellation {
    PreventedLaunch,
    RequestedForStartedWork,
    AlreadyTerminal,
}

#[derive(Debug)]
pub struct ReportOutcome {
    pub request_id: RequestId,
    pub completion: ReportCompletion,
}

#[derive(Debug)]
pub enum ReportCompletion {
    /// Existing synchronous report work cannot be interrupted once started.
    /// Its actual observed result wins, with cancellation recorded separately.
    Report {
        report: Box<ReportResult>,
        cancellation_requested: bool,
    },
    CancelledBeforeStart,
    Failed(EngineError),
    #[cfg(test)]
    Probe {
        kind: &'static str,
        cancellation_requested: bool,
    },
}
