//! UNREDACTED internal transport, not safe UI/audit output. Direct-root stop is
//! not descendant ownership. Dropping a handle does not abort its supervisor;
//! runtime destruction is not shutdown/cleanup acceptance.
//!
//! Conservative reservation, NOT RSS/allocator metering: each pipe reserves
//! 4KiB read + 4KiB prefix + 4KiB construction. Export reserves32*64KiB for
//! bounded frame growth, typed wire strings and serde escaped-token/error
//! scratch (fixed schema, default recursion limit). Expected-plan/validated
//! effect models are semantic state excluded from raw transport accounting.
//! Fixed reservations persist with shared state. Remaining capacity is split
//! equally per pipe; payloads are exactly4KiB, charged BEFORE allocation.
//! Records are not Clone. Retained consumer tails retain permits; arbitrary
//! external copies of borrowed text are outside this contract.

use super::program::ProgramSpec;
use crate::engine::export_protocol::{
    ExpectedFile, ExportProtocol, ProtocolError, MAX_RECORD_BYTES,
};
use std::io;
use std::process::Stdio;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::process::{Child, ChildStderr, ChildStdout, Command};
use tokio::sync::{mpsc, watch};
use tokio::task::JoinHandle;

pub const RAW_BUDGET: usize = 8 * 1024 * 1024;
const CHUNK: usize = 4096;
const QUEUE_SLOTS: usize = 1024;
const FIXED_PIPES: usize = 2 * 3 * CHUNK;
const PARSER_RESERVATION: usize = 32 * MAX_RECORD_BYTES;

pub enum OutputProtocol {
    Text,
    Export {
        digest: String,
        files: Vec<ExpectedFile>,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    Requested,
    Timeout,
    Protocol,
    Io,
}
/// Fixed diagnostics exclude child-controlled bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoOperation {
    StdoutRead,
    StderrRead,
    RootWait,
    RootStop,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IoFailure {
    pub operation: IoOperation,
    pub kind: io::ErrorKind,
}
#[derive(Debug, Default, Clone, Copy)]
pub struct StreamStats {
    pub bytes_read: u64,
    /// True only for an observed zero-byte read, never a read failure.
    pub eof: bool,
    /// Whole source record loss, including LF when present.
    pub dropped_bytes: u64,
    pub dropped_records: u64,
    /// Source content bytes omitted by fixed UTF8 prefix; excludes LF.
    pub truncated_bytes: u64,
    pub truncated_records: u64,
}
#[derive(Debug, Clone, Copy)]
pub struct BudgetSnapshot {
    pub limit: usize,
    pub current: usize,
    pub peak: usize,
    pub fixed_reservation: usize,
    pub per_stream_payload_limit: usize,
    pub stdout_payload: usize,
    pub stderr_payload: usize,
    pub stdout_peak_payload: usize,
    pub stderr_peak_payload: usize,
}
struct Budget {
    fixed: usize,
    per_stream: usize,
    current: AtomicUsize,
    peak: AtomicUsize,
    payload: [AtomicUsize; 2],
    payload_peak: [AtomicUsize; 2],
}
impl Budget {
    fn new(export: bool) -> Arc<Self> {
        let fixed = FIXED_PIPES + if export { PARSER_RESERVATION } else { 0 };
        Arc::new(Self {
            fixed,
            per_stream: ((RAW_BUDGET - fixed) / 2 / CHUNK) * CHUNK,
            current: AtomicUsize::new(fixed),
            peak: AtomicUsize::new(fixed),
            payload: [AtomicUsize::new(0), AtomicUsize::new(0)],
            payload_peak: [AtomicUsize::new(0), AtomicUsize::new(0)],
        })
    }
    fn acquire(self: &Arc<Self>, stream: usize) -> Option<PayloadPermit> {
        let mut previous = self.payload[stream].load(Ordering::SeqCst);
        loop {
            if previous > self.per_stream - CHUNK {
                return None;
            }
            match self.payload[stream].compare_exchange_weak(
                previous,
                previous + CHUNK,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => break,
                Err(current) => previous = current,
            }
        }
        self.payload_peak[stream].fetch_max(previous + CHUNK, Ordering::SeqCst);
        let current = self.current.fetch_add(CHUNK, Ordering::SeqCst) + CHUNK;
        self.peak.fetch_max(current, Ordering::SeqCst);
        Some(PayloadPermit {
            budget: Arc::clone(self),
            stream,
        })
    }
    fn snapshot(&self) -> BudgetSnapshot {
        BudgetSnapshot {
            limit: RAW_BUDGET,
            current: self.current.load(Ordering::SeqCst),
            peak: self.peak.load(Ordering::SeqCst),
            fixed_reservation: self.fixed,
            per_stream_payload_limit: self.per_stream,
            stdout_payload: self.payload[0].load(Ordering::SeqCst),
            stderr_payload: self.payload[1].load(Ordering::SeqCst),
            stdout_peak_payload: self.payload_peak[0].load(Ordering::SeqCst),
            stderr_peak_payload: self.payload_peak[1].load(Ordering::SeqCst),
        }
    }
}
struct PayloadPermit {
    budget: Arc<Budget>,
    stream: usize,
}
impl Drop for PayloadPermit {
    fn drop(&mut self) {
        // Global release first prevents transient overstatement across a
        // consumer release and the single supervisor's next acquisition.
        self.budget.current.fetch_sub(CHUNK, Ordering::SeqCst);
        self.budget.payload[self.stream].fetch_sub(CHUNK, Ordering::SeqCst);
    }
}
/// Internal unredacted record, deliberately not Clone or Debug.
pub struct OutputRecord {
    // Declaration order frees allocation before releasing its permit.
    payload: Box<[u8; CHUNK]>,
    _permit: PayloadPermit,
    len: usize,
    pub truncated: bool,
}
impl OutputRecord {
    pub fn text(&self) -> &str {
        // emit() exclusively writes this private payload using valid UTF8 spans.
        std::str::from_utf8(&self.payload[..self.len]).unwrap_or("")
    }
}
pub struct OutputCompletion {
    pub exit_code: Option<i32>,
    pub stop_reason: Option<StopReason>,
    pub stdout: StreamStats,
    pub stderr: StreamStats,
    pub root_wait_observed: bool,
    pub root_reaped: bool,
    pub descendant_cleanup_verified: bool,
    pub io_error: Option<IoFailure>,
    pub protocol_error: Option<ProtocolError>,
    /// Retains validated effects even when later protocol/transport input fails.
    export: Option<ExportProtocol>,
    _budget: Arc<Budget>,
}
impl OutputCompletion {
    /// Read-only protocol observation; raw framing cannot be extracted and fed
    /// again outside its retained reservation by mutating this completion.
    pub fn export(&self) -> Option<&ExportProtocol> {
        self.export.as_ref()
    }
}
pub struct BoundedOutputHandle {
    stdout: Option<mpsc::Receiver<OutputRecord>>,
    stderr: Option<mpsc::Receiver<OutputRecord>>,
    stop: watch::Sender<bool>,
    terminal: watch::Receiver<Option<Arc<OutputCompletion>>>,
    supervisor: JoinHandle<()>,
    joined: bool,
    join_failed: bool,
    budget: Arc<Budget>,
}
impl BoundedOutputHandle {
    pub fn take_stdout(&mut self) -> Option<mpsc::Receiver<OutputRecord>> {
        self.stdout.take()
    }
    pub fn take_stderr(&mut self) -> Option<mpsc::Receiver<OutputRecord>> {
        self.stderr.take()
    }
    /// Direct-root stop only; caller must still observe wait().
    pub fn request_root_stop(&self) {
        self.stop.send_replace(true);
    }
    pub fn budget(&self) -> BudgetSnapshot {
        self.budget.snapshot()
    }
    /// JoinHandle remains in self across await: cancelling this observer cannot
    /// detach readers or discard the reliable terminal result. Repeated waits
    /// share the same Arc, after observing the entire supervisor's completion.
    pub async fn wait(&mut self) -> anyhow::Result<Arc<OutputCompletion>> {
        if !self.joined {
            self.join_failed = (&mut self.supervisor).await.is_err();
            self.joined = true;
        }
        if self.join_failed {
            anyhow::bail!("bounded output supervisor failed");
        }
        self.terminal
            .borrow()
            .as_ref()
            .map(Arc::clone)
            .ok_or_else(|| anyhow::anyhow!("bounded output supervisor closed without completion"))
    }
}
struct Display {
    prefix: [u8; CHUNK],
    len: usize,
    content_bytes: u64,
    stats: StreamStats,
    stream: usize,
    tx: mpsc::Sender<OutputRecord>,
    budget: Arc<Budget>,
}
impl Display {
    fn new(stream: usize, tx: mpsc::Sender<OutputRecord>, budget: Arc<Budget>) -> Self {
        Self {
            prefix: [0; CHUNK],
            len: 0,
            content_bytes: 0,
            stats: StreamStats::default(),
            stream,
            tx,
            budget,
        }
    }
    fn feed(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            if byte == b'\n' {
                self.emit(true);
            } else {
                self.content_bytes += 1;
                if self.len < CHUNK {
                    self.prefix[self.len] = byte;
                    self.len += 1;
                }
            }
        }
    }
    fn finish(&mut self) {
        if self.content_bytes > 0 {
            self.emit(false);
        }
    }
    fn emit(&mut self, newline: bool) {
        // Fixed construction scratch, no allocating lossy UTF8 intermediates.
        let mut rendered = [0u8; CHUNK];
        let (mut written, mut consumed) = (0, 0);
        while consumed < self.len && written < CHUNK {
            let remaining = &self.prefix[consumed..self.len];
            let (valid, invalid) = match std::str::from_utf8(remaining) {
                Ok(_) => (remaining.len(), None),
                Err(e) => (e.valid_up_to(), Some(e.error_len())),
            };
            let mut take = valid.min(CHUNK - written);
            while take > 0 && std::str::from_utf8(&remaining[..take]).is_err() {
                take -= 1;
            }
            rendered[written..written + take].copy_from_slice(&remaining[..take]);
            written += take;
            consumed += take;
            if take < valid {
                break;
            }
            match invalid {
                None => break,
                // Don't render an incomplete prefix cut as an invalid sequence.
                Some(None) if self.content_bytes > self.len as u64 => break,
                Some(length) => {
                    if CHUNK - written < 3 {
                        break;
                    }
                    rendered[written..written + 3].copy_from_slice("�".as_bytes());
                    written += 3;
                    consumed += length.unwrap_or(remaining.len() - valid);
                }
            }
        }
        let omitted = self.content_bytes - consumed as u64;
        let source_bytes = self.content_bytes + u64::from(newline);
        let sent = match self.tx.try_reserve() {
            Ok(slot) => match self.budget.acquire(self.stream) {
                Some(permit) => {
                    let payload = Box::new(rendered);
                    slot.send(OutputRecord {
                        payload,
                        _permit: permit,
                        len: written,
                        truncated: omitted > 0,
                    });
                    true
                }
                None => false,
            },
            Err(_) => false,
        };
        if sent && omitted > 0 {
            self.stats.truncated_records += 1;
            self.stats.truncated_bytes += omitted;
        } else if !sent {
            self.stats.dropped_records += 1;
            self.stats.dropped_bytes += source_bytes;
        }
        self.len = 0;
        self.content_bytes = 0;
    }
}
pub(crate) async fn spawn(
    program: &ProgramSpec,
    timeout: Duration,
    protocol: OutputProtocol,
) -> anyhow::Result<BoundedOutputHandle> {
    // Fresh plan validation occurs BEFORE child launch or output allocation.
    let parser = match protocol {
        OutputProtocol::Text => None,
        OutputProtocol::Export { digest, files } => Some(ExportProtocol::new(digest, files)?),
    };
    let budget = Budget::new(parser.is_some());
    let mut command = Command::from(program.command()?);
    command
        .kill_on_drop(true)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn()?;
    let pipes = (child.stdout.take(), child.stderr.take());
    let (stdout, stderr) = match pipes {
        (Some(stdout), Some(stderr)) => (stdout, stderr),
        _ => {
            let stop = child.start_kill();
            let waited = child.wait().await;
            stop?;
            waited?;
            anyhow::bail!("bounded subprocess pipes unavailable");
        }
    };
    let (out_tx, out_rx) = mpsc::channel(QUEUE_SLOTS);
    let (err_tx, err_rx) = mpsc::channel(QUEUE_SLOTS);
    let (stop_tx, stop_rx) = watch::channel(false);
    let (terminal_tx, terminal_rx) = watch::channel(None);
    let stdout_display = Display::new(0, out_tx, Arc::clone(&budget));
    let stderr_display = Display::new(1, err_tx, Arc::clone(&budget));
    let supervisor = tokio::spawn(async move {
        let completion = supervise(
            child,
            stdout,
            stderr,
            stdout_display,
            stderr_display,
            parser,
            stop_rx,
            timeout,
        )
        .await;
        terminal_tx.send_replace(Some(Arc::new(completion)));
    });
    Ok(BoundedOutputHandle {
        stdout: Some(out_rx),
        stderr: Some(err_rx),
        stop: stop_tx,
        terminal: terminal_rx,
        supervisor,
        joined: false,
        join_failed: false,
        budget,
    })
}
#[allow(clippy::too_many_arguments)]
async fn supervise(
    mut child: Child,
    mut stdout: ChildStdout,
    mut stderr: ChildStderr,
    mut out: Display,
    mut err: Display,
    mut parser: Option<ExportProtocol>,
    mut stop: watch::Receiver<bool>,
    timeout: Duration,
) -> OutputCompletion {
    let mut out_buffer = [0; CHUNK];
    let mut err_buffer = [0; CHUNK];
    let (mut out_done, mut err_done, mut wait_done) = (false, false, false);
    let (mut requested_stop, mut stop_open, mut root_reaped) = (false, true, false);
    let mut exit_code = None;
    let mut stop_reason = None;
    let mut io_error = None;
    let mut protocol_error = None;
    let deadline = tokio::time::sleep(timeout);
    tokio::pin!(deadline);
    while !(out_done && err_done && wait_done) {
        // Unbiased select; locked Tokio read()/Child::wait() are cancel safe.
        // Separate quotas ensure a hot stdout cannot take stderr capacity.
        tokio::select! {
            read = stdout.read(&mut out_buffer), if !out_done => {
                match read {
                    Ok(0) => { out_done = true; out.stats.eof = true; if parser.is_none() { out.finish(); } }
                    Ok(n) => {
                        out.stats.bytes_read += n as u64;
                        if let Some(parser) = parser.as_mut() {
                            if protocol_error.is_none() {
                                if let Err(error) = parser.feed(&out_buffer[..n]) {
                                    protocol_error = Some(error); stop_reason.get_or_insert(StopReason::Protocol);
                                }
                            }
                        } else { out.feed(&out_buffer[..n]); }
                    }
                    Err(error) => {
                        out_done = true; out.finish();
                        io_error.get_or_insert(IoFailure { operation: IoOperation::StdoutRead, kind: error.kind() });
                        stop_reason.get_or_insert(StopReason::Io);
                    }
                }
            }
            read = stderr.read(&mut err_buffer), if !err_done => {
                match read {
                    Ok(0) => { err_done = true; err.stats.eof = true; err.finish(); }
                    Ok(n) => { err.stats.bytes_read += n as u64; err.feed(&err_buffer[..n]); }
                    Err(error) => {
                        err_done = true; err.finish();
                        io_error.get_or_insert(IoFailure { operation: IoOperation::StderrRead, kind: error.kind() });
                        stop_reason.get_or_insert(StopReason::Io);
                    }
                }
            }
            status = child.wait(), if !wait_done => {
                wait_done = true;
                match status {
                    Ok(status) => { root_reaped = true; exit_code = status.code(); }
                    Err(error) => {
                        io_error.get_or_insert(IoFailure { operation: IoOperation::RootWait, kind: error.kind() });
                        stop_reason.get_or_insert(StopReason::Io);
                    }
                }
            }
            changed = stop.changed(), if stop_open && stop_reason.is_none() => {
                if changed.is_err() { stop_open = false; }
                else if *stop.borrow() { stop_reason = Some(StopReason::Requested); }
            }
            _ = &mut deadline, if stop_reason.is_none() => { stop_reason = Some(StopReason::Timeout); }
        }
        if stop_reason.is_some() && !requested_stop {
            requested_stop = true;
            if !root_reaped {
                if let Err(error) = child.start_kill() {
                    io_error.get_or_insert(IoFailure {
                        operation: IoOperation::RootStop,
                        kind: error.kind(),
                    });
                }
            }
        }
    }
    if let Some(parser) = parser.as_mut() {
        if let Err(error) = parser.finish(exit_code) {
            protocol_error.get_or_insert(error);
            stop_reason.get_or_insert(StopReason::Protocol);
        }
    }
    OutputCompletion {
        exit_code,
        stop_reason,
        stdout: out.stats,
        stderr: err.stats,
        root_wait_observed: wait_done,
        root_reaped,
        descendant_cleanup_verified: false,
        io_error,
        protocol_error,
        export: parser,
        _budget: Arc::clone(&out.budget),
    }
}
