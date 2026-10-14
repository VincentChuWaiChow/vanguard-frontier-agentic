# Packet 02-integration-map: repository integration

## Objective

Map exact integration surfaces and existing restrictions for the mods task.

## Context

Implementation is gated on native API and contract verification by the parent.

## Sources

`package.json`, `catalog/index.json`, `tests/validate-asset-integrity.py`,
`tests/validate-catalog.py`, `tests/fuzz-properties.test.mjs`,
`.plugin-scanner.toml`, scoped AGENTS guides, existing generator conventions.

## Ownership

Read-only, no writes. Return findings to parent for workflow recording.

## Do

Identify coverage assumptions, schema constraints, test dependencies, ignored
workflow artifacts, and minimum exact edits. Cite file lines.

## Do not

Edit, run mutating generators, run full tests, commit, push, spawn agents, read
credentials, or inspect pre-existing untracked `.kiro/kiro-agents/` contents.

## Expected output

Concise integration map, hidden conflicts, and verification commands.

## Verification

Inspect actual code rather than trusting spec assumptions or stale graph.

## Handoff format

Summary, file:line evidence, risks, recommended parent action.
