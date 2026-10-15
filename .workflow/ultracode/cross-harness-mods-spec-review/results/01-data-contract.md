# Result 01-data-contract: R18 contract review

## Summary

The new model rejects the current generated catalog; reload and failure behavior
are underspecified. Read-only Codex sidecar `spec_data_review` completed.

## Evidence

Baseline references: generator `scripts/generate-mods.mjs:612-621` emits eight
fields, no `type` or security notes. `loader.rs:14-31` maps all read failures to
`CatalogNotFound`; it cannot establish optional absence. `loader.rs:446-489`
recursively checks catalog strings. `store.rs:330-348` rejects tainted registry
reloads. `error.rs:13-24` labels diagnostic coordinates as bytes/fields.

## Handoff

- Summary: remove invented discriminator; define absence, validation and reload.
- Changed surfaces: spec only; reviewer changed nothing.
- Contracts satisfied: bounded read-only discovery with source citations.
- Assumptions: no runtime implementation authorized.
- Local checks: source inspection only.
- Integration evidence: parent Rust probe independently reproduced missing type
  rejection and line/column versus byte-offset distinction.
- Risks: startup hashes are a second read in existing store logic.

## Files changed

None by worker.

## Decisions

Parent owns amendments. P8 must distinguish invariants from sampled evidence.

## Risks

Do not swallow permission/UTF-8 errors as missing. Do not copy the unvalidated
rules reload arm or assume `u8` enforces the narrower 0..3 domain.

## Verification run

Source review only; no tests or gates run by worker.

## Open questions

At discovery, UI versus foundation was unresolved. The user subsequently chose
the full browser; ADR-7 and the pending implementation tasks record that choice.
