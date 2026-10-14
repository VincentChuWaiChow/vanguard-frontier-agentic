# Packet 03 — secret scan coverage

## Concern

Findings SEC1 and SEC2. Broad path exclusions and an advisory repository-root
scan may leave tracked content outside any build-failing secret scan.

## Bounded questions

Establish, with `file:line` citations only, the full exclusion list; every
scanner job with its target path and failure thresholds; which job can fail the
build; what the blocking job actually scans; a per-pattern count of matching
tracked files; and whether any other secret-scanning mechanism exists.

## Constraint

Report file paths and match kinds only. Never reproduce a candidate secret
value in the report.

## Write scope

None. Read-only.

## Deliverable

`results/03-secret-scan-coverage.md`, ending with a `COVERAGE BOUNDARY` section
carrying counts.
