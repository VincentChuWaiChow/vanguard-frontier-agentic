//! Installation-free report bridge; deliberately no native GPUI admission claim.

use std::collections::BTreeMap;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use std::thread;

use serde_json::{json, Value};
use tempfile::TempDir;
use vfa_tui::engine::contracts::{
    EngineError, ReportCompletion, ShutdownChoice, MAX_RETAINED_REQUESTS,
};
use vfa_tui::engine::runtime::EngineHost;
use vfa_tui::headless::{ReportProducer, ReportRequest, ReportService};
use vfa_tui::models::report::{OutputFormat, ReportType};

struct ThreadWake(thread::Thread);

impl Wake for ThreadWake {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
}

/// No Tokio runtime or executor is constructed by this receiving adapter.
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

struct Fixture {
    _temp: TempDir,
    root: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("runtime report Ω 空 spaces");
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        std::fs::create_dir_all(root.join("catalog")).unwrap();
        for name in [
            "agents.json",
            "skills.json",
            "rules.json",
            "mcp-references.json",
            "install-roles.json",
            "asset-integrity.json",
        ] {
            std::fs::copy(
                source.join("catalog").join(name),
                root.join("catalog").join(name),
            )
            .expect("required baseline report input");
        }
        std::fs::copy(source.join("package.json"), root.join("package.json")).unwrap();
        Self { _temp: temp, root }
    }

    fn request(&self) -> ReportRequest {
        ReportRequest {
            catalog_root: self.root.clone(),
            registry_path: self.root.join("not-configured-registry.toml"),
            policy_path: self.root.join("not-configured-policy.toml"),
            index_path: self.root.join("never-created-index").join("legacy.sqlite"),
            report_types: vec![ReportType::Summary],
            format: OutputFormat::Json,
            workspace_filter: None,
            producer: ReportProducer {
                name: "runtime-bridge-fixture".into(),
                version: "fixture-owned-version".into(),
            },
        }
    }

    fn bytes(&self) -> BTreeMap<PathBuf, Vec<u8>> {
        let mut files = BTreeMap::new();
        for entry in std::fs::read_dir(self.root.join("catalog")).unwrap() {
            let entry = entry.unwrap();
            files.insert(entry.path(), std::fs::read(entry.path()).unwrap());
        }
        files.insert(
            self.root.join("package.json"),
            std::fs::read(self.root.join("package.json")).unwrap(),
        );
        files
    }
}

fn normalized(mut payload: Value) -> Value {
    let timestamp = payload["timestamp"].as_str().expect("observed report time");
    chrono::DateTime::parse_from_rfc3339(timestamp).unwrap();
    payload["timestamp"] = json!("<observed-time>");
    payload
}

#[test]
fn real_report_service_owned_results_survive_observer_loss_without_runtime_on_caller() {
    let fixture = Fixture::new();
    let request = fixture.request();
    let bytes = fixture.bytes();
    let expected = ReportService::generate(&request);
    let (host, client) = EngineHost::start().unwrap();
    let ticket = client.build_report(request.clone()).unwrap();
    let id = ticket.id();
    drop(ticket);
    let outcome = block_on(client.observe(id).unwrap().wait()).unwrap();
    assert_eq!(outcome.request_id, id);
    let ReportCompletion::Report {
        report,
        cancellation_requested,
    } = &outcome.completion
    else {
        panic!("actual existing ReportService must return its owned result");
    };
    assert!(!cancellation_requested);
    assert_eq!(
        normalized(report.payload.clone()),
        normalized(expected.payload)
    );
    assert_eq!(report.producer, request.producer);
    assert_eq!(report.exit_code, expected.exit_code);
    assert_eq!(report.provenance.catalog_root, request.catalog_root);
    assert_eq!(report.provenance.index_path, request.index_path);
    assert!(Arc::ptr_eq(&outcome, &client.outcome(id).unwrap().unwrap()));
    assert_eq!(fixture.bytes(), bytes);
    assert!(!request.index_path.exists());
    assert!(!request.registry_path.exists());
    assert!(!request.policy_path.exists());
    assert_eq!(std::fs::read_dir(&fixture.root).unwrap().count(), 2);
    block_on(client.request_shutdown(ShutdownChoice::Wait).wait()).unwrap();
    host.join().unwrap();
    assert!(Arc::ptr_eq(
        &outcome,
        &block_on(client.observe(id).unwrap().wait()).unwrap()
    ));
    client.release(id).unwrap();
    assert!(matches!(client.observe(id), Err(EngineError::UnknownRequest(found)) if found == id));
}

#[test]
fn actual_terminal_report_retention_is_bounded_until_explicit_release() {
    let fixture = Fixture::new();
    let (host, client) = EngineHost::start().unwrap();
    let mut ids = Vec::new();
    for _ in 0..MAX_RETAINED_REQUESTS {
        let ticket = client.build_report(fixture.request()).unwrap();
        ids.push(ticket.id());
        assert!(matches!(
            &block_on(ticket.wait()).unwrap().completion,
            ReportCompletion::Report { .. }
        ));
    }
    assert!(matches!(
        client.build_report(fixture.request()),
        Err(EngineError::Busy)
    ));
    let first = ids[0];
    let retained = client.outcome(first).unwrap().unwrap();
    client.release(first).unwrap();
    let replacement = client.build_report(fixture.request()).unwrap();
    assert!(!ids.contains(&replacement.id()));
    block_on(replacement.wait()).unwrap();
    assert!(matches!(
        &retained.completion,
        ReportCompletion::Report { .. }
    ));
    block_on(host.request_shutdown(ShutdownChoice::Wait).wait()).unwrap();
    host.join().unwrap();
    for id in ids.into_iter().skip(1) {
        assert!(client.outcome(id).unwrap().is_some());
        client.release(id).unwrap();
    }
}
