//! Engine-only source/session contracts; no native Desktop or execution proof.

use std::path::{Path, PathBuf};

use vfa_tui::engine::session::{SessionError, SourceIdentity};

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

fn fixture() -> (tempfile::TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("source Ω 東京 spaces");
    copy_tree(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/compatibility/core"),
        &root,
    );
    std::fs::write(
        root.join("package.json"),
        r#"{"name":"@raishin/vanguard-frontier-agentic"}"#,
    )
    .unwrap();
    (temp, root)
}

#[cfg(unix)]
#[test]
fn exact_selected_root_has_canonical_native_identity() {
    let (_temp, root) = fixture();
    let identity = SourceIdentity::open(&root).unwrap();
    assert_eq!(identity.root(), root.canonicalize().unwrap());
    assert_eq!(identity, SourceIdentity::open(&root).unwrap());
    identity.verify_current().unwrap();
    assert!(matches!(
        SourceIdentity::open(Path::new(".")),
        Err(SessionError::AbsolutePathRequired)
    ));
    let nested = root.join("nested");
    std::fs::create_dir(&nested).unwrap();
    assert!(
        SourceIdentity::open(&nested).is_err(),
        "must not walk to a parent checkout"
    );
}

#[cfg(unix)]
#[test]
fn root_markers_and_collection_health_are_separate() {
    let (_temp, root) = fixture();
    std::fs::remove_file(root.join("catalog/agents.json")).unwrap();
    SourceIdentity::open(&root).unwrap();
    std::fs::write(root.join("package.json"), r#"{"name":"wrong"}"#).unwrap();
    assert!(SourceIdentity::open(&root).is_err());
    std::fs::write(root.join("package.json"), b"not json").unwrap();
    assert!(SourceIdentity::open(&root).is_err());
    std::fs::remove_file(root.join("package.json")).unwrap();
    assert!(SourceIdentity::open(&root).is_err());
}

#[cfg(unix)]
#[test]
fn aliases_share_identity_but_replaced_roots_do_not() {
    use std::os::unix::fs::symlink;
    let (temp, root) = fixture();
    let alias = temp.path().join("alias");
    symlink(&root, &alias).unwrap();
    let original = SourceIdentity::open(&root).unwrap();
    assert_eq!(SourceIdentity::open(&alias).unwrap(), original);
    let retained = temp.path().join("retained root");
    std::fs::rename(&root, &retained).unwrap();
    copy_tree(&retained, &root);
    assert_ne!(SourceIdentity::open(&root).unwrap(), original);
    assert!(matches!(
        original.verify_current(),
        Err(SessionError::SourceChanged)
    ));
}

#[cfg(unix)]
#[test]
fn native_non_utf8_root_is_preserved_without_lossy_identity() {
    use std::os::unix::ffi::OsStringExt;
    let (temp, root) = fixture();
    let native = temp
        .path()
        .join(std::ffi::OsString::from_vec(b"source-\xff".to_vec()));
    std::fs::rename(root, &native).unwrap();
    let identity = SourceIdentity::open(&native).unwrap();
    assert_eq!(
        identity.root().as_os_str(),
        native.canonicalize().unwrap().as_os_str()
    );
    assert!(identity.root().to_str().is_none());
}

#[cfg(unix)]
#[test]
fn invalid_directory_and_oversized_or_linked_markers_are_named_failures() {
    use std::os::unix::fs::symlink;
    let (temp, root) = fixture();
    let package = root.join("package.json");
    std::fs::write(&package, vec![b' '; 1024 * 1024 + 1]).unwrap();
    assert!(SourceIdentity::open(&root).is_err());
    std::fs::remove_file(&package).unwrap();
    std::fs::write(
        temp.path().join("external.json"),
        r#"{"name":"@raishin/vanguard-frontier-agentic"}"#,
    )
    .unwrap();
    symlink(temp.path().join("external.json"), &package).unwrap();
    assert!(SourceIdentity::open(&root).is_err());
    std::fs::remove_file(&package).unwrap();
    std::fs::create_dir(&package).unwrap();
    assert!(SourceIdentity::open(&root).is_err());
    assert!(SourceIdentity::open(&root.join("missing")).is_err());
}

#[cfg(not(unix))]
#[test]
fn unsupported_native_identity_backend_is_explicit() {
    let (_temp, root) = fixture();
    assert!(matches!(
        SourceIdentity::open(&root),
        Err(SessionError::IdentityUnavailable)
    ));
}

#[cfg(unix)]
mod snapshots_and_switching {
    use super::*;
    use serde_json::{json, Value};
    use sha2::{Digest, Sha256};
    use std::sync::Arc;
    use vfa_tui::engine::session::{SourceSession, MAX_SESSION_OPERATIONS};
    use vfa_tui::engine::snapshot::{CatalogSnapshot, CollectionKind, CollectionStatus};
    use vfa_tui::persistence::operations::OperationStatus;

    fn edit(root: &Path, name: &str, change: impl FnOnce(&mut Value)) {
        let path = root.join("catalog").join(name);
        let mut value: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        change(&mut value);
        std::fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
    }

    fn load(root: &Path) -> Arc<CatalogSnapshot> {
        SourceSession::new().select(root).unwrap()
    }

    #[test]
    fn snapshot_reuses_fixed_models_and_hashes_the_actual_parsed_inputs() {
        fn send_sync<T: Send + Sync>() {}
        send_sync::<CatalogSnapshot>();
        let (_temp, root) = fixture();
        let snapshot = load(&root);
        assert_eq!(snapshot.context().epoch(), 1);
        assert_eq!(
            snapshot
                .catalog()
                .agents
                .iter()
                .map(|a| a.id.as_str())
                .collect::<Vec<_>>(),
            ["alpha-agent", "Beta-agent", "zeta-agent"]
        );
        assert_eq!(snapshot.catalog().provider_names(), ["aws", "azure"]);
        assert_eq!(snapshot.catalog().harness_names(), ["codex", "cursor"]);
        for (kind, count) in [
            (CollectionKind::Agents, 3),
            (CollectionKind::Skills, 1),
            (CollectionKind::Roles, 1),
            (CollectionKind::McpReferences, 1),
            (CollectionKind::Rules, 1),
            (CollectionKind::Integrity, 1),
        ] {
            let state = snapshot.collection(kind);
            assert_eq!(state.status, CollectionStatus::Current);
            assert_eq!(state.count, count);
            assert!(state.issues.is_empty());
        }
        for (path, hash) in &snapshot.catalog().content_hashes {
            assert_eq!(
                hash,
                &format!("{:x}", Sha256::digest(std::fs::read(path).unwrap()))
            );
        }
        assert_eq!(snapshot.catalog().content_hashes.len(), 6);
    }

    #[test]
    fn actual_checkout_catalogs_are_current_and_keep_every_sap_entity() {
        use std::collections::BTreeSet;
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let snapshot = load(&root);
        for (kind, name) in [
            (CollectionKind::Agents, "agents.json"),
            (CollectionKind::Skills, "skills.json"),
        ] {
            let rows: Value =
                serde_json::from_slice(&std::fs::read(root.join("catalog").join(name)).unwrap())
                    .unwrap();
            let rows = rows.as_array().unwrap();
            let state = snapshot.collection(kind);
            assert_eq!(
                state.status,
                CollectionStatus::Current,
                "{name}: {:?}",
                state.issues
            );
            assert_eq!(state.count, rows.len());
            let expected_ids: BTreeSet<_> = rows
                .iter()
                .map(|row| row["id"].as_str().unwrap().to_owned())
                .collect();
            let expected_sap: BTreeSet<_> = rows
                .iter()
                .filter(|row| row["provider"] == "sap")
                .map(|row| row["id"].as_str().unwrap().to_owned())
                .collect();
            assert!(
                !expected_sap.is_empty(),
                "canonical source must exercise SAP"
            );
            let (ids, sap): (BTreeSet<_>, BTreeSet<_>) = if kind == CollectionKind::Agents {
                (
                    snapshot
                        .catalog()
                        .agents
                        .iter()
                        .map(|row| row.id.clone())
                        .collect(),
                    snapshot
                        .catalog()
                        .agents
                        .iter()
                        .filter(|row| row.provider.to_string() == "sap")
                        .map(|row| row.id.clone())
                        .collect(),
                )
            } else {
                (
                    snapshot
                        .catalog()
                        .skills
                        .iter()
                        .map(|row| row.id.clone())
                        .collect(),
                    snapshot
                        .catalog()
                        .skills
                        .iter()
                        .filter(|row| row.provider.to_string() == "sap")
                        .map(|row| row.id.clone())
                        .collect(),
                )
            };
            assert_eq!(ids, expected_ids);
            assert_eq!(sap, expected_sap);
        }
    }

    #[test]
    fn missing_malformed_and_empty_collections_keep_independent_quality() {
        let (_temp, root) = fixture();
        std::fs::remove_file(root.join("catalog/agents.json")).unwrap();
        std::fs::write(root.join("catalog/skills.json"), b"not json").unwrap();
        std::fs::write(root.join("catalog/rules.json"), b"[]").unwrap();
        let snapshot = load(&root);
        for kind in [CollectionKind::Agents, CollectionKind::Skills] {
            assert_eq!(
                snapshot.collection(kind).status,
                CollectionStatus::Unavailable
            );
            assert_eq!(snapshot.collection(kind).count, 0);
            assert!(!snapshot.collection(kind).issues.is_empty());
        }
        assert_eq!(
            snapshot.collection(CollectionKind::Rules).status,
            CollectionStatus::Empty
        );
        assert_eq!(
            snapshot.collection(CollectionKind::Roles).status,
            CollectionStatus::Current
        );
        assert_eq!(snapshot.catalog().roles.len(), 1);
    }

    #[test]
    fn tainted_entries_are_partial_or_unavailable_never_healthy_empty() {
        let (_temp, root) = fixture();
        edit(&root, "agents.json", |v| {
            v[0]["summary"] = json!("\u{001b}tainted")
        });
        let partial = load(&root);
        assert_eq!(
            partial.collection(CollectionKind::Agents).status,
            CollectionStatus::Partial
        );
        assert_eq!(partial.catalog().agents.len(), 2);
        edit(&root, "agents.json", |v| {
            for row in v.as_array_mut().unwrap() {
                row["summary"] = json!("\u{001b}tainted");
            }
        });
        let unavailable = load(&root);
        assert_eq!(
            unavailable.collection(CollectionKind::Agents).status,
            CollectionStatus::Unavailable
        );
        assert!(unavailable.catalog().agents.is_empty());
    }

    #[test]
    fn duplicate_ids_are_named_collection_failures_and_are_kind_scoped() {
        for (name, kind) in [
            ("agents.json", CollectionKind::Agents),
            ("skills.json", CollectionKind::Skills),
            ("mcp-references.json", CollectionKind::McpReferences),
            ("rules.json", CollectionKind::Rules),
        ] {
            let (_temp, root) = fixture();
            edit(&root, name, |v| {
                let first = v[0].clone();
                v.as_array_mut().unwrap().push(first);
            });
            let snapshot = load(&root);
            let state = snapshot.collection(kind);
            assert_eq!(state.status, CollectionStatus::Unavailable, "{name}");
            assert!(
                state
                    .issues
                    .iter()
                    .any(|issue| issue.contains("duplicate entity ID")),
                "{:?}",
                state.issues
            );
            assert_eq!(
                snapshot.collection(CollectionKind::Roles).status,
                CollectionStatus::Current
            );
        }
        let (_temp, root) = fixture();
        edit(&root, "skills.json", |v| v[0]["id"] = json!("alpha-agent"));
        let snapshot = load(&root);
        assert_eq!(
            snapshot.collection(CollectionKind::Agents).status,
            CollectionStatus::Current
        );
        assert_eq!(
            snapshot.collection(CollectionKind::Skills).status,
            CollectionStatus::Current
        );
    }

    #[test]
    fn duplicate_decoded_json_map_keys_do_not_silently_overwrite_roles() {
        let (_temp, root) = fixture();
        let path = root.join("catalog/install-roles.json");
        let roles: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let role = &roles["roles"]["reviewer"];
        let raw = format!(
            r#"{{"version":"v","description":"d","roles":{{"reviewer":{role},"\u0072eviewer":{role}}}}}"#
        );
        std::fs::write(path, raw).unwrap();
        let snapshot = load(&root);
        assert_eq!(
            snapshot.collection(CollectionKind::Roles).status,
            CollectionStatus::Unavailable
        );
        assert!(snapshot
            .collection(CollectionKind::Roles)
            .issues
            .iter()
            .any(|i| i.contains("duplicate JSON key")));
        assert!(snapshot.catalog().roles.is_empty());
        assert_eq!(snapshot.catalog().agents.len(), 3);
    }

    #[test]
    fn no_op_and_failed_switches_preserve_snapshot_identity_and_epoch() {
        use std::os::unix::fs::symlink;
        let (temp, root) = fixture();
        let mut session = SourceSession::new();
        assert!(session.snapshot().is_none());
        assert!(matches!(
            session.register_operation(),
            Err(SessionError::NoSource)
        ));
        let first = session.select(&root).unwrap();
        let alias = temp.path().join("alias");
        symlink(&root, &alias).unwrap();
        assert!(Arc::ptr_eq(&first, &session.select(&alias).unwrap()));
        assert!(session.select(&root.join("missing")).is_err());
        assert!(Arc::ptr_eq(&first, &session.snapshot().unwrap()));
        assert_eq!(session.epoch(), 1);
        assert!(session.is_current(first.context()));
    }

    #[test]
    fn successful_switch_invalidates_tags_but_retains_old_snapshot_and_operation_context() {
        let (_temp, root) = fixture();
        let (_temp2, root2) = fixture();
        std::fs::write(root2.join("catalog/agents.json"), b"[]").unwrap();
        let mut session = SourceSession::new();
        let first = session.select(&root).unwrap();
        let id = session.register_operation().unwrap();
        session
            .transition_operation(id, OperationStatus::Running)
            .unwrap();
        session
            .transition_operation(id, OperationStatus::Succeeded)
            .unwrap();
        let second = session.select(&root2).unwrap();
        assert_eq!(session.epoch(), 2);
        assert!(!session.is_current(first.context()));
        assert!(session.is_current(second.context()));
        assert_eq!(first.catalog().agents.len(), 3);
        assert!(second.catalog().agents.is_empty());
        assert_eq!(session.operation(id).unwrap().context(), first.context());
    }

    #[test]
    fn every_nonterminal_state_blocks_switch_and_dropped_observers_do_not_release_it() {
        for stage in [
            OperationStatus::Queued,
            OperationStatus::Running,
            OperationStatus::Cancelling,
        ] {
            let (_temp, root) = fixture();
            let (_temp2, root2) = fixture();
            let mut session = SourceSession::new();
            let first = session.select(&root).unwrap();
            let id = session.register_operation().unwrap();
            if stage != OperationStatus::Queued {
                session
                    .transition_operation(id, OperationStatus::Running)
                    .unwrap();
            }
            if stage == OperationStatus::Cancelling {
                session.transition_operation(id, stage).unwrap();
            }
            drop(session.snapshot()); // Lost presentation observer is not cleanup.
            assert!(
                matches!(session.select(&root2), Err(SessionError::Busy)),
                "{stage:?}"
            );
            assert_eq!(session.operation(id).unwrap().status(), stage);
            assert!(Arc::ptr_eq(&first, &session.snapshot().unwrap()));
            assert_eq!(session.epoch(), 1);
        }
    }

    #[test]
    fn terminal_states_allow_switch_only_after_explicit_valid_transition() {
        for terminal in [
            OperationStatus::Succeeded,
            OperationStatus::Failed,
            OperationStatus::Cancelled,
            OperationStatus::TimedOut,
            OperationStatus::Interrupted,
        ] {
            let (_temp, root) = fixture();
            let (_temp2, root2) = fixture();
            let mut session = SourceSession::new();
            session.select(&root).unwrap();
            let id = session.register_operation().unwrap();
            session
                .transition_operation(id, OperationStatus::Running)
                .unwrap();
            session
                .transition_operation(id, OperationStatus::Cancelling)
                .unwrap();
            session.transition_operation(id, terminal).unwrap();
            session.transition_operation(id, terminal).unwrap(); // Identical delivery is harmless.
            session.select(&root2).unwrap();
            session.release_operation(id).unwrap();
            assert!(session.operation(id).is_none());
        }
    }

    #[test]
    fn invalid_foreign_and_premature_release_requests_cannot_clear_guards() {
        let (_temp, root) = fixture();
        let mut session = SourceSession::new();
        let mut other = SourceSession::new();
        session.select(&root).unwrap();
        other.select(&root).unwrap();
        assert!(
            !session.is_current(other.snapshot().unwrap().context()),
            "matching source and epoch in a different session is not current"
        );
        let id = session.register_operation().unwrap();
        let foreign = other.register_operation().unwrap();
        assert!(matches!(
            session.transition_operation(foreign, OperationStatus::Failed),
            Err(SessionError::UnknownOperation)
        ));
        assert!(matches!(
            session.release_operation(foreign),
            Err(SessionError::UnknownOperation)
        ));
        assert!(matches!(
            session.transition_operation(id, OperationStatus::Succeeded),
            Err(SessionError::InvalidTransition)
        ));
        assert!(matches!(
            session.release_operation(id),
            Err(SessionError::OperationNotTerminal)
        ));
        assert_eq!(
            session.operation(id).unwrap().status(),
            OperationStatus::Queued
        );
        session
            .transition_operation(id, OperationStatus::Failed)
            .unwrap();
        assert!(matches!(
            session.transition_operation(id, OperationStatus::Running),
            Err(SessionError::InvalidTransition)
        ));
    }

    #[test]
    fn operation_retention_is_bounded_until_explicit_terminal_release() {
        let (_temp, root) = fixture();
        let mut session = SourceSession::new();
        session.select(&root).unwrap();
        let ids = (0..MAX_SESSION_OPERATIONS)
            .map(|_| session.register_operation().unwrap())
            .collect::<Vec<_>>();
        assert!(matches!(
            session.register_operation(),
            Err(SessionError::OperationLimit)
        ));
        for id in &ids {
            session
                .transition_operation(*id, OperationStatus::Failed)
                .unwrap();
        }
        assert!(matches!(
            session.register_operation(),
            Err(SessionError::OperationLimit)
        ));
        session.release_operation(ids[0]).unwrap();
        session.register_operation().unwrap();
    }

    #[test]
    fn root_replacement_blocks_new_operation_admission() {
        let (temp, root) = fixture();
        let mut session = SourceSession::new();
        let first = session.select(&root).unwrap();
        let old = temp.path().join("old");
        std::fs::rename(&root, &old).unwrap();
        copy_tree(&old, &root);
        assert!(matches!(
            session.register_operation(),
            Err(SessionError::SourceChanged)
        ));
        assert!(Arc::ptr_eq(&first, &session.snapshot().unwrap()));
        session.select(&root).unwrap();
        assert_eq!(session.epoch(), 2);
    }

    #[test]
    fn selecting_fifo_source_is_rejected_without_blocking() {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;
        use std::process::{Command, Stdio};
        use std::time::{Duration, Instant};

        // Run this same test in a disposable child: a broken selector may block
        // in open(), and a timeout must not leave a blocked test thread behind.
        if let Some(path) = std::env::var_os("VFA_TASK31_FIFO_PROBE") {
            assert!(matches!(
                SourceIdentity::open(Path::new(&path)),
                Err(SessionError::SourceUnavailable { .. })
            ));
            return;
        }
        let temp = tempfile::tempdir().unwrap();
        let fifo = temp.path().join("not-a-checkout.fifo");
        let native = CString::new(fifo.as_os_str().as_bytes()).unwrap();
        // SAFETY: native is a live NUL-terminated CString with no interior NUL;
        // mkfifo reads it only during this call and receives a valid mode.
        assert_eq!(unsafe { libc::mkfifo(native.as_ptr(), 0o600) }, 0);
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "snapshots_and_switching::selecting_fifo_source_is_rejected_without_blocking",
            ])
            .env("VFA_TASK31_FIFO_PROBE", &fifo)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    assert!(status.success(), "FIFO source was not rejected correctly");
                    break;
                }
                Ok(None) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(10))
                }
                other => {
                    let kill = child.kill();
                    let reap = child.wait();
                    panic!("selector blocked or child observation failed: {other:?}; kill={kill:?}, reap={reap:?}");
                }
            }
        }
    }

    #[test]
    fn browse_does_not_execute_package_helpers_or_write_any_source_content() {
        fn files(root: &Path) -> std::collections::BTreeMap<PathBuf, Vec<u8>> {
            let mut result = std::collections::BTreeMap::new();
            for entry in std::fs::read_dir(root).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    result.extend(files(&path));
                } else {
                    result.insert(path.clone(), std::fs::read(path).unwrap());
                }
            }
            result
        }
        let (temp, root) = fixture();
        std::fs::write(
            root.join("package.json"),
            r#"{"name":"@raishin/vanguard-frontier-agentic","scripts":{"postinstall":"exit 99"}}"#,
        )
        .unwrap();
        let sentinel = temp.path().join("helper-was-executed");
        std::fs::write(
            root.join("helper.sh"),
            format!("#!/bin/sh\ntouch '{}'\n", sentinel.display()),
        )
        .unwrap();
        let before = files(&root);
        load(&root);
        assert_eq!(files(&root), before);
        assert!(!sentinel.exists());
    }
}
