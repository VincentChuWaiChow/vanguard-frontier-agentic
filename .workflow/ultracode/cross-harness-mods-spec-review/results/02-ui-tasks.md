# Result 02-ui-tasks: UI and task review

## Summary

R18 promises display/footprint while T9 only loads metadata. Read-only Codex
sidecar `spec_ui_tasks_review` completed.

## Evidence

Baseline `requirements.md:338` promises display; `design.md:1201-1203` excludes
it. `ui/nav.rs:101-144` and `app.rs:2023-2048` have no mods view. Generator
`generate-mods.mjs:612-620` has no footprint. Exhaustive store literals also
exist at `src/test_fixtures.rs:83-99`, `src/app.rs:3485-3502`,
`tests/property/reverse_lookup.rs:42-58`, `tests/property/ui.rs:165-180`.
Watcher `src/catalog/watcher.rs:166-169` already classifies all catalog paths;
app `src/app.rs:1551-1561` delegates reloads to the store.

## Handoff

- Summary: resolve UI scope; include fixtures and genuine changed-reload tests.
- Changed surfaces: spec only; worker changed nothing.
- Contracts satisfied: read-only audit with source citations.
- Assumptions: watcher filename dispatch is already generic.
- Local checks: inspected navigation, store consumers, fixtures and graph.
- Integration evidence: parent checked navigation/render dispatch and generator.
- Risks: previous graph dispatches whole-repo generators concurrently and omits
  checkpoints; N=5 and Unchanged tests do not prove P8/hot reload.

## Files changed

None by worker.

## Decisions

Parent must define UI scope; no speculative watcher rewrite is justified.

## Risks

Do not retain the claim that adding a store field requires no fixture changes.
Do not claim earlier T8 checks verify later T9 changes.

## Verification run

Source inspection only; no tests or gates run by worker.

## Open questions

Resolved after discovery: the user explicitly chose the full browser. Generator
footprint metadata, UI tasks and actual App render/reload tests are now planned.
