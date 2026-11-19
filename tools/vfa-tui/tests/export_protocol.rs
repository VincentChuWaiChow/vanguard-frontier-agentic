//! Parser preparation only; not output-budget, journal or native integration proof.

use std::path::Path;
use std::process::Command;

use serde_json::{json, Value};
use vfa_tui::engine::export_protocol::{
    ApplyStatus, ExpectedFile, ExportProtocol, FinalArtifact, ProtocolError, MAX_PROTOCOL_BYTES,
    MAX_RECORD_BYTES,
};

fn digest() -> String {
    "a".repeat(64)
}

fn files() -> Vec<ExpectedFile> {
    vec![
        ExpectedFile {
            path: ".codex/agents/Ω 東京.toml".into(),
            final_artifact: FinalArtifact {
                sha256: "b".repeat(64),
                size: 42,
                mode: 0o644,
            },
        },
        ExpectedFile {
            path: ".codex/skills/example/SKILL.md".into(),
            final_artifact: FinalArtifact {
                sha256: "c".repeat(64),
                size: 17,
                mode: 0o755,
            },
        },
    ]
}

fn parser() -> ExportProtocol {
    ExportProtocol::new(digest(), files()).unwrap()
}

fn record(kind: &str, seq: usize, fields: Value) -> Value {
    let mut result =
        json!({"protocol":"vfa-export/1", "plan_digest":digest(), "seq":seq, "type":kind});
    result
        .as_object_mut()
        .unwrap()
        .extend(fields.as_object().unwrap().clone());
    result
}

fn start() -> Value {
    record("apply-start", 1, json!({"file_count":2}))
}

fn outcome(index: usize) -> Value {
    let file = &files()[index];
    record(
        "file-outcome",
        index + 2,
        json!({
            "file_index":index, "path":file.path, "status":"WRITTEN",
            "final":{"sha256":file.final_artifact.sha256,"size":file.final_artifact.size,"mode":file.final_artifact.mode}
        }),
    )
}

fn summary() -> Value {
    record(
        "apply-summary",
        4,
        json!({"status":"SUCCEEDED", "planned":2, "written":2,
        "failed_index":null, "unattempted":0, "uncertain":false, "code":null}),
    )
}

fn failed_summary(attempted: bool) -> Value {
    record(
        "apply-summary",
        3,
        json!({"status":"FAILED", "planned":2, "written":1,
        "failed_index":if attempted {json!(1)} else {Value::Null},
        "unattempted":if attempted {0} else {1}, "uncertain":attempted, "code":"WRITE_FAILED"}),
    )
}

fn line(value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(value).unwrap();
    bytes.push(b'\n');
    bytes
}

fn success_bytes() -> Vec<u8> {
    [start(), outcome(0), outcome(1), summary()]
        .iter()
        .flat_map(line)
        .collect()
}

fn feed_prefix(parser: &mut ExportProtocol) {
    parser.feed(&line(&start())).unwrap();
    parser.feed(&line(&outcome(0))).unwrap();
}

#[test]
fn success_survives_every_byte_split_including_utf8() {
    let bytes = success_bytes();
    for split in 0..=bytes.len() {
        let mut parser = parser();
        parser.feed(&bytes[..split]).unwrap();
        parser.feed(&bytes[split..]).unwrap();
        assert_eq!(
            parser.finish(Some(0)).unwrap().status,
            ApplyStatus::Succeeded
        );
        assert_eq!(parser.outcomes().len(), 2);
        assert_eq!(parser.outcomes()[0].file, files()[0]);
    }
    let mut parser = parser();
    for byte in bytes {
        parser.feed(&[byte]).unwrap();
    }
    parser.finish(Some(0)).unwrap();
}

#[test]
fn empty_expected_plan_is_rejected_like_the_producer() {
    assert!(ExportProtocol::new(digest(), vec![]).is_err());
}

#[test]
fn invalid_expected_plan_is_not_accepted_as_a_trust_anchor() {
    for path in [
        "",
        "/absolute",
        "../escape",
        "a/../b",
        "a//b",
        "\\\\server\\share",
        "C:/drive",
        "C:\\drive",
        "a/./b",
        "a\0b",
    ] {
        let mut expected = files();
        expected[0].path = path.into();
        assert!(ExportProtocol::new(digest(), expected).is_err(), "{path:?}");
    }
    for value in ["", "A", &"A".repeat(64), &"g".repeat(64)] {
        assert!(ExportProtocol::new(value.into(), files()).is_err());
    }
    let mut expected = files();
    expected[1].path = expected[0].path.clone();
    assert!(ExportProtocol::new(digest(), expected).is_err());
    for (hash, size, mode) in [
        ("bad".into(), 1, 0o644),
        (digest(), 256 * 1024 * 1024 + 1, 0o644),
        (digest(), 1, 0o2000),
    ] {
        let mut expected = files();
        expected[0].final_artifact = FinalArtifact {
            sha256: hash,
            size,
            mode,
        };
        assert!(ExportProtocol::new(digest(), expected).is_err());
    }
}

#[test]
fn start_must_match_protocol_identity_count_and_order() {
    for (field, value) in [
        ("protocol", json!("vfa-export/2")),
        ("plan_digest", json!("b".repeat(64))),
        ("seq", json!(0)),
        ("file_count", json!(3)),
        ("extra", json!(true)),
        ("type", json!("unknown")),
    ] {
        let mut bad = start();
        bad[field] = value;
        assert!(parser().feed(&line(&bad)).is_err(), "{field}");
    }
    assert!(parser().feed(&line(&outcome(0))).is_err());
    assert!(parser().feed(&line(&summary())).is_err());
    let mut parser = parser();
    parser.feed(&line(&start())).unwrap();
    assert!(parser.feed(&line(&start())).is_err());
}

#[test]
fn outcomes_must_match_every_authoritative_plan_field() {
    for (field, value) in [
        ("protocol", json!("wrong")),
        ("plan_digest", json!("b".repeat(64))),
        ("seq", json!(4)),
        ("file_index", json!(1)),
        ("path", json!("../escape")),
        ("status", json!("FAILED")),
        ("extra", json!(true)),
        ("file_index", json!(-1)),
        ("file_index", json!(0.0)),
        ("path", json!(null)),
    ] {
        let mut parser = parser();
        parser.feed(&line(&start())).unwrap();
        let mut bad = outcome(0);
        bad[field] = value;
        assert!(parser.feed(&line(&bad)).is_err(), "{field}");
        assert!(parser.outcomes().is_empty());
    }
    for (field, value) in [
        ("sha256", json!("d".repeat(64))),
        ("size", json!(43)),
        ("mode", json!(0o600)),
        ("extra", json!(true)),
    ] {
        let mut parser = parser();
        parser.feed(&line(&start())).unwrap();
        let mut bad = outcome(0);
        bad["final"][field] = value;
        assert!(parser.feed(&line(&bad)).is_err(), "final.{field}");
    }
}

#[test]
fn strict_json_rejects_duplicate_missing_and_unknown_fields() {
    for original in [start(), outcome(0), summary()] {
        for key in original.as_object().unwrap().keys() {
            let mut missing = original.clone();
            missing.as_object_mut().unwrap().remove(key);
            let mut duplicate = String::from_utf8(line(&original)).unwrap();
            duplicate.insert_str(
                1,
                &format!("{}:{},", serde_json::to_string(key).unwrap(), original[key]),
            );
            for bytes in [line(&missing), duplicate.into_bytes()] {
                let mut parser = parser();
                if original["type"] != "apply-start" {
                    parser.feed(&line(&start())).unwrap();
                }
                if original["type"] == "apply-summary" {
                    parser.feed(&line(&outcome(0))).unwrap();
                    parser.feed(&line(&outcome(1))).unwrap();
                }
                assert!(parser.feed(&bytes).is_err(), "{} {key}", original["type"]);
            }
        }
    }
    for bytes in [
        b"\n".as_slice(),
        b"{bad}\n",
        b"\xff\n",
        b"{} {}\n",
        b"null\n",
    ] {
        assert!(parser().feed(bytes).is_err());
    }
}

#[test]
fn summary_cannot_contradict_observed_writes_or_success() {
    for (field, value) in [
        ("seq", json!(5)),
        ("planned", json!(3)),
        ("written", json!(1)),
        ("failed_index", json!(0)),
        ("unattempted", json!(1)),
        ("uncertain", json!(true)),
        ("code", json!("WRITE_FAILED")),
        ("extra", json!(0)),
        ("status", json!("FAILED")),
    ] {
        let mut parser = parser();
        feed_prefix(&mut parser);
        parser.feed(&line(&outcome(1))).unwrap();
        let mut bad = summary();
        bad[field] = value;
        assert!(parser.feed(&line(&bad)).is_err(), "{field}");
        assert_eq!(parser.outcomes().len(), 2);
    }
    let mut parser = parser();
    feed_prefix(&mut parser);
    let mut premature = summary();
    premature["seq"] = json!(3);
    assert!(parser.feed(&line(&premature)).is_err());
}

#[test]
fn failed_summaries_preserve_known_effects_and_uncertainty() {
    for attempted in [false, true] {
        let mut parser = parser();
        feed_prefix(&mut parser);
        parser.feed(&line(&failed_summary(attempted))).unwrap();
        assert_eq!(parser.summary().unwrap().uncertain, attempted);
        assert_eq!(parser.finish(Some(0)), Err(ProtocolError::ApplyFailed));
        assert_eq!(parser.outcomes().len(), 1);
        assert_eq!(parser.summary().unwrap().status, ApplyStatus::Failed);
    }
    for code in ["OUTPUT_FAILED", "PROTOCOL_LIMIT"] {
        let mut parser = parser();
        parser.feed(&line(&start())).unwrap();
        parser
            .feed(&line(&record(
                "apply-summary",
                2,
                json!({"status":"FAILED","planned":2,
            "written":0,"failed_index":null,"unattempted":2,"uncertain":false,"code":code}),
            )))
            .unwrap();
        assert!(parser.finish(Some(1)).is_err());
    }
}

#[test]
fn contradictory_failed_summary_arithmetic_is_rejected() {
    for (field, value) in [
        ("failed_index", json!(0)),
        ("failed_index", json!(null)),
        ("uncertain", json!(false)),
        ("unattempted", json!(1)),
        ("written", json!(2)),
        ("code", json!(null)),
        ("code", json!("arbitrary secret message")),
    ] {
        let mut parser = parser();
        feed_prefix(&mut parser);
        let mut bad = failed_summary(true);
        bad[field] = value;
        assert!(parser.feed(&line(&bad)).is_err(), "{field}");
        assert_eq!(parser.outcomes().len(), 1);
    }
}

#[test]
fn eof_exit_and_terminal_order_are_independent_success_requirements() {
    assert_eq!(parser().finish(Some(0)), Err(ProtocolError::Incomplete));
    let bytes = success_bytes();
    for end in [line(&start()).len(), bytes.len() - 1] {
        let mut parser = parser();
        parser.feed(&bytes[..end]).unwrap();
        assert_eq!(parser.finish(Some(0)), Err(ProtocolError::Incomplete));
    }
    for exit in [Some(1), Some(-1), None] {
        let mut parser = parser();
        parser.feed(&bytes).unwrap();
        assert_eq!(parser.finish(exit), Err(ProtocolError::ChildExit));
        assert_eq!(parser.outcomes().len(), 2);
    }
    for tail in [line(&summary()), line(&outcome(0)), vec![b' ']] {
        let mut parser = parser();
        parser.feed(&bytes).unwrap();
        assert!(parser.feed(&tail).is_err());
        assert!(parser.finish(Some(0)).is_err());
        assert_eq!(parser.outcomes().len(), 2);
    }
    let mut parser = parser();
    parser.feed(&bytes).unwrap();
    parser.finish(Some(0)).unwrap();
    assert_eq!(parser.feed(b"x"), Err(ProtocolError::Closed));
}

#[test]
fn errors_latch_without_erasing_preceding_effects_even_in_one_chunk() {
    let mut bytes = [start(), outcome(0)]
        .iter()
        .flat_map(line)
        .collect::<Vec<_>>();
    bytes.extend_from_slice(b"not json\n");
    bytes.extend_from_slice(&line(&outcome(1)));
    let mut parser = parser();
    let error = parser.feed(&bytes).unwrap_err();
    assert_eq!(parser.outcomes().len(), 1);
    assert_eq!(parser.feed(&success_bytes()), Err(error));
    assert_eq!(parser.finish(Some(0)), Err(error));
    assert!(parser.summary().is_none());
}

fn padded(value: &Value, length: usize) -> Vec<u8> {
    let mut bytes = line(value);
    assert!(bytes.len() <= length);
    bytes.pop();
    bytes.resize(length - 1, b' ');
    bytes.push(b'\n');
    bytes
}

#[test]
fn record_limit_includes_newline_and_never_buffers_an_oversized_chunk() {
    let mut exact = parser();
    exact.feed(&padded(&start(), MAX_RECORD_BYTES)).unwrap();
    for bytes in [
        padded(&start(), MAX_RECORD_BYTES + 1),
        vec![b'x'; 64 * 1024 * 1024],
    ] {
        let mut parser = parser();
        feed_prefix(&mut parser);
        assert_eq!(parser.feed(&bytes), Err(ProtocolError::RecordLimit));
        assert_eq!(parser.outcomes().len(), 1);
        assert_eq!(parser.buffered_bytes(), 0);
    }
    let mut partial = parser();
    partial.feed(&vec![b' '; MAX_RECORD_BYTES - 1]).unwrap();
    assert_eq!(partial.feed(b" "), Err(ProtocolError::RecordLimit));
}

#[test]
fn cumulative_protocol_limit_has_an_exact_boundary_and_preserves_prefix() {
    let expected = (0..128)
        .map(|i| ExpectedFile {
            path: format!("files/{i}.md"),
            ..files()[0].clone()
        })
        .collect::<Vec<_>>();
    for extra in [0, 1] {
        let mut parser = ExportProtocol::new(digest(), expected.clone()).unwrap();
        let mut records = vec![record("apply-start", 1, json!({"file_count":128}))];
        records.extend(expected.iter().enumerate().map(|(i, f)| record("file-outcome", i+2,
            json!({"file_index":i,"path":f.path,"status":"WRITTEN","final":{"sha256":f.final_artifact.sha256,"size":42,"mode":0o644}}))));
        records.push(record(
            "apply-summary",
            130,
            json!({"status":"SUCCEEDED", "planned":128,
            "written":128,"failed_index":null,"unattempted":0,"uncertain":false,"code":null}),
        ));
        // 130 records. Pad 127 fully; distribute the remaining one-record budget.
        let last_lengths = [MAX_RECORD_BYTES - 2048, 1024, 1024 + extra];
        for (index, value) in records.iter().enumerate() {
            let length = if index < 127 {
                MAX_RECORD_BYTES
            } else {
                last_lengths[index - 127]
            };
            let result = parser.feed(&padded(value, length));
            if extra == 1 && index == 129 {
                assert_eq!(result, Err(ProtocolError::ProtocolLimit));
            } else {
                result.unwrap();
            }
        }
        assert_eq!(parser.outcomes().len(), 128);
        if extra == 0 {
            assert_eq!(parser.bytes_received(), MAX_PROTOCOL_BYTES);
            parser.finish(Some(0)).unwrap();
        } else {
            assert!(parser.finish(Some(0)).is_err());
        }
    }
}

fn copy_tree(source: &Path, target: &Path) {
    std::fs::create_dir_all(target).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = target.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn actual_node_apply(extra_paths: &[&str], no_skills: bool) {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let temp = tempfile::tempdir().unwrap();
    let package = temp.path().join("source 東京 spaces");
    copy_tree(
        &repository.join("tests/fixtures/export-plan/package"),
        &package,
    );
    for path in extra_paths {
        std::fs::write(
            package.join("skills/aws/alpha/references").join(path),
            b"literal companion bytes\n",
        )
        .unwrap();
    }
    std::fs::create_dir(package.join("scripts")).unwrap();
    let script = package.join("scripts/export-marketplace-agents.mjs");
    std::fs::copy(
        repository.join("scripts/export-marketplace-agents.mjs"),
        &script,
    )
    .unwrap();
    let target = temp.path().join("target Ω");
    let mut command = Command::new("node");
    command
        .arg(&script)
        .args([
            "--platform",
            "codex",
            "--agents",
            "alpha-agent",
            "--plan-json",
            "--repo",
        ])
        .arg(&target);
    if no_skills {
        command.arg("--no-skills");
    }
    let preview = command
        .output()
        .expect("installed Node is required; never skip or install");
    assert!(
        preview.status.success(),
        "{}",
        String::from_utf8_lossy(&preview.stderr)
    );
    assert!(!target.exists(), "preview must not write the target");
    let plan: Value = serde_json::from_slice(&preview.stdout).unwrap();
    let expected = plan["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|file| ExpectedFile {
            path: file["destination"]["relative"].as_str().unwrap().into(),
            final_artifact: serde_json::from_value(file["final"].clone()).unwrap(),
        })
        .collect::<Vec<_>>();
    if no_skills {
        assert_eq!(expected.len(), 1);
    }
    for path in extra_paths {
        assert!(expected
            .iter()
            .any(|file| file.path == format!(".codex/skills/alpha/references/{path}")));
    }
    assert_eq!(expected[0].path, ".codex/agents/alpha-agent.toml");
    let plan_file = temp.path().join("approved.json");
    std::fs::write(&plan_file, preview.stdout).unwrap();
    let apply = Command::new("node")
        .arg(script)
        .arg("--apply-plan")
        .arg(plan_file)
        .output()
        .unwrap();
    assert!(
        apply.status.success(),
        "{}",
        String::from_utf8_lossy(&apply.stderr)
    );
    let mut parser =
        ExportProtocol::new(plan["digest"].as_str().unwrap().into(), expected.clone()).unwrap();
    for chunk in apply.stdout.chunks(7) {
        parser.feed(chunk).unwrap();
    }
    parser.finish(apply.status.code()).unwrap();
    assert_eq!(parser.outcomes()[0].file, expected[0]);
    assert_eq!(parser.outcomes().len(), expected.len());
    for file in expected {
        let actual = std::fs::read(target.join(&file.path)).unwrap();
        use sha2::{Digest, Sha256};
        assert_eq!(
            format!("{:x}", Sha256::digest(&actual)),
            file.final_artifact.sha256
        );
        assert_eq!(actual.len() as u64, file.final_artifact.size);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(target.join(&file.path))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o7777,
                file.final_artifact.mode
            );
        }
    }
}

#[test]
fn consumes_actual_node_apply_stdout_against_the_preview() {
    actual_node_apply(&[], true);
}

#[cfg(unix)]
#[test]
fn consumes_actual_node_legal_posix_filenames_without_display_normalization() {
    actual_node_apply(&["time:zone.md", "a\\b.md", "line\nname.md"], false);
}
