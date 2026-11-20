//! Synthetic engine-only checks; no native frontend or cross-platform proof.
#![cfg(unix)]
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use vfa_tui::engine::findings::{
    dependency_findings, integrity_findings, FileStatus, FindingsQuality,
};
use vfa_tui::engine::session::SourceSession;

const HELLO: &str = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
fn copy_tree(source: &Path, target: &Path) {
    fs::create_dir_all(target).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target.join(entry.file_name()));
        } else {
            fs::copy(entry.path(), target.join(entry.file_name())).unwrap();
        }
    }
}
fn fixture() -> (tempfile::TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("synthetic source Ω");
    copy_tree(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/compatibility/core"),
        &root,
    );
    fs::write(
        root.join("package.json"),
        r#"{"name":"@raishin/vanguard-frontier-agentic"}"#,
    )
    .unwrap();
    (temp, root)
}
fn write_json(root: &Path, name: &str, value: Value) {
    fs::write(root.join("catalog").join(name), value.to_string()).unwrap();
}
fn manifest(root: &Path, paths: &[&str]) {
    write_json(
        root,
        "asset-integrity.json",
        json!({"manifest_version":1,"algorithm":"sha256","scope":{"trees":[],"root_files":paths},"trees":[],"root_files":paths.iter().map(|p| json!({"path":p,"sha256":HELLO,"bytes":11})).collect::<Vec<_>>(),"aggregate_sha256":"0".repeat(64)}),
    );
}
fn snapshot(root: &Path) -> std::sync::Arc<vfa_tui::engine::snapshot::CatalogSnapshot> {
    SourceSession::new().select(root).unwrap()
}

#[test]
fn exact_graph_provenance_and_traversal_are_snapshot_bound() {
    let (_temp, root) = fixture();
    let snapshot = snapshot(&root);
    let findings = dependency_findings(&snapshot);
    assert_eq!(findings.quality, FindingsQuality::Current);
    assert_eq!(findings.context, *snapshot.context());
    assert_eq!(findings.inputs.len(), 5);
    for input in &findings.inputs {
        assert_eq!(
            input.sha256.as_deref(),
            snapshot.catalog().content_hash_for(&input.path)
        );
    }
    let graph = findings.graph.as_ref().unwrap();
    let edges: Vec<_> = graph
        .edges
        .iter()
        .map(|e| (e.from.as_str(), e.to.as_str(), format!("{:?}", e.edge_type)))
        .collect();
    assert_eq!(
        edges,
        vec![
            ("alpha-agent", "codex-rule", "Configures".into()),
            ("alpha-agent", "identity-mcp", "References".into()),
            ("alpha-agent", "identity-skill", "DependsOn".into()),
            ("reviewer", "alpha-agent", "Contains".into()),
            ("reviewer", "zeta-agent", "Contains".into())
        ]
    );
    assert!(findings.cycles.is_empty());
    assert_eq!(
        findings.node_ids,
        vec![
            "Beta-agent",
            "alpha-agent",
            "codex-rule",
            "identity-mcp",
            "identity-skill",
            "reviewer",
            "zeta-agent"
        ]
    );
    assert_eq!(
        graph.blast_radius("identity-skill").dependents,
        vec!["alpha-agent", "reviewer"]
    );
    assert!(findings.assets.windows(2).all(
        |w| (&w[0].id, w[0].collection, &w[0].path) <= (&w[1].id, w[1].collection, &w[1].path)
    ));
    fs::write(root.join("catalog/agents.json"), b"[]").unwrap();
    let retained = dependency_findings(&snapshot);
    assert_eq!(retained.graph.as_ref().unwrap().edges.len(), 5);
    assert_eq!(retained.inputs, findings.inputs);
}
#[test]
fn collision_blocks_untyped_graph_before_identity_overwrite() {
    let (_temp, root) = fixture();
    let mut agents: Value =
        serde_json::from_slice(&fs::read(root.join("catalog/agents.json")).unwrap()).unwrap();
    agents[2]["id"] = json!("identity-skill");
    write_json(&root, "agents.json", agents);
    let findings = dependency_findings(&snapshot(&root));
    assert_eq!(findings.quality, FindingsQuality::Partial);
    assert!(findings.graph.is_none());
    assert_eq!(findings.collisions.len(), 1);
    assert_eq!(findings.collisions[0].id, "identity-skill");
    assert_eq!(findings.collisions[0].collections.len(), 2);
    assert_eq!(
        findings
            .unresolved
            .iter()
            .map(|r| r.to.as_str())
            .collect::<Vec<_>>(),
        vec!["alpha-agent"]
    );
}
#[test]
fn unresolved_reference_and_missing_input_are_not_healthy_zero() {
    let (_temp, root) = fixture();
    fs::remove_file(root.join("catalog/skills.json")).unwrap();
    let findings = dependency_findings(&snapshot(&root));
    assert_eq!(findings.quality, FindingsQuality::Partial);
    assert_eq!(findings.unresolved.len(), 1);
    assert_eq!(findings.unresolved[0].to, "identity-skill");
    assert!(findings.inputs.iter().any(|i| !i.issues.is_empty()));
}
#[test]
fn shared_cycle_algorithm_and_stable_results() {
    let (_temp, root) = fixture();
    let mut roles: Value =
        serde_json::from_slice(&fs::read(root.join("catalog/install-roles.json")).unwrap())
            .unwrap();
    roles["roles"]["reviewer"]["agents"] = json!(["reviewer"]);
    write_json(&root, "install-roles.json", roles);
    let snap = snapshot(&root);
    let findings = dependency_findings(&snap);
    assert_eq!(findings.cycles, vec![vec!["reviewer".to_string()]]);
    assert_eq!(findings.unresolved.len(), 1); // typed role target is not an agent
    assert_eq!(findings.cycles, dependency_findings(&snap).cycles);
}
#[test]
fn matched_mismatched_missing_and_captured_manifest_hash() {
    let (_temp, root) = fixture();
    manifest(&root, &["z-missing", "b-mismatch", "a-match"]);
    fs::write(root.join("a-match"), b"hello world").unwrap();
    fs::write(root.join("b-mismatch"), b"changed").unwrap();
    let snap = snapshot(&root);
    let findings = integrity_findings(&snap, 1024);
    assert_eq!(findings.quality, FindingsQuality::Current);
    assert_eq!(
        findings
            .files
            .iter()
            .map(|f| (&*f.path, &f.status))
            .collect::<Vec<_>>(),
        vec![
            ("a-match", &FileStatus::Pass),
            ("b-mismatch", &FileStatus::Fail),
            ("z-missing", &FileStatus::Missing)
        ]
    );
    assert_eq!(findings.files[0].actual_hash.as_deref(), Some(HELLO));
    assert_eq!(findings.files[0].expected_hash, HELLO);
    assert_eq!(findings.context, *snap.context());
    let manifest_input = &findings.inputs[0];
    assert_eq!(
        manifest_input.sha256.as_deref(),
        snap.catalog().content_hash_for(&manifest_input.path)
    );
    fs::write(root.join("catalog/asset-integrity.json"), b"not json").unwrap();
    assert_eq!(integrity_findings(&snap, 1024).inputs, findings.inputs);
}
#[test]
fn missing_malformed_unsupported_and_incomplete_manifests_are_unavailable() {
    for mode in 0..6 {
        let (_temp, root) = fixture();
        manifest(&root, &["file"]);
        let path = root.join("catalog/asset-integrity.json");
        match mode {
            0 => fs::remove_file(&path).unwrap(),
            1 => fs::write(&path, b"not json").unwrap(),
            _ => {
                let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                match mode {
                    2 => value["algorithm"] = json!("sha512"),
                    3 => value["manifest_version"] = json!(2),
                    4 => value["root_files"] = json!([]),
                    _ => value["root_files"][0]["sha256"] = json!("invalid"),
                };
                write_json(&root, "asset-integrity.json", value);
            }
        }
        let findings = integrity_findings(&snapshot(&root), 1024);
        assert_eq!(
            findings.quality,
            FindingsQuality::Unavailable,
            "mode {mode}"
        );
        assert!(!findings.issues.is_empty());
        assert!(findings.files.is_empty());
    }
}
#[test]
fn traversal_absolute_drive_backslash_and_ambiguous_paths_are_rejected() {
    let (temp, root) = fixture();
    fs::write(temp.path().join("outside"), b"hello world").unwrap();
    let absolute = temp.path().join("outside").to_str().unwrap().to_string();
    let paths = [
        "../outside",
        absolute.as_str(),
        "C:/outside",
        "a\\b",
        "a//b",
        "./outside",
        "a/../outside",
    ];
    manifest(&root, &paths);
    let findings = integrity_findings(&snapshot(&root), 1024);
    assert_eq!(findings.quality, FindingsQuality::Partial);
    assert_eq!(findings.files.len(), paths.len());
    for file in findings.files {
        assert_eq!(file.status, FileStatus::Unavailable);
        assert!(file.actual_hash.is_none());
        assert!(file.issue.is_some());
    }
}
#[test]
fn symlinks_parent_symlinks_directories_and_fifo_fail_closed_without_blocking() {
    use std::os::unix::fs::symlink;
    let (temp, root) = fixture();
    let outside = temp.path().join("outside");
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("file"), b"hello world").unwrap();
    symlink(outside.join("file"), root.join("leaf")).unwrap();
    symlink(&outside, root.join("parent")).unwrap();
    fs::create_dir(root.join("directory")).unwrap();
    let fifo = std::ffi::CString::new(root.join("fifo").as_os_str().as_encoded_bytes()).unwrap();
    // SAFETY: CString is NUL terminated and valid through this call; mode has no memory effects.
    assert_eq!(unsafe { libc::mkfifo(fifo.as_ptr(), 0o600) }, 0);
    manifest(&root, &["leaf", "parent/file", "directory", "fifo"]);
    let findings = integrity_findings(&snapshot(&root), 1024);
    for file in findings.files {
        assert_eq!(file.status, FileStatus::Unavailable, "{}", file.path);
        assert!(file.actual_hash.is_none());
    }
}
#[test]
fn oversized_files_do_not_return_truncated_success() {
    let (_temp, root) = fixture();
    manifest(&root, &["large"]);
    fs::write(root.join("large"), b"hello world").unwrap();
    let findings = integrity_findings(&snapshot(&root), 10);
    assert_eq!(findings.files[0].status, FileStatus::Unavailable);
    assert!(findings.files[0].actual_hash.is_none());
}

#[test]
fn hard_byte_ceiling_applies_even_when_caller_requests_unlimited() {
    let (_temp, root) = fixture();
    manifest(&root, &["sparse-large"]);
    fs::File::create(root.join("sparse-large"))
        .unwrap()
        .set_len(vfa_tui::engine::findings::MAX_INTEGRITY_FILE_BYTES + 1)
        .unwrap();
    let findings = integrity_findings(&snapshot(&root), u64::MAX);
    assert_eq!(findings.files[0].status, FileStatus::Unavailable);
    assert!(findings.files[0].actual_hash.is_none());
}

#[test]
fn incomplete_dependency_inputs_retain_exact_named_subset() {
    let (_temp, root) = fixture();
    let mut agents: Value =
        serde_json::from_slice(&fs::read(root.join("catalog/agents.json")).unwrap()).unwrap();
    let mut tainted = agents[0].clone();
    tainted["id"] = json!("tainted-agent");
    tainted["summary"] = json!("synthetic\u{0001}control");
    agents.as_array_mut().unwrap().push(tainted);
    write_json(&root, "agents.json", agents);
    let findings = dependency_findings(&snapshot(&root));
    assert_eq!(findings.quality, FindingsQuality::Partial);
    assert_eq!(findings.graph.as_ref().unwrap().edges.len(), 5);
    assert!(findings.inputs.iter().any(|i| i.collection
        == vfa_tui::engine::snapshot::CollectionKind::Agents
        && i.status == vfa_tui::engine::snapshot::CollectionStatus::Partial
        && !i.issues.is_empty()));
    for name in [
        "agents.json",
        "skills.json",
        "install-roles.json",
        "mcp-references.json",
        "rules.json",
    ] {
        fs::remove_file(root.join("catalog").join(name)).unwrap();
    }
    let unavailable = dependency_findings(&snapshot(&root));
    assert_eq!(unavailable.quality, FindingsQuality::Unavailable);
    assert!(unavailable
        .inputs
        .iter()
        .all(|i| !i.issues.is_empty() && i.sha256.is_none()));
}

#[test]
fn non_directory_parent_is_unavailable_not_missing() {
    let (_temp, root) = fixture();
    manifest(&root, &["parent/file"]);
    fs::write(root.join("parent"), b"synthetic").unwrap();
    let findings = integrity_findings(&snapshot(&root), 1024);
    assert_eq!(findings.files[0].status, FileStatus::Unavailable);
    assert!(findings.files[0].actual_hash.is_none());
}
#[test]
fn root_replacement_never_rebinds_snapshot() {
    let (temp, root) = fixture();
    manifest(&root, &["file"]);
    fs::write(root.join("file"), b"hello world").unwrap();
    let snap = snapshot(&root);
    fs::rename(&root, temp.path().join("retained")).unwrap();
    copy_tree(&temp.path().join("retained"), &root);
    let findings = integrity_findings(&snap, 1024);
    assert_eq!(findings.quality, FindingsQuality::Unavailable);
    assert!(findings.files.iter().all(|f| f.actual_hash.is_none()));
    assert!(!findings.issues.is_empty());
}
#[test]
fn replacement_by_symlink_after_snapshot_cannot_read_outside() {
    use std::os::unix::fs::symlink;
    let (temp, root) = fixture();
    manifest(&root, &["file"]);
    fs::write(root.join("file"), b"hello world").unwrap();
    let snap = snapshot(&root);
    fs::write(temp.path().join("synthetic"), b"hello world").unwrap();
    fs::remove_file(root.join("file")).unwrap();
    symlink(temp.path().join("synthetic"), root.join("file")).unwrap();
    assert_eq!(
        integrity_findings(&snap, 1024).files[0].status,
        FileStatus::Unavailable
    );
}
