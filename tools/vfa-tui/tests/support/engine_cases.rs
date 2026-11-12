//! Shared fixed domain compatibility cases for both frontend consumer graphs.
//! Expected JSON is hand-reviewed data: tests must never rewrite or infer it.

use std::error::Error;
use std::path::Path;

use serde_json::json;

use serde_json::Value;
use vfa_tui::catalog::loader;
use vfa_tui::catalog::store::{CatalogStore, EdgeKind, ReloadOutcome};
use vfa_tui::error::TuiError;
use vfa_tui::models::{Agent, Harness, Provider, Skill};
use vfa_tui::search::fuzzy::SearchEngine;

pub type TestResult = Result<(), Box<dyn Error>>;

const CATALOG_FILES: [&str; 6] = [
    "agents.json",
    "skills.json",
    "mcp-references.json",
    "rules.json",
    "install-roles.json",
    "asset-integrity.json",
];

fn read_expected(path: &Path) -> Result<Value, Box<dyn Error>> {
    Ok(serde_json::from_slice(&std::fs::read(path)?)?)
}

fn load_fixture(fixture_root: &Path) -> CatalogStore {
    let store = CatalogStore::load(fixture_root);
    assert!(store.load_errors.is_empty(), "{:?}", store.load_errors);
    store
}

fn agent_ids(agents: &[Agent]) -> Vec<&str> {
    agents.iter().map(|agent| agent.id.as_str()).collect()
}

fn search_ids<'a>(agents: &'a [Agent], indices: &[usize]) -> Vec<&'a str> {
    indices
        .iter()
        .map(|&index| agents[index].id.as_str())
        .collect()
}

// Compare the complete stable value, not a subset of keys or a production-derived
// digest. Extra/missing keys and nested changes all invalidate the same oracle.
fn matches_contract(actual: &Value, expected: &Value) -> bool {
    actual == expected
}

fn observe(store: &CatalogStore) -> Result<Value, Box<dyn Error>> {
    let mut roles: Vec<_> = store.roles.keys().collect();
    roles.sort();
    let alpha = store.agent_by_id("alpha-agent").ok_or("missing alpha")?;
    let beta = store.agent_by_id("Beta-agent").ok_or("missing Beta")?;
    let beta_value = serde_json::to_value(beta)?;
    let mut defaults = serde_json::Map::new();
    for field in [
        "companion_skills",
        "execution_tier",
        "lifecycle",
        "harness_variants",
        "author",
        "version",
        "provider_coverage",
    ] {
        defaults.insert(
            field.to_owned(),
            beta_value
                .get(field)
                .ok_or("missing serialized field")?
                .clone(),
        );
    }
    let mut edges: Vec<_> = store
        .dependency_edges()
        .into_iter()
        .map(|(from, to, kind)| {
            let kind = match kind {
                EdgeKind::AgentSkill => "AgentSkill",
                EdgeKind::RoleAgent => "RoleAgent",
                EdgeKind::AgentMcp => "AgentMcp",
                EdgeKind::AgentRule => "AgentRule",
            };
            (from, to, kind)
        })
        .collect();
    // Edge iteration traverses a HashMap; its order is not a public guarantee.
    edges.sort();
    let groups: Vec<_> = store
        .agents_for_role_by_provider("reviewer")
        .into_iter()
        .map(|(provider, agents)| {
            (
                provider,
                agents.iter().map(|a| a.id.clone()).collect::<Vec<_>>(),
            )
        })
        .collect();
    let mut engine = SearchEngine::new();
    let mut searches = serde_json::Map::new();
    for (name, query, provider, harness) in [
        ("combined", "IAM", Some("AWS"), Some("CODEX")),
        ("provider", "", Some("aws"), None),
        ("harness", "", None, Some("cursor")),
        ("unicode", "café", None, None),
        ("none", "zzzzzzzzzz", None, None),
        ("cleared", "", None, None),
    ] {
        let indices = engine.search_agents(query, &store.agents, provider, harness);
        searches.insert(name.to_owned(), json!(search_ids(&store.agents, &indices)));
    }
    let skill_indices = engine.search_skills("café", &store.skills);
    let skill_ids = skill_indices
        .iter()
        .map(|&i| store.skills[i].id.as_str())
        .collect::<Vec<_>>();
    searches.insert("skill".to_owned(), json!(skill_ids));
    Ok(json!({
        "collections": {
            "agents": agent_ids(&store.agents),
            "skills": store.skills.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
            "mcp_refs": store.mcp_refs.iter().map(|m| m.id.as_str()).collect::<Vec<_>>(),
            "rules": store.rules.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(),
            "roles": roles, "providers": store.provider_count(),
            "all_assets": store.all_asset_ids()
        },
        "alpha_detail": alpha,
        "default_agent_fields": defaults,
        "role_members": store.agents_for_role("reviewer").iter().map(|a| a.id.as_str()).collect::<Vec<_>>(),
        "role_groups": groups, "edges": edges, "search": searches
    }))
}

pub fn public_domain_imports_and_fixed_catalog_contract(fixture_root: &Path) -> TestResult {
    let store = load_fixture(fixture_root);
    let expected = read_expected(&fixture_root.join("expected.json"))?;
    let actual = observe(&store)?;
    assert!(
        matches_contract(&actual, &expected),
        "actual: {actual:#}\nexpected: {expected:#}"
    );
    assert_eq!(store.agent_count(), 3);
    assert_eq!(store.skill_count(), 1);
    assert_eq!(store.role_catalog_version, "fixture-1");
    assert_eq!(store.role_catalog_description, "Compatibility role catalog");
    assert_eq!(store.content_hashes.len(), 6);
    // Explicitly retain both re-exported and concrete public model API paths.
    let alpha: &Agent = store.agent_by_id("alpha-agent").ok_or("missing alpha")?;
    let skill: &Skill = store.skill_by_id("identity-skill").ok_or("missing skill")?;
    assert_eq!(alpha.provider, Provider::Aws);
    assert_eq!(alpha.harnesses, [Harness::Codex]);
    assert_eq!(skill.production_allowed, Some(false));
    assert_eq!(
        skill.companion_agents.as_deref(),
        Some(["alpha-agent".to_owned()].as_slice())
    );
    assert!(store.agent_by_id("ALPHA-AGENT").is_none());
    assert!(store.skill_by_id("missing").is_none());
    assert!(store.agents_for_role("missing").is_empty());
    let zeta = serde_json::to_value(store.agent_by_id("zeta-agent").ok_or("missing zeta")?)?;
    assert_eq!(zeta["companion_skills"], json!([]));
    let integrity = store.integrity.as_ref().ok_or("missing integrity")?;
    assert_eq!(integrity.manifest_version, 1);
    assert_eq!(integrity.algorithm, "sha256");
    Ok(())
}

pub fn same_oracle_rejects_payload_schema_order_and_missing_fixture_mutants(
    fixture_root: &Path,
) -> TestResult {
    let actual = observe(&load_fixture(fixture_root))?;
    let expected = read_expected(&fixture_root.join("expected.json"))?;
    assert!(matches_contract(&actual, &expected));
    let mut payload = expected.clone();
    payload["alpha_detail"]["provider_coverage"]["regions"][0] = json!("wrong-region");
    assert!(!matches_contract(&actual, &payload));
    let mut schema = expected.clone();
    schema
        .as_object_mut()
        .ok_or("expected object")?
        .remove("search");
    assert!(!matches_contract(&actual, &schema));
    let mut order = expected;
    order["collections"]["agents"] = json!(["Beta-agent", "alpha-agent", "zeta-agent"]);
    assert!(!matches_contract(&actual, &order));
    let temp = tempfile::tempdir()?;
    assert!(read_expected(&temp.path().join("absent.json")).is_err());
    Ok(())
}

pub fn catalog_schema_rejects_unknown_fields_and_wrong_discriminator(
    fixture_root: &Path,
) -> TestResult {
    let expected = read_expected(&fixture_root.join("expected.json"))?;
    let mut unknown = expected["alpha_detail"].clone();
    unknown["unrecognized_contract_field"] = json!(true);
    assert!(serde_json::from_value::<Agent>(unknown).is_err());
    let mut wrong_type = expected["alpha_detail"].clone();
    wrong_type["type"] = json!("skill");
    assert!(serde_json::from_value::<Agent>(wrong_type).is_err());
    Ok(())
}

fn copy_catalog(fixture_root: &Path, destination: &Path) -> TestResult {
    std::fs::create_dir(destination.join("catalog"))?;
    for filename in CATALOG_FILES {
        std::fs::copy(
            fixture_root.join("catalog").join(filename),
            destination.join("catalog").join(filename),
        )?;
    }
    Ok(())
}

pub fn malformed_collection_preserves_other_collections_and_reload_retains_last_good(
    fixture_root: &Path,
) -> TestResult {
    let temp = tempfile::tempdir()?;
    copy_catalog(fixture_root, temp.path())?;
    let mut store = CatalogStore::load(temp.path());
    assert!(store.load_errors.is_empty());
    let agents_path = temp.path().join("catalog/agents.json");
    assert!(matches!(
        store.reload_file(&agents_path),
        ReloadOutcome::Unchanged
    ));
    std::fs::write(&agents_path, "not JSON")?;
    assert!(matches!(
        store.reload_file(&agents_path),
        ReloadOutcome::RetainedPrevious { .. }
    ));
    assert_eq!(
        agent_ids(&store.agents),
        ["alpha-agent", "Beta-agent", "zeta-agent"]
    );
    let reloaded = CatalogStore::load(temp.path());
    assert!(reloaded.agents.is_empty());
    assert!(matches!(
        reloaded.load_errors.as_slice(),
        [TuiError::CatalogParse { .. }]
    ));
    assert_eq!(reloaded.skills.len(), 1);
    assert_eq!(reloaded.mcp_refs.len(), 1);
    assert_eq!(reloaded.rules.len(), 1);
    assert_eq!(reloaded.roles.len(), 1);
    Ok(())
}

pub fn missing_catalog_file_is_an_error_not_a_healthy_empty_collection(
    _fixture_root: &Path,
) -> TestResult {
    let temp = tempfile::tempdir()?;
    let (agents, errors) = loader::load_agents(temp.path());
    assert!(agents.is_empty());
    assert!(matches!(
        errors.as_slice(),
        [TuiError::CatalogNotFound { .. }]
    ));
    Ok(())
}
