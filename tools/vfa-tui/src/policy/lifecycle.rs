//! Lifecycle gate evaluation — flag assets installed below a minimum stage.
//!
//! # Lifecycle ordering
//!
//! The canonical linear order for gate purposes is:
//!
//! ```text
//! Experimental (0) < Beta (1) < Stable (2)
//! ```
//!
//! `Deprecated` (rank 3) is treated as **past stable** — it has already
//! satisfied the stable gate at some point in its history.  Therefore:
//! - `min_stage = "stable"` → Deprecated PASSES (rank 3 ≥ rank 2)
//! - `min_stage = "deprecated"` → Deprecated PASSES (rank 3 == rank 3)
//!
//! This may be counter-intuitive ("deprecated" sounds bad), but the lifecycle
//! gate is a **minimum maturity** gate, not a "still in good standing" gate.
//! Separate policies can flag deprecated assets if desired.
//!
//! Req 13.1–13.3.

#![deny(warnings)]

use std::collections::HashMap;

use crate::catalog::store::CatalogStore;
use crate::federation::scanner::InstalledAsset;
use crate::models::agent::Lifecycle;
use crate::models::policy::{PolicyRule, PolicyRuleType, PolicyScope, PolicyViolation, Severity};

use super::engine::lifecycle_rank;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Evaluate all `installed` assets against `min_stage` and return violations
/// for any asset whose catalog lifecycle is strictly below `min_stage`.
///
/// Assets not found in the catalog (agents only; skills/mcp_refs are not
/// lifecycle-gated here) are skipped.
pub fn evaluate_lifecycle(
    installed: &[InstalledAsset],
    catalog: &CatalogStore,
    min_stage: Lifecycle,
) -> Vec<PolicyViolation> {
    let synthetic_rule = PolicyRule {
        id: "lifecycle_gate".to_string(),
        rule_type: PolicyRuleType::LifecycleGate { min_stage },
        severity: Severity::Critical,
        scope: PolicyScope::All,
        description: format!(
            "All installed assets must be at least at lifecycle stage '{}'",
            min_stage
        ),
    };

    let min_rank = lifecycle_rank(min_stage);
    let mut violations = Vec::new();

    for asset in installed {
        let agent = match catalog.agent_by_id(&asset.asset_id) {
            Some(a) => a,
            None => continue, // not a catalog agent — skip
        };

        let lc = match agent.lifecycle {
            Some(l) => l,
            None => continue, // no lifecycle field — skip
        };

        if lifecycle_rank(lc) < min_rank {
            violations.push(PolicyViolation {
                rule: synthetic_rule.clone(),
                workspace: asset
                    .workspace_path
                    .parent()
                    .and_then(|p| p.parent())
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                asset_id: Some(asset.asset_id.clone()),
                first_detected: crate::clock::now_millis(),
                details: format!(
                    "asset '{}' is at lifecycle stage '{}' but minimum required is '{}'",
                    asset.asset_id, lc, min_stage
                ),
                remediation: format!(
                    "Upgrade '{}' to at least '{}' or remove it from this workspace",
                    asset.asset_id, min_stage
                ),
            });
        }
    }

    violations
}

/// Detect lifecycle transitions between two snapshots.
///
/// Returns `(asset_id, previous_stage, new_stage)` for every asset whose
/// lifecycle changed.  Assets present in only one snapshot (added/removed) are
/// not reported here.
///
/// This is used to drive audit log entries (Req 13.3).
pub fn detect_lifecycle_transitions(
    prev: &HashMap<String, Lifecycle>,
    curr: &HashMap<String, Lifecycle>,
) -> Vec<(String, Lifecycle, Lifecycle)> {
    let mut transitions = Vec::new();

    for (asset_id, prev_lc) in prev {
        if let Some(curr_lc) = curr.get(asset_id) {
            if curr_lc != prev_lc {
                transitions.push((asset_id.clone(), *prev_lc, *curr_lc));
            }
        }
    }

    // Sort by asset_id for determinism (Req 11.3)
    transitions.sort_by(|a, b| a.0.cmp(&b.0));

    transitions
}

/// Build a snapshot of `asset_id → Lifecycle` for all agents in `catalog`.
pub fn build_lifecycle_snapshot(catalog: &CatalogStore) -> HashMap<String, Lifecycle> {
    catalog
        .agents
        .iter()
        .filter_map(|a| a.lifecycle.map(|lc| (a.id.clone(), lc)))
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::store::CatalogStore;
    use crate::federation::scanner::{DetectionMethod, InstalledAsset};
    use crate::models::agent::{AgentType, Lifecycle};
    use crate::models::harness::{Harness, SourceType};
    use crate::models::provider::Provider;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn make_agent(id: &str, lc: Lifecycle) -> crate::models::agent::Agent {
        crate::models::agent::Agent {
            id: id.to_string(),
            name: id.to_string(),
            entity_type: AgentType::Agent,
            provider: Provider::Aws,
            harnesses: vec![Harness::ClaudeCode],
            summary: String::new(),
            source_type: SourceType::Original,
            official_docs: vec![],
            security_notes: String::new(),
            last_verified: "2024-01-01".to_string(),
            path: format!("agents/{id}.md"),
            companion_skills: vec![],
            execution_tier: None,
            lifecycle: Some(lc),
            harness_variants: HashMap::new(),
            author: None,
            version: Some("1.0.0".to_string()),
            provider_coverage: None,
        }
    }

    fn make_installed(asset_id: &str, workspace: &str) -> InstalledAsset {
        InstalledAsset {
            workspace_path: PathBuf::from(format!("/workspaces/{workspace}/.claude/{asset_id}.md")),
            asset_id: asset_id.to_string(),
            installed_version: None,
            content_hash: "abc".to_string(),
            detection_methods: vec![DetectionMethod::Filename, DetectionMethod::MetadataComment],
            confirmed: true,
            harness: ".claude".to_string(),
        }
    }

    fn store_with_agents(agents: Vec<crate::models::agent::Agent>) -> CatalogStore {
        CatalogStore::from_parts(agents, vec![], HashMap::new(), vec![], vec![])
    }

    // -----------------------------------------------------------------------
    // evaluate_lifecycle
    // -----------------------------------------------------------------------

    #[test]
    fn no_violation_when_stable_asset_and_min_stable() {
        let agent = make_agent("my-agent", Lifecycle::Stable);
        let store = store_with_agents(vec![agent]);
        let installed = vec![make_installed("my-agent", "prod")];
        let violations = evaluate_lifecycle(&installed, &store, Lifecycle::Stable);
        assert!(violations.is_empty());
    }

    #[test]
    fn no_violation_when_deprecated_and_min_stable() {
        // Deprecated is past Stable → passes min_stage=Stable
        let agent = make_agent("my-agent", Lifecycle::Deprecated);
        let store = store_with_agents(vec![agent]);
        let installed = vec![make_installed("my-agent", "prod")];
        let violations = evaluate_lifecycle(&installed, &store, Lifecycle::Stable);
        assert!(violations.is_empty());
    }

    #[test]
    fn violation_when_experimental_and_min_stable() {
        let agent = make_agent("exp-agent", Lifecycle::Experimental);
        let store = store_with_agents(vec![agent]);
        let installed = vec![make_installed("exp-agent", "prod")];
        let violations = evaluate_lifecycle(&installed, &store, Lifecycle::Stable);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].asset_id, Some("exp-agent".to_string()));
        assert!(violations[0].details.contains("experimental"));
    }

    #[test]
    fn violation_when_beta_and_min_stable() {
        let agent = make_agent("beta-agent", Lifecycle::Beta);
        let store = store_with_agents(vec![agent]);
        let installed = vec![make_installed("beta-agent", "prod")];
        let violations = evaluate_lifecycle(&installed, &store, Lifecycle::Stable);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn no_violation_when_asset_not_in_catalog() {
        let store = store_with_agents(vec![]);
        let installed = vec![make_installed("unknown-agent", "prod")];
        let violations = evaluate_lifecycle(&installed, &store, Lifecycle::Stable);
        assert!(violations.is_empty());
    }

    #[test]
    fn multiple_violations_returned() {
        let agents = vec![
            make_agent("exp-a", Lifecycle::Experimental),
            make_agent("beta-a", Lifecycle::Beta),
            make_agent("stable-a", Lifecycle::Stable),
        ];
        let store = store_with_agents(agents);
        let installed = vec![
            make_installed("exp-a", "prod"),
            make_installed("beta-a", "prod"),
            make_installed("stable-a", "prod"),
        ];
        let violations = evaluate_lifecycle(&installed, &store, Lifecycle::Stable);
        assert_eq!(violations.len(), 2);
    }

    // -----------------------------------------------------------------------
    // detect_lifecycle_transitions
    // -----------------------------------------------------------------------

    #[test]
    fn detects_upgrade_transition() {
        let mut prev = HashMap::new();
        prev.insert("agent-a".to_string(), Lifecycle::Beta);

        let mut curr = HashMap::new();
        curr.insert("agent-a".to_string(), Lifecycle::Stable);

        let transitions = detect_lifecycle_transitions(&prev, &curr);
        assert_eq!(transitions.len(), 1);
        assert_eq!(transitions[0].0, "agent-a");
        assert_eq!(transitions[0].1, Lifecycle::Beta);
        assert_eq!(transitions[0].2, Lifecycle::Stable);
    }

    #[test]
    fn no_transition_when_unchanged() {
        let mut prev = HashMap::new();
        prev.insert("agent-a".to_string(), Lifecycle::Stable);

        let mut curr = HashMap::new();
        curr.insert("agent-a".to_string(), Lifecycle::Stable);

        let transitions = detect_lifecycle_transitions(&prev, &curr);
        assert!(transitions.is_empty());
    }

    #[test]
    fn no_transition_for_removed_asset() {
        let mut prev = HashMap::new();
        prev.insert("agent-a".to_string(), Lifecycle::Stable);

        let curr: HashMap<String, Lifecycle> = HashMap::new();

        let transitions = detect_lifecycle_transitions(&prev, &curr);
        assert!(transitions.is_empty());
    }

    #[test]
    fn transitions_sorted_by_asset_id() {
        let mut prev = HashMap::new();
        prev.insert("z-agent".to_string(), Lifecycle::Beta);
        prev.insert("a-agent".to_string(), Lifecycle::Experimental);

        let mut curr = HashMap::new();
        curr.insert("z-agent".to_string(), Lifecycle::Stable);
        curr.insert("a-agent".to_string(), Lifecycle::Beta);

        let transitions = detect_lifecycle_transitions(&prev, &curr);
        assert_eq!(transitions.len(), 2);
        assert_eq!(transitions[0].0, "a-agent");
        assert_eq!(transitions[1].0, "z-agent");
    }

    // -----------------------------------------------------------------------
    // lifecycle_rank ordering
    // -----------------------------------------------------------------------

    #[test]
    fn lifecycle_rank_ordering() {
        assert!(lifecycle_rank(Lifecycle::Experimental) < lifecycle_rank(Lifecycle::Beta));
        assert!(lifecycle_rank(Lifecycle::Beta) < lifecycle_rank(Lifecycle::Stable));
        assert!(lifecycle_rank(Lifecycle::Stable) < lifecycle_rank(Lifecycle::Deprecated));
    }

    // -----------------------------------------------------------------------
    // Property 24 (Req 13.2): lifecycle — violation iff below min_stage
    // -----------------------------------------------------------------------

    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(256))]

        #[test]
        fn prop24b_lifecycle_violation_iff_below_min_stage(
            asset_lc_idx in 0usize..4,
            min_stage_idx in 0usize..4,
        ) {
            let stages = [
                Lifecycle::Experimental,
                Lifecycle::Beta,
                Lifecycle::Stable,
                Lifecycle::Deprecated,
            ];
            let asset_lc = stages[asset_lc_idx];
            let min_stage = stages[min_stage_idx];

            let agent = make_agent("test-agent", asset_lc);
            let store = store_with_agents(vec![agent]);
            let installed = vec![make_installed("test-agent", "prod")];

            let violations = evaluate_lifecycle(&installed, &store, min_stage);

            let expected_violation = lifecycle_rank(asset_lc) < lifecycle_rank(min_stage);
            proptest::prop_assert_eq!(
                !violations.is_empty(),
                expected_violation,
                "asset_lc={:?}, min_stage={:?}",
                asset_lc, min_stage
            );
        }
    }
}
