//! Immutable owned catalog observations with independent collection quality.
//!
//! Hashes describe the exact captured bytes passed to existing shared parsers.
//! This is not a simultaneous multi-file filesystem snapshot, a trust grant, or
//! a claim of freshness after subsequent edits. Refresh/watcher wiring is later.

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
            catalog,
            collections,
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
