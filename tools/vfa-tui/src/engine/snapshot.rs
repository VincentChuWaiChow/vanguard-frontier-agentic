//! Immutable owned catalog observations with independent collection quality.
//!
//! Hashes describe the exact captured bytes passed to existing shared parsers.
//! This is not a simultaneous multi-file filesystem snapshot, a trust grant, or
//! a claim of freshness after subsequent edits. Native watcher wiring is later.

use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::path::Path;

use serde::de::{Error, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::Value;

use super::session::{read_regular_file, ContextTag, SessionError};
use crate::catalog::{loader, store::CatalogStore};
use crate::error::TuiError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CollectionKind {
    Agents,
    Skills,
    Roles,
    McpReferences,
    Rules,
    Integrity,
}

impl CollectionKind {
    fn filename(self) -> &'static str {
        match self {
            Self::Agents => "agents.json",
            Self::Skills => "skills.json",
            Self::Roles => "install-roles.json",
            Self::McpReferences => "mcp-references.json",
            Self::Rules => "rules.json",
            Self::Integrity => "asset-integrity.json",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionStatus {
    Current,
    Empty,
    Partial,
    Unavailable,
    /// Retained last-good data after a failed refresh.
    Stale,
}

#[derive(Debug)]
pub struct CollectionState {
    pub status: CollectionStatus,
    pub count: usize,
    /// Internal diagnostic strings; UI/audit adapters must sanitize/redact them.
    pub issues: Vec<String>,
}

pub struct CatalogSnapshot {
    context: ContextTag,
    revision: u64,
    observed_at: std::time::SystemTime,
    catalog: CatalogStore,
    collections: BTreeMap<CollectionKind, CollectionState>,
}

impl fmt::Debug for CatalogSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CatalogSnapshot")
            .field("context", &self.context)
            .field("collections", &self.collections)
            .finish_non_exhaustive()
    }
}

impl CatalogSnapshot {
    pub(crate) fn load(context: ContextTag) -> Result<Self, SessionError> {
        let root = context.source().root();
        context.source().verify_current()?;
        let catalog = CatalogStore::load_with_reader(root, |path| {
            let bytes = read_regular_file(path, loader::MAX_CATALOG_FILE_SIZE as usize)
                .map_err(|error| parse_error(path, error.to_string()))?;
            validate_input(path, &bytes)?;
            Ok(bytes)
        });
        context.source().verify_current()?;
        let mut collections = BTreeMap::new();
        for (kind, count) in [
            (CollectionKind::Agents, catalog.agents.len()),
            (CollectionKind::Skills, catalog.skills.len()),
            (CollectionKind::Roles, catalog.roles.len()),
            (CollectionKind::McpReferences, catalog.mcp_refs.len()),
            (CollectionKind::Rules, catalog.rules.len()),
            (
                CollectionKind::Integrity,
                usize::from(catalog.integrity.is_some()),
            ),
        ] {
            let path = root
                .join("catalog")
                .join(kind.filename())
                .display()
                .to_string();
            let issues: Vec<_> = catalog
                .load_errors
                .iter()
                .filter(|error| match error {
                    TuiError::CatalogNotFound { path: p }
                    | TuiError::CatalogParse { path: p, .. }
                    | TuiError::TaintedEntry { path: p, .. } => *p == path,
                    _ => false,
                })
                .map(ToString::to_string)
                .collect();
            let status = match (issues.is_empty(), count == 0) {
                (true, true) => CollectionStatus::Empty,
                (true, false) => CollectionStatus::Current,
                (false, true) => CollectionStatus::Unavailable,
                (false, false) => CollectionStatus::Partial,
            };
            collections.insert(
                kind,
                CollectionState {
                    status,
                    count,
                    issues,
                },
            );
        }
        Ok(Self {
            context,
            revision: 1,
            observed_at: std::time::SystemTime::now(),
            catalog,
            collections,
        })
    }

    /// Reload all observations once through the existing bounded reader/shared
    /// parsers. Failed OR partial collections retain their prior accepted data
    /// and hashes independently; diagnostics describe the failed new observation.
    /// Other successfully loaded collections can advance the content revision.
    pub(crate) fn refresh(previous: &Self) -> Result<Self, SessionError> {
        let mut candidate = Self::load(previous.context.clone())?;
        for kind in [
            CollectionKind::Agents,
            CollectionKind::Skills,
            CollectionKind::Roles,
            CollectionKind::McpReferences,
            CollectionKind::Rules,
            CollectionKind::Integrity,
        ] {
            if matches!(
                candidate.collection(kind).status,
                CollectionStatus::Partial | CollectionStatus::Unavailable
            ) {
                let old = previous.collection(kind);
                // Do not promote an originally unavailable collection to stale
                // last-good data: there was no successful observation to retain.
                if matches!(
                    old.status,
                    CollectionStatus::Current
                        | CollectionStatus::Empty
                        | CollectionStatus::Stale
                        | CollectionStatus::Partial
                ) {
                    match kind {
                        CollectionKind::Agents => {
                            candidate.catalog.agents = previous.catalog.agents.clone()
                        }
                        CollectionKind::Skills => {
                            candidate.catalog.skills = previous.catalog.skills.clone()
                        }
                        CollectionKind::Roles => {
                            candidate.catalog.roles = previous.catalog.roles.clone();
                            candidate.catalog.role_catalog_version =
                                previous.catalog.role_catalog_version.clone();
                            candidate.catalog.role_catalog_description =
                                previous.catalog.role_catalog_description.clone();
                        }
                        CollectionKind::McpReferences => {
                            candidate.catalog.mcp_refs = previous.catalog.mcp_refs.clone()
                        }
                        CollectionKind::Rules => {
                            candidate.catalog.rules = previous.catalog.rules.clone()
                        }
                        CollectionKind::Integrity => {
                            candidate.catalog.integrity = previous.catalog.integrity.clone()
                        }
                    }
                    let state = candidate
                        .collections
                        .get_mut(&kind)
                        .ok_or(SessionError::NoSource)?;
                    state.status = CollectionStatus::Stale;
                    state.count = old.count;
                    let path = previous
                        .context
                        .source()
                        .root()
                        .join("catalog")
                        .join(kind.filename());
                    candidate.catalog.content_hashes.remove(&path);
                    if let Some(hash) = previous.catalog.content_hashes.get(&path) {
                        candidate.catalog.content_hashes.insert(path, hash.clone());
                    }
                } else {
                    // No accepted data existed. Keep it unavailable, do not
                    // publish partial new records or failing-byte provenance.
                    let state = candidate
                        .collections
                        .get_mut(&kind)
                        .ok_or(SessionError::NoSource)?;
                    state.status = CollectionStatus::Unavailable;
                    state.count = 0;
                    match kind {
                        CollectionKind::Agents => candidate.catalog.agents.clear(),
                        CollectionKind::Skills => candidate.catalog.skills.clear(),
                        CollectionKind::Roles => {
                            candidate.catalog.roles.clear();
                            candidate.catalog.role_catalog_version.clear();
                            candidate.catalog.role_catalog_description.clear();
                        }
                        CollectionKind::McpReferences => candidate.catalog.mcp_refs.clear(),
                        CollectionKind::Rules => candidate.catalog.rules.clear(),
                        CollectionKind::Integrity => candidate.catalog.integrity = None,
                    }
                    let path = previous
                        .context
                        .source()
                        .root()
                        .join("catalog")
                        .join(kind.filename());
                    candidate.catalog.content_hashes.remove(&path);
                    if let Some(hash) = previous.catalog.content_hashes.get(&path) {
                        candidate.catalog.content_hashes.insert(path, hash.clone());
                    }
                }
            }
        }
        candidate.revision = if candidate.catalog.content_hashes == previous.catalog.content_hashes
        {
            previous.revision
        } else {
            previous
                .revision
                .checked_add(1)
                .ok_or(SessionError::EpochExhausted)?
        };
        Ok(candidate)
    }

    pub(crate) fn source_failed(previous: &Self, error: &SessionError) -> Self {
        let c = &previous.catalog;
        let catalog = CatalogStore {
            agents: c.agents.clone(),
            skills: c.skills.clone(),
            roles: c.roles.clone(),
            role_catalog_version: c.role_catalog_version.clone(),
            role_catalog_description: c.role_catalog_description.clone(),
            mcp_refs: c.mcp_refs.clone(),
            rules: c.rules.clone(),
            integrity: c.integrity.clone(),
            load_errors: vec![parse_error(
                previous.context.source().root(),
                error.to_string(),
            )],
            content_hashes: c.content_hashes.clone(),
            catalog_root: c.catalog_root.clone(),
        };
        let collections = previous
            .collections
            .iter()
            .map(|(kind, state)| {
                let issues = vec![error.to_string()];
                (
                    *kind,
                    CollectionState {
                        status: if state.status == CollectionStatus::Unavailable {
                            CollectionStatus::Unavailable
                        } else {
                            CollectionStatus::Stale
                        },
                        count: state.count,
                        issues,
                    },
                )
            })
            .collect();
        Self {
            context: previous.context.clone(),
            revision: previous.revision,
            observed_at: std::time::SystemTime::now(),
            catalog,
            collections,
        }
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Time of this load/refresh attempt, NOT the original acquisition time of
    /// retained stale collections. Their retained content hashes are provenance.
    pub fn observed_at(&self) -> std::time::SystemTime {
        self.observed_at
    }

    /// Conservative freshness guard, not an execution/trust permit.
    pub fn is_current_data(&self) -> bool {
        self.collections.values().all(|s| {
            matches!(
                s.status,
                CollectionStatus::Current | CollectionStatus::Empty
            )
        })
    }

    pub fn context(&self) -> &ContextTag {
        &self.context
    }

    /// Shared domain values cannot be mutated through a published snapshot.
    ///
    /// ```compile_fail
    /// fn mutate(s: &vfa_tui::engine::snapshot::CatalogSnapshot) {
    ///     s.catalog().agents.clear();
    /// }
    /// ```
    pub fn catalog(&self) -> &CatalogStore {
        &self.catalog
    }

    pub fn collection(&self, kind: CollectionKind) -> &CollectionState {
        // Every enum variant is inserted during construction; avoid indexing a
        // caller-provided string or exposing the map for external mutation.
        &self.collections[&kind]
    }
}

fn parse_error(path: &Path, detail: String) -> TuiError {
    TuiError::CatalogParse {
        path: path.display().to_string(),
        offset: 0,
        detail,
    }
}

fn validate_input(path: &Path, bytes: &[u8]) -> Result<(), TuiError> {
    let UniqueValue(value) =
        serde_json::from_slice(bytes).map_err(|error| parse_error(path, error.to_string()))?;
    if matches!(
        path.file_name().and_then(|n| n.to_str()),
        Some("agents.json" | "skills.json" | "mcp-references.json" | "rules.json")
    ) {
        let mut ids = HashSet::new();
        if let Some(rows) = value.as_array() {
            for row in rows {
                if let Some(id) = row.get("id").and_then(Value::as_str) {
                    if !ids.insert(id) {
                        return Err(parse_error(path, "duplicate entity ID".into()));
                    }
                }
            }
        }
    }
    Ok(())
}

// Ordinary Value/HashMap deserialization silently overwrites map keys. Retain
// decoded-key uniqueness during this preflight, before using the shared model
// parsers on the ORIGINAL captured bytes. No schema/rendering logic is copied.
struct UniqueValue(Value);

impl<'de> Deserialize<'de> for UniqueValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct UniqueVisitor;
        impl<'de> Visitor<'de> for UniqueVisitor {
            type Value = UniqueValue;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("JSON with unique object keys")
            }
            fn visit_bool<E: Error>(self, v: bool) -> Result<Self::Value, E> {
                Ok(UniqueValue(Value::Bool(v)))
            }
            fn visit_i64<E: Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(UniqueValue(Value::Number(v.into())))
            }
            fn visit_u64<E: Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(UniqueValue(Value::Number(v.into())))
            }
            fn visit_f64<E: Error>(self, v: f64) -> Result<Self::Value, E> {
                serde_json::Number::from_f64(v)
                    .map(|n| UniqueValue(Value::Number(n)))
                    .ok_or_else(|| E::custom("invalid JSON number"))
            }
            fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(UniqueValue(Value::String(v.into())))
            }
            fn visit_unit<E: Error>(self) -> Result<Self::Value, E> {
                Ok(UniqueValue(Value::Null))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut values = Vec::new();
                while let Some(UniqueValue(value)) = seq.next_element()? {
                    values.push(value);
                }
                Ok(UniqueValue(Value::Array(values)))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut values = serde_json::Map::new();
                while let Some(key) = map.next_key::<String>()? {
                    if values.contains_key(&key) {
                        return Err(A::Error::custom("duplicate JSON key"));
                    }
                    let UniqueValue(value) = map.next_value()?;
                    values.insert(key, value);
                }
                Ok(UniqueValue(Value::Object(values)))
            }
        }
        deserializer.deserialize_any(UniqueVisitor)
    }
}
