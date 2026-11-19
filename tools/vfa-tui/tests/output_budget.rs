//! Actual pipe pressure contracts, not descendant/native cleanup acceptance.
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use vfa_tui::engine::export_protocol::{ExpectedFile, FinalArtifact, ProtocolError};
use vfa_tui::subprocess::bounded::{BoundedOutputHandle, OutputProtocol, StopReason, RAW_BUDGET};
use vfa_tui::subprocess::program::ProgramSpec;
use vfa_tui::subprocess::SubprocessExecutor;

struct Helper {
    _temp: tempfile::TempDir,
    exe: PathBuf,
}
fn helper() -> &'static Helper {
    static HELPER: OnceLock<Helper> = OnceLock::new();
    HELPER.get_or_init(|| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/output-budget-wave");
        std::fs::create_dir_all(&root).unwrap();
        let temp = tempfile::tempdir_in(root).unwrap();
        let exe = temp
            .path()
            .join(format!("output-helper{}", std::env::consts::EXE_SUFFIX));
        let status = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
            .args(["--edition=2021", "--crate-name=vfa_output_helper"])
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/process_helper.rs"))
            .arg("-o")
            .arg(&exe)
            .status()
            .unwrap();
        assert!(status.success());
        Helper { _temp: temp, exe }
    })
}
fn spec(args: &[&str]) -> ProgramSpec {
    let h = helper();
    ProgramSpec::resolve(
        &h.exe,
        args.iter().map(Into::into).collect(),
        h._temp.path(),
        vec![],
    )
    .unwrap()
}
async fn launch(args: &[&str], protocol: OutputProtocol) -> BoundedOutputHandle {
    SubprocessExecutor::spawn_bounded(&spec(args), Duration::from_secs(30), protocol)
        .await
        .unwrap()
}
async fn completed(
    handle: &mut BoundedOutputHandle,
) -> Arc<vfa_tui::subprocess::bounded::OutputCompletion> {
    match tokio::time::timeout(Duration::from_secs(40), handle.wait()).await {
        Ok(result) => result.unwrap(),
        Err(_) => {
            handle.request_root_stop();
            let _ = handle.wait().await;
            panic!("bounded completion deadline");
        }
    }
}
fn plan() -> OutputProtocol {
    OutputProtocol::Export {
        digest: "a".repeat(64),
        files: vec![ExpectedFile {
            path: "result.txt".into(),
            final_artifact: FinalArtifact {
                sha256: "b".repeat(64),
                size: 7,
                mode: 420,
            },
        }],
    }
}
#[tokio::test]
async fn dual_64m_unterminated_pipes_drain_without_consumers() {
    let mut h = launch(&["output-dual", "67108864", "plain"], OutputProtocol::Text).await;
    let c = completed(&mut h).await;
    assert_eq!(c.exit_code, Some(0));
    assert_eq!(c.stop_reason, None);
    for s in [&c.stdout, &c.stderr] {
        assert_eq!(s.bytes_read, 67108864);
        assert!(s.eof);
        assert_eq!(s.truncated_records, 1);
        assert_eq!(s.truncated_bytes, 67108864 - 4096);
    }
    assert!(!c.descendant_cleanup_verified);
    let b = h.budget();
    assert_eq!(b.limit, 8388608);
    assert_eq!(RAW_BUDGET, b.limit);
    assert!(b.peak <= b.limit);
    assert!(Arc::ptr_eq(&c, &completed(&mut h).await));
}
#[tokio::test]
async fn closed_long_records_count_only_whole_record_loss() {
    let mut h = launch(&["output-dual", "67108864", "plain"], OutputProtocol::Text).await;
    // The current-thread runtime cannot poll the supervisor until our next
    // await, so both display receivers are deterministically closed first.
    drop(h.take_stdout());
    drop(h.take_stderr());
    let c = completed(&mut h).await;
    assert_eq!(c.exit_code, Some(0));
    assert_eq!(c.stop_reason, None);
    assert!(c.root_wait_observed && c.root_reaped);
    for stream in [&c.stdout, &c.stderr] {
        assert_eq!(stream.bytes_read, 67108864);
        assert!(stream.eof);
        assert_eq!(stream.dropped_bytes, stream.bytes_read);
        assert_eq!(stream.dropped_records, 1);
        assert_eq!(stream.truncated_bytes, 0);
        assert_eq!(stream.truncated_records, 0);
    }
}
#[tokio::test]
async fn held_records_and_closed_consumers_do_not_block_and_keep_permits() {
    let mut h = launch(&["output-dual", "16777216", "lines"], OutputProtocol::Text).await;
    let mut out = h.take_stdout().unwrap();
    let mut err = h.take_stderr().unwrap();
    let first = out.recv().await.unwrap();
    assert_eq!(first.text().len(), 4095);
    drop(err.recv().await.unwrap());
    drop(err);
    let c = completed(&mut h).await;
    assert_eq!(c.stdout.bytes_read, 16777216);
    assert_eq!(c.stderr.bytes_read, 16777216);
    assert!(c.stdout.dropped_records > 0);
    assert!(c.stderr.dropped_records > 0);
    assert_eq!(c.stdout.dropped_bytes, c.stdout.dropped_records * 4096);
    assert_eq!(c.stderr.dropped_bytes, c.stderr.dropped_records * 4096);
    let b = h.budget();
    assert!(b.stdout_payload > 0);
    assert!(b.stdout_payload <= b.per_stream_payload_limit);
    assert!(b.peak <= RAW_BUDGET);
    assert!(b.stderr_peak_payload > 0);
    drop(out);
    let held = h.budget();
    assert_eq!(held.stdout_payload, 4096);
    drop(first);
    assert_eq!(h.budget().stdout_payload, 0);
}
#[tokio::test]
async fn both_streams_have_reserved_capacity_and_one_cumulative_cap() {
    let mut h = launch(&["output-dual", "16777216", "lines"], OutputProtocol::Text).await;
    let c = completed(&mut h).await;
    assert!(c.stdout.dropped_records > 0 && c.stderr.dropped_records > 0);
    let b = h.budget();
    assert!(b.stdout_payload > 0 && b.stderr_payload > 0);
    assert!(
        b.stdout_payload <= b.per_stream_payload_limit
            && b.stderr_payload <= b.per_stream_payload_limit
    );
    assert!(b.stdout_payload + b.stderr_payload + b.fixed_reservation <= RAW_BUDGET);
    assert!(b.peak <= RAW_BUDGET);
}
#[tokio::test]
async fn unicode_invalid_and_split_sequences_are_bounded_valid_text() {
    let mut h = launch(&["output-unicode"], OutputProtocol::Text).await;
    let mut out = h.take_stdout().unwrap();
    let c = completed(&mut h).await;
    assert_eq!(c.exit_code, Some(0));
    let a = out.recv().await.unwrap();
    assert_eq!(a.text(), "hello Ω 🦀");
    assert!(!a.truncated);
    let b = out.recv().await.unwrap();
    assert_eq!(b.text(), "x�y");
    let c = out.recv().await.unwrap();
    assert!(c.text().len() <= 4096);
    assert!(c.truncated);
    assert!(c.text().ends_with('a'));
}
#[tokio::test]
async fn empty_records_are_slot_bounded() {
    let mut h = launch(&["output-empty", "20000"], OutputProtocol::Text).await;
    let c = completed(&mut h).await;
    assert_eq!(c.stdout.bytes_read, 20000);
    assert!(c.stdout.dropped_records > 0);
    assert_eq!(c.stdout.dropped_records, c.stdout.dropped_bytes);
    assert!(h.budget().stdout_payload <= h.budget().per_stream_payload_limit);
}
#[tokio::test]
async fn cancelled_wait_observer_and_explicit_stop_reap_root() {
    let mut h = launch(&["output-pause", "1000"], OutputProtocol::Text).await;
    assert!(tokio::time::timeout(Duration::from_millis(10), h.wait())
        .await
        .is_err());
    h.request_root_stop();
    let c = completed(&mut h).await;
    assert_eq!(c.stop_reason, Some(StopReason::Requested));
    assert!(c.root_wait_observed);
    assert!(c.stdout.eof && c.stderr.eof);
    assert!(!c.descendant_cleanup_verified);
    assert!(Arc::ptr_eq(&c, &completed(&mut h).await));
}
#[tokio::test]
async fn timeout_and_nonzero_are_reliably_observed() {
    let mut h = SubprocessExecutor::spawn_bounded(
        &spec(&["output-pause", "1000"]),
        Duration::from_millis(10),
        OutputProtocol::Text,
    )
    .await
    .unwrap();
    assert_eq!(
        completed(&mut h).await.stop_reason,
        Some(StopReason::Timeout)
    );
    let mut h = launch(&["exit", "7"], OutputProtocol::Text).await;
    let c = completed(&mut h).await;
    assert_eq!(c.exit_code, Some(7));
    assert_eq!(c.stop_reason, None);
}
#[tokio::test]
async fn export_uses_raw_stdout_before_loss_and_retains_effects_on_error() {
    for mode in ["valid", "overflow", "invalid", "hostile", "wire-overflow"] {
        let mut protocol = plan();
        if mode == "wire-overflow" {
            if let OutputProtocol::Export { files, .. } = &mut protocol {
                for index in 1..200 {
                    files.push(ExpectedFile {
                        path: format!("f{index}"),
                        final_artifact: files[0].final_artifact.clone(),
                    });
                }
            }
        }
        let mut h = launch(&["output-export", mode], protocol).await;
        drop(h.take_stdout());
        drop(h.take_stderr());
        let c = completed(&mut h).await;
        let parser = c.export().unwrap();
        if mode == "wire-overflow" {
            assert_eq!(parser.outcomes().len(), 127);
        } else {
            assert_eq!(parser.outcomes().len(), 1, "{mode}");
        }
        assert!(c.stderr.bytes_read > 0);
        if mode == "valid" {
            assert_eq!(c.protocol_error, None);
            assert!(parser.summary().is_some());
        } else {
            let expected = match mode {
                "overflow" => ProtocolError::RecordLimit,
                "wire-overflow" => ProtocolError::ProtocolLimit,
                _ => ProtocolError::InvalidRecord,
            };
            assert_eq!(c.protocol_error, Some(expected));
            assert_eq!(c.stop_reason, Some(StopReason::Protocol));
        }
        assert_eq!(h.budget().stdout_payload, 0);
        assert!(c.root_wait_observed && c.root_reaped);
        assert_eq!(h.budget().fixed_reservation, 2 * 3 * 4096 + 32 * 65536);
        assert!(h.budget().peak <= RAW_BUDGET);
    }
}
#[tokio::test]
async fn invalid_export_plan_rejects_before_child_launch() {
    let h = SubprocessExecutor::spawn_bounded(
        &spec(&["output-marker"]),
        Duration::from_secs(3),
        OutputProtocol::Export {
            digest: "invalid".into(),
            files: vec![],
        },
    )
    .await;
    assert!(h.is_err());
    assert!(!helper()._temp.path().join("must-not-launch").exists());
}
