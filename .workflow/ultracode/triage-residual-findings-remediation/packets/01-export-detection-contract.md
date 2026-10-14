# Packet 01 — export detection contract

## Concern

Finding N1. Assets written by `vfa-export-agents` are not confirmed by the
console scanner, so `require_asset` reports a correctly installed agent as
missing.

## Bounded questions

Establish, with `file:line` citations only, which detection signals can fire in
headless mode; where each `DetectionMethod` is set; the exact `ExportMeta`
payload a producer must emit; whether the headless catalog supplies template
content; every destination path and extension the exporter writes; whether the
exporter transforms content or copies bytes; and which written formats begin
with YAML frontmatter.

## Write scope

None. Read-only.

## Deliverable

`results/01-export-detection-contract.md`, ending with a `CONTRACT GAP` section.
