# Eval contract

## Goal

Ship the full approved mods browser with robust catalog reload, not a narrower
loading-only implementation.

## Success criteria

| Requirements | Observable grader |
|---|---|
| 18.1–2 | Strict legacy/new typed models; missing/unknown fields fail; imports compile |
| 18.3–4,15 | Bounded 20 MiB reads, only actual NotFound optional, recursive taint/reach and coordinate regressions |
| 18.5–10 | Exact-byte hashes, deterministic order, lookup, all exhaustive literals compile |
| 18.7,13–14 | Repeated invalid/missing reload retains data/hash; correction, changed and empty recover |
| 18.11 | Isolated varied-N and real generated catalog parity, not only count=5 |
| 18.16–20 | Real App navigation/render/detail/scroll/Back; legacy vs empty footprint; removed-ID and failed reload |
| 8.7 | Generator/validator footprint parity and deliberate mismatch rejection |
| 16.3 | 200 seeded production append sequences N=1..50, nested JSON, explicit boundaries and null prototype |
| 17.5 | Every mod README matches its manifest reach and footprint |
| 18.12,8.5 | All integrated repository/Rust/lint gates and exact commit/remote proof |

## Integration surfaces

Shared `Mod`, `ModFootprint`, `CatalogStore.mods`, query/reload APIs and generated
catalog footprint. Parent owns producer; UI worker consumes documented contract.

## Downstream consumers

Primary App/browser, Rust fixtures/property tests, Python catalog validator and
Node generation/property tests. Existing agents/skills/rules remain compatible.

## Required checks

Targeted Rust regressions plus `cargo fmt --check`, `cargo clippy --all-targets
-- -D warnings`, `cargo test`; `npm run test:fuzz`, `npm run validate:mods`,
generator `--check`, `npm run validate`, spelling, full Markdown and diff checks.
Tests use disposable local fixtures, not real external services.

## Deliverables

Implementation, regression tests, generated catalog/integrity, task updates,
review evidence and final audit/report in this run root.

## Blocking conditions

Unresolved API/security conflict, lost/ambiguous external write, unavailable
required gate or exhausted delegation/repair budget. No false green from
missing child output; inspect lifecycle and retain live handles before retrying.
