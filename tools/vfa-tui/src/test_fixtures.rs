//! Test fixtures: minimal valid examples for integration testing.
//!
//! Provides builder functions and factories for creating test instances of:
//! - Catalog (agents, skills, rules, MCP references)
//! - Workspaces (registry entries, detection results)
//! - Policies (rules, gates, lifecycle gates)
//! - Gate execution (DAGs, outcomes)
//! - Integrity (asset hashes, verification results)

use std::collections::HashMap;
use std::path::PathBuf;

use crate::catalog::store::CatalogStore;
use crate::models::agent::{Agent, AgentType};
use crate::models::harness::Harness;
use crate::models::harness::SourceType;
use crate::models::provider::Provider;
use crate::models::skill::{Skill, SkillType};

// ─────────────────────────────────────────────────────────────────────────────
// Catalog Fixtures
// ─────────────────────────────────────────────────────────────────────────────

/// Factory: minimal valid agent with sensible defaults.
pub fn agent_fixture(id: &str) -> Agent {
    Agent {
        id: id.to_string(),
        name: format!("Agent {}", id),
        entity_type: AgentType::Agent,
        provider: Provider::Aws,
        harnesses: vec![Harness::ClaudeCode],
        summary: format!("Test agent {}", id),
        companion_skills: vec![],
        source_type: SourceType::Original,
        official_docs: vec![],
        security_notes: String::new(),
        last_verified: "2025-01-01".to_string(),
        path: format!("agents/aws/{}", id),
        execution_tier: None,
        lifecycle: None,
        harness_variants: HashMap::new(),
        author: None,
        version: None,
        provider_coverage: None,
    }
}

/// Factory: minimal valid skill with sensible defaults.
pub fn skill_fixture(id: &str) -> Skill {
    Skill {
        id: id.to_string(),
        name: format!("Skill {}", id),
        entity_type: SkillType::Skill,
        summary: format!("Test skill {}", id),
        provider: Provider::Aws,
        source_type: SourceType::Original,
        official_docs: vec![],
        security_notes: String::new(),
        last_verified: "2025-01-01".to_string(),
        path: format!("skills/{}", id),
        companion_agents: None,
        harnesses: vec![Harness::ClaudeCode],
        author: None,
        version: None,
        category: None,
        certifications: None,
        companion_review_skills: None,
        companion_skills: None,
        execution_tier: None,
        feeds_skills: None,
        lifecycle: None,
        mcp_servers: None,
        oauth_scopes: None,
        production_allowed: None,
        run_as_permissions: None,
        sandbox_only: None,
        source_attribution: None,
        verify_before_merge: None,
    }
}

/// Factory: catalog store with minimal valid structure (empty, but valid).
pub fn catalog_store_fixture() -> CatalogStore {
    CatalogStore {
        agents: vec![],
        skills: vec![],
        roles: HashMap::new(),
        role_catalog_version: "1.0".to_string(),
        role_catalog_description: "Test catalog".to_string(),
        mcp_refs: vec![],
        rules: vec![],
        integrity: None,
        model_assignments: None,
        model_registry: None,
        workflows: None,
        load_errors: vec![],
        content_hashes: HashMap::new(),
        catalog_root: PathBuf::from("/tmp/test-catalog"),
    }
}

/// Factory: catalog with a few agents and skills.
pub fn catalog_store_with_data(num_agents: usize, num_skills: usize) -> CatalogStore {
    let mut catalog = catalog_store_fixture();
    catalog.agents = (0..num_agents)
        .map(|i| agent_fixture(&format!("agent-{}", i)))
        .collect();
    catalog.skills = (0..num_skills)
        .map(|i| skill_fixture(&format!("skill-{}", i)))
        .collect();
    catalog
}

// ─────────────────────────────────────────────────────────────────────────────
// Workspace Fixtures
// ─────────────────────────────────────────────────────────────────────────────

/// Factory: minimal valid workspace registry entry (TOML).
pub fn workspace_registry_fixture() -> String {
    r#"
# Test workspace registry
[[workspace]]
name = "test-workspace"
root = "/tmp/test-workspace"
federation_scope = "all"
"#
    .to_string()
}

/// Factory: workspace registry with multiple entries.
pub fn workspace_registry_multi_fixture() -> String {
    r#"
[[workspace]]
name = "prod-east"
root = "/mnt/workspaces/prod-east"
federation_scope = "all"

[[workspace]]
name = "prod-west"
root = "/mnt/workspaces/prod-west"
federation_scope = "all"

[[workspace]]
name = "staging"
root = "/mnt/workspaces/staging"
federation_scope = "all"
"#
    .to_string()
}

// ─────────────────────────────────────────────────────────────────────────────
// Policy Fixtures
// ─────────────────────────────────────────────────────────────────────────────

/// Factory: minimal valid policy (TOML).
pub fn policy_fixture() -> String {
    r#"
[[rule]]
type = "require_asset"
asset_type = "skill"
asset_id = "test-skill"
scope = "all"
"#
    .to_string()
}

/// Factory: policy with mixed rule types.
pub fn policy_complex_fixture() -> String {
    r#"
[[rule]]
type = "require_asset"
asset_type = "agent"
asset_id = "required-agent"
scope = "all"

[[rule]]
type = "lifecycle_gate"
min_lifecycle = "stable"
scope = "all"

[[rule]]
type = "trust_boundary"
mcp_source = "external"
action = "deny"
scope = "all"
"#
    .to_string()
}

// ─────────────────────────────────────────────────────────────────────────────
// Gate Fixtures
// ─────────────────────────────────────────────────────────────────────────────

/// Factory: minimal valid validation gate (TOML).
pub fn gate_fixture(name: &str) -> String {
    format!(
        r#"
[[gate]]
id = "{}"
name = "{}"
command = "true"
description = "Test gate"
timeout_secs = 30
"#,
        name, name
    )
}

/// Factory: validation gates with dependencies.
pub fn gate_dag_fixture() -> String {
    r#"
[[gate]]
id = "gate-a"
name = "Gate A"
command = "true"
description = "Root gate"
timeout_secs = 30

[[gate]]
id = "gate-b"
name = "Gate B"
command = "true"
description = "Depends on A"
timeout_secs = 30
depends_on = ["gate-a"]

[[gate]]
id = "gate-c"
name = "Gate C"
command = "true"
description = "Depends on B"
timeout_secs = 30
depends_on = ["gate-b"]
"#
    .to_string()
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_fixture_is_valid() {
        let agent = agent_fixture("test-agent");
        assert_eq!(agent.id, "test-agent");
        assert!(!agent.name.is_empty());
        assert!(!agent.summary.is_empty());
    }

    #[test]
    fn skill_fixture_is_valid() {
        let skill = skill_fixture("test-skill");
        assert_eq!(skill.id, "test-skill");
        assert!(!skill.name.is_empty());
        assert!(!skill.summary.is_empty());
    }

    #[test]
    fn catalog_store_fixture_is_valid() {
        let catalog = catalog_store_fixture();
        assert!(catalog.agents.is_empty());
        assert!(catalog.skills.is_empty());
    }

    #[test]
    fn catalog_with_data_has_correct_counts() {
        let catalog = catalog_store_with_data(5, 3);
        assert_eq!(catalog.agents.len(), 5);
        assert_eq!(catalog.skills.len(), 3);
    }

    #[test]
    fn workspace_registry_fixture_is_valid_toml() {
        let toml = workspace_registry_fixture();
        assert!(toml::from_str::<toml::Value>(&toml).is_ok());
    }

    #[test]
    fn workspace_registry_multi_fixture_has_three_entries() {
        let toml = workspace_registry_multi_fixture();
        let parsed = toml::from_str::<toml::Value>(&toml).expect("parse");
        let workspaces = parsed
            .get("workspace")
            .and_then(|w| w.as_array())
            .expect("workspace array");
        assert_eq!(workspaces.len(), 3);
    }

    #[test]
    fn policy_fixture_is_valid_toml() {
        let toml = policy_fixture();
        assert!(toml::from_str::<toml::Value>(&toml).is_ok());
    }

    #[test]
    fn policy_complex_fixture_is_valid_toml() {
        let toml = policy_complex_fixture();
        assert!(toml::from_str::<toml::Value>(&toml).is_ok());
    }

    #[test]
    fn gate_fixture_is_valid_toml() {
        let toml = gate_fixture("test-gate");
        assert!(toml::from_str::<toml::Value>(&toml).is_ok());
    }

    #[test]
    fn gate_dag_fixture_is_valid_toml() {
        let toml = gate_dag_fixture();
        let parsed = toml::from_str::<toml::Value>(&toml).expect("parse");
        let gates = parsed
            .get("gate")
            .and_then(|g| g.as_array())
            .expect("gate array");
        assert_eq!(gates.len(), 3);
    }

    #[test]
    fn agent_with_companion_skills() {
        let mut agent = agent_fixture("test");
        agent.companion_skills = vec!["skill-1".to_string(), "skill-2".to_string()];
        assert_eq!(agent.companion_skills.len(), 2);
    }

    #[test]
    fn skill_with_companion_agents() {
        let mut skill = skill_fixture("test");
        skill.companion_agents = Some(vec!["agent-1".to_string()]);
        assert_eq!(skill.companion_agents.as_ref().unwrap().len(), 1);
    }
}
