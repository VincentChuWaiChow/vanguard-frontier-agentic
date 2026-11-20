//! Presentation-free search coordination with observation-bound publication.
//! Requests own immutable observations; results never grant execution authority.
use std::path::Path;
use std::sync::Arc;

use super::session::{SessionError, SourceSession};
use super::snapshot::{CatalogSnapshot, CollectionKind};
use crate::search::fuzzy::SearchEngine;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityId {
    pub collection: CollectionKind,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchQuery {
    pub collection: CollectionKind,
    pub text: String,
    pub provider: Option<String>,
    pub harness: Option<String>,
}

pub struct SearchRequest {
    snapshot: Arc<CatalogSnapshot>,
    sequence: u64,
    query: SearchQuery,
}

pub struct SearchResponse {
    snapshot: Arc<CatalogSnapshot>,
    sequence: u64,
    query: SearchQuery,
    entities: Vec<EntityId>,
}

impl SearchResponse {
    pub fn entities(&self) -> &[EntityId] {
        &self.entities
    }
    pub fn query(&self) -> &SearchQuery {
        &self.query
    }
    pub fn revision(&self) -> u64 {
        self.snapshot.revision()
    }
    pub fn sequence(&self) -> u64 {
        self.sequence
    }
}

impl SearchRequest {
    /// Intended for a bounded background worker, never a render callback.
    pub fn execute(self) -> SearchResponse {
        let catalog = self.snapshot.catalog();
        let q = &self.query;
        let mut search = SearchEngine::new();
        let ids: Vec<String> = match q.collection {
            CollectionKind::Agents => search
                .search_agents(
                    &q.text,
                    &catalog.agents,
                    q.provider.as_deref(),
                    q.harness.as_deref(),
                )
                .into_iter()
                .map(|i| catalog.agents[i].id.clone())
                .collect(),
            CollectionKind::Skills => search
                .search_skills(&q.text, &catalog.skills)
                .into_iter()
                .filter(|&i| {
                    let s = &catalog.skills[i];
                    filters(
                        q,
                        &s.provider.to_string(),
                        s.harnesses.iter().map(ToString::to_string),
                    )
                })
                .map(|i| catalog.skills[i].id.clone())
                .collect(),
            _ => other_ids(&self.snapshot, q),
        };
        let entities = ids
            .into_iter()
            .map(|id| EntityId {
                collection: q.collection,
                id,
            })
            .collect();
        SearchResponse {
            snapshot: self.snapshot,
            sequence: self.sequence,
            query: self.query,
            entities,
        }
    }
}

fn filters(q: &SearchQuery, provider: &str, harnesses: impl Iterator<Item = String>) -> bool {
    q.provider
        .as_ref()
        .map_or(true, |p| p.eq_ignore_ascii_case(provider))
        && q.harness.as_ref().map_or(true, |h| {
            harnesses.into_iter().any(|v| h.eq_ignore_ascii_case(&v))
        })
}

/// Non-ranked ID/name/summary substring matching; sort by lowercase ID then exact
/// ID. Roles/integrity have no provider/harness metadata: active filters exclude
/// them rather than inventing metadata. Unicode text is preserved unchanged.
fn other_ids(snapshot: &CatalogSnapshot, q: &SearchQuery) -> Vec<String> {
    let c = snapshot.catalog();
    let text = q.text.to_lowercase();
    let mut rows: Vec<(String, String)> = Vec::new();
    match q.collection {
        CollectionKind::Roles if q.provider.is_none() && q.harness.is_none() => {
            rows.extend(
                c.roles
                    .iter()
                    .map(|(id, r)| (id.clone(), format!("{id} {} {}", r.label, r.description))),
            );
        }
        CollectionKind::McpReferences => {
            rows.extend(
                c.mcp_refs
                    .iter()
                    .filter(|r| {
                        filters(
                            q,
                            &r.provider.to_string(),
                            r.harnesses.iter().map(ToString::to_string),
                        )
                    })
                    .map(|r| (r.id.clone(), format!("{} {} {}", r.id, r.name, r.summary))),
            );
        }
        CollectionKind::Rules => {
            rows.extend(
                c.rules
                    .iter()
                    .filter(|r| {
                        filters(
                            q,
                            &r.provider.to_string(),
                            r.harnesses.iter().map(ToString::to_string),
                        )
                    })
                    .map(|r| (r.id.clone(), format!("{} {} {}", r.id, r.name, r.summary))),
            );
        }
        CollectionKind::Integrity if q.provider.is_none() && q.harness.is_none() => {
            if let Some(integrity) = &c.integrity {
                rows.extend(
                    integrity
                        .root_files
                        .iter()
                        .chain(integrity.trees.iter().flat_map(|t| &t.files))
                        .map(|f| {
                            let name = Path::new(&f.path)
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("");
                            (f.path.clone(), format!("{} {name}", f.path))
                        }),
                );
            }
        }
        _ => {}
    }
    rows.retain(|(_, haystack)| haystack.to_lowercase().contains(&text));
    rows.sort_by(|a, b| {
        a.0.to_lowercase()
            .cmp(&b.0.to_lowercase())
            .then(a.0.cmp(&b.0))
    });
    rows.dedup_by(|a, b| a.0 == b.0);
    rows.into_iter().map(|(id, _)| id).collect()
}

#[derive(Default)]
pub struct CatalogController {
    sequence: u64,
    pending: Option<(Arc<CatalogSnapshot>, u64, SearchQuery)>,
    visible: Option<SearchResponse>,
    selection: Option<(Arc<CatalogSnapshot>, EntityId)>,
}
impl CatalogController {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn request(
        &mut self,
        session: &SourceSession,
        query: SearchQuery,
    ) -> Result<SearchRequest, SessionError> {
        self.synchronize(session);
        let snapshot = session.snapshot().ok_or(SessionError::NoSource)?;
        self.sequence = self
            .sequence
            .checked_add(1)
            .ok_or(SessionError::EpochExhausted)?;
        self.pending = Some((Arc::clone(&snapshot), self.sequence, query.clone()));
        Ok(SearchRequest {
            snapshot,
            sequence: self.sequence,
            query,
        })
    }
    /// Reject foreign source/session, old revision, old observation and reordered
    /// request results without changing published data. Duplicate delivery rejects.
    pub fn accept(&mut self, session: &SourceSession, result: SearchResponse) -> bool {
        self.synchronize(session);
        let Some(current) = session.snapshot() else {
            return false;
        };
        let Some((snapshot, sequence, query)) = &self.pending else {
            return false;
        };
        if !Arc::ptr_eq(&current, &result.snapshot)
            || !Arc::ptr_eq(snapshot, &result.snapshot)
            || *sequence != result.sequence
            || *query != result.query
        {
            return false;
        }
        if self
            .selection
            .as_ref()
            .is_some_and(|(_, id)| !result.entities.contains(id))
        {
            self.selection = None;
        }
        self.pending = None;
        self.visible = Some(result);
        true
    }
    /// Old views cannot leak through when a caller refreshes/switches the session
    /// without explicitly synchronizing this controller.
    pub fn visible(&self, session: &SourceSession) -> Option<&SearchResponse> {
        self.visible.as_ref().filter(|r| {
            session
                .snapshot()
                .is_some_and(|s| Arc::ptr_eq(&s, &r.snapshot))
        })
    }
    pub fn selection(&self, session: &SourceSession) -> Option<&EntityId> {
        self.selection
            .as_ref()
            .filter(|(old, _)| session.snapshot().is_some_and(|s| Arc::ptr_eq(&s, old)))
            .map(|(_, id)| id)
    }
    pub fn select(&mut self, session: &SourceSession, id: EntityId) -> bool {
        let Some(current) = session.snapshot() else {
            return false;
        };
        if !self
            .visible
            .as_ref()
            .is_some_and(|r| Arc::ptr_eq(&r.snapshot, &current) && r.entities.contains(&id))
        {
            return false;
        }
        self.selection = Some((current, id));
        true
    }
    /// Fresh selection is necessary but never sufficient for future confirmation.
    pub fn selection_is_current(&self, session: &SourceSession) -> bool {
        self.selection.as_ref().is_some_and(|(s, _)| {
            s.is_current_data()
                && session
                    .snapshot()
                    .is_some_and(|current| Arc::ptr_eq(s, &current))
        })
    }
    /// Explicit serialized adapter boundary after refresh/source change. Stable
    /// selection survives successful reload only if its ID still exists.
    pub fn synchronize(&mut self, session: &SourceSession) {
        let current = session.snapshot();
        if self.visible.as_ref().is_some_and(|r| {
            current
                .as_ref()
                .map_or(true, |s| !Arc::ptr_eq(s, &r.snapshot))
        }) {
            self.visible = None;
        }
        if let Some((old, id)) = &self.selection {
            let retained = current
                .as_ref()
                .filter(|s| s.context() == old.context())
                .filter(|s| {
                    let q = SearchQuery {
                        collection: id.collection,
                        text: String::new(),
                        provider: None,
                        harness: None,
                    };
                    SearchRequest {
                        snapshot: Arc::clone(s),
                        sequence: 0,
                        query: q,
                    }
                    .execute()
                    .entities
                    .contains(id)
                });
            self.selection = retained.map(|s| (Arc::clone(s), id.clone()));
        }
        if self
            .pending
            .as_ref()
            .is_some_and(|(old, _, _)| current.as_ref().map_or(true, |s| !Arc::ptr_eq(s, old)))
        {
            self.pending = None;
        }
    }
}
