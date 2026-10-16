use std::io::Read;
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::error::TuiError;
use crate::models::{
    Agent, AssetIntegrity, McpReference, Mod, ModelAssignments, ModelRegistry, RoleCatalog, Rule,
    Skill, WorkflowCatalog,
};
use crate::security::sanitize::{
    has_control_bytes, is_disallowed_control, sanitize_catalog_string,
};

const MAX_CATALOG_FILE_SIZE: u64 = 20 * 1024 * 1024;

#[derive(Debug)]
pub(crate) struct ModSnapshot {
    pub entries: Vec<Mod>,
    pub content_hash: String,
}

fn mod_error(path: &Path, offset: usize, detail: String) -> TuiError {
    TuiError::CatalogParse {
        path: sanitize_catalog_string(&path.display().to_string()),
        offset,
        detail: sanitize_catalog_string(&detail),
    }
}

/// Optional only when opening reports NotFound. Other failures remain visible.
pub fn load_mods(workspace_root: &Path) -> (Vec<Mod>, Vec<TuiError>) {
    match read_mods_snapshot(&workspace_root.join("catalog/mods.json")) {
        Ok(Some(snapshot)) => (snapshot.entries, Vec::new()),
        Ok(None) => (Vec::new(), Vec::new()),
        Err(error) => (Vec::new(), vec![error]),
    }
}

/// Validate and hash exactly one bounded read; never cache failed bytes.
pub(crate) fn read_mods_snapshot(path: &Path) -> Result<Option<ModSnapshot>, TuiError> {
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(mod_error(path, 0, format!("could not open: {error}"))),
    };
    read_mods_from(file, path).map(Some)
}

fn read_mods_from(reader: impl Read, path: &Path) -> Result<ModSnapshot, TuiError> {
    let mut bytes = Vec::new();
    reader
        .take(MAX_CATALOG_FILE_SIZE + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| mod_error(path, 0, format!("could not read: {error}")))?;
    if bytes.len() as u64 > MAX_CATALOG_FILE_SIZE {
        return Err(mod_error(
            path,
            0,
            format!("file too large: maximum is {MAX_CATALOG_FILE_SIZE} bytes"),
        ));
    }
    let text = std::str::from_utf8(&bytes)
        .map_err(|error| mod_error(path, error.valid_up_to(), format!("invalid UTF-8: {error}")))?;
    let entries: Vec<Mod> = serde_json::from_str(text).map_err(|error| {
        let offset = if error.is_eof() {
            bytes.len()
        } else {
            let line_start: usize = text
                .split_inclusive('\n')
                .take(error.line().saturating_sub(1))
                .map(str::len)
                .sum();
            (line_start + error.column().saturating_sub(1)).min(bytes.len())
        };
        mod_error(path, offset, error.to_string())
    })?;
    for (index, entry) in entries.iter().enumerate() {
        if entry.reach_level > 3 {
            return Err(mod_error(
                path,
                0,
                format!("mods[{index}].reach_level must be in 0..=3"),
            ));
        }
        let value =
            serde_json::to_value(entry).map_err(|error| mod_error(path, 0, error.to_string()))?;
        if let Some((field, offset)) = mod_taint(&value, &format!("mods[{index}]")) {
            return Err(TuiError::TaintedEntry {
                path: path.display().to_string(),
                offset,
                field,
            });
        }
    }
    use std::fmt::Write as _;
    let content_hash = Sha256::digest(&bytes)
        .iter()
        .fold(String::new(), |mut out, byte| {
            let _ = write!(out, "{byte:02x}");
            out
        });
    Ok(ModSnapshot {
        entries,
        content_hash,
    })
}

fn mod_taint(value: &serde_json::Value, field: &str) -> Option<(String, usize)> {
    match value {
        serde_json::Value::String(text) if has_control_bytes(text) => text
            .char_indices()
            .find(|(_, character)| is_disallowed_control(*character))
            .map(|(offset, _)| (field.to_string(), offset)),
        serde_json::Value::Array(items) => items
            .iter()
            .enumerate()
            .find_map(|(index, item)| mod_taint(item, &format!("{field}[{index}]"))),
        serde_json::Value::Object(items) => items
            .iter()
            .find_map(|(key, item)| mod_taint(item, &format!("{field}.{key}"))),
        _ => None,
    }
}

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

    fn mod_entry() -> serde_json::Value {
        serde_json::json!({
            "id": "example-mod", "name": "Example é", "version": "1.0.0",
            "description": "Example mod", "reach_level": 0,
            "events": ["PreToolUse"], "harnesses": ["codex"], "path": "mods/example-mod",
            "security_notes": "Declared only", "footprint": {
                "env_reads": ["EXAMPLE"], "file_writes": ["example/**"],
                "spawns": ["example-process"], "egress_hosts": ["example.invalid"]
            }
        })
    }

    fn mods_workspace(bytes: &[u8]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("catalog")).unwrap();
        std::fs::write(dir.path().join("catalog/mods.json"), bytes).unwrap();
        dir
    }

    #[test]
    fn load_mods_optional_empty_and_current_catalog() {
        let missing = tempfile::tempdir().unwrap();
        let (entries, errors) = load_mods(missing.path());
        assert!(entries.is_empty() && errors.is_empty());
        let empty = mods_workspace(b"[]");
        let (entries, errors) = load_mods(empty.path());
        assert!(entries.is_empty() && errors.is_empty());
        let actual: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(workspace_root().join("catalog/mods.json")).unwrap(),
        )
        .unwrap();
        let (entries, errors) = load_mods(workspace_root());
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(entries.len(), actual.as_array().unwrap().len());
        for entry in entries {
            let raw = actual
                .as_array()
                .unwrap()
                .iter()
                .find(|raw| raw["id"] == entry.id)
                .unwrap();
            let expected: crate::models::Mod = serde_json::from_value(raw.clone()).unwrap();
            assert_eq!(entry, expected);
        }
    }

    #[test]
    fn load_mods_rejects_all_tainted_fields_without_partial_adoption() {
        for field in [
            "/id",
            "/name",
            "/version",
            "/description",
            "/path",
            "/security_notes",
            "/events/0",
            "/harnesses/0",
            "/footprint/env_reads/0",
            "/footprint/file_writes/0",
            "/footprint/spawns/0",
            "/footprint/egress_hosts/0",
        ] {
            let mut tainted = mod_entry();
            *tainted.pointer_mut(field).unwrap() = serde_json::json!("é\u{1b}bad");
            let dir = mods_workspace(&serde_json::to_vec(&vec![mod_entry(), tainted]).unwrap());
            let (entries, errors) = load_mods(dir.path());
            assert!(entries.is_empty(), "partially adopted {field}");
            match &errors[0] {
                TuiError::TaintedEntry {
                    offset,
                    field: diagnostic,
                    ..
                } => {
                    assert_eq!(*offset, 2, "byte offset within the UTF-8 field");
                    assert!(diagnostic.starts_with("mods[1]."), "{diagnostic}");
                    assert!(diagnostic
                        .contains(field.trim_start_matches('/').split('/').next().unwrap()));
                }
                other => panic!("expected taint diagnostic for {field}, got {other:?}"),
            }
        }
        let mut allowed = mod_entry();
        allowed["description"] = serde_json::json!("allowed\tand\nnewlines");
        let dir = mods_workspace(&serde_json::to_vec(&vec![allowed]).unwrap());
        assert!(load_mods(dir.path()).1.is_empty());
    }

    #[test]
    fn load_mods_enforces_reach_and_schema() {
        for level in [
            serde_json::json!(-1),
            serde_json::json!(4),
            serde_json::json!(255),
            serde_json::json!(1.5),
        ] {
            let mut entry = mod_entry();
            entry["reach_level"] = level;
            let dir = mods_workspace(&serde_json::to_vec(&vec![entry]).unwrap());
            let (entries, errors) = load_mods(dir.path());
            assert!(entries.is_empty() && !errors.is_empty());
        }
        for level in 0..=3 {
            let mut entry = mod_entry();
            entry["reach_level"] = serde_json::json!(level);
            let dir = mods_workspace(&serde_json::to_vec(&vec![entry]).unwrap());
            assert!(load_mods(dir.path()).1.is_empty());
        }
        let mut entry = mod_entry();
        entry["unknown\u{1b}field"] = serde_json::json!(true);
        let dir = mods_workspace(&serde_json::to_vec(&vec![entry]).unwrap());
        let (entries, errors) = load_mods(dir.path());
        assert!(entries.is_empty() && !errors.is_empty());
        assert!(!errors[0].to_string().contains('\u{1b}'));
    }

    #[test]
    fn load_mods_bounds_reads_and_reports_real_io_failures() {
        let mut limit = vec![b' '; MAX_CATALOG_FILE_SIZE as usize];
        limit[0] = b'[';
        *limit.last_mut().unwrap() = b']';
        let dir = mods_workspace(&limit);
        assert!(load_mods(dir.path()).1.is_empty());
        limit.push(b' ');
        std::fs::write(dir.path().join("catalog/mods.json"), &limit).unwrap();
        let (entries, errors) = load_mods(dir.path());
        assert!(entries.is_empty());
        assert!(errors[0].to_string().contains("too large"));
        std::fs::remove_file(dir.path().join("catalog/mods.json")).unwrap();
        std::fs::create_dir(dir.path().join("catalog/mods.json")).unwrap();
        assert!(
            !load_mods(dir.path()).1.is_empty(),
            "directory is not optional absence"
        );
        struct Denied;
        impl std::io::Read for Denied {
            fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied))
            }
        }
        let error = read_mods_from(Denied, Path::new("catalog/mods.json")).unwrap_err();
        assert!(error
            .to_string()
            .to_lowercase()
            .contains("permission denied"));
    }

    #[test]
    fn load_mods_reports_utf8_multiline_and_eof_byte_positions() {
        let dir = mods_workspace(b"\xff");
        let errors = load_mods(dir.path()).1;
        assert!(matches!(
            &errors[0],
            TuiError::CatalogParse { offset: 0, .. }
        ));
        let entry = serde_json::to_string(&mod_entry()).unwrap();
        let invalid = format!("[\n{entry},\n ?\n]");
        std::fs::write(dir.path().join("catalog/mods.json"), &invalid).unwrap();
        match &load_mods(dir.path()).1[0] {
            TuiError::CatalogParse { offset, detail, .. } => {
                assert_eq!(*offset, invalid.find('?').unwrap());
                assert!(detail.contains("line 3 column 2"), "{detail}");
            }
            other => panic!("{other:?}"),
        }
        let eof = format!("[\n{entry},\n");
        std::fs::write(dir.path().join("catalog/mods.json"), &eof).unwrap();
        assert!(
            matches!(&load_mods(dir.path()).1[0], TuiError::CatalogParse { offset, .. } if *offset == eof.len())
        );
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
