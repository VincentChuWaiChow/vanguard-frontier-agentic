use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::error::TuiError;
use crate::models::{
    Agent, AssetIntegrity, McpReference, ModelAssignments, ModelRegistry, Role, Rule, Skill,
    WorkflowCatalog,
};

use super::loader;

// ---------------------------------------------------------------------------
// ReloadOutcome
// ---------------------------------------------------------------------------

/// Outcome returned by [`CatalogStore::reload_file`].
#[derive(Debug)]
pub enum ReloadOutcome {
    /// The file was re-parsed successfully and the named catalog was updated.
    Reloaded { catalog: String },
    /// The new content failed to parse; the previous valid state was retained.
    RetainedPrevious { error: String },
    /// The file's content hash matched the already-loaded version — no update needed.
    Unchanged,
}

// ---------------------------------------------------------------------------
// EdgeKind
// ---------------------------------------------------------------------------

/// Classifies an edge in the catalog dependency graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdgeKind {
    /// Agent → Skill via `companion_skills`.
    AgentSkill,
    /// Role → Agent via role membership.
    RoleAgent,
    /// Agent → McpReference (detected from harness variants / mcp_refs cross-ref).
    AgentMcp,
    /// Agent → Rule (agent is harness-specific and rule shares harness).
    AgentRule,
}

// ---------------------------------------------------------------------------
// CatalogStore
// ---------------------------------------------------------------------------

/// In-memory catalog store holding all loaded catalog data.
pub struct CatalogStore {
    pub agents: Vec<Agent>,
    pub skills: Vec<Skill>,
    pub roles: HashMap<String, Role>,
    pub role_catalog_version: String,
    pub role_catalog_description: String,
    pub mcp_refs: Vec<McpReference>,
    pub rules: Vec<Rule>,
    pub integrity: Option<AssetIntegrity>,
    /// Resolved per-agent/per-harness model + reasoning assignments
    /// (catalog/model-assignments.json); absent on older checkouts.
    pub model_assignments: Option<ModelAssignments>,
    /// Verified model / reasoning-effort matrix (catalog/model-registry.json).
    /// Drives the Model Policy builder's pickers; `None` degrades them to
    /// free-text entry rather than blocking the view.
    pub model_registry: Option<ModelRegistry>,
    /// Workflows declared in `.claude/workflows/`, generated into
    /// `catalog/workflows.json`; absent on checkouts without any workflow.
    pub workflows: Option<WorkflowCatalog>,
    pub load_errors: Vec<TuiError>,
    /// SHA-256 hex digest of each catalog JSON file's raw bytes, keyed by absolute path.
    pub content_hashes: HashMap<PathBuf, String>,
    /// Root directory from which catalogs were loaded.
    pub catalog_root: PathBuf,
}

/// Compute SHA-256 hex digest of a byte slice.
fn sha256_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    result.iter().fold(String::new(), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}

/// Compute SHA-256 hex digest of a file on disk; returns None on I/O error.
fn hash_file(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    Some(sha256_hex(&bytes))
}

impl CatalogStore {
    /// Load all catalog files from the workspace root.
    /// Sorts agents, skills, mcp_refs, and rules by ID (case-insensitive).
    pub fn load(workspace_root: &Path) -> Self {
        let mut load_errors = Vec::new();

        let (mut agents, errs) = loader::load_agents(workspace_root);
        load_errors.extend(errs);

        let (mut skills, errs) = loader::load_skills(workspace_root);
        load_errors.extend(errs);

        let (mut mcp_refs, errs) = loader::load_mcp_refs(workspace_root);
        load_errors.extend(errs);

        let (mut rules, errs) = loader::load_rules(workspace_root);
        load_errors.extend(errs);

        let (role_catalog, errs) = loader::load_roles(workspace_root);
        load_errors.extend(errs);

        let (integrity, errs) = loader::load_integrity(workspace_root);
        load_errors.extend(errs);

        let (model_assignments, errs) = loader::load_model_assignments(workspace_root);
        load_errors.extend(errs);

        let (model_registry, errs) = loader::load_model_registry(workspace_root);
        load_errors.extend(errs);

        let (workflows, errs) = loader::load_workflows(workspace_root);
        load_errors.extend(errs);

        // Sort by ID, case-insensitive
        agents.sort_by_key(|a| a.id.to_lowercase());
        skills.sort_by_key(|a| a.id.to_lowercase());
        mcp_refs.sort_by_key(|a| a.id.to_lowercase());
        rules.sort_by_key(|a| a.id.to_lowercase());

        let (roles, role_catalog_version, role_catalog_description) = match role_catalog {
            Some(rc) => (rc.roles, rc.version, rc.description),
            None => (HashMap::new(), String::new(), String::new()),
        };

        // Build content hashes for the catalog JSON files
        let catalog_dir = workspace_root.join("catalog");
        let mut content_hashes = HashMap::new();
        for filename in &[
            "agents.json",
            "skills.json",
            "mcp-references.json",
            "rules.json",
            "install-roles.json",
            "asset-integrity.json",
            "model-assignments.json",
            "model-registry.json",
            "workflows.json",
        ] {
            let path = catalog_dir.join(filename);
            if let Some(hash) = hash_file(&path) {
                content_hashes.insert(path, hash);
            }
        }

        CatalogStore {
            agents,
            skills,
            roles,
            role_catalog_version,
            role_catalog_description,
            mcp_refs,
            rules,
            integrity,
            model_assignments,
            model_registry,
            workflows,
            load_errors,
            content_hashes,
            catalog_root: workspace_root.to_path_buf(),
        }
    }

    /// Re-parse a single catalog file identified by its path.
    ///
    /// - If the file content hash is unchanged from the previously loaded hash,
    ///   returns `ReloadOutcome::Unchanged`.
    /// - If the file parses successfully, updates the corresponding in-memory
    ///   slice and content hash, then returns `ReloadOutcome::Reloaded`.
    /// - If the file fails to parse (invalid JSON), retains the previous valid
    ///   state and returns `ReloadOutcome::RetainedPrevious` with error detail
    ///   (including byte offset when available from serde_json).
    pub fn reload_file(&mut self, path: &Path) -> ReloadOutcome {
        // Read raw bytes for hashing, fall back gracefully on I/O error
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(e) => {
                return ReloadOutcome::RetainedPrevious {
                    error: format!("could not read {}: {}", path.display(), e),
                };
            }
        };

        let new_hash = sha256_hex(&bytes);

        // Check if the content changed at all
        let abs_path = path.to_path_buf();
        if let Some(existing) = self.content_hashes.get(&abs_path) {
            if *existing == new_hash {
                return ReloadOutcome::Unchanged;
            }
        }

        let content = match String::from_utf8(bytes) {
            Ok(s) => s,
            Err(e) => {
                return ReloadOutcome::RetainedPrevious {
                    error: format!("invalid UTF-8 in {}: {}", path.display(), e),
                };
            }
        };

        // Determine which catalog this is by filename
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        match filename {
            "agents.json" => match serde_json::from_str::<Vec<Agent>>(&content) {
                Ok(mut new_agents) => {
                    new_agents.sort_by_key(|a| a.id.to_lowercase());
                    self.agents = new_agents;
                    self.content_hashes.insert(abs_path, new_hash);
                    ReloadOutcome::Reloaded {
                        catalog: "agents".to_string(),
                    }
                }
                Err(e) => ReloadOutcome::RetainedPrevious {
                    error: format!(
                        "parse error in {} at offset {}: {}",
                        path.display(),
                        e.column(),
                        e
                    ),
                },
            },
            "skills.json" => match serde_json::from_str::<Vec<Skill>>(&content) {
                Ok(mut new_skills) => {
                    new_skills.sort_by_key(|s| s.id.to_lowercase());
                    self.skills = new_skills;
                    self.content_hashes.insert(abs_path, new_hash);
                    ReloadOutcome::Reloaded {
                        catalog: "skills".to_string(),
                    }
                }
                Err(e) => ReloadOutcome::RetainedPrevious {
                    error: format!(
                        "parse error in {} at offset {}: {}",
                        path.display(),
                        e.column(),
                        e
                    ),
                },
            },
            "mcp-references.json" => match serde_json::from_str::<Vec<McpReference>>(&content) {
                Ok(mut new_refs) => {
                    new_refs.sort_by_key(|r| r.id.to_lowercase());
                    self.mcp_refs = new_refs;
                    self.content_hashes.insert(abs_path, new_hash);
                    ReloadOutcome::Reloaded {
                        catalog: "mcp-references".to_string(),
                    }
                }
                Err(e) => ReloadOutcome::RetainedPrevious {
                    error: format!(
                        "parse error in {} at offset {}: {}",
                        path.display(),
                        e.column(),
                        e
                    ),
                },
            },
            "rules.json" => match serde_json::from_str::<Vec<Rule>>(&content) {
                Ok(mut new_rules) => {
                    new_rules.sort_by_key(|r| r.id.to_lowercase());
                    self.rules = new_rules;
                    self.content_hashes.insert(abs_path, new_hash);
                    ReloadOutcome::Reloaded {
                        catalog: "rules".to_string(),
                    }
                }
                Err(e) => ReloadOutcome::RetainedPrevious {
                    error: format!(
                        "parse error in {} at offset {}: {}",
                        path.display(),
                        e.column(),
                        e
                    ),
                },
            },
            "install-roles.json" => {
                match serde_json::from_str::<crate::models::RoleCatalog>(&content) {
                    Ok(rc) => {
                        self.roles = rc.roles;
                        self.role_catalog_version = rc.version;
                        self.role_catalog_description = rc.description;
                        self.content_hashes.insert(abs_path, new_hash);
                        ReloadOutcome::Reloaded {
                            catalog: "install-roles".to_string(),
                        }
                    }
                    Err(e) => ReloadOutcome::RetainedPrevious {
                        error: format!(
                            "parse error in {} at offset {}: {}",
                            path.display(),
                            e.column(),
                            e
                        ),
                    },
                }
            }
            "asset-integrity.json" => match serde_json::from_str::<AssetIntegrity>(&content) {
                Ok(new_integrity) => {
                    self.integrity = Some(new_integrity);
                    self.content_hashes.insert(abs_path, new_hash);
                    ReloadOutcome::Reloaded {
                        catalog: "asset-integrity".to_string(),
                    }
                }
                Err(e) => ReloadOutcome::RetainedPrevious {
                    error: format!(
                        "parse error in {} at offset {}: {}",
                        path.display(),
                        e.column(),
                        e
                    ),
                },
            },
            "model-registry.json" => match serde_json::from_str::<ModelRegistry>(&content) {
                Ok(new_registry) => {
                    // Mirror loader::load_model_registry: refuse a tainted
                    // reload rather than adopting what startup would reject.
                    if loader::check_model_registry_tainted(&new_registry) {
                        ReloadOutcome::RetainedPrevious {
                            error: TuiError::TaintedEntry {
                                path: path.display().to_string(),
                                offset: 0,
                                field: "control bytes detected".to_string(),
                            }
                            .to_string(),
                        }
                    } else {
                        self.model_registry = Some(new_registry);
                        self.content_hashes.insert(abs_path, new_hash);
                        ReloadOutcome::Reloaded {
                            catalog: "model-registry".to_string(),
                        }
                    }
                }
                Err(e) => ReloadOutcome::RetainedPrevious {
                    error: format!(
                        "parse error in {} at offset {}: {}",
                        path.display(),
                        e.column(),
                        e
                    ),
                },
            },
            "model-assignments.json" => match serde_json::from_str::<ModelAssignments>(&content) {
                Ok(new_assignments) => {
                    // Mirror the initial-load path (catalog::loader::load_model_assignments):
                    // reject a reload that carries control-byte-tainted data
                    // rather than adopting it, so a hot-reload can't smuggle
                    // in what the startup path would have refused.
                    if loader::check_model_assignments_tainted(&new_assignments) {
                        ReloadOutcome::RetainedPrevious {
                            error: TuiError::TaintedEntry {
                                path: path.display().to_string(),
                                offset: 0,
                                field: "control bytes detected".to_string(),
                            }
                            .to_string(),
                        }
                    } else {
                        self.model_assignments = Some(new_assignments);
                        self.content_hashes.insert(abs_path, new_hash);
                        ReloadOutcome::Reloaded {
                            catalog: "model-assignments".to_string(),
                        }
                    }
                }
                Err(e) => ReloadOutcome::RetainedPrevious {
                    error: format!(
                        "parse error in {} at offset {}: {}",
                        path.display(),
                        e.column(),
                        e
                    ),
                },
            },
            other => ReloadOutcome::RetainedPrevious {
                error: format!("unknown catalog file: {other}"),
            },
        }
    }

    // -----------------------------------------------------------------------
    // v2 query methods
    // -----------------------------------------------------------------------

    /// Look up an agent by its ID.
    pub fn agent_by_id(&self, id: &str) -> Option<&Agent> {
        self.agents.iter().find(|a| a.id == id)
    }

    /// Look up a skill by its ID.
    pub fn skill_by_id(&self, id: &str) -> Option<&Skill> {
        self.skills.iter().find(|s| s.id == id)
    }

    /// Resolved model assignments for one agent (empty when the assignments
    /// index is absent or the agent has no capable harness variants).
    pub fn model_assignments_for_agent(
        &self,
        agent_id: &str,
    ) -> Vec<&crate::models::ModelAssignment> {
        self.model_assignments
            .as_ref()
            .map(|m| m.for_agent(agent_id))
            .unwrap_or_default()
    }

    /// Return every asset ID (agents + skills + mcp_refs + rules).
    pub fn all_asset_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self
            .agents
            .iter()
            .map(|a| a.id.clone())
            .chain(self.skills.iter().map(|s| s.id.clone()))
            .chain(self.mcp_refs.iter().map(|m| m.id.clone()))
            .chain(self.rules.iter().map(|r| r.id.clone()))
            .collect();
        ids.sort();
        ids
    }

    /// Return the SHA-256 hex digest for the catalog file at `path`, or `None`
    /// if that path was not loaded or could not be hashed.
    pub fn content_hash_for(&self, path: &Path) -> Option<&str> {
        self.content_hashes.get(path).map(|s| s.as_str())
    }

    /// Enumerate all dependency edges derivable from catalog data.
    ///
    /// Edge types:
    /// - `AgentSkill`: agent → skill via `companion_skills`
    /// - `RoleAgent`: role → agent via role membership
    /// - `AgentMcp`: agent → mcp-reference when an agent's `harness_variants`
    ///   value or `path` matches an mcp_ref id
    /// - `AgentRule`: agent → rule when the agent and rule share at least one
    ///   harness
    pub fn dependency_edges(&self) -> Vec<(String, String, EdgeKind)> {
        let mut edges: Vec<(String, String, EdgeKind)> = Vec::new();

        // AgentSkill edges
        for agent in &self.agents {
            for skill_id in &agent.companion_skills {
                edges.push((agent.id.clone(), skill_id.clone(), EdgeKind::AgentSkill));
            }
        }

        // RoleAgent edges
        for (role_id, role) in &self.roles {
            for agent_id in &role.agents {
                edges.push((role_id.clone(), agent_id.clone(), EdgeKind::RoleAgent));
            }
        }

        // AgentMcp edges: match harness_variants values against mcp_ref ids
        let mcp_ids: HashSet<&str> = self.mcp_refs.iter().map(|m| m.id.as_str()).collect();
        for agent in &self.agents {
            for val in agent.harness_variants.values() {
                if mcp_ids.contains(val.as_str()) {
                    edges.push((agent.id.clone(), val.clone(), EdgeKind::AgentMcp));
                }
            }
        }

        // AgentRule edges: agent and rule share at least one harness
        for agent in &self.agents {
            let agent_harnesses: HashSet<_> = agent.harnesses.iter().collect();
            for rule in &self.rules {
                let shares = rule.harnesses.iter().any(|h| agent_harnesses.contains(h));
                if shares {
                    edges.push((agent.id.clone(), rule.id.clone(), EdgeKind::AgentRule));
                }
            }
        }

        edges
    }

    // -----------------------------------------------------------------------
    // Existing query methods (unchanged)
    // -----------------------------------------------------------------------

    /// Number of loaded agents.
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    /// Number of loaded skills.
    pub fn skill_count(&self) -> usize {
        self.skills.len()
    }

    /// Number of distinct providers across all agents.
    pub fn provider_count(&self) -> usize {
        let providers: HashSet<_> = self.agents.iter().map(|a| a.provider).collect();
        providers.len()
    }

    /// Filter agents by provider (case-insensitive comparison via Provider enum serialization).
    pub fn agents_by_provider(&self, provider: &str) -> Vec<&Agent> {
        let provider_lower = provider.to_lowercase();
        self.agents
            .iter()
            .filter(|a| {
                let serialized = serde_json::to_value(a.provider)
                    .ok()
                    .and_then(|v| v.as_str().map(|s| s.to_string()));
                match serialized {
                    Some(p) => p.to_lowercase() == provider_lower,
                    None => false,
                }
            })
            .collect()
    }

    /// Get agents listed in a role's agents vec.
    pub fn agents_for_role(&self, role_id: &str) -> Vec<&Agent> {
        let role = match self.roles.get(role_id) {
            Some(r) => r,
            None => return Vec::new(),
        };

        let agent_ids: HashSet<&str> = role.agents.iter().map(|s| s.as_str()).collect();
        self.agents
            .iter()
            .filter(|a| agent_ids.contains(a.id.as_str()))
            .collect()
    }

    /// Get agents for a role grouped by provider.
    /// Returns a vec of (provider_name, agents) tuples sorted alphabetically by provider,
    /// with agents sorted alphabetically by ID within each group.
    pub fn agents_for_role_by_provider(&self, role_id: &str) -> Vec<(String, Vec<&Agent>)> {
        let agents = self.agents_for_role(role_id);

        // Group agents by their serialized provider name
        let mut groups: HashMap<String, Vec<&Agent>> = HashMap::new();
        for agent in agents {
            let provider_name = serde_json::to_value(agent.provider)
                .ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| format!("{:?}", agent.provider));
            groups.entry(provider_name).or_default().push(agent);
        }

        // Sort agents within each group by ID (case-insensitive)
        for agents_in_group in groups.values_mut() {
            agents_in_group.sort_by_key(|a| a.id.to_lowercase());
        }

        // Sort groups alphabetically by provider name
        let mut result: Vec<(String, Vec<&Agent>)> = groups.into_iter().collect();
        result.sort_by_key(|(provider, _)| provider.to_lowercase());
        result
    }

    /// Find skills whose id is in an agent's companion_skills.
    pub fn skills_for_agent(&self, agent_id: &str) -> Vec<&Skill> {
        let agent = match self.agents.iter().find(|a| a.id == agent_id) {
            Some(a) => a,
            None => return Vec::new(),
        };

        let skill_ids: HashSet<&str> = agent.companion_skills.iter().map(|s| s.as_str()).collect();
        self.skills
            .iter()
            .filter(|s| skill_ids.contains(s.id.as_str()))
            .collect()
    }

    /// Reverse lookup: agents whose companion_skills contains the given skill_id.
    pub fn agents_with_skill(&self, skill_id: &str) -> Vec<&Agent> {
        self.agents
            .iter()
            .filter(|a| a.companion_skills.iter().any(|s| s == skill_id))
            .collect()
    }

    /// Reverse lookup: roles that contain the given agent_id.
    pub fn roles_containing_agent(&self, agent_id: &str) -> Vec<&str> {
        self.roles
            .iter()
            .filter(|(_id, role)| role.agents.iter().any(|a| a == agent_id))
            .map(|(id, _)| id.as_str())
            .collect()
    }

    /// Get all distinct provider names from agents.
    pub fn provider_names(&self) -> Vec<String> {
        let mut providers: std::collections::HashSet<String> = std::collections::HashSet::new();
        for agent in &self.agents {
            let p = serde_json::to_value(agent.provider)
                .ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| format!("{:?}", agent.provider));
            providers.insert(p);
        }
        let mut list: Vec<String> = providers.into_iter().collect();
        list.sort();
        list
    }

    /// Get all distinct harness names from agents.
    pub fn harness_names(&self) -> Vec<String> {
        let mut harnesses: std::collections::HashSet<String> = std::collections::HashSet::new();
        for agent in &self.agents {
            for h in &agent.harnesses {
                let s = serde_json::to_value(h)
                    .ok()
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                    .unwrap_or_else(|| format!("{h:?}"));
                harnesses.insert(s);
            }
        }
        let mut list: Vec<String> = harnesses.into_iter().collect();
        list.sort();
        list
    }

    /// Get all role IDs sorted.
    pub fn role_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.roles.keys().cloned().collect();
        ids.sort();
        ids
    }

    /// Get all valid platform names for export.
    pub fn platform_names(&self) -> Vec<&'static str> {
        vec![
            "kiro",
            "claude-code",
            "cursor",
            "copilot",
            "codex",
            "gemini",
        ]
    }
}

// ---------------------------------------------------------------------------
// Test helpers (cfg(test) only)
// ---------------------------------------------------------------------------

#[cfg(test)]
impl CatalogStore {
    /// Construct a minimal in-memory store for property tests, without hitting disk.
    pub fn from_parts(
        agents: Vec<Agent>,
        skills: Vec<Skill>,
        roles: HashMap<String, Role>,
        mcp_refs: Vec<McpReference>,
        rules: Vec<Rule>,
    ) -> Self {
        CatalogStore {
            agents,
            skills,
            roles,
            role_catalog_version: String::new(),
            role_catalog_description: String::new(),
            mcp_refs,
            rules,
            integrity: None,
            model_assignments: None,
            model_registry: None,
            workflows: None,
            load_errors: Vec::new(),
            content_hashes: HashMap::new(),
            catalog_root: PathBuf::from("/tmp"),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn workspace_root() -> &'static Path {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let p = Path::new(manifest_dir).parent().unwrap().parent().unwrap();
        Box::leak(Box::new(p.to_path_buf()))
    }

    #[test]
    fn store_loads_successfully() {
        let store = CatalogStore::load(workspace_root());
        assert!(
            store.load_errors.is_empty(),
            "load errors: {:?}",
            store.load_errors
        );
        assert!(store.agent_count() > 0);
        assert!(store.skill_count() > 0);
        assert!(!store.mcp_refs.is_empty());
        assert!(!store.rules.is_empty());
        assert!(!store.roles.is_empty());
        assert!(store.integrity.is_some());
    }

    #[test]
    fn store_provider_count() {
        let store = CatalogStore::load(workspace_root());
        assert!(store.provider_count() > 1);
    }

    #[test]
    fn store_agents_by_provider() {
        let store = CatalogStore::load(workspace_root());
        let aws_agents = store.agents_by_provider("aws");
        assert!(!aws_agents.is_empty());
    }

    #[test]
    fn store_agents_sorted_case_insensitive() {
        let store = CatalogStore::load(workspace_root());
        for window in store.agents.windows(2) {
            assert!(
                window[0].id.to_lowercase() <= window[1].id.to_lowercase(),
                "agents not sorted: {} > {}",
                window[0].id,
                window[1].id
            );
        }
    }

    #[test]
    fn store_skills_sorted_case_insensitive() {
        let store = CatalogStore::load(workspace_root());
        for window in store.skills.windows(2) {
            assert!(
                window[0].id.to_lowercase() <= window[1].id.to_lowercase(),
                "skills not sorted: {} > {}",
                window[0].id,
                window[1].id
            );
        }
    }

    #[test]
    fn store_agents_for_role() {
        let store = CatalogStore::load(workspace_root());
        // Get the first role
        if let Some((role_id, role)) = store.roles.iter().next() {
            if !role.agents.is_empty() {
                let agents = store.agents_for_role(role_id);
                assert!(!agents.is_empty());
            }
        }
    }

    #[test]
    fn store_skills_for_agent() {
        let store = CatalogStore::load(workspace_root());
        // Find an agent with companion skills
        if let Some(agent) = store.agents.iter().find(|a| !a.companion_skills.is_empty()) {
            let skills = store.skills_for_agent(&agent.id);
            assert!(!skills.is_empty());
        }
    }

    #[test]
    fn store_agents_with_skill() {
        let store = CatalogStore::load(workspace_root());
        // Find a skill that some agent references
        if let Some(agent) = store.agents.iter().find(|a| !a.companion_skills.is_empty()) {
            let skill_id = &agent.companion_skills[0];
            let agents = store.agents_with_skill(skill_id);
            assert!(!agents.is_empty());
            assert!(agents.iter().any(|a| a.id == agent.id));
        }
    }

    #[test]
    fn store_agents_with_skill_returns_empty_for_unknown() {
        let store = CatalogStore::load(workspace_root());
        let agents = store.agents_with_skill("nonexistent-skill-id-xyz");
        assert!(agents.is_empty());
    }

    #[test]
    fn store_agents_for_role_by_provider() {
        let store = CatalogStore::load(workspace_root());
        // Find a role with agents from multiple providers
        if let Some((role_id, _role)) = store.roles.iter().find(|(_, r)| r.agents.len() > 2) {
            let grouped = store.agents_for_role_by_provider(role_id);
            assert!(!grouped.is_empty());

            // Verify provider groups are sorted alphabetically
            for window in grouped.windows(2) {
                assert!(
                    window[0].0.to_lowercase() <= window[1].0.to_lowercase(),
                    "provider groups not sorted: {} > {}",
                    window[0].0,
                    window[1].0
                );
            }

            // Verify agents within each group are sorted by ID
            for (_provider, agents) in &grouped {
                for window in agents.windows(2) {
                    assert!(
                        window[0].id.to_lowercase() <= window[1].id.to_lowercase(),
                        "agents in group not sorted: {} > {}",
                        window[0].id,
                        window[1].id
                    );
                }
            }

            // Verify total count matches flat query
            let flat = store.agents_for_role(role_id);
            let grouped_total: usize = grouped.iter().map(|(_, agents)| agents.len()).sum();
            assert_eq!(grouped_total, flat.len());
        } else {
            // If no role has >2 agents, just test with any role
            if let Some((role_id, _)) = store.roles.iter().next() {
                let grouped = store.agents_for_role_by_provider(role_id);
                let flat = store.agents_for_role(role_id);
                let grouped_total: usize = grouped.iter().map(|(_, agents)| agents.len()).sum();
                assert_eq!(grouped_total, flat.len());
            }
        }
    }

    #[test]
    fn store_agents_for_role_by_provider_unknown_role() {
        let store = CatalogStore::load(workspace_root());
        let grouped = store.agents_for_role_by_provider("nonexistent-role-xyz");
        assert!(grouped.is_empty());
    }

    // -----------------------------------------------------------------------
    // v2 query tests
    // -----------------------------------------------------------------------

    #[test]
    fn content_hashes_populated_after_load() {
        let store = CatalogStore::load(workspace_root());
        // At least the files that exist should have been hashed
        assert!(
            !store.content_hashes.is_empty(),
            "content_hashes should be populated"
        );
        // All hashes should be valid 64-char hex strings (SHA-256)
        for (path, hash) in &store.content_hashes {
            assert_eq!(
                hash.len(),
                64,
                "hash for {:?} has wrong length: {}",
                path,
                hash
            );
        }
    }

    #[test]
    fn content_hash_for_returns_known_path() {
        let store = CatalogStore::load(workspace_root());
        let agents_path = workspace_root().join("catalog").join("agents.json");
        let hash = store.content_hash_for(&agents_path);
        assert!(
            hash.is_some(),
            "content_hash_for agents.json should return Some"
        );
        assert_eq!(hash.unwrap().len(), 64);
    }

    #[test]
    fn content_hash_for_unknown_path_returns_none() {
        let store = CatalogStore::load(workspace_root());
        let result = store.content_hash_for(Path::new("/nonexistent/path.json"));
        assert!(result.is_none());
    }

    #[test]
    fn agent_by_id_found() {
        let store = CatalogStore::load(workspace_root());
        if let Some(first) = store.agents.first() {
            let id = first.id.clone();
            let found = store.agent_by_id(&id);
            assert!(found.is_some());
            assert_eq!(found.unwrap().id, id);
        }
    }

    #[test]
    fn agent_by_id_not_found() {
        let store = CatalogStore::load(workspace_root());
        assert!(store.agent_by_id("nonexistent-xyz-agent").is_none());
    }

    #[test]
    fn skill_by_id_found() {
        let store = CatalogStore::load(workspace_root());
        if let Some(first) = store.skills.first() {
            let id = first.id.clone();
            let found = store.skill_by_id(&id);
            assert!(found.is_some());
            assert_eq!(found.unwrap().id, id);
        }
    }

    #[test]
    fn all_asset_ids_covers_all_types() {
        let store = CatalogStore::load(workspace_root());
        let ids = store.all_asset_ids();
        // Should include agents, skills, mcp_refs, rules
        assert!(ids.len() >= store.agent_count() + store.skill_count());
        // Check a known agent ID is present
        if let Some(a) = store.agents.first() {
            assert!(ids.contains(&a.id));
        }
    }

    #[test]
    fn dependency_edges_agent_skill() {
        let store = CatalogStore::load(workspace_root());
        let edges = store.dependency_edges();
        // If any agent has companion_skills, we expect AgentSkill edges
        let has_companion = store.agents.iter().any(|a| !a.companion_skills.is_empty());
        if has_companion {
            let agent_skill_edges: Vec<_> = edges
                .iter()
                .filter(|(_, _, k)| *k == EdgeKind::AgentSkill)
                .collect();
            assert!(!agent_skill_edges.is_empty());
        }
    }

    #[test]
    fn dependency_edges_role_agent() {
        let store = CatalogStore::load(workspace_root());
        let edges = store.dependency_edges();
        let role_agent_edges: Vec<_> = edges
            .iter()
            .filter(|(_, _, k)| *k == EdgeKind::RoleAgent)
            .collect();
        // Roles have agents, so we expect some RoleAgent edges
        assert!(!role_agent_edges.is_empty());
    }

    #[test]
    fn reload_file_unchanged_on_same_content() {
        // Reload agents.json — content hasn't changed so should be Unchanged
        let mut store = CatalogStore::load(workspace_root());
        let agents_path = workspace_root().join("catalog").join("agents.json");
        let outcome = store.reload_file(&agents_path);
        assert!(
            matches!(outcome, ReloadOutcome::Unchanged),
            "Expected Unchanged, got {:?}",
            outcome
        );
    }

    #[test]
    fn reload_file_invalid_json_retains_previous() {
        use std::io::Write;

        let tmp = tempfile::TempDir::new().unwrap();
        let catalog_dir = tmp.path().join("catalog");
        std::fs::create_dir_all(&catalog_dir).unwrap();

        // Write valid agents.json
        let agents_path = catalog_dir.join("agents.json");
        std::fs::write(&agents_path, b"[]").unwrap();

        // Load from the temp directory (empty catalog is fine)
        let mut store = CatalogStore::load(tmp.path());
        // Initially no agents
        assert_eq!(store.agent_count(), 0);

        // Now write invalid JSON
        {
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&agents_path)
                .unwrap();
            f.write_all(b"{ invalid json {{{{").unwrap();
        }

        let outcome = store.reload_file(&agents_path);
        assert!(
            matches!(outcome, ReloadOutcome::RetainedPrevious { .. }),
            "Expected RetainedPrevious, got {:?}",
            outcome
        );
        // Previous valid state (empty vec) should still be there
        assert_eq!(store.agent_count(), 0);
    }

    #[test]
    fn reload_model_assignments_rejects_tainted_control_bytes() {
        use std::io::Write;

        let tmp = tempfile::TempDir::new().unwrap();
        let catalog_dir = tmp.path().join("catalog");
        std::fs::create_dir_all(&catalog_dir).unwrap();

        let assignments_path = catalog_dir.join("model-assignments.json");
        let clean = r#"{
          "manifest_version": 1,
          "generated_by": "scripts/model-policy.mjs",
          "policy_sha256": "abc123",
          "capabilities": {},
          "assignments": []
        }"#;
        std::fs::write(&assignments_path, clean).unwrap();

        // Initial load (catalog::loader::load_model_assignments) accepts the
        // clean file.
        let mut store = CatalogStore::load(tmp.path());
        assert!(store.model_assignments.is_some());
        assert_eq!(
            store.model_assignments.as_ref().unwrap().generated_by,
            "scripts/model-policy.mjs"
        );

        // Overwrite with content carrying a control byte (ESC, 0x1B) — the
        // same class of taint check_model_assignments_tainted rejects on the
        // initial load path. The hot-reload arm must reuse that check rather
        // than adopting the new value unchecked.
        let tainted = r#"{
          "manifest_version": 1,
          "generated_by": "scripts/model-policy.mjs\u001b[31m",
          "policy_sha256": "abc123",
          "capabilities": {},
          "assignments": []
        }"#;
        {
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&assignments_path)
                .unwrap();
            f.write_all(tainted.as_bytes()).unwrap();
        }

        let outcome = store.reload_file(&assignments_path);
        assert!(
            matches!(outcome, ReloadOutcome::RetainedPrevious { .. }),
            "Expected RetainedPrevious for a tainted reload, got {:?}",
            outcome
        );
        // The prior, untainted value must still be in memory — not clobbered
        // by the rejected parse.
        assert_eq!(
            store.model_assignments.as_ref().unwrap().generated_by,
            "scripts/model-policy.mjs"
        );
    }
}

// ---------------------------------------------------------------------------
// Property tests (Task 5.2)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod prop_tests {
    use super::*;
    use crate::models::agent::{AgentType, Lifecycle};
    use crate::models::harness::{Harness, SourceType};
    use crate::models::provider::Provider;
    use crate::models::role::Role;
    use crate::models::skill::{Skill, SkillType};
    use crate::search::fuzzy::SearchEngine;
    use proptest::prelude::*;
    use std::collections::HashMap;
    use std::io::Write;

    // -----------------------------------------------------------------------
    // Helpers for building test data
    // -----------------------------------------------------------------------

    fn make_agent(id: &str, provider: Provider, harness: Harness, skill_ids: Vec<String>) -> Agent {
        Agent {
            id: id.to_string(),
            name: format!("Agent {id}"),
            entity_type: AgentType::Agent,
            provider,
            harnesses: vec![harness],
            summary: format!("Summary for {id}"),
            source_type: SourceType::Original,
            official_docs: vec![],
            security_notes: String::new(),
            last_verified: "2024-01-01".to_string(),
            path: format!("agents/{id}.md"),
            companion_skills: skill_ids,
            execution_tier: None,
            lifecycle: Some(Lifecycle::Stable),
            harness_variants: HashMap::new(),
            author: None,
            version: Some("1.0.0".to_string()),
            provider_coverage: None,
        }
    }

    fn make_skill(id: &str, provider: Provider, harness: Harness) -> Skill {
        Skill {
            id: id.to_string(),
            name: format!("Skill {id}"),
            entity_type: SkillType::Skill,
            provider,
            harnesses: vec![harness],
            summary: format!("Summary for {id}"),
            source_type: SourceType::Original,
            official_docs: vec![],
            security_notes: String::new(),
            last_verified: "2024-01-01".to_string(),
            path: format!("skills/{id}.md"),
            author: None,
            version: Some("1.0.0".to_string()),
            category: None,
            certifications: None,
            companion_agents: None,
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

    fn make_role(agent_ids: Vec<String>) -> Role {
        Role {
            label: "Test Role".to_string(),
            description: "Test".to_string(),
            agents: agent_ids,
            skills: vec![],
        }
    }

    /// Arbitrary safe identifiers: lowercase alphanumeric + hyphens, non-empty.
    fn arb_id() -> impl Strategy<Value = String> {
        "[a-z][a-z0-9-]{1,15}".prop_map(|s| s)
    }

    fn arb_provider() -> impl Strategy<Value = Provider> {
        prop_oneof![
            Just(Provider::Aws),
            Just(Provider::Azure),
            Just(Provider::Gcp),
            Just(Provider::Claude),
            Just(Provider::Kubernetes),
            Just(Provider::Php),
        ]
    }

    fn arb_harness() -> impl Strategy<Value = Harness> {
        prop_oneof![
            Just(Harness::ClaudeCode),
            Just(Harness::Cursor),
            Just(Harness::Kiro),
            Just(Harness::Copilot),
        ]
    }

    // -----------------------------------------------------------------------
    // Property 1 (Req 1.3): reload_file on invalid JSON never panics and
    // retains previous valid state.
    // -----------------------------------------------------------------------
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        #[test]
        fn prop1_reload_invalid_json_retains_previous(
            agent_id in arb_id(),
            garbage in ".{0,200}",  // arbitrary bytes (invalidity forced by control-byte prefix below)
        ) {
            let tmp = tempfile::TempDir::new().unwrap();
            let catalog_dir = tmp.path().join("catalog");
            std::fs::create_dir_all(&catalog_dir).unwrap();

            // Write a valid agents.json with one agent
            let valid_agents = serde_json::json!([{
                "id": agent_id,
                "name": format!("Agent {agent_id}"),
                "type": "agent",
                "provider": "aws",
                "harnesses": ["claude-code"],
                "summary": "test",
                "source_type": "original",
                "official_docs": [],
                "security_notes": "",
                "last_verified": "2024-01-01",
                "path": "agents/test.md",
                "companion_skills": [],
                "harness_variants": {}
            }]);
            let agents_path = catalog_dir.join("agents.json");
            std::fs::write(&agents_path, valid_agents.to_string().as_bytes()).unwrap();

            // Also create other required files as empty arrays/objects
            std::fs::write(catalog_dir.join("skills.json"), b"[]").unwrap();
            std::fs::write(catalog_dir.join("mcp-references.json"), b"[]").unwrap();
            std::fs::write(catalog_dir.join("rules.json"), b"[]").unwrap();

            let mut store = CatalogStore::load(tmp.path());
            let agent_count_before = store.agent_count();

            // Overwrite with garbage (definitely invalid JSON)
            {
                let mut f = std::fs::OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .open(&agents_path)
                    .unwrap();
                // Force it to be invalid by prefixing with control chars
                f.write_all(b"\x00\x01\x02").unwrap();
                f.write_all(garbage.as_bytes()).unwrap();
            }

            // This must not panic
            let outcome = store.reload_file(&agents_path);

            // Outcome should be RetainedPrevious (bytes changed, but invalid JSON)
            match outcome {
                ReloadOutcome::RetainedPrevious { .. } => {
                    // Good — verify state is unchanged
                    prop_assert_eq!(store.agent_count(), agent_count_before,
                        "store should retain previous agents after invalid reload");
                }
                ReloadOutcome::Unchanged => {
                    // Content hash matched — also acceptable (hash collision with control bytes prefix is impossible in practice)
                    prop_assert_eq!(store.agent_count(), agent_count_before);
                }
                ReloadOutcome::Reloaded { .. } => {
                    // This would be a bug: should not have "succeeded" on garbage
                    prop_assert!(false, "reload should not succeed on garbage input");
                }
            }
        }

        // -----------------------------------------------------------------------
        // Property 2 (Req 16.2, 32.2): fuzzy search returns only matching items
        // — for any query that IS a substring of some item's id or name, that
        //   item appears in results.
        // -----------------------------------------------------------------------
        #[test]
        fn prop2_fuzzy_search_substring_hit(
            base_id in arb_id(),
            provider in arb_provider(),
            harness in arb_harness(),
        ) {
            // Build a store with one agent whose id contains `base_id`
            let agent = make_agent(&base_id, provider, harness, vec![]);
            let store = CatalogStore::from_parts(
                vec![agent],
                vec![],
                HashMap::new(),
                vec![],
                vec![],
            );

            let mut engine = SearchEngine::new();

            // Query using the exact id — must appear in results
            let results = engine.search_agents(&base_id, &store.agents, None, None);
            prop_assert!(
                results.contains(&0),
                "agent with id '{}' not found when querying '{}'",
                base_id, base_id
            );
        }

        // -----------------------------------------------------------------------
        // Property 3 (Req 32.3): combined filter (provider AND harness) returns
        // the correct intersection.
        // -----------------------------------------------------------------------
        #[test]
        fn prop3_combined_filter_intersection(
            id1 in "[a-z][a-z0-9]{3,8}",
            id2 in "[a-z][a-z0-9]{3,8}",
        ) {
            // Make two agents with different providers / harnesses
            let agent_aws_claude = make_agent(&id1, Provider::Aws, Harness::ClaudeCode, vec![]);
            let agent_gcp_cursor = make_agent(&id2, Provider::Gcp, Harness::Cursor, vec![]);

            let store = CatalogStore::from_parts(
                vec![agent_aws_claude, agent_gcp_cursor],
                vec![],
                HashMap::new(),
                vec![],
                vec![],
            );

            let mut engine = SearchEngine::new();

            // Filter: provider=aws AND harness=claude-code → only agent 0
            let results = engine.search_agents("", &store.agents, Some("aws"), Some("claude-code"));
            // Should contain agent at index 0 (aws/claude-code)
            prop_assert!(results.contains(&0), "aws/claude-code agent should be in results");
            // Should NOT contain agent at index 1 (gcp/cursor)
            prop_assert!(!results.contains(&1), "gcp/cursor agent should not be in aws/claude-code results");

            // Filter: provider=gcp AND harness=cursor → only agent 1
            let results2 = engine.search_agents("", &store.agents, Some("gcp"), Some("cursor"));
            prop_assert!(results2.contains(&1), "gcp/cursor agent should be in results");
            prop_assert!(!results2.contains(&0), "aws/claude-code agent should not be in gcp/cursor results");

            // Filter: provider=aws AND harness=cursor → empty (no agent matches both)
            let results3 = engine.search_agents("", &store.agents, Some("aws"), Some("cursor"));
            prop_assert!(results3.is_empty(), "no agent matches aws+cursor");
        }

        // -----------------------------------------------------------------------
        // Property 5 (Req 5.2, 5.3, 32.5): reverse-lookup consistency
        //   a) agents_with_skill(S) contains agent A when A.companion_skills ∋ S
        //   b) roles_containing_agent is consistent with role membership
        // -----------------------------------------------------------------------
        #[test]
        fn prop5_reverse_lookup_consistency(
            agent_id in arb_id(),
            skill_id in arb_id(),
            role_id in arb_id(),
            provider in arb_provider(),
            harness in arb_harness(),
        ) {
            // Build agent with companion skill
            let agent = make_agent(&agent_id, provider, harness, vec![skill_id.clone()]);
            let skill = make_skill(&skill_id, provider, harness);

            // Build role containing the agent
            let mut roles = HashMap::new();
            roles.insert(role_id.clone(), make_role(vec![agent_id.clone()]));

            let store = CatalogStore::from_parts(
                vec![agent],
                vec![skill],
                roles,
                vec![],
                vec![],
            );

            // (a) agents_with_skill reverse lookup
            let agents_with = store.agents_with_skill(&skill_id);
            prop_assert!(
                agents_with.iter().any(|a| a.id == agent_id),
                "agents_with_skill('{}') should contain agent '{}'",
                skill_id, agent_id
            );

            // (b) roles_containing_agent reverse lookup
            let roles_with = store.roles_containing_agent(&agent_id);
            prop_assert!(
                roles_with.contains(&role_id.as_str()),
                "roles_containing_agent('{}') should contain role '{}'",
                agent_id, role_id
            );

            // (c) consistency: agents_for_role(role_id) should include the agent
            let agents_for = store.agents_for_role(&role_id);
            prop_assert!(
                agents_for.iter().any(|a| a.id == agent_id),
                "agents_for_role('{}') should contain agent '{}'",
                role_id, agent_id
            );

            // (d) skills_for_agent should include the skill
            let skills_for = store.skills_for_agent(&agent_id);
            prop_assert!(
                skills_for.iter().any(|s| s.id == skill_id),
                "skills_for_agent('{}') should contain skill '{}'",
                agent_id, skill_id
            );
        }
    }
}
