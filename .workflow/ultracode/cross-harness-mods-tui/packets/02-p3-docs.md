# Packet 02: P3 and explicit reach docs

## Objective

Implement pending tasks 6.7 and 8.6 without changing runtime hook behavior.

## Context

Read current CLAUDE.md, tests/AGENTS.md, R16.3/R17.5, design P3 and deviations.

## Sources

Existing production-hook property test, audit core and each mod manifest/README.

## Ownership

Write only `tests/fuzz-properties.test.mjs` and `mods/*/README.md`.

## Do

Extend the production append workload to nested JSON, 200 seeded sequences
N=1..50, explicit N=1/50 for all three command adapters and null-prototype input.
Check real counts, sequence and canonical hashes; retain isolated temp cleanup.
State each README's numeric derived reach and actual declared footprint.

## Do not

Edit runtime/generator/validator/generated files, task status or personal config.
Do not run full fuzz before parent integration; do not run broad gates, Git,
network or recursive delegation. Do not reduce the approved workload for speed.

## Expected output

Changed paths, workload/seed, static checks and runtime verification still needed.

## Verification

Parent owns the final production fuzz run after generated data settles. Keep
test assertions tied to production code, not a duplicated append implementation.

## Coordination rule

You are not alone in the codebase. Do not revert edits made by others. Adapt to
nearby changes. Ask before expanding ownership.

## Handoff format

Summary; changed surfaces; contract criteria; assumptions; checks; risks.
