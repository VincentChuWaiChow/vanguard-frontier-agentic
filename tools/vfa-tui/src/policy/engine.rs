//! Policy evaluation engine — deterministic pass/fail for every workspace.
//!
//! Req 11.1–11.7.

#![deny(warnings)]

use std::collections::HashSet;

use crate::catalog::store::CatalogStore;
use crate::federation::scanner::InstalledAsset;
use crate::models::agent::Lifecycle;
use crate::models::policy::{PolicyEvaluation, PolicyRuleType, PolicyScope, RuleResult};
use crate::models::workspace::ResolvedWorkspace;

use super::parser::PolicyConfig;

// ---------------------------------------------------------------------------
// Local glob_match
// ---------------------------------------------------------------------------

/// Match `text` against a glob `pattern` supporting `*` (any run) and `?`
/// (any single char). Case-sensitive.
pub fn glob_match(pattern: &str, text: &str) -> bool {
    let pat: Vec<char> = pattern.chars().collect();
    let txt: Vec<char> = text.chars().collect();
    glob_inner(&pat, &txt)
}

fn glob_inner(pat: &[char], txt: &[char]) -> bool {
    match (pat.first(), txt.first()) {
        (None, None) => true,
        (Some(&'*'), _) => {
            glob_inner(&pat[1..], txt) || (!txt.is_empty() && glob_inner(pat, &txt[1..]))
        }
        (Some(&'?'), Some(_)) => glob_inner(&pat[1..], &txt[1..]),
        (Some(p), Some(t)) if p == t => glob_inner(&pat[1..], &txt[1..]),
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Round-half-up helper
// ---------------------------------------------------------------------------

/// Round `v` to 1 decimal place using half-up (0.5 rounds away from zero).
pub fn round_half_up_1dp(v: f64) -> f64 {
    (v * 10.0 + 0.5).floor() / 10.0
}

// ---------------------------------------------------------------------------
// Lifecycle ordering
// ---------------------------------------------------------------------------

/// Maps a lifecycle stage to a numeric rank for comparison.
/// Lower rank = earlier in the lifecycle progression.
/// Deprecated (rank 3) is past Stable (rank 2); a min_stage=Stable gate PASSES
/// for Deprecated assets.
pub fn lifecycle_rank(lc: Lifecycle) -> u8 {
    match lc {
        Lifecycle::Experimental => 0,
        Lifecycle::Beta => 1,
        Lifecycle::Stable => 2,
        Lifecycle::Deprecated => 3,
    }
}

/// Returns `true` when `lc` is strictly below `min_stage` in maturity order.
pub fn lifecycle_below(lc: Lifecycle, min_stage: Lifecycle) -> bool {
    lifecycle_rank(lc) < lifecycle_rank(min_stage)
}

// ---------------------------------------------------------------------------
// PolicyEngine
// ---------------------------------------------------------------------------

/// Evaluates declarative policy rules against workspace state (Req 11.1–11.7).
pub struct PolicyEngine;

impl PolicyEngine {
    /// Evaluate all rules in `config` against `workspace` / `installed` / `catalog`.
    ///
    /// Only rules whose scope matches the workspace and that are not suppressed
    /// contribute to the result. Deterministic: same inputs → identical output
    /// (Req 11.3).
    pub fn evaluate(
        config: &PolicyConfig,
        workspace: &ResolvedWorkspace,
        installed: &[InstalledAsset],
        catalog: &CatalogStore,
        now_date: &str,
    ) -> PolicyEvaluation {
        let mut results: Vec<RuleResult> = Vec::new();

        let installed_ids: HashSet<&str> = installed
            .iter()
            .filter(|a| a.confirmed)
            .map(|a| a.asset_id.as_str())
            .collect();

        for rule in &config.rules {
            if !Self::rule_applies(rule, workspace) {
                continue;
            }
            if Self::is_suppressed(rule, workspace, config, now_date) {
                continue;
            }

            let (passed, details) =
                Self::evaluate_rule_type(&rule.rule_type, installed, &installed_ids, catalog);

            results.push(RuleResult {
                rule_id: rule.id.clone(),
                passed,
                details: if passed { None } else { details },
            });
        }

        let total = results.len();
        let passed = results.iter().filter(|r| r.passed).count();
        let compliance_score = Self::compliance_score(passed, total);

        PolicyEvaluation {
            workspace: workspace.name.clone(),
            results,
            compliance_score,
        }
    }

    /// Return `true` when `rule.scope` matches `workspace` (Req 11.6).
    pub fn rule_applies(
        rule: &crate::models::policy::PolicyRule,
        workspace: &ResolvedWorkspace,
    ) -> bool {
        match &rule.scope {
            PolicyScope::All => true,
            PolicyScope::NamePattern(pat) => glob_match(pat, &workspace.name),
            PolicyScope::Team(team) => workspace
                .team
                .as_deref()
                .map(|t| t == team.as_str())
                .unwrap_or(false),
        }
    }

    /// Return `true` when a suppression covers `rule` for `workspace` and has
    /// not expired as of `now_date` (YYYY-MM-DD, Req 15.6).
    ///
    /// Lexicographic comparison is safe for ISO dates in this format.
    pub fn is_suppressed(
        rule: &crate::models::policy::PolicyRule,
        workspace: &ResolvedWorkspace,
        config: &PolicyConfig,
        now_date: &str,
    ) -> bool {
        config.suppressions.iter().any(|s| {
            s.rule_id == rule.id
                && s.workspace == workspace.name
                && (s.expires.is_empty() || s.expires.as_str() >= now_date)
        })
    }

    /// Compute compliance score: `(passed / total) * 100`, rounded half-up to
    /// 1 decimal place (Req 15.3). Returns 100.0 when `total == 0`.
    pub fn compliance_score(passed: usize, total: usize) -> f64 {
        if total == 0 {
            return 100.0;
        }
        let raw = (passed as f64 / total as f64) * 100.0;
        round_half_up_1dp(raw)
    }

    // -----------------------------------------------------------------------
    // Per-rule evaluation (private)
    // -----------------------------------------------------------------------

    fn evaluate_rule_type(
        rule_type: &PolicyRuleType,
        installed: &[InstalledAsset],
        installed_ids: &HashSet<&str>,
        catalog: &CatalogStore,
    ) -> (bool, Option<String>) {
        match rule_type {
            PolicyRuleType::RequireAsset { asset_id } => {
                if installed_ids.contains(asset_id.as_str()) {
                    (true, None)
                } else {
                    (
                        false,
                        Some(format!("asset '{}' is not installed", asset_id)),
                    )
                }
            }

            PolicyRuleType::RequireRole { role_id } => {
                let role_agents = catalog.agents_for_role(role_id);
                if role_agents.is_empty() {
                    return (true, None);
                }
                let missing: Vec<&str> = role_agents
                    .iter()
                    .filter(|a| !installed_ids.contains(a.id.as_str()))
                    .map(|a| a.id.as_str())
                    .collect();
                if missing.is_empty() {
                    (true, None)
                } else {
                    (
                        false,
                        Some(format!(
                            "role '{}' agents not installed: {}",
                            role_id,
                            missing.join(", ")
                        )),
                    )
                }
            }

            PolicyRuleType::MaxStale { threshold } => {
                let stale_count: usize = installed
                    .iter()
                    .filter(|a| is_asset_stale(a, catalog))
                    .count();
                let thr = *threshold as usize;
                if stale_count <= thr {
                    (true, None)
                } else {
                    (
                        false,
                        Some(format!(
                            "{} stale asset(s) exceed threshold of {}",
                            stale_count, thr
                        )),
                    )
                }
            }

            PolicyRuleType::TrustBoundary {
                max_mutation,
                max_egress,
                max_credentials,
            } => {
                let violations: Vec<String> = catalog
                    .mcp_refs
                    .iter()
                    .filter(|mcp| installed_ids.contains(mcp.id.as_str()))
                    .filter_map(|mcp| {
                        // No matrix means the posture was never declared, not
                        // that it is within the boundary. `?` here silently
                        // dropped such MCPs from evaluation entirely.
                        let Some(tm) = mcp.trust_matrix.as_ref() else {
                            return Some(format!(
                                "{}: no trust_matrix declared (posture unknown)",
                                mcp.id
                            ));
                        };
                        let mut problems = Vec::new();
                        if !max_mutation && tm.mutation_capable {
                            problems.push("mutation_capable");
                        }
                        if !max_egress && tm.requires_egress {
                            problems.push("requires_egress");
                        }
                        if !max_credentials && tm.requires_credentials {
                            problems.push("requires_credentials");
                        }
                        if problems.is_empty() {
                            None
                        } else {
                            Some(format!("{}: {}", mcp.id, problems.join(",")))
                        }
                    })
                    .collect();

                if violations.is_empty() {
                    (true, None)
                } else {
                    (
                        false,
                        Some(format!(
                            "trust boundary violated by MCP refs: {}",
                            violations.join("; ")
                        )),
                    )
                }
            }

            PolicyRuleType::LifecycleGate { min_stage } => {
                let violations: Vec<String> = installed
                    .iter()
                    .filter_map(|a| {
                        let agent = catalog.agent_by_id(&a.asset_id)?;
                        let lc = agent.lifecycle?;
                        if lifecycle_below(lc, *min_stage) {
                            Some(format!("{} ({})", a.asset_id, lc))
                        } else {
                            None
                        }
                    })
                    .collect();

                if violations.is_empty() {
                    (true, None)
                } else {
                    (
                        false,
                        Some(format!(
                            "assets below min lifecycle '{}': {}",
                            min_stage,
                            violations.join(", ")
                        )),
                    )
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Stale-asset helper
// ---------------------------------------------------------------------------

fn is_asset_stale(installed: &InstalledAsset, catalog: &CatalogStore) -> bool {
    let agent = match catalog.agent_by_id(&installed.asset_id) {
        Some(a) => a,
        None => return false,
    };
    let canonical_ver = match &agent.version {
        Some(v) => v,
        None => return false,
    };
    let installed_ver = match &installed.installed_version {
        Some(v) => v,
        None => return false,
    };
    installed_ver != canonical_ver
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::federation::scanner::InstalledAsset;
    use crate::models::agent::{AgentType, Lifecycle};
    use crate::models::harness::{Harness, SourceType};
    use crate::models::policy::{PolicyRule, PolicyScope, Severity, Suppression};
    use crate::models::provider::Provider;
    use crate::models::workspace::{ResolvedWorkspace, WorkspaceStatus};
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn make_workspace(name: &str, team: Option<&str>) -> ResolvedWorkspace {
        ResolvedWorkspace {
            canonical_path: PathBuf::from(format!("/workspaces/{}", name)),
            name: name.to_string(),
            team: team.map(|t| t.to_string()),
            tags: vec![],
            status: WorkspaceStatus::Available,
        }
    }

    fn make_rule(id: &str, rule_type: PolicyRuleType, scope: PolicyScope) -> PolicyRule {
        PolicyRule {
            id: id.to_string(),
            rule_type,
            severity: Severity::Warning,
            scope,
            description: "test".to_string(),
        }
    }

    fn make_installed(asset_id: &str) -> InstalledAsset {
        InstalledAsset {
            workspace_path: PathBuf::from(format!("/workspace/.claude/{}.md", asset_id)),
            asset_id: asset_id.to_string(),
            installed_version: Some("1.0.0".to_string()),
            content_hash: "abc".to_string(),
            detection_methods: vec![],
            confirmed: true,
            harness: ".claude".to_string(),
        }
    }

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
            path: format!("agents/{}.md", id),
            companion_skills: vec![],
            execution_tier: None,
            lifecycle: Some(lc),
            harness_variants: HashMap::new(),
            author: None,
            version: Some("1.0.0".to_string()),
            provider_coverage: None,
        }
    }

    fn empty_config() -> PolicyConfig {
        PolicyConfig::default()
    }

    fn config_with_rule(rule: PolicyRule) -> PolicyConfig {
        PolicyConfig {
            rules: vec![rule],
            ..Default::default()
        }
    }

    // glob_match tests
    #[test]
    fn glob_match_exact() {
        assert!(glob_match("production-infra", "production-infra"));
        assert!(!glob_match("production-infra", "staging-infra"));
    }

    #[test]
    fn glob_match_star() {
        assert!(glob_match("production-*", "production-infra"));
        assert!(!glob_match("production-*", "staging-infra"));
        assert!(glob_match("*", "anything"));
        assert!(glob_match("*", ""));
    }

    #[test]
    fn glob_match_question() {
        assert!(glob_match("prod?", "proda"));
        assert!(!glob_match("prod?", "prod"));
    }

    // compliance_score tests
    #[test]
    fn compliance_score_all_pass() {
        assert!((PolicyEngine::compliance_score(3, 3) - 100.0).abs() < 1e-9);
    }

    #[test]
    fn compliance_score_none_pass() {
        assert!((PolicyEngine::compliance_score(0, 3) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn compliance_score_zero_total() {
        assert!((PolicyEngine::compliance_score(0, 0) - 100.0).abs() < 1e-9);
    }

    #[test]
    fn compliance_score_rounds_half_up() {
        let s = PolicyEngine::compliance_score(1, 3);
        assert!((s - 33.3).abs() < 1e-9, "got {}", s);
        let s2 = PolicyEngine::compliance_score(2, 3);
        assert!((s2 - 66.7).abs() < 1e-9, "got {}", s2);
    }

    // rule_applies tests
    #[test]
    fn rule_applies_all() {
        let ws = make_workspace("any-workspace", None);
        let rule = make_rule(
            "r1",
            PolicyRuleType::MaxStale { threshold: 5 },
            PolicyScope::All,
        );
        assert!(PolicyEngine::rule_applies(&rule, &ws));
    }

    #[test]
    fn rule_applies_name_pattern_match() {
        let ws = make_workspace("production-infra", None);
        let rule = make_rule(
            "r1",
            PolicyRuleType::MaxStale { threshold: 5 },
            PolicyScope::NamePattern("production-*".to_string()),
        );
        assert!(PolicyEngine::rule_applies(&rule, &ws));
    }

    #[test]
    fn rule_applies_name_pattern_no_match() {
        let ws = make_workspace("staging-infra", None);
        let rule = make_rule(
            "r1",
            PolicyRuleType::MaxStale { threshold: 5 },
            PolicyScope::NamePattern("production-*".to_string()),
        );
        assert!(!PolicyEngine::rule_applies(&rule, &ws));
    }

    #[test]
    fn rule_applies_team_match() {
        let ws = make_workspace("ws", Some("platform"));
        let rule = make_rule(
            "r1",
            PolicyRuleType::MaxStale { threshold: 5 },
            PolicyScope::Team("platform".to_string()),
        );
        assert!(PolicyEngine::rule_applies(&rule, &ws));
    }

    #[test]
    fn rule_applies_team_no_match() {
        let ws = make_workspace("ws", Some("devex"));
        let rule = make_rule(
            "r1",
            PolicyRuleType::MaxStale { threshold: 5 },
            PolicyScope::Team("platform".to_string()),
        );
        assert!(!PolicyEngine::rule_applies(&rule, &ws));
    }

    // is_suppressed tests
    #[test]
    fn is_suppressed_active() {
        let ws = make_workspace("staging-infra", None);
        let rule = make_rule(
            "no-mutation",
            PolicyRuleType::MaxStale { threshold: 0 },
            PolicyScope::All,
        );
        let mut cfg = empty_config();
        cfg.suppressions.push(Suppression {
            rule_id: "no-mutation".to_string(),
            workspace: "staging-infra".to_string(),
            reason: "approved".to_string(),
            approver: "lead@example.com".to_string(),
            expires: "2099-12-31".to_string(),
        });
        assert!(PolicyEngine::is_suppressed(&rule, &ws, &cfg, "2026-06-13"));
    }

    #[test]
    fn is_suppressed_expired() {
        let ws = make_workspace("staging-infra", None);
        let rule = make_rule(
            "no-mutation",
            PolicyRuleType::MaxStale { threshold: 0 },
            PolicyScope::All,
        );
        let mut cfg = empty_config();
        cfg.suppressions.push(Suppression {
            rule_id: "no-mutation".to_string(),
            workspace: "staging-infra".to_string(),
            reason: "approved".to_string(),
            approver: "lead@example.com".to_string(),
            expires: "2020-01-01".to_string(),
        });
        assert!(!PolicyEngine::is_suppressed(&rule, &ws, &cfg, "2026-06-13"));
    }

    // evaluate tests
    #[test]
    fn evaluate_require_asset_pass() {
        let ws = make_workspace("prod", None);
        let installed = vec![make_installed("aws-iam-scanner")];
        let rule = make_rule(
            "r1",
            PolicyRuleType::RequireAsset {
                asset_id: "aws-iam-scanner".to_string(),
            },
            PolicyScope::All,
        );
        let cfg = config_with_rule(rule);
        let store = CatalogStore::from_parts(vec![], vec![], HashMap::new(), vec![], vec![]);
        let eval = PolicyEngine::evaluate(&cfg, &ws, &installed, &store, "2026-06-13");
        assert_eq!(eval.results.len(), 1);
        assert!(eval.results[0].passed);
        assert!((eval.compliance_score - 100.0).abs() < 1e-9);
    }

    #[test]
    fn evaluate_require_asset_fail() {
        let ws = make_workspace("prod", None);
        let installed: Vec<InstalledAsset> = vec![];
        let rule = make_rule(
            "r1",
            PolicyRuleType::RequireAsset {
                asset_id: "aws-iam-scanner".to_string(),
            },
            PolicyScope::All,
        );
        let cfg = config_with_rule(rule);
        let store = CatalogStore::from_parts(vec![], vec![], HashMap::new(), vec![], vec![]);
        let eval = PolicyEngine::evaluate(&cfg, &ws, &installed, &store, "2026-06-13");
        assert!(!eval.results[0].passed);
        assert!((eval.compliance_score - 0.0).abs() < 1e-9);
    }

    #[test]
    fn evaluate_lifecycle_gate_fail_experimental() {
        let ws = make_workspace("prod", None);
        let installed = vec![make_installed("my-agent")];
        let agent = make_agent("my-agent", Lifecycle::Experimental);
        let rule = make_rule(
            "lc",
            PolicyRuleType::LifecycleGate {
                min_stage: Lifecycle::Stable,
            },
            PolicyScope::All,
        );
        let cfg = config_with_rule(rule);
        let store = CatalogStore::from_parts(vec![agent], vec![], HashMap::new(), vec![], vec![]);
        let eval = PolicyEngine::evaluate(&cfg, &ws, &installed, &store, "2026-06-13");
        assert!(!eval.results[0].passed);
    }

    #[test]
    fn evaluate_lifecycle_gate_pass_deprecated() {
        // Deprecated > Stable in rank, so it should pass min_stage=Stable.
        let ws = make_workspace("prod", None);
        let installed = vec![make_installed("my-agent")];
        let agent = make_agent("my-agent", Lifecycle::Deprecated);
        let rule = make_rule(
            "lc",
            PolicyRuleType::LifecycleGate {
                min_stage: Lifecycle::Stable,
            },
            PolicyScope::All,
        );
        let cfg = config_with_rule(rule);
        let store = CatalogStore::from_parts(vec![agent], vec![], HashMap::new(), vec![], vec![]);
        let eval = PolicyEngine::evaluate(&cfg, &ws, &installed, &store, "2026-06-13");
        assert!(eval.results[0].passed);
    }

    #[test]
    fn evaluate_scope_filters_non_applicable() {
        let ws = make_workspace("staging-infra", None);
        let installed: Vec<InstalledAsset> = vec![];
        let rule = make_rule(
            "r1",
            PolicyRuleType::RequireAsset {
                asset_id: "scanner".to_string(),
            },
            PolicyScope::NamePattern("production-*".to_string()),
        );
        let cfg = config_with_rule(rule);
        let store = CatalogStore::from_parts(vec![], vec![], HashMap::new(), vec![], vec![]);
        let eval = PolicyEngine::evaluate(&cfg, &ws, &installed, &store, "2026-06-13");
        assert!(eval.results.is_empty());
        assert!((eval.compliance_score - 100.0).abs() < 1e-9);
    }

    // -----------------------------------------------------------------------
    // Property 22 (Req 11.3): determinism
    // -----------------------------------------------------------------------

    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(256))]

        #[test]
        fn prop22_evaluate_deterministic(
            ws_name in "[a-z][a-z0-9-]{1,15}",
            asset_id in "[a-z][a-z0-9-]{1,10}",
            installed_count in 0usize..5,
        ) {
            let ws = make_workspace(&ws_name, None);
            let installed: Vec<InstalledAsset> = (0..installed_count)
                .map(|i| make_installed(&format!("{}-{}", asset_id, i)))
                .collect();
            let rule = make_rule(
                "r1",
                PolicyRuleType::RequireAsset { asset_id: asset_id.clone() },
                PolicyScope::All,
            );
            let cfg = config_with_rule(rule);
            let store = CatalogStore::from_parts(vec![], vec![], HashMap::new(), vec![], vec![]);

            let eval1 = PolicyEngine::evaluate(&cfg, &ws, &installed, &store, "2026-06-13");
            let eval2 = PolicyEngine::evaluate(&cfg, &ws, &installed, &store, "2026-06-13");

            proptest::prop_assert_eq!(eval1.workspace.clone(), eval2.workspace.clone());
            proptest::prop_assert_eq!(eval1.results.len(), eval2.results.len());
            for (r1, r2) in eval1.results.iter().zip(eval2.results.iter()) {
                proptest::prop_assert_eq!(r1.rule_id.clone(), r2.rule_id.clone());
                proptest::prop_assert_eq!(r1.passed, r2.passed);
            }
            proptest::prop_assert_eq!(
                (eval1.compliance_score * 10.0).round() as i64,
                (eval2.compliance_score * 10.0).round() as i64
            );
        }

        // -----------------------------------------------------------------------
        // Property 23 (Req 11.6): scope matching correctness
        // -----------------------------------------------------------------------

        #[test]
        fn prop23_scope_matching(
            ws_name in "[a-z][a-z0-9-]{1,12}",
            team in proptest::option::of("[a-z][a-z0-9-]{1,8}"),
        ) {
            let ws = ResolvedWorkspace {
                canonical_path: std::path::PathBuf::from("/ws"),
                name: ws_name.clone(),
                team: team.clone(),
                tags: vec![],
                status: WorkspaceStatus::Available,
            };

            // Scope::All always applies
            let rule_all = make_rule("all", PolicyRuleType::MaxStale { threshold: 99 }, PolicyScope::All);
            proptest::prop_assert!(PolicyEngine::rule_applies(&rule_all, &ws));

            // Scope::NamePattern with exact name always applies
            let rule_exact = make_rule(
                "exact",
                PolicyRuleType::MaxStale { threshold: 99 },
                PolicyScope::NamePattern(ws_name.clone()),
            );
            proptest::prop_assert!(PolicyEngine::rule_applies(&rule_exact, &ws));

            // Scope::NamePattern "*" always applies
            let rule_star = make_rule(
                "star",
                PolicyRuleType::MaxStale { threshold: 99 },
                PolicyScope::NamePattern("*".to_string()),
            );
            proptest::prop_assert!(PolicyEngine::rule_applies(&rule_star, &ws));

            // Scope::Team matches only if workspace has that exact team
            let sentinel = "sentinel-team-zzz";
            let rule_team = make_rule(
                "team",
                PolicyRuleType::MaxStale { threshold: 99 },
                PolicyScope::Team(sentinel.to_string()),
            );
            let expected = ws.team.as_deref() == Some(sentinel);
            proptest::prop_assert_eq!(PolicyEngine::rule_applies(&rule_team, &ws), expected);
        }
    }
}
