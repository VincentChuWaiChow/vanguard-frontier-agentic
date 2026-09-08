// Feature: rust-tui, Property 5: Reverse-lookup returns correct associated agents
// **Validates: Requirements 2.2**
//
// For any skill ID and for any list of agents, the reverse-lookup function SHALL
// return exactly the set of agents whose `companion_skills` array contains the
// given skill ID, and no other agents.

use proptest::prelude::*;
use proptest::test_runner::Config;
use std::collections::HashMap;

use vfa_tui::catalog::store::CatalogStore;
use vfa_tui::models::agent::{Agent, AgentType};
use vfa_tui::models::harness::{Harness, SourceType};
use vfa_tui::models::provider::Provider;

/// Generate a minimal valid Agent with the given id and companion_skills.
fn make_agent(id: &str, companion_skills: Vec<String>) -> Agent {
    Agent {
        id: id.to_string(),
        name: format!("Agent {}", id),
        entity_type: AgentType::Agent,
        provider: Provider::Aws,
        harnesses: vec![Harness::ClaudeCode],
        summary: format!("Summary for {}", id),
        source_type: SourceType::Original,
        official_docs: vec![],
        security_notes: String::new(),
        last_verified: "2025-01-01".to_string(),
        path: format!("agents/aws/{}", id),
        companion_skills,
        execution_tier: None,
        lifecycle: None,
        harness_variants: HashMap::new(),
        author: None,
        version: None,
        provider_coverage: None,
    }
}

/// Build a CatalogStore with the given agents (no file loading).
fn store_from_agents(agents: Vec<Agent>) -> CatalogStore {
    CatalogStore {
        agents,
        skills: vec![],
        roles: HashMap::new(),
        role_catalog_version: String::new(),
        role_catalog_description: String::new(),
        mcp_refs: vec![],
        rules: vec![],
        integrity: None,
        model_assignments: None,
        model_registry: None,
        workflows: None,
        load_errors: vec![],
        content_hashes: HashMap::new(),
        catalog_root: std::path::PathBuf::from("/tmp"),
    }
}

/// Strategy to generate a skill ID from a small pool.
fn skill_id_strategy() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "skill-a".to_string(),
        "skill-b".to_string(),
        "skill-c".to_string(),
        "skill-d".to_string(),
        "skill-e".to_string(),
    ])
}

/// Strategy to generate a list of companion_skills (0-4 skills from the pool).
fn companion_skills_strategy() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(skill_id_strategy(), 0..=4).prop_map(|mut skills| {
        skills.sort();
        skills.dedup();
        skills
    })
}

/// Strategy to generate a list of agents (2-10) with random companion_skills.
fn agents_strategy() -> impl Strategy<Value = Vec<Agent>> {
    prop::collection::vec(companion_skills_strategy(), 2..=10).prop_map(|skills_lists| {
        skills_lists
            .into_iter()
            .enumerate()
            .map(|(i, skills)| make_agent(&format!("agent-{}", i), skills))
            .collect()
    })
}

proptest! {
    #![proptest_config(Config::with_cases(256))]

    /// Property 5: For any skill ID and any list of agents, agents_with_skill
    /// returns exactly the agents whose companion_skills contains that skill ID.
    ///
    /// This tests both directions:
    /// 1. Every returned agent has the skill in its companion_skills (no false positives)
    /// 2. Every agent with the skill in companion_skills is returned (no false negatives)
    #[test]
    fn reverse_lookup_returns_exact_set(
        agents in agents_strategy(),
        query_skill in skill_id_strategy(),
    ) {
        let store = store_from_agents(agents.clone());
        let result = store.agents_with_skill(&query_skill);

        // Compute expected set: agents whose companion_skills contains query_skill
        let expected_ids: Vec<&str> = agents
            .iter()
            .filter(|a| a.companion_skills.iter().any(|s| s == &query_skill))
            .map(|a| a.id.as_str())
            .collect();

        let result_ids: Vec<&str> = result.iter().map(|a| a.id.as_str()).collect();

        // Direction 1: No false positives — every returned agent has the skill
        for agent in &result {
            prop_assert!(
                agent.companion_skills.iter().any(|s| s == &query_skill),
                "False positive: agent '{}' returned but does not have skill '{}' in companion_skills {:?}",
                agent.id,
                query_skill,
                agent.companion_skills
            );
        }

        // Direction 2: No false negatives — every agent with the skill is returned
        for agent in &agents {
            if agent.companion_skills.iter().any(|s| s == &query_skill) {
                prop_assert!(
                    result_ids.contains(&agent.id.as_str()),
                    "False negative: agent '{}' has skill '{}' but was not returned. Got: {:?}",
                    agent.id,
                    query_skill,
                    result_ids
                );
            }
        }

        // Cardinality check: result set size equals expected set size
        prop_assert_eq!(
            result_ids.len(),
            expected_ids.len(),
            "Result count {} != expected count {} for skill '{}'",
            result_ids.len(),
            expected_ids.len(),
            query_skill
        );
    }

    /// Property 5 (supplementary): When skill_id is not in any agent's
    /// companion_skills, agents_with_skill returns an empty set.
    #[test]
    fn reverse_lookup_empty_for_absent_skill(
        agents in agents_strategy(),
    ) {
        let store = store_from_agents(agents);
        // Use a skill ID that is never in the pool
        let result = store.agents_with_skill("nonexistent-skill-xyz-999");
        prop_assert!(
            result.is_empty(),
            "Expected empty result for nonexistent skill, got {} agents",
            result.len()
        );
    }
}
