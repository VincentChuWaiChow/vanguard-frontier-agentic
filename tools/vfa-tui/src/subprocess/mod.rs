pub mod executor;
pub mod program;
pub mod signal;
pub mod stream;

use std::time::{Duration, Instant};

use tokio::process::Child;
use tokio::sync::mpsc;

#[allow(unused_imports)]
pub use executor::SubprocessExecutor;
#[allow(unused_imports)]
pub use stream::{OutputLine, OutputStream};

/// Handle to a running subprocess with streaming output capture.
pub struct SubprocessHandle {
    child: Child,
    stdout_rx: mpsc::Receiver<OutputLine>,
    stderr_rx: mpsc::Receiver<OutputLine>,
    start_time: Instant,
    timeout: Duration,
    finished: bool,
    exit_code: Option<i32>,
}

impl SubprocessHandle {
    pub(crate) fn new(
        child: Child,
        stdout_rx: mpsc::Receiver<OutputLine>,
        stderr_rx: mpsc::Receiver<OutputLine>,
        timeout: Duration,
    ) -> Self {
        Self {
            child,
            stdout_rx,
            stderr_rx,
            start_time: Instant::now(),
            timeout,
            finished: false,
            exit_code: None,
        }
    }

    /// Cancel the subprocess by sending SIGTERM, then SIGKILL if needed.
    pub async fn cancel(&mut self) -> anyhow::Result<()> {
        if !self.finished {
            signal::graceful_kill(&mut self.child).await?;
            self.finished = true;
        }
        Ok(())
    }

    /// Try to receive the next stdout line without blocking.
    pub fn try_recv_stdout(&mut self) -> Option<OutputLine> {
        self.stdout_rx.try_recv().ok()
    }

    /// Try to receive the next stderr line without blocking.
    pub fn try_recv_stderr(&mut self) -> Option<OutputLine> {
        self.stderr_rx.try_recv().ok()
    }

    /// Returns true if the subprocess is still running.
    pub fn is_running(&self) -> bool {
        !self.finished
    }

    /// Returns the exit code if the subprocess has finished.
    pub fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }

    /// Check if the subprocess has exceeded its timeout (synchronous check).
    /// This does not kill the process; it only checks the elapsed time.
    pub fn is_timed_out(&self) -> bool {
        !self.finished && self.start_time.elapsed() > self.timeout
    }

    /// Non-blocking check if the subprocess has exited.
    /// If it has, sets `finished` and `exit_code` and returns true.
    /// This allows synchronous tick loops to detect completion.
    pub fn try_poll_exit(&mut self) -> bool {
        if self.finished {
            return true;
        }
        match self.child.try_wait() {
            Ok(Some(status)) => {
                self.finished = true;
                self.exit_code = Some(status.code().unwrap_or(-1));
                true
            }
            _ => false,
        }
    }

    /// Check if the subprocess has exceeded its timeout.
    /// Returns true if timed out (and kills the process).
    pub async fn check_timeout(&mut self) -> bool {
        if self.finished {
            return false;
        }
        if self.start_time.elapsed() > self.timeout {
            self.cancel().await.ok();
            self.finished = true;
            true
        } else {
            false
        }
    }

    /// Wait for the subprocess to complete and return its exit code.
    pub async fn wait(&mut self) -> anyhow::Result<i32> {
        if let Some(code) = self.exit_code {
            return Ok(code);
        }

        let status = self.child.wait().await?;
        self.finished = true;
        let code = status.code().unwrap_or(-1);
        self.exit_code = Some(code);
        Ok(code)
    }
}
