//! Persistent presentation-neutral Tokio host, currently serving read-only reports.
//!
//! All started blocking work stays tracked until its JoinHandle completes. A
//! lost observer is not cancellation or cleanup. Terminal outcomes remain in a
//! bounded registry until explicit release. No profile/journal is opened here.
//!
//! `ReportTicket::wait` and `ShutdownTicket::wait` use runtime-neutral channel
//! futures. The native adapter must own subscription handles and foreground
//! delivery. Own `EngineHost` off the UI thread: explicit `join` and its Drop
//! fallback block until the host exits, and never detach started work.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{mpsc as std_mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};

use tokio::sync::{mpsc, watch, Notify};

use crate::headless::{ReportRequest, ReportService};

use super::contracts::{
    Cancellation, EngineError, ReportCompletion, ReportOutcome, RequestId, ShutdownChoice,
    MAX_RETAINED_REQUESTS,
};

const QUEUED: u8 = 0;
const RUNNING: u8 = 1;
const CANCELLED_QUEUED: u8 = 2;
const CANCELLED_RUNNING: u8 = 3;
const TERMINAL: u8 = 4;
const HOST_RUNNING: u8 = 0;
const HOST_STOPPED: u8 = 1;
const HOST_FAILED: u8 = 2;

type RetainedOutcome = Option<Arc<ReportOutcome>>;
type StoppedResult = Option<Result<(), EngineError>>;

struct RequestCell {
    phase: AtomicU8,
    outcome: watch::Sender<RetainedOutcome>,
}

impl RequestCell {
    fn cancel(&self) -> Cancellation {
        loop {
            let phase = self.phase.load(Ordering::Acquire);
            let (next, answer) = match phase {
                QUEUED => (CANCELLED_QUEUED, Cancellation::PreventedLaunch),
                RUNNING => (CANCELLED_RUNNING, Cancellation::RequestedForStartedWork),
                CANCELLED_QUEUED => return Cancellation::PreventedLaunch,
                CANCELLED_RUNNING => return Cancellation::RequestedForStartedWork,
                _ => return Cancellation::AlreadyTerminal,
            };
            if self
                .phase
                .compare_exchange(phase, next, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return answer;
            }
        }
    }

    fn begin(&self) -> bool {
        self.phase
            .compare_exchange(QUEUED, RUNNING, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    fn publish(&self, id: RequestId, result: WorkResult) {
        let previous = self.phase.swap(TERMINAL, Ordering::AcqRel);
        let cancellation_requested = previous == CANCELLED_RUNNING;
        let completion = match result {
            WorkResult::Report(report) => ReportCompletion::Report {
                report,
                cancellation_requested,
            },
            WorkResult::Cancelled => ReportCompletion::CancelledBeforeStart,
            WorkResult::Failed(error) => ReportCompletion::Failed(error),
            #[cfg(test)]
            WorkResult::Probe(kind) => ReportCompletion::Probe {
                kind,
                cancellation_requested,
            },
        };
        // send_replace retains the value even when every observer was dropped.
        self.outcome.send_replace(Some(Arc::new(ReportOutcome {
            request_id: id,
            completion,
        })));
    }
}

struct Registry {
    next_id: u64,
    requests: BTreeMap<RequestId, Arc<RequestCell>>,
}

struct Shared {
    registry: Mutex<Registry>,
    shutdown: AtomicU8,
    control: Notify,
    status: AtomicU8,
    stopped: watch::Receiver<StoppedResult>,
}

impl Shared {
    fn shutdown(&self, choice: ShutdownChoice) -> ShutdownTicket {
        let requested = match choice {
            ShutdownChoice::Wait => 1,
            ShutdownChoice::Cancel => 2,
        };
        self.shutdown.fetch_max(requested, Ordering::AcqRel);
        if choice == ShutdownChoice::Cancel {
            let registry = match self.registry.lock() {
                Ok(registry) => registry,
                Err(poisoned) => {
                    self.status.store(HOST_FAILED, Ordering::Release);
                    // Cleanup only: admission stays closed and runtime-stop/join
                    // report StatePoisoned. Never clear poison or reuse state.
                    poisoned.into_inner()
                }
            };
            // Includes dequeued requests not yet dispatched by the host. Each
            // cell's CAS linearizes cancellation against its actual begin().
            for cell in registry.requests.values() {
                cell.cancel();
            }
        }
        self.control.notify_one();
        ShutdownTicket {
            stopped: self.stopped.clone(),
        }
    }

    fn cell(&self, id: RequestId) -> Result<Arc<RequestCell>, EngineError> {
        self.registry
            .lock()
            .map_err(|_| EngineError::StatePoisoned)?
            .requests
            .get(&id)
            .cloned()
            .ok_or(EngineError::UnknownRequest(id))
    }
}

enum Work {
    Report(ReportRequest),
    #[cfg(test)]
    Probe(Probe),
}

struct Request {
    id: RequestId,
    cell: Arc<RequestCell>,
    work: Work,
}

enum WorkResult {
    Report(Box<crate::headless::ReportResult>),
    Cancelled,
    Failed(EngineError),
    #[cfg(test)]
    Probe(&'static str),
}

#[derive(Clone)]
pub struct EngineClient {
    sender: mpsc::Sender<Request>,
    shared: Arc<Shared>,
}

impl EngineClient {
    /// Nonblocking ordinary admission; retained terminal results consume slots.
    pub fn build_report(&self, request: ReportRequest) -> Result<ReportTicket, EngineError> {
        self.submit(Work::Report(request))
    }

    fn submit(&self, work: Work) -> Result<ReportTicket, EngineError> {
        if self.shared.status.load(Ordering::Acquire) != HOST_RUNNING {
            return Err(EngineError::Stopped);
        }
        if self.shared.shutdown.load(Ordering::Acquire) != 0 {
            return Err(EngineError::ShuttingDown);
        }
        let mut registry = self
            .shared
            .registry
            .try_lock()
            .map_err(|error| match error {
                std::sync::TryLockError::WouldBlock => EngineError::Busy,
                std::sync::TryLockError::Poisoned(_) => EngineError::StatePoisoned,
            })?;
        // Synchronize reservation with the global Cancel sweep: an admission
        // that passed the outer checks cannot enter after the sweep returned.
        if self.shared.status.load(Ordering::Acquire) != HOST_RUNNING {
            return Err(EngineError::Stopped);
        }
        if self.shared.shutdown.load(Ordering::Acquire) != 0 {
            return Err(EngineError::ShuttingDown);
        }
        if registry.requests.len() >= MAX_RETAINED_REQUESTS {
            return Err(EngineError::Busy);
        }
        let id = RequestId(registry.next_id);
        registry.next_id = registry
            .next_id
            .checked_add(1)
            .ok_or(EngineError::IdExhausted)?;
        let (outcome, receiver) = watch::channel(None);
        let cell = Arc::new(RequestCell {
            phase: AtomicU8::new(QUEUED),
            outcome,
        });
        registry.requests.insert(id, cell.clone());
        if let Err(error) = self.sender.try_send(Request {
            id,
            cell: cell.clone(),
            work,
        }) {
            registry.requests.remove(&id);
            return Err(match error {
                mpsc::error::TrySendError::Full(_) => EngineError::Busy,
                mpsc::error::TrySendError::Closed(_) => EngineError::Stopped,
            });
        }
        Ok(ReportTicket {
            id,
            cell,
            outcome: receiver,
            stopped: self.shared.stopped.clone(),
        })
    }

    /// Reattach after a lost observer, including after runtime shutdown.
    pub fn observe(&self, id: RequestId) -> Result<ReportTicket, EngineError> {
        let cell = self.shared.cell(id)?;
        Ok(ReportTicket {
            id,
            outcome: cell.outcome.subscribe(),
            cell,
            stopped: self.shared.stopped.clone(),
        })
    }

    pub fn outcome(&self, id: RequestId) -> Result<RetainedOutcome, EngineError> {
        let cell = self.shared.cell(id)?;
        let outcome = cell.outcome.borrow().clone();
        Ok(outcome)
    }

    /// Direct atomic control: ordinary queue saturation cannot block cancellation.
    pub fn cancel(&self, id: RequestId) -> Result<Cancellation, EngineError> {
        Ok(self.shared.cell(id)?.cancel())
    }

    /// Only explicit terminal release returns admission budget to the engine.
    pub fn release(&self, id: RequestId) -> Result<(), EngineError> {
        let mut registry = self
            .shared
            .registry
            .lock()
            .map_err(|_| EngineError::StatePoisoned)?;
        let cell = registry
            .requests
            .get(&id)
            .ok_or(EngineError::UnknownRequest(id))?;
        if cell.outcome.borrow().is_none() {
            return Err(EngineError::NotTerminal(id));
        }
        registry.requests.remove(&id);
        Ok(())
    }

    /// Repeated requests are idempotent; Wait can escalate to Cancel, never back.
    pub fn request_shutdown(&self, choice: ShutdownChoice) -> ShutdownTicket {
        self.shared.shutdown(choice)
    }
}

pub struct ReportTicket {
    id: RequestId,
    cell: Arc<RequestCell>,
    outcome: watch::Receiver<RetainedOutcome>,
    stopped: watch::Receiver<StoppedResult>,
}

impl ReportTicket {
    pub fn id(&self) -> RequestId {
        self.id
    }

    pub fn cancel(&self) -> Cancellation {
        self.cell.cancel()
    }

    /// Owned runtime-neutral future; no Tokio executor is required to await it.
    pub async fn wait(mut self) -> Result<Arc<ReportOutcome>, EngineError> {
        loop {
            if let Some(outcome) = self.outcome.borrow().clone() {
                return Ok(outcome);
            }
            if self.stopped.borrow().is_some() {
                return self.outcome.borrow().clone().ok_or(EngineError::Stopped);
            }
            tokio::select! {
                biased;
                changed = self.outcome.changed() => { changed.map_err(|_| EngineError::Stopped)?; }
                changed = self.stopped.changed() => {
                    if changed.is_err() {
                        return self.outcome.borrow().clone().ok_or(EngineError::Stopped);
                    }
                }
            }
        }
    }
}

pub struct ShutdownTicket {
    stopped: watch::Receiver<StoppedResult>,
}

impl ShutdownTicket {
    /// Acknowledges runtime destruction, NOT exit of the std host thread.
    pub async fn wait(mut self) -> Result<(), EngineError> {
        loop {
            if let Some(result) = self.stopped.borrow().clone() {
                return result;
            }
            self.stopped
                .changed()
                .await
                .map_err(|_| EngineError::Stopped)?;
        }
    }
}

pub struct EngineHost {
    shared: Arc<Shared>,
    worker: Option<JoinHandle<Result<(), EngineError>>>,
}

impl EngineHost {
    /// Start once, outside render/input callbacks. Startup waits for runtime creation.
    pub fn start() -> Result<(Self, EngineClient), EngineError> {
        let (sender, receiver) = mpsc::channel(MAX_RETAINED_REQUESTS);
        let (stopped_sender, stopped) = watch::channel(None);
        let shared = Arc::new(Shared {
            registry: Mutex::new(Registry {
                next_id: 1,
                requests: BTreeMap::new(),
            }),
            shutdown: AtomicU8::new(0),
            control: Notify::new(),
            status: AtomicU8::new(HOST_RUNNING),
            stopped,
        });
        let worker_shared = shared.clone();
        let (ready_sender, ready_receiver) = std_mpsc::sync_channel(1);
        let worker = thread::Builder::new()
            .name("vfa-engine-host".into())
            .spawn(move || {
                let runtime = match tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .enable_all()
                    .build()
                {
                    Ok(runtime) => runtime,
                    Err(error) => {
                        let error = EngineError::Startup(error.to_string());
                        let _ = ready_sender.send(Err(error.clone()));
                        return Err(error);
                    }
                };
                let _ = ready_sender.send(Ok(()));
                let mut result = runtime.block_on(run_host(worker_shared.clone(), receiver));
                drop(runtime); // All started work was joined before runtime destruction.
                if result.is_ok() && worker_shared.status.load(Ordering::Acquire) == HOST_FAILED {
                    result = Err(EngineError::StatePoisoned);
                }
                worker_shared.status.store(
                    if result.is_ok() {
                        HOST_STOPPED
                    } else {
                        HOST_FAILED
                    },
                    Ordering::Release,
                );
                stopped_sender.send_replace(Some(result.clone()));
                result
            })
            .map_err(|error| EngineError::Startup(error.to_string()))?;
        match ready_receiver.recv() {
            Ok(Ok(())) => {
                let client = EngineClient {
                    sender,
                    shared: shared.clone(),
                };
                Ok((
                    Self {
                        shared,
                        worker: Some(worker),
                    },
                    client,
                ))
            }
            result => {
                drop(sender);
                let joined = worker.join().map_err(|_| EngineError::HostPanicked)?;
                match result {
                    Ok(Err(error)) => Err(error),
                    _ => joined.and(Err(EngineError::Stopped)),
                }
            }
        }
    }

    pub fn request_shutdown(&self, choice: ShutdownChoice) -> ShutdownTicket {
        self.shared.shutdown(choice)
    }

    /// Blocking, off-UI join is the only proof that the std host thread exited.
    pub fn join(mut self) -> Result<(), EngineError> {
        self.shared.shutdown(ShutdownChoice::Wait);
        self.finish()
    }

    fn finish(&mut self) -> Result<(), EngineError> {
        match self.worker.take() {
            Some(worker) => worker.join().map_err(|_| EngineError::HostPanicked)?,
            None => Ok(()),
        }
    }
}

impl Drop for EngineHost {
    fn drop(&mut self) {
        // Non-UI fallback only. Never detach a runtime or started blocking work.
        self.shared.shutdown(ShutdownChoice::Cancel);
        let _ = self.finish();
    }
}

struct Active {
    id: RequestId,
    cell: Arc<RequestCell>,
    task: tokio::task::JoinHandle<WorkResult>,
}

async fn run_host(
    shared: Arc<Shared>,
    mut receiver: mpsc::Receiver<Request>,
) -> Result<(), EngineError> {
    let mut active: Option<Active> = None;
    loop {
        let shutdown = shared.shutdown.load(Ordering::Acquire);
        if shutdown != 0 {
            receiver.close();
        }
        if shutdown == 2 {
            if let Some(active) = &active {
                active.cell.cancel();
            }
            while let Ok(request) = receiver.try_recv() {
                request.cell.cancel();
                request.cell.publish(request.id, WorkResult::Cancelled);
            }
        }
        if active.is_none() && shutdown != 0 && receiver.is_empty() {
            return Ok(());
        }
        let has_active = active.is_some();
        tokio::select! {
            biased;
            _ = shared.control.notified() => {}
            result = async { (&mut active.as_mut().expect("active branch is enabled").task).await }, if has_active => {
                let completed = active.take().expect("tracked active work");
                match result {
                    Ok(result) => completed.cell.publish(completed.id, result),
                    Err(error) => {
                        let failure = EngineError::WorkFailed(error.to_string());
                        shared.status.store(HOST_FAILED, Ordering::Release);
                        shared.shutdown.fetch_max(2, Ordering::AcqRel);
                        completed.cell.publish(completed.id, WorkResult::Failed(failure.clone()));
                        receiver.close();
                        while let Some(request) = receiver.recv().await {
                            request.cell.publish(request.id, WorkResult::Failed(failure.clone()));
                        }
                        return Err(failure);
                    }
                }
            }
            request = receiver.recv(), if !has_active => {
                match request {
                    Some(request) => {
                        if shared.shutdown.load(Ordering::Acquire) == 2 { request.cell.cancel(); }
                        if request.cell.phase.load(Ordering::Acquire) == CANCELLED_QUEUED {
                            request.cell.publish(request.id, WorkResult::Cancelled);
                        } else {
                            #[cfg(test)]
                            let launch_observed = match &request.work {
                                Work::Probe(Probe::BeforeDispatch { observed, gate, launch_observed, .. }) => {
                                    observed.send(()).unwrap();
                                    gate.wait();
                                    Some(launch_observed.clone())
                                }
                                _ => None,
                            };
                            let cell = request.cell.clone();
                            active = Some(Active { id: request.id, task: start_work(request.work, cell.clone()), cell });
                            #[cfg(test)]
                            if let Some(launch_observed) = launch_observed { launch_observed.wait(); }
                        }
                    }
                    None => return Ok(()),
                }
            }
        }
    }
}

fn start_work(work: Work, cell: Arc<RequestCell>) -> tokio::task::JoinHandle<WorkResult> {
    match work {
        Work::Report(request) => tokio::task::spawn_blocking(move || {
            if !cell.begin() {
                return WorkResult::Cancelled;
            }
            WorkResult::Report(Box::new(ReportService::generate(&request)))
        }),
        #[cfg(test)]
        Work::Probe(probe) => start_probe(probe, cell),
    }
}

#[cfg(test)]
enum Probe {
    Timer,
    Io(std::path::PathBuf),
    Blocking {
        started: std_mpsc::Sender<()>,
        gate: Arc<TestGate>,
    },
    BeforeDispatch {
        observed: std_mpsc::Sender<()>,
        gate: Arc<TestGate>,
        launch_observed: Arc<TestGate>,
        effects: Arc<std::sync::atomic::AtomicUsize>,
    },
    Panic,
}

#[cfg(test)]
struct TestGate {
    released: Mutex<bool>,
    condition: std::sync::Condvar,
}

#[cfg(test)]
impl TestGate {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            released: Mutex::new(false),
            condition: std::sync::Condvar::new(),
        })
    }
    fn release(&self) {
        *self.released.lock().unwrap() = true;
        self.condition.notify_all();
    }
    fn wait(&self) {
        let mut released = self.released.lock().unwrap();
        while !*released {
            released = self.condition.wait(released).unwrap();
        }
    }
}

#[cfg(test)]
fn start_probe(probe: Probe, cell: Arc<RequestCell>) -> tokio::task::JoinHandle<WorkResult> {
    match probe {
        Probe::Timer => tokio::spawn(async move {
            if !cell.begin() {
                return WorkResult::Cancelled;
            }
            assert_eq!(
                tokio::runtime::Handle::current().runtime_flavor(),
                tokio::runtime::RuntimeFlavor::MultiThread
            );
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            WorkResult::Probe("timer")
        }),
        Probe::Io(executable) => tokio::spawn(async move {
            if !cell.begin() {
                return WorkResult::Cancelled;
            }
            let output = tokio::process::Command::new(executable)
                .arg("--list")
                .stdout(std::process::Stdio::piped())
                .output()
                .await
                .unwrap();
            assert!(output.status.success());
            assert!(!output.stdout.is_empty());
            WorkResult::Probe("piped-io")
        }),
        Probe::Blocking { started, gate } => tokio::task::spawn_blocking(move || {
            if !cell.begin() {
                return WorkResult::Cancelled;
            }
            started.send(()).unwrap();
            gate.wait();
            WorkResult::Probe("blocking")
        }),
        Probe::BeforeDispatch {
            launch_observed,
            effects,
            ..
        } => tokio::task::spawn_blocking(move || {
            if !cell.begin() {
                launch_observed.release();
                return WorkResult::Cancelled;
            }
            effects.fetch_add(1, Ordering::SeqCst);
            launch_observed.release();
            WorkResult::Probe("prelaunch-side-effect")
        }),
        Probe::Panic => tokio::task::spawn_blocking(move || {
            if !cell.begin() {
                return WorkResult::Cancelled;
            }
            panic!("controlled engine work panic");
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::FutureExt;
    use std::future::Future;
    use std::task::{Context, Poll, Wake, Waker};
    use std::time::Duration;

    struct ThreadWake(thread::Thread);

    impl Wake for ThreadWake {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
    }

    fn block_on<T>(future: impl Future<Output = T>) -> T {
        let waker = Waker::from(Arc::new(ThreadWake(thread::current())));
        let mut context = Context::from_waker(&waker);
        let mut future = std::pin::pin!(future);
        loop {
            match future.as_mut().poll(&mut context) {
                Poll::Ready(value) => return value,
                Poll::Pending => thread::park(),
            }
        }
    }

    struct GateOwner(Arc<TestGate>);

    impl Drop for GateOwner {
        fn drop(&mut self) {
            self.0.release();
        }
    }

    fn held(client: &EngineClient) -> (ReportTicket, GateOwner) {
        let gate = GateOwner(TestGate::new());
        let (started, observed) = std_mpsc::channel();
        let ticket = client
            .submit(Work::Probe(Probe::Blocking {
                started,
                gate: gate.0.clone(),
            }))
            .unwrap();
        observed
            .recv_timeout(Duration::from_secs(10))
            .expect("blocking work actually started");
        (ticket, gate)
    }

    #[test]
    fn persistent_multithread_runtime_drives_real_timer_and_piped_process_io() {
        let (host, client) = EngineHost::start().unwrap();
        assert_ne!(
            host.worker.as_ref().unwrap().thread().id(),
            thread::current().id()
        );
        let timer = client.submit(Work::Probe(Probe::Timer)).unwrap();
        assert!(matches!(
            &block_on(timer.wait()).unwrap().completion,
            ReportCompletion::Probe { kind: "timer", .. }
        ));
        let io = client
            .submit(Work::Probe(Probe::Io(std::env::current_exe().unwrap())))
            .unwrap();
        assert!(matches!(
            &block_on(io.wait()).unwrap().completion,
            ReportCompletion::Probe {
                kind: "piped-io",
                ..
            }
        ));
        block_on(host.request_shutdown(ShutdownChoice::Wait).wait()).unwrap();
        host.join().unwrap();
    }

    #[test]
    fn prelaunch_cancellation_skips_work_and_release_requires_terminal() {
        let (host, client) = EngineHost::start().unwrap();
        let (first, gate) = held(&client);
        let second_gate = GateOwner(TestGate::new());
        let (started, observed) = std_mpsc::channel();
        let second = client
            .submit(Work::Probe(Probe::Blocking {
                started,
                gate: second_gate.0.clone(),
            }))
            .unwrap();
        let id = second.id();
        assert_eq!(client.release(id), Err(EngineError::NotTerminal(id)));
        assert_eq!(second.cancel(), Cancellation::PreventedLaunch);
        gate.0.release();
        block_on(first.wait()).unwrap();
        assert!(matches!(
            &block_on(second.wait()).unwrap().completion,
            ReportCompletion::CancelledBeforeStart
        ));
        assert!(
            observed.try_recv().is_err(),
            "cancelled queued work must not start"
        );
        block_on(host.request_shutdown(ShutdownChoice::Wait).wait()).unwrap();
        host.join().unwrap();
        assert_eq!(
            observed.try_recv(),
            Err(std_mpsc::TryRecvError::Disconnected)
        );
        client.release(id).unwrap();
    }

    #[test]
    fn global_cancel_after_dequeue_before_dispatch_prevents_launch() {
        let (host, client) = EngineHost::start().unwrap();
        let gate = GateOwner(TestGate::new());
        let (observed, dequeued) = std_mpsc::channel();
        let effects = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let ticket = client
            .submit(Work::Probe(Probe::BeforeDispatch {
                observed,
                gate: gate.0.clone(),
                launch_observed: TestGate::new(),
                effects: effects.clone(),
            }))
            .unwrap();
        dequeued
            .recv_timeout(Duration::from_secs(10))
            .expect("host paused after its Cancel check");
        let stopped = client.request_shutdown(ShutdownChoice::Cancel);
        gate.0.release();
        let outcome = block_on(ticket.wait()).unwrap();
        block_on(stopped.wait()).unwrap();
        host.join().unwrap();
        assert_eq!(
            effects.load(Ordering::SeqCst),
            0,
            "global Cancel returned before any launch side effect"
        );
        assert!(matches!(
            &outcome.completion,
            ReportCompletion::CancelledBeforeStart
        ));
    }

    #[test]
    fn lost_observer_has_recoverable_immutable_outcome_and_bounded_retention() {
        let (host, client) = EngineHost::start().unwrap();
        let (ticket, gate) = held(&client);
        let id = ticket.id();
        drop(ticket);
        assert!(client.outcome(id).unwrap().is_none());
        let recovered = client.observe(id).unwrap();
        gate.0.release();
        let outcome = block_on(recovered.wait()).unwrap();
        let retained = client.outcome(id).unwrap().unwrap();
        assert!(Arc::ptr_eq(&outcome, &retained));
        assert!(matches!(
            &outcome.completion,
            ReportCompletion::Probe {
                kind: "blocking",
                cancellation_requested: false
            }
        ));
        for _ in 1..MAX_RETAINED_REQUESTS {
            let ticket = client.submit(Work::Probe(Probe::Timer)).unwrap();
            block_on(ticket.wait()).unwrap();
        }
        assert!(matches!(
            client.submit(Work::Probe(Probe::Timer)),
            Err(EngineError::Busy)
        ));
        client.release(id).unwrap();
        assert!(
            matches!(client.outcome(id), Err(EngineError::UnknownRequest(found)) if found == id)
        );
        let replacement = client.submit(Work::Probe(Probe::Timer)).unwrap();
        assert_ne!(replacement.id(), id);
        block_on(replacement.wait()).unwrap();
        block_on(host.request_shutdown(ShutdownChoice::Wait).wait()).unwrap();
        host.join().unwrap();
        assert!(matches!(
            &retained.completion,
            ReportCompletion::Probe {
                kind: "blocking",
                ..
            }
        ));
    }

    #[test]
    fn saturated_ordinary_budget_cannot_starve_cancel_or_escalated_shutdown() {
        let (host, client) = EngineHost::start().unwrap();
        let (started, gate) = held(&client);
        let started_id = started.id();
        let mut queued_ids = Vec::new();
        for _ in 1..MAX_RETAINED_REQUESTS {
            let queued = client.submit(Work::Probe(Probe::Timer)).unwrap();
            queued_ids.push(queued.id());
            drop(queued);
        }
        assert!(matches!(
            client.submit(Work::Probe(Probe::Timer)),
            Err(EngineError::Busy)
        ));
        assert_eq!(
            client.cancel(started_id).unwrap(),
            Cancellation::RequestedForStartedWork
        );
        let wait = client.request_shutdown(ShutdownChoice::Wait);
        assert!(wait.wait().now_or_never().is_none());
        let cancel = client.request_shutdown(ShutdownChoice::Cancel);
        assert!(
            cancel.wait().now_or_never().is_none(),
            "a dropped wait future is not cleanup proof"
        );
        client.request_shutdown(ShutdownChoice::Wait);
        assert_eq!(client.shared.shutdown.load(Ordering::Acquire), 2);
        drop(started);
        assert!(client.outcome(started_id).unwrap().is_none());
        gate.0.release();
        block_on(client.request_shutdown(ShutdownChoice::Wait).wait()).unwrap();
        host.join().unwrap();
        assert!(matches!(
            &client.outcome(started_id).unwrap().unwrap().completion,
            ReportCompletion::Probe {
                kind: "blocking",
                cancellation_requested: true
            }
        ));
        for id in queued_ids {
            assert!(matches!(
                &client.outcome(id).unwrap().unwrap().completion,
                ReportCompletion::CancelledBeforeStart
            ));
        }
    }

    #[test]
    fn wait_shutdown_drains_accepted_work_and_repeated_acknowledgments_are_stable() {
        let (host, client) = EngineHost::start().unwrap();
        let (first, gate) = held(&client);
        let second = client.submit(Work::Probe(Probe::Timer)).unwrap();
        assert!(host
            .request_shutdown(ShutdownChoice::Wait)
            .wait()
            .now_or_never()
            .is_none());
        gate.0.release();
        block_on(first.wait()).unwrap();
        assert!(matches!(
            &block_on(second.wait()).unwrap().completion,
            ReportCompletion::Probe {
                kind: "timer",
                cancellation_requested: false
            }
        ));
        block_on(host.request_shutdown(ShutdownChoice::Wait).wait()).unwrap();
        block_on(client.request_shutdown(ShutdownChoice::Cancel).wait()).unwrap();
        block_on(client.request_shutdown(ShutdownChoice::Wait).wait()).unwrap();
        host.join().unwrap();
        assert!(matches!(
            client.submit(Work::Probe(Probe::Timer)),
            Err(EngineError::Stopped)
        ));
    }

    #[test]
    fn work_panic_fails_closed_publishes_failures_and_never_reuses_host() {
        let (host, client) = EngineHost::start().unwrap();
        let (first, gate) = held(&client);
        let panicking = client.submit(Work::Probe(Probe::Panic)).unwrap();
        let queued = client.submit(Work::Probe(Probe::Timer)).unwrap();
        gate.0.release();
        block_on(first.wait()).unwrap();
        assert!(matches!(
            &block_on(panicking.wait()).unwrap().completion,
            ReportCompletion::Failed(EngineError::WorkFailed(_))
        ));
        assert!(matches!(
            &block_on(queued.wait()).unwrap().completion,
            ReportCompletion::Failed(EngineError::WorkFailed(_))
        ));
        assert!(matches!(
            client.submit(Work::Probe(Probe::Timer)),
            Err(EngineError::Stopped)
        ));
        assert!(matches!(
            block_on(client.request_shutdown(ShutdownChoice::Wait).wait()),
            Err(EngineError::WorkFailed(_))
        ));
        assert!(matches!(host.join(), Err(EngineError::WorkFailed(_))));
    }

    #[test]
    fn non_ui_host_drop_waits_instead_of_detaching_started_blocking_work() {
        let (host, client) = EngineHost::start().unwrap();
        let (ticket, gate) = held(&client);
        drop(ticket);
        let (entered, observed_entry) = std_mpsc::channel();
        let (finished, observed_finish) = std_mpsc::channel();
        let dropping_thread = thread::spawn(move || {
            entered.send(()).unwrap();
            drop(host);
            finished.send(()).unwrap();
        });
        observed_entry
            .recv_timeout(Duration::from_secs(10))
            .unwrap();
        assert_eq!(
            observed_finish.try_recv(),
            Err(std_mpsc::TryRecvError::Empty)
        );
        gate.0.release();
        observed_finish
            .recv_timeout(Duration::from_secs(10))
            .unwrap();
        dropping_thread.join().unwrap();
        block_on(client.request_shutdown(ShutdownChoice::Wait).wait()).unwrap();
    }
}
