use std::path::Path;
use std::process::Stdio;
use std::time::{Duration, Instant};

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

use crate::security::redact::sanitized_child_env;

use super::program::ProgramSpec;
use super::stream::{OutputLine, OutputStream};
use super::SubprocessHandle;

/// Maximum length of a single output line in bytes. Lines exceeding this are truncated.
const MAX_LINE_LENGTH: usize = 4096;

/// Spawns subprocesses without using a shell.
pub struct SubprocessExecutor;

impl SubprocessExecutor {
    /// Spawn a subprocess with the given command, arguments, working directory, and timeout.
    ///
    /// The subprocess inherits a sanitized environment (secrets stripped) and captures
    /// both stdout and stderr as line-by-line streams.
    ///
    /// Note: Shell injection is prevented by using `Command::new` (no shell).
    /// User-provided arguments should be validated via `ExportCommand::validate()`
    /// before reaching this layer.
    pub async fn spawn(
        command: &str,
        args: &[String],
        working_dir: &Path,
        timeout: Duration,
    ) -> anyhow::Result<SubprocessHandle> {
        let env = sanitized_child_env();

        let mut cmd = Command::new(command);
        cmd.args(args)
            .current_dir(working_dir)
            .env_clear()
            .envs(env);

        Self::spawn_command(cmd, timeout).await
    }

    /// Execute resolved native argv and only its explicitly selected environment.
    /// Uses legacy capture/handle behavior: this is not D7 stream-budget or
    /// descendant cleanup acceptance. Typed launch defaults stdin to null.
    pub async fn spawn_spec(
        program: &ProgramSpec,
        timeout: Duration,
    ) -> anyhow::Result<SubprocessHandle> {
        Self::spawn_command(Command::from(program.command()?), timeout).await
    }

    async fn spawn_command(
        mut cmd: Command,
        timeout: Duration,
    ) -> anyhow::Result<SubprocessHandle> {
        cmd.kill_on_drop(true)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        #[cfg(unix)]
        cmd.process_group(0);

        let mut child = cmd.spawn()?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("stdout not captured from subprocess"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow::anyhow!("stderr not captured from subprocess"))?;

        let (stdout_tx, stdout_rx) = mpsc::channel(10_000);
        let (stderr_tx, stderr_rx) = mpsc::channel(10_000);

        // Spawn stdout reader task
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let content = if line.len() > MAX_LINE_LENGTH {
                    // Find a valid UTF-8 boundary at or before MAX_LINE_LENGTH
                    let mut end = MAX_LINE_LENGTH;
                    while end > 0 && !line.is_char_boundary(end) {
                        end -= 1;
                    }
                    let mut truncated = line[..end].to_string();
                    truncated.push_str(" [truncated]");
                    truncated
                } else {
                    line
                };
                let output_line = OutputLine {
                    content,
                    timestamp: Instant::now(),
                    stream: OutputStream::Stdout,
                };
                // Use try_send to apply backpressure without blocking;
                // if the buffer is full, drop the line (acceptable for TUI display).
                if stdout_tx.try_send(output_line).is_err() {
                    break;
                }
            }
        });

        // Spawn stderr reader task
        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let content = if line.len() > MAX_LINE_LENGTH {
                    // Find a valid UTF-8 boundary at or before MAX_LINE_LENGTH
                    let mut end = MAX_LINE_LENGTH;
                    while end > 0 && !line.is_char_boundary(end) {
                        end -= 1;
                    }
                    let mut truncated = line[..end].to_string();
                    truncated.push_str(" [truncated]");
                    truncated
                } else {
                    line
                };
                let output_line = OutputLine {
                    content,
                    timestamp: Instant::now(),
                    stream: OutputStream::Stderr,
                };
                // Use try_send to apply backpressure without blocking;
                // if the buffer is full, drop the line (acceptable for TUI display).
                if stderr_tx.try_send(output_line).is_err() {
                    break;
                }
            }
        });

        Ok(SubprocessHandle::new(child, stdout_rx, stderr_rx, timeout))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn spawn_echo_captures_stdout() {
        let tmp = TempDir::new().unwrap();
        let mut handle = SubprocessExecutor::spawn(
            "echo",
            &["hello world".to_string()],
            tmp.path(),
            Duration::from_secs(10),
        )
        .await
        .unwrap();

        let code = handle.wait().await.unwrap();
        assert_eq!(code, 0);

        // Give the reader a moment to deliver
        tokio::time::sleep(Duration::from_millis(50)).await;

        let mut lines = Vec::new();
        while let Some(line) = handle.try_recv_stdout() {
            lines.push(line.content);
        }
        assert!(lines.contains(&"hello world".to_string()));
    }

    #[tokio::test]
    async fn spawn_false_returns_nonzero() {
        let tmp = TempDir::new().unwrap();
        let mut handle =
            SubprocessExecutor::spawn("false", &[], tmp.path(), Duration::from_secs(10))
                .await
                .unwrap();

        let code = handle.wait().await.unwrap();
        assert_ne!(code, 0);
    }

    #[tokio::test]
    async fn spawn_captures_stderr() {
        let tmp = TempDir::new().unwrap();
        let mut handle = SubprocessExecutor::spawn(
            "sh",
            &["-c".to_string(), "echo error >&2".to_string()],
            tmp.path(),
            Duration::from_secs(10),
        )
        .await
        .unwrap();

        let code = handle.wait().await.unwrap();
        assert_eq!(code, 0);

        tokio::time::sleep(Duration::from_millis(50)).await;

        let mut lines = Vec::new();
        while let Some(line) = handle.try_recv_stderr() {
            lines.push(line.content);
        }
        assert!(lines.contains(&"error".to_string()));
    }
}
