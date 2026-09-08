use std::path::Path;

use crate::error::TuiError;
use crate::models::{
    Agent, AssetIntegrity, McpReference, ModelAssignments, ModelRegistry, RoleCatalog, Rule, Skill,
    WorkflowCatalog,
};
use crate::security::sanitize::has_control_bytes;

const MAX_CATALOG_FILE_SIZE: u64 = 20 * 1024 * 1024;

/// Read a catalog file with size validation.
/// Returns an error if the file exceeds `max_size`.
fn read_catalog_file_with_limit(path: &std::path::Path, max_size: u64) -> Result<String, TuiError> {
    let metadata = std::fs::metadata(path).map_err(|_| TuiError::CatalogNotFound {
        path: path.display().to_string(),
    })?;
    if metadata.len() > max_size {
        return Err(TuiError::CatalogParse {
            path: path.display().to_string(),
            offset: 0,
            detail: format!(
                "file too large: {} bytes exceeds maximum of {} bytes",
                metadata.len(),
                max_size
            ),
        });
    }
    std::fs::read_to_string(path).map_err(|_| TuiError::CatalogNotFound {
        path: path.display().to_string(),
    })
}

/// Read a catalog file with the default size limit (100MB).
fn read_catalog_file(path: &std::path::Path) -> Result<String, TuiError> {
    read_catalog_file_with_limit(path, MAX_CATALOG_FILE_SIZE)
}

/// Load agents from catalog/agents.json.
/// Returns loaded agents and any errors encountered.
pub fn load_agents(workspace_root: &Path) -> (Vec<Agent>, Vec<TuiError>) {
    let file_path = workspace_root.join("catalog").join("agents.json");
    let path = file_path.display().to_string();
    let mut errors = Vec::new();

    let data = match read_catalog_file(&file_path) {
        Ok(d) => d,
        Err(e) => {
            errors.push(e);
            return (Vec::new(), errors);
        }
    };

    let all_agents: Vec<Agent> = match serde_json::from_str(&data) {
        Ok(a) => a,
        Err(e) => {
            errors.push(TuiError::CatalogParse {
                path,
                offset: e.column(),
                detail: e.to_string(),
            });
            return (Vec::new(), errors);
        }
    };

    let mut agents = Vec::new();
    for (idx, agent) in all_agents.into_iter().enumerate() {
        if check_agent_tainted(&agent) {
            errors.push(TuiError::TaintedEntry {
                path: path.clone(),
                offset: idx,
                field: "control bytes detected".to_string(),
            });
        } else {
            agents.push(agent);
        }
    }

    (agents, errors)
}

/// Load skills from catalog/skills.json.
pub fn load_skills(workspace_root: &Path) -> (Vec<Skill>, Vec<TuiError>) {
    let file_path = workspace_root.join("catalog").join("skills.json");
    let path = file_path.display().to_string();
    let mut errors = Vec::new();

    let data = match read_catalog_file(&file_path) {
        Ok(d) => d,
        Err(e) => {
            errors.push(e);
            return (Vec::new(), errors);
        }
    };

    let all_skills: Vec<Skill> = match serde_json::from_str(&data) {
        Ok(s) => s,
        Err(e) => {
            errors.push(TuiError::CatalogParse {
                path,
                offset: e.column(),
                detail: e.to_string(),
            });
            return (Vec::new(), errors);
        }
    };

    let mut skills = Vec::new();
    for (idx, skill) in all_skills.into_iter().enumerate() {
        if check_skill_tainted(&skill) {
            errors.push(TuiError::TaintedEntry {
                path: path.clone(),
                offset: idx,
                field: "control bytes detected".to_string(),
            });
        } else {
            skills.push(skill);
        }
    }

    (skills, errors)
}

/// Load MCP references from catalog/mcp-references.json.
pub fn load_mcp_refs(workspace_root: &Path) -> (Vec<McpReference>, Vec<TuiError>) {
    let file_path = workspace_root.join("catalog").join("mcp-references.json");
    let path = file_path.display().to_string();
    let mut errors = Vec::new();

    let data = match read_catalog_file(&file_path) {
        Ok(d) => d,
        Err(e) => {
            errors.push(e);
            return (Vec::new(), errors);
        }
    };

    let all_refs: Vec<McpReference> = match serde_json::from_str(&data) {
        Ok(r) => r,
        Err(e) => {
            errors.push(TuiError::CatalogParse {
                path,
                offset: e.column(),
                detail: e.to_string(),
            });
            return (Vec::new(), errors);
        }
    };

    let mut refs = Vec::new();
    for (idx, mcp_ref) in all_refs.into_iter().enumerate() {
        if check_mcp_ref_tainted(&mcp_ref) {
            errors.push(TuiError::TaintedEntry {
                path: path.clone(),
                offset: idx,
                field: "control bytes detected".to_string(),
            });
        } else {
            refs.push(mcp_ref);
        }
    }

    (refs, errors)
}

/// Load rules from catalog/rules.json.
pub fn load_rules(workspace_root: &Path) -> (Vec<Rule>, Vec<TuiError>) {
    let file_path = workspace_root.join("catalog").join("rules.json");
    let path = file_path.display().to_string();
    let mut errors = Vec::new();

    let data = match read_catalog_file(&file_path) {
        Ok(d) => d,
        Err(e) => {
            errors.push(e);
            return (Vec::new(), errors);
        }
    };

    let all_rules: Vec<Rule> = match serde_json::from_str(&data) {
        Ok(r) => r,
        Err(e) => {
            errors.push(TuiError::CatalogParse {
                path,
                offset: e.column(),
                detail: e.to_string(),
            });
            return (Vec::new(), errors);
        }
    };

    let mut rules = Vec::new();
    for (idx, rule) in all_rules.into_iter().enumerate() {
        if check_rule_tainted(&rule) {
            errors.push(TuiError::TaintedEntry {
                path: path.clone(),
                offset: idx,
                field: "control bytes detected".to_string(),
            });
        } else {
            rules.push(rule);
        }
    }

    (rules, errors)
}

/// Load roles from catalog/install-roles.json.
/// This is an object, not an array.
pub fn load_roles(workspace_root: &Path) -> (Option<RoleCatalog>, Vec<TuiError>) {
    let file_path = workspace_root.join("catalog").join("install-roles.json");
    let path = file_path.display().to_string();
    let mut errors = Vec::new();

    let data = match read_catalog_file(&file_path) {
        Ok(d) => d,
        Err(e) => {
            errors.push(e);
            return (None, errors);
        }
    };

    match serde_json::from_str::<RoleCatalog>(&data) {
        Ok(catalog) => {
            if check_role_catalog_tainted(&catalog) {
                errors.push(TuiError::TaintedEntry {
                    path,
                    offset: 0,
                    field: "control bytes detected".to_string(),
                });
                (None, errors)
            } else {
                (Some(catalog), errors)
            }
        }
        Err(e) => {
            errors.push(TuiError::CatalogParse {
                path,
                offset: e.column(),
                detail: e.to_string(),
            });
            (None, errors)
        }
    }
}

/// Load asset integrity from catalog/asset-integrity.json.
pub fn load_integrity(workspace_root: &Path) -> (Option<AssetIntegrity>, Vec<TuiError>) {
    let file_path = workspace_root.join("catalog").join("asset-integrity.json");
    let path = file_path.display().to_string();
    let mut errors = Vec::new();

    let data = match read_catalog_file(&file_path) {
        Ok(d) => d,
        Err(e) => {
            errors.push(e);
            return (None, errors);
        }
    };

    match serde_json::from_str::<AssetIntegrity>(&data) {
        Ok(integrity) => {
            if check_integrity_tainted(&integrity) {
                errors.push(TuiError::TaintedEntry {
                    path,
                    offset: 0,
                    field: "control bytes detected".to_string(),
                });
                (None, errors)
            } else {
                (Some(integrity), errors)
            }
        }
        Err(e) => {
            errors.push(TuiError::CatalogParse {
                path,
                offset: e.column(),
                detail: e.to_string(),
            });
            (None, errors)
        }
    }
}

/// Load resolved model assignments from catalog/model-assignments.json.
///
/// A missing file is not an error — the model-policy feature is additive and
/// older checkouts simply render without model information.
pub fn load_model_assignments(workspace_root: &Path) -> (Option<ModelAssignments>, Vec<TuiError>) {
    let file_path = workspace_root
        .join("catalog")
        .join("model-assignments.json");
    let path = file_path.display().to_string();
    let mut errors = Vec::new();

    if !file_path.exists() {
        return (None, errors);
    }

    let data = match read_catalog_file(&file_path) {
        Ok(d) => d,
        Err(e) => {
            errors.push(e);
            return (None, errors);
        }
    };

    match serde_json::from_str::<ModelAssignments>(&data) {
        Ok(assignments) => {
            if check_model_assignments_tainted(&assignments) {
                errors.push(TuiError::TaintedEntry {
                    path,
                    offset: 0,
                    field: "control bytes detected".to_string(),
                });
                (None, errors)
            } else {
                (Some(assignments), errors)
            }
        }
        Err(e) => {
            errors.push(TuiError::CatalogParse {
                path,
                offset: e.column(),
                detail: e.to_string(),
            });
            (None, errors)
        }
    }
}

/// Load the verified model registry from catalog/model-registry.json.
///
/// A missing file is not an error — the picker degrades to free-text entry
/// and the harness-level effort union, exactly as it behaved before the
/// registry was wired in. A malformed file is reported and treated the same
/// way, so a bad catalog can never wedge the builder.
pub fn load_model_registry(workspace_root: &Path) -> (Option<ModelRegistry>, Vec<TuiError>) {
    let file_path = workspace_root.join("catalog").join("model-registry.json");
    let path = file_path.display().to_string();
    let mut errors = Vec::new();

    if !file_path.exists() {
        return (None, errors);
    }

    let data = match read_catalog_file(&file_path) {
        Ok(d) => d,
        Err(e) => {
            errors.push(e);
            return (None, errors);
        }
    };

    match serde_json::from_str::<ModelRegistry>(&data) {
        Ok(registry) => {
            if check_model_registry_tainted(&registry) {
                errors.push(TuiError::TaintedEntry {
                    path,
                    offset: 0,
                    field: "control bytes detected".to_string(),
                });
                (None, errors)
            } else {
                (Some(registry), errors)
            }
        }
        Err(e) => {
            errors.push(TuiError::CatalogParse {
                path,
                offset: e.column(),
                detail: e.to_string(),
            });
            (None, errors)
        }
    }
}

/// Load the workflow catalog from catalog/workflows.json.
///
/// A missing file is not an error — the workflow catalog is additive and a checkout
/// without it simply renders no workflows. The file is generated from the `meta` block
/// of each script in `.claude/workflows/` by
/// `scripts/generate-workflow-catalog.mjs`; the TUI never parses the scripts itself.
pub fn load_workflows(workspace_root: &Path) -> (Option<WorkflowCatalog>, Vec<TuiError>) {
    let file_path = workspace_root.join("catalog").join("workflows.json");
    let path = file_path.display().to_string();
    let mut errors = Vec::new();

    if !file_path.exists() {
        return (None, errors);
    }

    let data = match read_catalog_file(&file_path) {
        Ok(d) => d,
        Err(e) => {
            errors.push(e);
            return (None, errors);
        }
    };

    match serde_json::from_str::<WorkflowCatalog>(&data) {
        Ok(catalog) => {
            if check_workflows_tainted(&catalog) {
                errors.push(TuiError::TaintedEntry {
                    path,
                    offset: 0,
                    field: "control bytes detected".to_string(),
                });
                (None, errors)
            } else {
                (Some(catalog), errors)
            }
        }
        Err(e) => {
            errors.push(TuiError::CatalogParse {
                path,
                offset: e.column(),
                detail: e.to_string(),
            });
            (None, errors)
        }
    }
}

// Taint checks for each model type.

pub(crate) fn check_workflows_tainted(catalog: &WorkflowCatalog) -> bool {
    serde_json::to_value(catalog)
        .map(|value| value_has_control_bytes(&value))
        .unwrap_or(true)
}

pub(crate) fn check_model_registry_tainted(registry: &ModelRegistry) -> bool {
    serde_json::to_value(registry)
        .map(|v| value_has_control_bytes(&v))
        .unwrap_or(true)
}

pub(crate) fn check_model_assignments_tainted(assignments: &ModelAssignments) -> bool {
    serde_json::to_value(assignments)
        .map(|value| value_has_control_bytes(&value))
        .unwrap_or(true)
}

fn check_agent_tainted(agent: &Agent) -> bool {
    serde_json::to_value(agent)
        .map(|value| value_has_control_bytes(&value))
        .unwrap_or(true)
}

fn check_skill_tainted(skill: &Skill) -> bool {
    serde_json::to_value(skill)
        .map(|value| value_has_control_bytes(&value))
        .unwrap_or(true)
}

fn check_mcp_ref_tainted(mcp_ref: &McpReference) -> bool {
    serde_json::to_value(mcp_ref)
        .map(|value| value_has_control_bytes(&value))
        .unwrap_or(true)
}

fn check_rule_tainted(rule: &Rule) -> bool {
    serde_json::to_value(rule)
        .map(|value| value_has_control_bytes(&value))
        .unwrap_or(true)
}

fn check_role_catalog_tainted(role_catalog: &RoleCatalog) -> bool {
    serde_json::to_value(role_catalog)
        .map(|value| value_has_control_bytes(&value))
        .unwrap_or(true)
}

fn check_integrity_tainted(integrity: &AssetIntegrity) -> bool {
    serde_json::to_value(integrity)
        .map(|value| value_has_control_bytes(&value))
        .unwrap_or(true)
}

fn value_has_control_bytes(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::String(s) => has_control_bytes(s),
        serde_json::Value::Array(items) => items.iter().any(value_has_control_bytes),
        serde_json::Value::Object(map) => map
            .iter()
            .any(|(key, value)| has_control_bytes(key) || value_has_control_bytes(value)),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn workspace_root() -> &'static Path {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        // tools/vfa-tui -> project root
        let p = Path::new(manifest_dir).parent().unwrap().parent().unwrap();
        // Leak to get static lifetime for testing
        Box::leak(Box::new(p.to_path_buf()))
    }

    #[test]
    fn load_agents_succeeds() {
        let (agents, errors) = load_agents(workspace_root());
        assert!(errors.is_empty(), "errors: {errors:?}");
        assert!(!agents.is_empty());
    }

    #[test]
    fn load_skills_succeeds() {
        let (skills, errors) = load_skills(workspace_root());
        assert!(errors.is_empty(), "errors: {errors:?}");
        assert!(!skills.is_empty());
    }

    #[test]
    fn load_workflows_parses_the_real_catalog() {
        // Loads the committed catalog/workflows.json, so a generator change that emits a
        // key this struct does not declare fails here rather than silently vanishing
        // from the display. `deny_unknown_fields` only bites if something exercises it
        // against real data.
        let (catalog, errors) = load_workflows(workspace_root());
        assert!(errors.is_empty(), "errors: {errors:?}");
        let catalog = catalog.expect("catalog/workflows.json should be present");
        assert!(
            !catalog.workflows.is_empty(),
            "expected at least one workflow in the catalog",
        );
        for wf in &catalog.workflows {
            assert!(!wf.id.is_empty(), "workflow id must not be empty");
            assert_eq!(wf.id, wf.name, "id and name are the same identifier");
            assert!(
                wf.path.starts_with(".claude/workflows/"),
                "workflow path should be repo-relative: {}",
                wf.path,
            );
        }
    }

    #[test]
    fn load_workflows_tolerates_a_missing_file() {
        // Additive feature: a checkout without any workflow renders none rather than
        // reporting a load error.
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("catalog")).unwrap();
        let (catalog, errors) = load_workflows(tmp.path());
        assert!(catalog.is_none());
        assert!(errors.is_empty(), "missing file must not be an error");
    }

    #[test]
    fn load_mcp_refs_succeeds() {
        let (refs, errors) = load_mcp_refs(workspace_root());
        assert!(errors.is_empty(), "errors: {errors:?}");
        assert!(!refs.is_empty());
    }

    #[test]
    fn load_rules_succeeds() {
        let (rules, errors) = load_rules(workspace_root());
        assert!(errors.is_empty(), "errors: {errors:?}");
        assert!(!rules.is_empty());
    }

    #[test]
    fn load_roles_succeeds() {
        let (roles, errors) = load_roles(workspace_root());
        assert!(errors.is_empty(), "errors: {errors:?}");
        assert!(roles.is_some());
        let catalog = roles.unwrap();
        assert!(!catalog.roles.is_empty());
    }

    #[test]
    fn load_integrity_succeeds() {
        let (integrity, errors) = load_integrity(workspace_root());
        assert!(errors.is_empty(), "errors: {errors:?}");
        assert!(integrity.is_some());
    }

    #[test]
    fn load_roles_rejects_tainted_strings() {
        let tmp = tempfile::TempDir::new().unwrap();
        let catalog_dir = tmp.path().join("catalog");
        std::fs::create_dir(&catalog_dir).unwrap();
        std::fs::write(
            catalog_dir.join("install-roles.json"),
            r#"{
              "version": "1",
              "description": "roles",
              "roles": {
                "security": {
                  "label": "Security\u001b[31m",
                  "description": "bad",
                  "agents": []
                }
              }
            }"#,
        )
        .unwrap();

        let (roles, errors) = load_roles(tmp.path());
        assert!(roles.is_none());
        assert!(matches!(
            errors.first(),
            Some(TuiError::TaintedEntry { .. })
        ));
    }

    #[test]
    fn load_integrity_rejects_tainted_strings() {
        let tmp = tempfile::TempDir::new().unwrap();
        let catalog_dir = tmp.path().join("catalog");
        std::fs::create_dir(&catalog_dir).unwrap();
        std::fs::write(
            catalog_dir.join("asset-integrity.json"),
            r#"{
              "manifest_version": 1,
              "algorithm": "sha256",
              "scope": { "trees": [], "root_files": [] },
              "trees": [{
                "tree": "assets\u001b[31m",
                "aggregate_sha256": "abc",
                "files": []
              }],
              "root_files": [],
              "aggregate_sha256": "abc"
            }"#,
        )
        .unwrap();

        let (integrity, errors) = load_integrity(tmp.path());
        assert!(integrity.is_none());
        assert!(matches!(
            errors.first(),
            Some(TuiError::TaintedEntry { .. })
        ));
    }

    #[test]
    fn load_agents_missing_file() {
        let tmp = tempfile::TempDir::new().unwrap();
        let (agents, errors) = load_agents(tmp.path());
        assert!(agents.is_empty());
        assert_eq!(errors.len(), 1);
        match &errors[0] {
            TuiError::CatalogNotFound { .. } => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn read_catalog_file_rejects_oversized() {
        // Use the configurable limit function to test the rejection path
        // without creating a 100MB file
        let tmp = tempfile::TempDir::new().unwrap();
        let file = tmp.path().join("small.json");
        std::fs::write(&file, "[1,2,3]").unwrap(); // 7 bytes

        // With a limit smaller than the file, it should be rejected
        let result = read_catalog_file_with_limit(&file, 5);
        assert!(result.is_err());
        match result.unwrap_err() {
            TuiError::CatalogParse { detail, .. } => {
                assert!(
                    detail.contains("file too large"),
                    "unexpected detail: {detail}"
                );
                assert!(
                    detail.contains("7 bytes"),
                    "should report actual size: {detail}"
                );
            }
            other => panic!("expected CatalogParse error, got: {other:?}"),
        }

        // With a limit larger than the file, it should succeed
        let result = read_catalog_file_with_limit(&file, 100);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "[1,2,3]");
    }

    #[test]
    fn read_catalog_file_rejects_missing() {
        let tmp = tempfile::TempDir::new().unwrap();
        let missing = tmp.path().join("missing.json");
        let result = read_catalog_file(&missing);
        assert!(result.is_err());
    }
}
