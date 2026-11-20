//! Safe display projection tests; raw protocol validation is a separate boundary.
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use std::time::Duration;
use vfa_tui::subprocess::bounded::{OutputProtocol, RAW_BUDGET};
use vfa_tui::subprocess::program::ProgramSpec;
use vfa_tui::subprocess::SubprocessExecutor;

struct Helper {
    _temp: tempfile::TempDir,
    exe: PathBuf,
}
fn helper() -> &'static Helper {
    static H: OnceLock<Helper> = OnceLock::new();
    H.get_or_init(|| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/no-install-three/2.5");
        std::fs::create_dir_all(&root).unwrap();
        let temp = tempfile::tempdir_in(root).unwrap();
        let src = temp.path().join("helper.rs");
        let exe = temp.path().join(format!("helper{}", std::env::consts::EXE_SUFFIX));
        std::fs::write(&src, r#"
use std::io::{self,Write};
fn main() {
 let mode=std::env::args().nth(1).unwrap();
 let mut out=io::stdout().lock(); let mut err=io::stderr().lock();
 let token="ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmn";
 if mode=="truncate" { writeln!(out,"{}{}", " ".repeat(4080),token).unwrap(); }
 else if mode=="export" {
 let digest="a".repeat(64); let hash="b".repeat(64);
 writeln!(out,"{{\"type\":\"apply-start\",\"protocol\":\"vfa-export/1\",\"plan_digest\":\"{digest}\",\"seq\":1,\"file_count\":1}}").unwrap();
 writeln!(out,"{{\"type\":\"file-outcome\",\"protocol\":\"vfa-export/1\",\"plan_digest\":\"{digest}\",\"seq\":2,\"file_index\":0,\"path\":\"{token}.txt\",\"status\":\"WRITTEN\",\"final\":{{\"sha256\":\"{hash}\",\"size\":7,\"mode\":420}}}}").unwrap();
 writeln!(out,"{{\"type\":\"apply-summary\",\"protocol\":\"vfa-export/1\",\"plan_digest\":\"{digest}\",\"seq\":3,\"status\":\"SUCCEEDED\",\"planned\":1,\"written\":1,\"failed_index\":null,\"unattempted\":0,\"uncertain\":false,\"code\":null}}").unwrap();
 writeln!(err,"{token}").unwrap();
 }
 else if mode=="pem" { write!(out,"-----BEGIN RSA PRIVATE KEY-----\n{}\n-----END RSA PRIVATE KEY-----\nafter\n", "body\n".repeat(100000)).unwrap(); }
 else if mode=="cancel" { write!(out,"-----BEGIN PRIVATE KEY-----\n{}", "body".repeat(10000)).unwrap(); out.flush().unwrap(); std::thread::sleep(std::time::Duration::from_secs(30)); }
 else { for _ in 0..5000 { writeln!(out,"{token}").unwrap(); writeln!(err,"{token}").unwrap(); } }
}
"#).unwrap();
        assert!(Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
            .args(["--edition=2021", "--crate-name=stream_redact_helper"])
            .arg(src).arg("-o").arg(&exe).status().unwrap().success());
        Helper { _temp: temp, exe }
    })
}
async fn launch(mode: &str) -> vfa_tui::subprocess::bounded::BoundedOutputHandle {
    let h = helper();
    let p = ProgramSpec::resolve(&h.exe, vec![mode.into()], h._temp.path(), vec![]).unwrap();
    SubprocessExecutor::spawn_bounded_redacted(&p, Duration::from_secs(10), OutputProtocol::Text)
        .await
        .unwrap()
}
#[tokio::test]
async fn secret_is_redacted_before_display_prefix_cut() {
    let mut h = launch("truncate").await;
    let mut rx = h.take_stdout().unwrap();
    let c = h.wait().await.unwrap();
    let record = rx.recv().await.unwrap();
    assert!(
        !record.text().contains("ghp_"),
        "unsafe prefix: {}",
        record.text()
    );
    assert!(record.text().contains("[REDACTED]"));
    assert_eq!(c.stdout.bytes_read, 4125);
    assert_eq!(c.stdout.truncated_bytes, 28);
    assert!(record.truncated);
    assert!(h.budget().peak <= RAW_BUDGET);
}

use vfa_tui::security::redact::StreamingRedactor;
fn project(input: &[u8], split: usize) -> String {
    let mut r = StreamingRedactor::default();
    let mut out = Vec::new();
    r.feed(&input[..split], |s| out.extend_from_slice(s));
    r.feed(&input[split..], |s| out.extend_from_slice(s));
    r.finish(|s| out.extend_from_slice(s));
    String::from_utf8(out).unwrap()
}
#[test]
fn every_byte_split_suppresses_all_documented_classes() {
    for token in [
        "ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmn",
        "github_pat_ABCDEFGHIJKLMNOPQRSTUVWXYZ_123456",
        "npm_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmn",
        "sk-abcdefghijklmnopqrstuvwxyz",
        "xoxb-abcdefghijklmnopqrstuvwxyz",
        "xoxp-abcdefghijklmnopqrstuvwxyz",
        "AKIAIOSFODNN7EXAMPLE1",
        "abcdefghijk.abcdefghijk.abcdefghijk",
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijk+mnopqrs=",
        "-----BEGIN RSA PRIVATE KEY-----\nabc\n-----END RSA PRIVATE KEY-----",
    ] {
        let input = format!("before {token} after");
        for split in 0..=input.len() {
            let got = project(input.as_bytes(), split);
            assert!(!got.contains(token), "{split}: {got}");
            assert!(got.contains("[REDACTED]"), "{split}: {got}");
            assert!(got.ends_with(" after"), "{split}: {got}");
        }
    }
}
#[test]
fn bytewise_ansi_controls_and_unicode_do_not_break_recognition() {
    let input =
        "café 🦀 ghp_ABC\u{1b}[31mDEFGHIJKLM\u{0}NOPQRSTUVWXYZabcdefghijklmn\u{1b}[0m done\n";
    let mut r = StreamingRedactor::default();
    let mut out = Vec::new();
    for b in input.as_bytes() {
        r.feed(&[*b], |s| out.extend_from_slice(s));
    }
    r.finish(|s| out.extend_from_slice(s));
    assert_eq!(String::from_utf8(out).unwrap(), "café 🦀 [REDACTED] done\n");
}
#[test]
fn huge_ambiguous_and_unterminated_pem_are_bounded_loss() {
    let input = format!("before {}+ after", "a".repeat(100000));
    let got = project(input.as_bytes(), 50000);
    assert!(got.contains("[REDACTED: ambiguous output suppressed]"));
    assert!(!got.contains("aaaa"));
    assert!(got.ends_with(" after"));
    let input = format!("-----BEGIN PRIVATE KEY-----\n{}", "secret\n".repeat(100000));
    let got = project(input.as_bytes(), 17);
    assert!(!got.contains("secret"));
    assert!(got.contains("[REDACTED]"));
    assert!(got.contains("[REDACTED: incomplete output suppressed]"));
    assert!(std::mem::size_of::<StreamingRedactor>() <= 1024);
}
#[tokio::test]
async fn long_pem_and_slow_or_dropped_receivers_keep_terminal_reliable() {
    let mut h = launch("pem").await;
    let mut rx = h.take_stdout().unwrap();
    let c = tokio::time::timeout(Duration::from_secs(15), h.wait())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(c.exit_code, Some(0));
    assert!(c.stdout.dropped_records > 0);
    while let Some(r) = rx.recv().await {
        assert!(!r.text().contains("body"));
    }
    assert!(h.budget().peak <= RAW_BUDGET);
    let mut h = launch("pressure").await;
    drop(h.take_stdout());
    drop(h.take_stderr());
    let c = tokio::time::timeout(Duration::from_secs(15), h.wait())
        .await
        .unwrap()
        .unwrap();
    assert!(c.root_reaped && c.stdout.eof && c.stderr.eof);
    assert_eq!(c.exit_code, Some(0));
    assert!(c.stdout.dropped_records > 0 && c.stderr.dropped_records > 0);
}
#[tokio::test]
async fn cancel_flushes_pending_safe_state_without_secret_disclosure() {
    let mut h = launch("cancel").await;
    let mut rx = h.take_stdout().unwrap();
    // Wait for a completed header record: this is pipe progress, not a sleep race.
    let first = tokio::time::timeout(Duration::from_secs(5), rx.recv())
        .await
        .unwrap()
        .unwrap();
    assert!(first.text().contains("[REDACTED]"));
    h.request_root_stop();
    let c = tokio::time::timeout(Duration::from_secs(5), h.wait())
        .await
        .unwrap()
        .unwrap();
    assert!(c.root_reaped);
    while let Some(r) = rx.recv().await {
        assert!(!r.text().contains("body"));
    }
}

#[tokio::test]
async fn raw_export_validation_is_not_redacted_or_exposed_as_display() {
    use vfa_tui::engine::export_protocol::{ExpectedFile, FinalArtifact};
    let h = helper();
    let p = ProgramSpec::resolve(&h.exe, vec!["export".into()], h._temp.path(), vec![]).unwrap();
    let path = "ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmn.txt";
    let protocol = OutputProtocol::Export {
        digest: "a".repeat(64),
        files: vec![ExpectedFile {
            path: path.into(),
            final_artifact: FinalArtifact {
                sha256: "b".repeat(64),
                size: 7,
                mode: 420,
            },
        }],
    };
    let mut handle =
        SubprocessExecutor::spawn_bounded_redacted(&p, Duration::from_secs(10), protocol)
            .await
            .unwrap();
    let mut out = handle.take_stdout().unwrap();
    let mut err = handle.take_stderr().unwrap();
    let completion = handle.wait().await.unwrap();
    assert_eq!(completion.protocol_error, None);
    assert!(completion.export().unwrap().summary().is_some());
    assert_eq!(completion.export().unwrap().outcomes()[0].file.path, path);
    assert!(out.recv().await.is_none());
    assert_eq!(err.recv().await.unwrap().text(), "[REDACTED]");
    assert!(handle.budget().peak <= RAW_BUDGET);
}
#[test]
fn controls_invalid_utf8_and_incomplete_escape_remain_inert() {
    for input in [
        b"hello\xffworld".as_slice(),
        b"hello\x1b]hidden",
        b"hello\xc3",
    ] {
        for split in 0..=input.len() {
            let output = project(input, split);
            assert!(!output.contains('\u{1b}'));
            assert!(!output.contains("hidden"));
            assert!(output.starts_with("hello"));
        }
    }
    let input = "café\u{009b}🦀";
    assert_eq!(project(input.as_bytes(), 6), "café🦀");
    assert!(project(b"ghp_abcd", 4).contains("incomplete output suppressed"));
}

#[test]
fn ansi_insertions_inside_each_class_are_recognized_bytewise() {
    for token in [
        "ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmn",
        "github_pat_ABCDEFGHIJKLMNOPQRSTUVWXYZ_123456",
        "npm_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmn",
        "sk-abcdefghijklmnopqrstuvwxyz",
        "xoxb-abcdefghijklmnopqrstuvwxyz",
        "xoxp-abcdefghijklmnopqrstuvwxyz",
        "AKIAIOSFODNN7EXAMPLE1",
        "abcdefghijk.abcdefghijk.abcdefghijk",
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijk+mnopqrs=",
        "-----BEGIN PRIVATE KEY-----\nabc\n-----END PRIVATE KEY-----",
    ] {
        for split in 0..token.len() {
            let input = format!(
                "{}{}{} ",
                &token[..split],
                "\u{1b}]title\u{1b}\\",
                &token[split..]
            );
            let mut r = StreamingRedactor::default();
            let mut out = Vec::new();
            for byte in input.as_bytes() {
                r.feed(&[*byte], |s| out.extend_from_slice(s));
            }
            r.finish(|s| out.extend_from_slice(s));
            let got = String::from_utf8(out).unwrap();
            assert!(got.contains("[REDACTED]"), "{split}: {got}");
            assert!(!got.contains(token), "{split}: {got}");
            assert!(!got.contains("title"));
        }
    }
}
#[tokio::test]
async fn safe_queues_consume_reduced_quotas_without_blocking_drain() {
    let mut h = launch("pressure").await;
    let c = tokio::time::timeout(Duration::from_secs(15), h.wait())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(c.exit_code, Some(0));
    assert!(c.stdout.dropped_records > 0 && c.stderr.dropped_records > 0);
    let budget = h.budget();
    assert_eq!(budget.fixed_reservation, 2 * 3 * 4096 + 2 * 4 * 4096);
    assert!(budget.peak <= RAW_BUDGET);
    assert_eq!(budget.stdout_peak_payload, budget.per_stream_payload_limit);
    assert_eq!(budget.stderr_peak_payload, budget.per_stream_payload_limit);
    let mut out = h.take_stdout().unwrap();
    let mut err = h.take_stderr().unwrap();
    while let Some(record) = out.recv().await {
        assert_eq!(record.text(), "[REDACTED]");
    }
    while let Some(record) = err.recv().await {
        assert_eq!(record.text(), "[REDACTED]");
    }
    assert_eq!(h.budget().current, budget.fixed_reservation);
}
