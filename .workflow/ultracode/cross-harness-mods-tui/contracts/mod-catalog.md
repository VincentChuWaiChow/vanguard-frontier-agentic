# Downstream contract: mod catalog

Producer: parent models/loader/store/generator.
Consumers: UI worker and existing catalog tests/fixtures.
Surface: `crate::models::{Mod, ModFootprint}`, `CatalogStore.mods` and queries.
Location: `tools/vfa-tui/src/models/mod_catalog.rs`, catalog loader/store.

## Guarantees

`Mod` fields: id/name/version/description/path strings, reach_level u8 (validated
0..3), events/harnesses Vec<String>, security_notes Option<String>, footprint
Option<ModFootprint>. Footprint has env_reads/file_writes/spawns/egress_hosts,
all Vec<String>. Types derive Clone, Debug, Serialize, Deserialize, PartialEq, Eq.
No type/provider discriminator. Missing/null footprint is unknown, not empty.

Store: public `mods: Vec<Mod>`, `mod_count()`, exact `mod_by_id(&str)`; sorted by
lowercase ID then original ID. Startup absence is optional; invalid startup
does not partially adopt. Reload failure/deletion retains data/hash. Success
and hash correspond to one validated bounded read. Empty array deliberately clears.

## Compatibility checks

Worker app literal gains `mods: Vec::new()`. Parent owns other literals.
Primary Tab::CatalogBrowser must actually route appended Mods sidebar and new
View::ModList/ModDetail(String) variants. Rendering never reads manifests/hooks.

## Allowed changes

Approved spec's additive fields, views and generated footprint only.

## Forbidden changes

No mutation actions, provider filters hiding mods, fabricated empty legacy
footprints, independent manifest semantics or runtime-enforcement claims.
