//! Engine-only freshness and result-ordering contracts.
#![cfg(unix)]
use std::path::{Path, PathBuf};
use vfa_tui::engine::catalog::{CatalogController, EntityId, SearchQuery};
use vfa_tui::engine::session::{RefreshTrigger, SourceSession};
use vfa_tui::engine::snapshot::{CollectionKind, CollectionStatus};
use vfa_tui::search::fuzzy::SearchEngine;

fn copy(source: &Path, dest: &Path) {
    std::fs::create_dir_all(dest).unwrap();
    for e in std::fs::read_dir(source).unwrap() {
        let e = e.unwrap();
        if e.file_type().unwrap().is_dir() {
            copy(&e.path(), &dest.join(e.file_name()));
        } else {
            std::fs::copy(e.path(), dest.join(e.file_name())).unwrap();
        }
    }
}
fn fixture() -> (tempfile::TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("Ω 東京");
    copy(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/compatibility/core"),
        &root,
    );
    std::fs::write(
        root.join("package.json"),
        r#"{"name":"@raishin/vanguard-frontier-agentic"}"#,
    )
    .unwrap();
    let path = root.join("catalog/asset-integrity.json");
    let mut integrity: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    integrity["root_files"] =
        serde_json::json!([{"path":"README.md","sha256":"fixture","bytes":12}]);
    std::fs::write(path, serde_json::to_vec(&integrity).unwrap()).unwrap();
    (temp, root)
}
fn query(kind: CollectionKind, text: &str) -> SearchQuery {
    SearchQuery {
        collection: kind,
        text: text.into(),
        provider: None,
        harness: None,
    }
}
#[test]
fn shared_ranking_filters_unicode_and_other_collections() {
    let (_t, root) = fixture();
    let mut session = SourceSession::new();
    session.select(&root).unwrap();
    let mut controller = CatalogController::new();
    let mut shared = SearchEngine::new();
    for text in ["", "a", "café", "東京", "zzzz-no-match"] {
        for provider in [None, Some("aws"), Some("azure")] {
            for harness in [None, Some("codex"), Some("cursor")] {
                let mut q = query(CollectionKind::Agents, text);
                q.provider = provider.map(str::to_owned);
                q.harness = harness.map(str::to_owned);
                let request = controller.request(&session, q).unwrap();
                let result = request.execute();
                let snapshot = session.snapshot().unwrap();
                let expected: Vec<_> = shared
                    .search_agents(text, &snapshot.catalog().agents, provider, harness)
                    .into_iter()
                    .map(|i| snapshot.catalog().agents[i].id.clone())
                    .collect();
                assert_eq!(
                    result
                        .entities()
                        .iter()
                        .map(|e| e.id.clone())
                        .collect::<Vec<_>>(),
                    expected
                );
                assert!(controller.accept(&session, result));
            }
        }
        let request = controller
            .request(&session, query(CollectionKind::Skills, text))
            .unwrap();
        let snapshot = session.snapshot().unwrap();
        let expected: Vec<_> = shared
            .search_skills(text, &snapshot.catalog().skills)
            .into_iter()
            .map(|i| snapshot.catalog().skills[i].id.clone())
            .collect();
        assert_eq!(
            request
                .execute()
                .entities()
                .iter()
                .map(|e| e.id.clone())
                .collect::<Vec<_>>(),
            expected
        );
    }
    for kind in [
        CollectionKind::Roles,
        CollectionKind::McpReferences,
        CollectionKind::Rules,
        CollectionKind::Integrity,
    ] {
        let result = controller
            .request(&session, query(kind, ""))
            .unwrap()
            .execute();
        assert!(!result.entities().is_empty(), "{kind:?}");
        let none = controller
            .request(&session, query(kind, "zzzz-no-match"))
            .unwrap()
            .execute();
        assert!(none.entities().is_empty());
    }
}
#[test]
fn stale_queries_source_and_revision_cannot_publish() {
    let (_t, root) = fixture();
    let (_t2, other) = fixture();
    let mut session = SourceSession::new();
    session.select(&root).unwrap();
    let mut c = CatalogController::new();
    let old = c
        .request(&session, query(CollectionKind::Agents, "Alpha"))
        .unwrap()
        .execute();
    let new = c
        .request(&session, query(CollectionKind::Agents, "Beta"))
        .unwrap()
        .execute();
    assert!(c.accept(&session, new));
    assert!(!c.accept(&session, old));
    assert_eq!(c.visible(&session).unwrap().entities()[0].id, "Beta-agent");
    let old = c
        .request(&session, query(CollectionKind::Agents, ""))
        .unwrap()
        .execute();
    std::fs::write(root.join("catalog/agents.json"), "[]").unwrap();
    session.refresh(RefreshTrigger::Manual).unwrap();
    assert!(!c.accept(&session, old));
    let old = c
        .request(&session, query(CollectionKind::Agents, ""))
        .unwrap()
        .execute();
    session.select(&other).unwrap();
    assert!(!c.accept(&session, old));
    let mut foreign = SourceSession::new();
    foreign.select(&other).unwrap();
    let result = c
        .request(&foreign, query(CollectionKind::Agents, ""))
        .unwrap()
        .execute();
    assert!(!c.accept(&session, result));
}
#[test]
fn refresh_retains_failed_collections_independently_and_hashes_captured_bytes() {
    use sha2::{Digest, Sha256};
    let (_t, root) = fixture();
    let mut session = SourceSession::new();
    let original = session.select(&root).unwrap();
    let same = session.refresh(RefreshTrigger::ExternalChange).unwrap();
    assert_eq!(same.revision(), original.revision());
    std::fs::write(root.join("catalog/agents.json"), b"broken").unwrap();
    std::fs::write(root.join("catalog/skills.json"), b"[]").unwrap();
    let refreshed = session.refresh(RefreshTrigger::Manual).unwrap();
    assert_eq!(refreshed.revision(), original.revision() + 1);
    assert_eq!(refreshed.catalog().agents.len(), 3);
    assert!(refreshed.catalog().skills.is_empty());
    assert_eq!(
        refreshed.collection(CollectionKind::Agents).status,
        CollectionStatus::Stale
    );
    assert!(!refreshed
        .collection(CollectionKind::Agents)
        .issues
        .is_empty());
    assert!(!refreshed.is_current_data());
    assert_eq!(
        refreshed
            .catalog()
            .content_hash_for(&root.join("catalog/agents.json")),
        original
            .catalog()
            .content_hash_for(&root.join("catalog/agents.json"))
    );
    assert_eq!(
        refreshed
            .catalog()
            .content_hash_for(&root.join("catalog/skills.json")),
        Some(format!("{:x}", Sha256::digest(b"[]")).as_str())
    );
    std::fs::remove_file(root.join("catalog/agents.json")).unwrap();
    let missing = session.refresh(RefreshTrigger::ExternalChange).unwrap();
    assert_eq!(missing.revision(), refreshed.revision());
    assert_eq!(
        missing.collection(CollectionKind::Agents).status,
        CollectionStatus::Stale
    );
    std::fs::write(root.join("catalog/agents.json"), "[]").unwrap();
    let recovered = session.refresh(RefreshTrigger::Manual).unwrap();
    assert_eq!(recovered.revision(), missing.revision() + 1);
    assert!(recovered.is_current_data());
    assert_eq!(original.catalog().agents.len(), 3);
}
#[test]
fn stable_selection_clears_on_filter_or_removal_and_cannot_authorize_stale_data() {
    let (_t, root) = fixture();
    let mut s = SourceSession::new();
    s.select(&root).unwrap();
    let mut c = CatalogController::new();
    let result = c
        .request(&s, query(CollectionKind::Agents, ""))
        .unwrap()
        .execute();
    assert!(c.accept(&s, result));
    let id = EntityId {
        collection: CollectionKind::Agents,
        id: "alpha-agent".into(),
    };
    assert!(c.select(&s, id.clone()));
    assert!(c.selection_is_current(&s));
    let result = c
        .request(&s, query(CollectionKind::Agents, "Beta"))
        .unwrap()
        .execute();
    assert!(c.accept(&s, result));
    assert!(c.selection(&s).is_none());
    let result = c
        .request(&s, query(CollectionKind::Agents, ""))
        .unwrap()
        .execute();
    c.accept(&s, result);
    c.select(&s, id);
    std::fs::write(root.join("catalog/agents.json"), "broken").unwrap();
    s.refresh(RefreshTrigger::Manual).unwrap();
    assert!(!c.selection_is_current(&s));
    std::fs::write(root.join("catalog/agents.json"), "[]").unwrap();
    s.refresh(RefreshTrigger::Manual).unwrap();
    c.synchronize(&s);
    assert!(c.selection(&s).is_none());
}

#[test]
fn partial_reloads_retain_prior_records_and_freshness_events_reject_pending_results() {
    let (_t, root) = fixture();
    let mut s = SourceSession::new();
    let first = s.select(&root).unwrap();
    let mut c = CatalogController::new();
    let pending = c
        .request(&s, query(CollectionKind::Agents, ""))
        .unwrap()
        .execute();
    let path = root.join("catalog/agents.json");
    let original = std::fs::read(&path).unwrap();
    let mut rows: serde_json::Value = serde_json::from_slice(&original).unwrap();
    rows[0]["summary"] = serde_json::json!("\u{1b}tainted");
    std::fs::write(&path, serde_json::to_vec(&rows).unwrap()).unwrap();
    let stale = s.refresh(RefreshTrigger::ExternalChange).unwrap();
    assert_eq!(stale.revision(), first.revision());
    assert_eq!(stale.catalog().agents.len(), 3);
    assert_eq!(
        stale.collection(CollectionKind::Agents).status,
        CollectionStatus::Stale
    );
    assert!(!c.accept(&s, pending));
    std::fs::write(path, original).unwrap();
    let recovered = s.refresh(RefreshTrigger::Manual).unwrap();
    assert_eq!(recovered.revision(), first.revision());
    assert!(recovered.is_current_data());
    for (path, hash) in &recovered.catalog().content_hashes {
        use sha2::{Digest, Sha256};
        assert_eq!(
            *hash,
            format!("{:x}", Sha256::digest(std::fs::read(path).unwrap()))
        );
    }
    assert!(recovered.observed_at() >= first.observed_at());
}

#[test]
fn unchanged_refresh_invalidates_old_observation_but_not_content_revision() {
    let (_t, root) = fixture();
    let mut s = SourceSession::new();
    let first = s.select(&root).unwrap();
    let mut c = CatalogController::new();
    let old = c
        .request(&s, query(CollectionKind::Agents, ""))
        .unwrap()
        .execute();
    let second = s.refresh(RefreshTrigger::Manual).unwrap();
    assert_eq!(first.revision(), second.revision());
    assert!(!c.accept(&s, old));
    let result = c
        .request(&s, query(CollectionKind::Agents, ""))
        .unwrap()
        .execute();
    assert!(c.accept(&s, result));
    let id = EntityId {
        collection: CollectionKind::Agents,
        id: "alpha-agent".into(),
    };
    assert!(c.select(&s, id.clone()));
    s.refresh(RefreshTrigger::ExternalChange).unwrap();
    c.synchronize(&s);
    assert_eq!(c.selection(&s), Some(&id));
    assert!(c.selection_is_current(&s));
    assert!(c.visible(&s).is_none());
}

#[test]
fn nonagent_text_sort_and_filter_intersection_are_deterministic() {
    let (_t, root) = fixture();
    let path = root.join("catalog/install-roles.json");
    std::fs::write(
        path,
        serde_json::to_vec(
            &serde_json::json!({"version":"1", "description":"fixture", "roles":{
                "z-role":{"label":"東京", "description":"café", "agents":[]},
                "a-role":{"label":"東京", "description":"café", "agents":[]},
                "A-role":{"label":"東京", "description":"café", "agents":[]}
            }}),
        )
        .unwrap(),
    )
    .unwrap();
    let mut s = SourceSession::new();
    s.select(&root).unwrap();
    let mut c = CatalogController::new();
    for text in ["", "東京", "café"] {
        let result = c
            .request(&s, query(CollectionKind::Roles, text))
            .unwrap()
            .execute();
        assert_eq!(
            result
                .entities()
                .iter()
                .map(|e| e.id.as_str())
                .collect::<Vec<_>>(),
            ["A-role", "a-role", "z-role"]
        );
        assert_eq!(result.query().text, text);
    }
    for kind in [
        CollectionKind::Skills,
        CollectionKind::McpReferences,
        CollectionKind::Rules,
    ] {
        let snapshot = s.snapshot().unwrap();
        let (provider, harness) = match kind {
            CollectionKind::Skills => {
                let r = &snapshot.catalog().skills[0];
                (r.provider.to_string(), r.harnesses[0].to_string())
            }
            CollectionKind::McpReferences => {
                let r = &snapshot.catalog().mcp_refs[0];
                (r.provider.to_string(), r.harnesses[0].to_string())
            }
            _ => {
                let r = &snapshot.catalog().rules[0];
                (r.provider.to_string(), r.harnesses[0].to_string())
            }
        };
        let mut q = query(kind, "");
        q.provider = Some(provider);
        q.harness = Some(harness);
        assert_eq!(
            c.request(&s, q.clone()).unwrap().execute().entities().len(),
            1
        );
        q.harness = Some("nonexistent".into());
        assert!(c.request(&s, q).unwrap().execute().entities().is_empty());
    }
}

#[test]
fn initially_unavailable_collection_does_not_advance_for_invalid_new_bytes() {
    let (_t, root) = fixture();
    let path = root.join("catalog/agents.json");
    std::fs::write(&path, "{}").unwrap();
    let mut s = SourceSession::new();
    let first = s.select(&root).unwrap();
    assert_eq!(
        first.collection(CollectionKind::Agents).status,
        CollectionStatus::Unavailable
    );
    std::fs::write(path, "[{}]").unwrap();
    let still_bad = s.refresh(RefreshTrigger::Manual).unwrap();
    assert_eq!(still_bad.revision(), first.revision());
    assert_eq!(
        still_bad.collection(CollectionKind::Agents).status,
        CollectionStatus::Unavailable
    );
    assert_eq!(
        still_bad.catalog().content_hashes,
        first.catalog().content_hashes
    );
}

#[test]
fn source_failure_marks_retained_data_stale_and_cannot_publish_pending_observations() {
    let (_t, root) = fixture();
    let mut s = SourceSession::new();
    let first = s.select(&root).unwrap();
    let mut c = CatalogController::new();
    let result = c
        .request(&s, query(CollectionKind::Agents, ""))
        .unwrap()
        .execute();
    assert!(c.accept(&s, result));
    let id = EntityId {
        collection: CollectionKind::Agents,
        id: "alpha-agent".into(),
    };
    c.select(&s, id);
    let pending = c
        .request(&s, query(CollectionKind::Agents, ""))
        .unwrap()
        .execute();
    std::fs::remove_file(root.join("package.json")).unwrap();
    assert!(s.refresh(RefreshTrigger::Manual).is_err());
    let failed = s.snapshot().unwrap();
    assert_eq!(failed.revision(), first.revision());
    assert!(!failed.is_current_data());
    assert_eq!(
        failed.catalog().content_hashes,
        first.catalog().content_hashes
    );
    assert_eq!(failed.catalog().agents.len(), 3);
    assert!(c.visible(&s).is_none());
    assert!(c.selection(&s).is_none());
    assert!(!c.selection_is_current(&s));
    assert!(!c.accept(&s, pending));
}

#[test]
fn replaced_root_is_not_rebound_by_refresh_and_old_selection_and_result_are_rejected() {
    let (_t, root) = fixture();
    let mut s = SourceSession::new();
    let first = s.select(&root).unwrap();
    let mut c = CatalogController::new();
    let result = c
        .request(&s, query(CollectionKind::Agents, ""))
        .unwrap()
        .execute();
    let seq = result.sequence();
    assert!(c.accept(&s, result));
    c.select(
        &s,
        EntityId {
            collection: CollectionKind::Agents,
            id: "alpha-agent".into(),
        },
    );
    let pending = c
        .request(&s, query(CollectionKind::Agents, ""))
        .unwrap()
        .execute();
    assert!(pending.sequence() > seq);
    let moved = root.with_file_name("old checkout");
    std::fs::rename(&root, &moved).unwrap();
    copy(&moved, &root);
    assert!(s.refresh(RefreshTrigger::ExternalChange).is_err());
    let failed = s.snapshot().unwrap();
    assert_eq!(failed.context(), first.context());
    assert!(!failed.is_current_data());
    assert!(!c.selection_is_current(&s));
    assert!(!c.accept(&s, pending));
    assert!(c.visible(&s).is_none());
    assert!(c.selection(&s).is_some());
    // A retained stable ID is browse-only; the stale snapshot is not current
    // evidence. Reselecting deliberately establishes a new native source context.
    s.select(&root).unwrap();
    assert_ne!(s.snapshot().unwrap().context(), first.context());
    assert!(c.selection(&s).is_none());
}

#[tokio::test]
async fn owned_watch_routes_linux_rename_delete_recreate_through_shared_refresh() {
    use std::time::Duration;
    use tokio::time::timeout;
    use vfa_tui::engine::session::WatchedSourceSession;
    let (_temp, root) = fixture();
    let mut owner = WatchedSourceSession::new();
    let first = owner.select(&root).await.unwrap();
    let path = root.join("catalog/agents.json");
    let staged = root.join("catalog/replacement.json");
    std::fs::write(&staged, "[]").unwrap();
    std::fs::rename(&staged, &path).unwrap();
    let renamed = timeout(Duration::from_secs(5), owner.next_change())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert!(renamed.catalog().agents.is_empty());
    assert_eq!(renamed.revision(), first.revision() + 1);
    std::fs::remove_file(&path).unwrap();
    let deleted = timeout(Duration::from_secs(5), owner.next_change())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(
        deleted.collection(CollectionKind::Agents).status,
        CollectionStatus::Stale
    );
    assert_eq!(deleted.revision(), renamed.revision());
    std::fs::write(&path, "[]").unwrap();
    let recreated = timeout(Duration::from_secs(5), owner.next_change())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(
        recreated.collection(CollectionKind::Agents).status,
        CollectionStatus::Empty
    );
    assert_eq!(recreated.revision(), deleted.revision());
    assert!(recreated.is_current_data());
    timeout(Duration::from_secs(5), owner.shutdown())
        .await
        .unwrap()
        .unwrap();
    assert!(owner.next_change().await.unwrap().is_none());
}

#[tokio::test]
async fn real_old_source_notification_cannot_refresh_selected_source() {
    use std::time::Duration;
    use tokio::time::timeout;
    use vfa_tui::catalog::watcher::spawn_watcher;
    let (_temp, root) = fixture();
    let (_other_temp, other) = fixture();
    let mut session = SourceSession::new();
    let original = session.select(&root).unwrap();
    let old_context = original.context().clone();
    let (mut watch, mut events) = spawn_watcher(&root, None, &[]).unwrap();
    std::fs::write(root.join("catalog/agents.json"), "[]").unwrap();
    timeout(Duration::from_secs(5), events.recv())
        .await
        .unwrap()
        .unwrap();
    watch.shutdown_async().await.unwrap();
    let selected = session.select(&other).unwrap();
    std::fs::write(other.join("catalog/agents.json"), "[]").unwrap();
    assert!(session.refresh_external(&old_context).unwrap().is_none());
    assert!(
        std::sync::Arc::ptr_eq(&selected, &session.snapshot().unwrap()),
        "old-generation event must not even publish a new observation"
    );
    assert_eq!(session.snapshot().unwrap().catalog().agents.len(), 3);
    assert!(session
        .refresh_external(selected.context())
        .unwrap()
        .unwrap()
        .catalog()
        .agents
        .is_empty());
}

#[tokio::test]
async fn owned_source_switch_drains_old_notifications_and_watches_new_source() {
    use std::time::Duration;
    use tokio::time::timeout;
    use vfa_tui::engine::session::WatchedSourceSession;
    let (_temp, root) = fixture();
    let (_other_temp, other) = fixture();
    let mut owner = WatchedSourceSession::new();
    let old = owner.select(&root).await.unwrap();
    let selected = owner.select(&other).await.unwrap();
    assert_ne!(selected.context(), old.context());
    // Both roots change: only new-source events can reach the owned receiver.
    std::fs::write(root.join("catalog/agents.json"), "broken").unwrap();
    std::fs::write(other.join("catalog/agents.json"), "[]").unwrap();
    let observed = timeout(Duration::from_secs(5), owner.next_change())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(observed.context(), selected.context());
    assert!(observed.catalog().agents.is_empty());
    assert!(observed.is_current_data());
    timeout(Duration::from_secs(5), owner.shutdown())
        .await
        .unwrap()
        .unwrap();
}
