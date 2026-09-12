# Packet 01: Requirement traceability

## Objective
Map the supplied bundle integration contract and module map to committed repository evidence.

## Context
Audit commit `d3b0693` against `/mnt/c/Users/Vincent/Downloads/istio-review-suite`.

## Sources
Bundle `integration/INTEGRATE.md`, `integration/INTEGRATION_PROMPT.md`, `module-map.json`; repository `CLAUDE.md`, schemas, changed paths, tests.

## Ownership
Read-only agent; no file edits.

## Do
Classify each material requirement PASS, MISS, or UNVERIFIED with exact path/line or command evidence. Look especially for silently omitted modules and renamed IDs.

## Do not
Do not edit, run live operations, or accept catalog presence as proof of semantic correctness.

## Expected output
Concise matrix plus promotion blockers and residual risks.

## Verification
Use direct file inspection and safe local commands.

## Handoff format
Summary, evidence, decisions, risks, checks run, open questions.
