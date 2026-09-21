//! Trust boundary enforcement — flag MCP refs exceeding policy constraints.
//!
//! [`evaluate_trust`] evaluates each installed MCP reference against a
//! [`TrustBoundaryPolicy`] and returns a [`PolicyViolation`] for any reference
//! that exceeds the policy.  Per-workspace overrides (approved exceptions) are
//! supported via `workspace_overrides` (Req 12.5).
//!
//! Req 12.1–12.5.

#![deny(warnings)]

use crate::catalog::store::CatalogStore;
use crate::federation::scanner::InstalledAsset;
use crate::models::mcp_ref::McpReference;
use crate::models::policy::{PolicyRule, PolicyRuleType, PolicyScope, PolicyViolation, Severity};

// ---------------------------------------------------------------------------
// TrustBoundaryPolicy
// ---------------------------------------------------------------------------

/// The constraints from a `trust_boundary` policy rule.
///
/// Each `max_*` field represents the **maximum allowed** value for that
/// attribute.  Setting `max_mutation = false` means "mutation-capable MCP refs
/// are NOT allowed"; `true` means "mutation-capable refs are fine".
#[derive(Debug, Clone, PartialEq)]
pub struct TrustBoundaryPolicy {
    /// If `false`, any MCP ref with `mutation_capable = true` is flagged.
    pub max_mutation: bool,
    /// If `false`, any MCP ref with `requires_egress = true` is flagged.
    pub max_egress: bool,
    /// If `false`, any MCP ref with `requires_credentials = true` is flagged.
    pub max_credentials: bool,
}

// ---------------------------------------------------------------------------
// WorkspaceTrustOverride
// ---------------------------------------------------------------------------

/// A per-workspace approved exception that suppresses a specific MCP violation.
#[derive(Debug, Clone)]
pub struct WorkspaceTrustOverride {
    /// ID of the MCP reference for which the exception applies.
    pub mcp_ref_id: String,
    /// Human-readable reason for the exception.
    pub reason: String,
    /// Approver identifier (e.g. email).
    pub approver: String,
}

/// Record applied per-workspace trust overrides in the audit log (Req 12.5 /
/// Task 7.8).
///
/// Each override becomes an [`crate::models::audit::AuditEventType::ConfigChange`]
/// entry naming the MCP ref, approver, reason, and workspace, preserving the
/// hash chain. Returns the appended entries.
pub fn log_trust_overrides(
    logger: &mut crate::persistence::audit::AuditLogger<'_>,
    workspace_name: &str,
    overrides: &[WorkspaceTrustOverride],
) -> Result<Vec<crate::models::audit::AuditEntry>, crate::error::TuiError> {
    use crate::models::audit::AuditEventType;

    let mut entries = Vec::with_capacity(overrides.len());
    for ov in overrides {
        let entry = logger.log(
            AuditEventType::ConfigChange,
            &format!("trust_override:{}", ov.mcp_ref_id),
            serde_json::json!({
                "workspace": workspace_name,
                "mcp_ref_id": ov.mcp_ref_id,
                "approver": ov.approver,
                "reason": ov.reason,
            }),
            "operator",
        )?;
        entries.push(entry);
    }
    Ok(entries)
}

// ---------------------------------------------------------------------------
// TrustViolation — internal detail
// ---------------------------------------------------------------------------

/// Which trust attributes were exceeded by a single MCP reference.
#[derive(Debug, Clone)]
struct TrustViolation {
    mcp_id: String,
    /// The MCP declared no trust matrix, so its posture is unknown rather than
    /// known-safe. Treated as a violation: a zero-trust boundary cannot be
    /// satisfied by metadata that does not exist.
    unknown_trust: bool,
    exceeded_mutation: bool,
    exceeded_egress: bool,
    exceeded_credentials: bool,
    /// Note if an override suppressed part of the violation.
    suppressed_by_override: bool,
    override_note: Option<String>,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Evaluate `installed` MCP references against `boundary` and return policy
/// violations.
///
/// `workspace_name` is used for violation attribution.
/// `workspace_overrides` may exempt specific MCP refs by ID; when an override
/// is applied an audit note is embedded in the violation's `details`.
pub fn evaluate_trust(
    installed: &[InstalledAsset],
    catalog: &CatalogStore,
    boundary: &TrustBoundaryPolicy,
    workspace_name: &str,
    workspace_overrides: &[WorkspaceTrustOverride],
) -> Vec<PolicyViolation> {
    // Build a synthetic PolicyRule representing the trust_boundary constraint,
    // so the returned PolicyViolation.rule is populated correctly.
    let synthetic_rule = PolicyRule {
        id: "trust_boundary".to_string(),
        rule_type: PolicyRuleType::TrustBoundary {
            max_mutation: boundary.max_mutation,
            max_egress: boundary.max_egress,
            max_credentials: boundary.max_credentials,
        },
        severity: Severity::Critical,
        scope: PolicyScope::All,
        description: "MCP trust boundary enforcement".to_string(),
    };

    let mut violations = Vec::new();

    for asset in installed {
        // Find the MCP reference in the catalog by asset_id.
        let mcp_ref = match catalog.mcp_refs.iter().find(|m| m.id == asset.asset_id) {
            Some(r) => r,
            None => continue,
        };

        let tv = check_mcp_against_boundary(mcp_ref, boundary, workspace_overrides);
        if tv.unknown_trust || tv.exceeded_mutation || tv.exceeded_egress || tv.exceeded_credentials
        {
            let details = build_violation_details(&tv);
            violations.push(PolicyViolation {
                rule: synthetic_rule.clone(),
                workspace: workspace_name.to_string(),
                asset_id: Some(mcp_ref.id.clone()),
                first_detected: chrono::Utc::now().to_rfc3339(),
                details,
                remediation: format!(
                    "Remove or replace '{}' with a reference that satisfies the trust boundary",
                    mcp_ref.id
                ),
            });
        }
    }

    violations
}

/// Evaluate trust for a policy rule of type `TrustBoundary`.
/// Returns `(passed, detail_str)`.
pub fn check_trust_boundary_rule(
    installed: &[InstalledAsset],
    catalog: &CatalogStore,
    max_mutation: bool,
    max_egress: bool,
    max_credentials: bool,
    workspace_overrides: &[WorkspaceTrustOverride],
) -> (bool, Option<String>) {
    let boundary = TrustBoundaryPolicy {
        max_mutation,
        max_egress,
        max_credentials,
    };
    let violations: Vec<String> = installed
        .iter()
        .filter_map(|a| {
            let mcp_ref = catalog.mcp_refs.iter().find(|m| m.id == a.asset_id)?;
            let tv = check_mcp_against_boundary(mcp_ref, &boundary, workspace_overrides);
            if tv.unknown_trust
                || tv.exceeded_mutation
                || tv.exceeded_egress
                || tv.exceeded_credentials
            {
                Some(format!(
                    "'{}': {}",
                    mcp_ref.id,
                    build_violation_details(&tv)
                ))
            } else {
                None
            }
        })
        .collect();

    if violations.is_empty() {
        (true, None)
    } else {
        (false, Some(violations.join("; ")))
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn check_mcp_against_boundary(
    mcp: &McpReference,
    boundary: &TrustBoundaryPolicy,
    overrides: &[WorkspaceTrustOverride],
) -> TrustViolation {
    let tm = match &mcp.trust_matrix {
        Some(tm) => tm,
        None => {
            // No trust matrix → UNKNOWN, not safe. Silently passing an MCP
            // whose mutation/egress/credential posture was never declared let
            // undeclared capability satisfy a zero-trust boundary.
            return TrustViolation {
                mcp_id: mcp.id.clone(),
                unknown_trust: true,
                exceeded_mutation: false,
                exceeded_egress: false,
                exceeded_credentials: false,
                suppressed_by_override: false,
                override_note: None,
            };
        }
    };

    let raw_mutation = !boundary.max_mutation && tm.mutation_capable;
    let raw_egress = !boundary.max_egress && tm.requires_egress;
    let raw_credentials = !boundary.max_credentials && tm.requires_credentials;

    // Check for per-workspace override for this MCP ref.
    let override_entry = overrides.iter().find(|o| o.mcp_ref_id == mcp.id);
    if let Some(ov) = override_entry {
        // Override suppresses all violations for this MCP ref.
        let note = format!(
            "override applied: reason='{}', approver='{}'",
            ov.reason, ov.approver
        );
        return TrustViolation {
            mcp_id: mcp.id.clone(),
            // A matrix exists on this path, so the posture is known.
            unknown_trust: false,
            exceeded_mutation: false,
            exceeded_egress: false,
            exceeded_credentials: false,
            suppressed_by_override: raw_mutation || raw_egress || raw_credentials,
            override_note: if raw_mutation || raw_egress || raw_credentials {
                Some(note)
            } else {
                None
            },
        };
    }

    TrustViolation {
        mcp_id: mcp.id.clone(),
        unknown_trust: false,
        exceeded_mutation: raw_mutation,
        exceeded_egress: raw_egress,
        exceeded_credentials: raw_credentials,
        suppressed_by_override: false,
        override_note: None,
    }
}

fn build_violation_details(tv: &TrustViolation) -> String {
    let mut parts = Vec::new();
    if tv.unknown_trust {
        parts.push("no trust_matrix declared (posture unknown)");
    }
    if tv.exceeded_mutation {
        parts.push("mutation_capable exceeds boundary");
    }
    if tv.exceeded_egress {
        parts.push("requires_egress exceeds boundary");
    }
    if tv.exceeded_credentials {
        parts.push("requires_credentials exceeds boundary");
    }
    if tv.suppressed_by_override {
        if let Some(note) = &tv.override_note {
            return format!("[suppressed] {}", note);
        }
    }
    format!("MCP ref '{}': {}", tv.mcp_id, parts.join(", "))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::store::CatalogStore;
    use crate::federation::scanner::{DetectionMethod, InstalledAsset};
    use crate::models::harness::{Harness, SourceType};
    use crate::models::mcp_ref::{McpReference, McpType, PinStrategy, SignedRelease, TrustMatrix};
    use crate::models::provider::Provider;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn make_mcp(id: &str, mutation: bool, egress: bool, credentials: bool) -> McpReference {
        McpReference {
            id: id.to_string(),
            name: id.to_string(),
            entity_type: McpType::McpReference,
            provider: Provider::Aws,
            harnesses: vec![Harness::ClaudeCode],
            summary: String::new(),
            source_type: SourceType::Original,
            official_docs: vec![],
            security_notes: String::new(),
            last_verified: "2024-01-01".to_string(),
            path: format!("mcp/{id}"),
            official_project_url: String::new(),
            vendor: String::new(),
            auth_model: String::new(),
            install_example: String::new(),
            unofficial_warning: String::new(),
            trust_matrix: Some(TrustMatrix {
                mutation_capable: mutation,
                requires_egress: egress,
                requires_credentials: credentials,
                signed_release: SignedRelease::Unsigned,
                pin_strategy: PinStrategy::None,
            }),
        }
    }

    fn make_installed(asset_id: &str) -> InstalledAsset {
        InstalledAsset {
            workspace_path: PathBuf::from(format!("/ws/.claude/{asset_id}")),
            asset_id: asset_id.to_string(),
            installed_version: None,
            content_hash: "abc".to_string(),
            detection_methods: vec![DetectionMethod::Filename, DetectionMethod::MetadataComment],
            confirmed: true,
            harness: ".claude".to_string(),
        }
    }

    fn store_with_mcp(mcp: McpReference) -> CatalogStore {
        CatalogStore::from_parts(vec![], vec![], HashMap::new(), vec![mcp], vec![])
    }

    // -----------------------------------------------------------------------
    // Basic violation detection
    // -----------------------------------------------------------------------

    #[test]
    fn no_violation_when_safe_mcp() {
        let mcp = make_mcp("safe-server", false, false, false);
        let store = store_with_mcp(mcp);
        let installed = vec![make_installed("safe-server")];
        let boundary = TrustBoundaryPolicy {
            max_mutation: false,
            max_egress: false,
            max_credentials: false,
        };
        let violations = evaluate_trust(&installed, &store, &boundary, "prod", &[]);
        assert!(
            violations.is_empty(),
            "expected no violations, got {:?}",
            violations
        );
    }

    /// Regression: an MCP in the catalog with no trust_matrix used to return
    /// all-false from the boundary check, so undeclared capability satisfied a
    /// zero-trust boundary. Every MCP in catalog/mcp-references.json was in
    /// exactly that state, which made the whole trust gate decorative.
    #[test]
    fn undeclared_trust_matrix_is_a_violation_not_a_pass() {
        let mut mcp = make_mcp("undeclared-server", false, false, false);
        mcp.trust_matrix = None;
        let store = store_with_mcp(mcp);
        let installed = vec![make_installed("undeclared-server")];
        // Strictest possible boundary: nothing is permitted.
        let boundary = TrustBoundaryPolicy {
            max_mutation: false,
            max_egress: false,
            max_credentials: false,
        };
        let violations = evaluate_trust(&installed, &store, &boundary, "prod", &[]);
        assert_eq!(
            violations.len(),
            1,
            "an undeclared trust posture must not pass a zero-trust boundary"
        );
        let details = format!("{:?}", violations[0]);
        assert!(
            details.contains("no trust_matrix declared"),
            "violation should name the undeclared posture, got {details}"
        );
    }

    #[test]
    fn violation_when_mutation_exceeds_boundary() {
        let mcp = make_mcp("mutating-server", true, false, false);
        let store = store_with_mcp(mcp);
        let installed = vec![make_installed("mutating-server")];
        let boundary = TrustBoundaryPolicy {
            max_mutation: false, // mutation NOT allowed
            max_egress: true,
            max_credentials: true,
        };
        let violations = evaluate_trust(&installed, &store, &boundary, "prod", &[]);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].asset_id, Some("mutating-server".to_string()));
        assert!(violations[0].details.contains("mutation_capable"));
    }

    #[test]
    fn violation_when_egress_exceeds_boundary() {
        let mcp = make_mcp("egress-server", false, true, false);
        let store = store_with_mcp(mcp);
        let installed = vec![make_installed("egress-server")];
        let boundary = TrustBoundaryPolicy {
            max_mutation: true,
            max_egress: false, // egress NOT allowed
            max_credentials: true,
        };
        let violations = evaluate_trust(&installed, &store, &boundary, "prod", &[]);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].details.contains("requires_egress"));
    }

    #[test]
    fn no_violation_when_mcp_not_in_catalog() {
        // Installed but not in catalog → skip (no trust matrix known)
        let store = CatalogStore::from_parts(vec![], vec![], HashMap::new(), vec![], vec![]);
        let installed = vec![make_installed("unknown-server")];
        let boundary = TrustBoundaryPolicy {
            max_mutation: false,
            max_egress: false,
            max_credentials: false,
        };
        let violations = evaluate_trust(&installed, &store, &boundary, "prod", &[]);
        assert!(violations.is_empty());
    }

    #[test]
    fn override_suppresses_violation() {
        let mcp = make_mcp("mutating-server", true, false, false);
        let store = store_with_mcp(mcp);
        let installed = vec![make_installed("mutating-server")];
        let boundary = TrustBoundaryPolicy {
            max_mutation: false,
            max_egress: true,
            max_credentials: true,
        };
        let overrides = vec![WorkspaceTrustOverride {
            mcp_ref_id: "mutating-server".to_string(),
            reason: "approved for staging mutation".to_string(),
            approver: "security-lead@example.com".to_string(),
        }];
        let violations = evaluate_trust(&installed, &store, &boundary, "staging", &overrides);
        // Override suppresses — no violation returned
        assert!(
            violations.is_empty(),
            "expected no violations with override, got {:?}",
            violations
        );
    }

    // -----------------------------------------------------------------------
    // Property 24 (Req 12.2): trust boundary — violation iff exceeds boundary
    // -----------------------------------------------------------------------

    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(256))]

        #[test]
        fn prop24a_trust_violation_iff_exceeds_boundary(
            mutation_capable in proptest::bool::ANY,
            requires_egress in proptest::bool::ANY,
            requires_credentials in proptest::bool::ANY,
            max_mutation in proptest::bool::ANY,
            max_egress in proptest::bool::ANY,
            max_credentials in proptest::bool::ANY,
        ) {
            let mcp = make_mcp("test-server", mutation_capable, requires_egress, requires_credentials);
            let store = store_with_mcp(mcp);
            let installed = vec![make_installed("test-server")];
            let boundary = TrustBoundaryPolicy { max_mutation, max_egress, max_credentials };

            let violations = evaluate_trust(&installed, &store, &boundary, "prod", &[]);

            // Violation should occur iff at least one attribute exceeds boundary
            let expected_violation =
                (!max_mutation && mutation_capable)
                || (!max_egress && requires_egress)
                || (!max_credentials && requires_credentials);

            proptest::prop_assert_eq!(
                !violations.is_empty(),
                expected_violation,
                "mutation_capable={}, requires_egress={}, requires_credentials={}, \
                 max_mutation={}, max_egress={}, max_credentials={}",
                mutation_capable, requires_egress, requires_credentials,
                max_mutation, max_egress, max_credentials
            );
        }
    }
}
