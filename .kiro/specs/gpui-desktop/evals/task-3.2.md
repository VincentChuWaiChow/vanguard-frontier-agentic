# Task 3.2: no-install catalog search and freshness preparation

## Admission and pre-implementation contract

The user's three-task approval admits an engine-only slice using current
task 3.1 snapshot/session preparation. Full task 3.1/native prerequisites stay
pending; full task 3.2 is not dependency-complete.

Owned code: `tools/vfa-tui/src/engine/catalog.rs`, narrow additive changes to
`session.rs`/`snapshot.rs` beside it, `tools/vfa-tui/src/search/`, and
`tools/vfa-tui/tests/catalog_session.rs`. Parent owns module exports. Preserve
existing loader/store/watcher APIs and behavior: do not silently connect the
legacy lossy watcher or its retry lifecycle. Engine refresh takes a common path
for manual and externally supplied change notifications; native wiring is later.

Before implementation all following criteria are **UNVERIFIED**:

1. Shared existing agent/skill ranking/filter intersection remains exact;
   deterministic other-entity matching, Unicode, empty and no-match queries work.
2. A result binds current source context, content revision and query sequence.
   Reordered query, old-source and old-revision results cannot replace current data.
3. Manual and change-notification refresh share snapshot parsing/provenance.
   Unchanged bytes preserve revision; successful changes advance it; failed or
   deleted inputs retain last-good data marked stale, never current-empty.
4. Removed entities clear stable selection; current selection cannot authorize
   stale work. Existing source-switch guards and snapshots remain valid.

Graders: `cargo test --locked --offline --test catalog_session`, existing
`--test session_contracts`, complete suites in both feature modes, fmt and strict
all-target clippy. Parent owns integrated failures. Missing-API compile RED and
behavioral RED must be distinguished. No installs, dependencies, UI/native/Git,
frozen input edits or broad store/watcher refactor.

Results and exact evidence will be appended after verification.

## Implementation and capability results

**PASS (scoped):** `CatalogController` reuses shared agent/skill ranking and
provides deterministic other-collection matching and typed stable selection.
Requests own immutable observations; acceptance and getters check the actual
current session observation, sequence/query and source/revision identity. Old
results reject even when the adapter omitted an explicit synchronize call.

`SourceSession::refresh` routes manual/external-change triggers through the
same bounded snapshot reader. Unchanged bytes keep content revision, while a
new observation invalidates prior requests. Successful changes advance revision.
Failed collections independently retain prior data and corresponding hashes as
STALE. Fatal marker/root failures publish a stale observation before returning
the error, never silently rebind the source. Removed/filtered selection clears.
Attempt time is not represented as the original acquisition time of stale data;
retained hashes remain its provenance. Selection freshness is not an execution
permit. Existing watcher/retry tasks and native controls are not wired.

TDD: `3.2/red-missing-api.log` is compile-only evidence; actual runtime
`red-runtime.log` rejects the no-op refresh stub (revision 1 instead of 2).
Targeted `green-default.log` / `green-no-default.log` each pass 10 new catalog
and 20 existing session contracts, including partial/deleted refresh, captured
hash retention, externally changed context, root replacement, fatal refresh and
selection invalidation. Parent independently reran the complete suites.

## Integrated checks and handoff

See the [parent integrated result](../../../../.workflow/ultracode/gpui-desktop/final-report.md) for exact complete-suite, strict lint,
feature-boundary, package, repository and scope commands/status. Parent rerun:
1,168 default / 947 engine tests pass, zero failures, one existing ignored
doctest each; both clippy modes pass. Targeted capability commands:

```sh
cargo test --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --test catalog_session
cargo test --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --no-default-features --test catalog_session
```

No native/GUI/MSRV/coverage or allocator instrumentation acceptance is inferred.
Full task stays unchecked. Task-local RED/GREEN logs are under ignored
`tools/vfa-tui/target/no-install-three/3.2/`.
