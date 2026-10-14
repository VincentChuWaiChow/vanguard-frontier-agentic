# Integration

## Accepted

Resumed the committed scaffold and preserved pre-existing implementations.
Added production regression/property tests, user-facing docs, npm shipping and
commands, catalog index, CI gate, mod integrity coverage and generated gate counts.
Parent verified worker changes and integrated the initial repair plus the explicitly
approved focused anchored-extension repair.

## Rejected

No unsupported APIs, fabricated writes to justify reach, executable scanner
suppression, duplicated test-only security implementation, personal installation,
commit, push, or deployment.

## Conflicts

Historical spec assumptions are superseded by the already-approved plan and
recorded deviations. Task checkboxes refer to approved equivalents, not literal
fulfillment of obsolete APIs or historical staged checkpoint executions.

## Decisions

Audit stale locks require explicit recovery, not age-based takeover. Generator
symlinks are refused before access. Missing output under sandbox subprocess
restrictions was not counted as success; exact reviewed local checks were rerun.

## Final changes

See `git diff --stat` and untracked task files. Existing `.kiro/kiro-agents/` is
untouched. The branch remains `feat/cross-harness-mods`; verified feature commit is `16cc3100`.

## Verification still needed

The anchored-extension bug is repaired and independently reviewed. All required
local checks pass. No required local repair remains; live installation and push
are skipped outside the user-requested commit scope.

## Remaining risks

No live installed Kiro/Codex proof. Native CLI tests have attribution warnings.
Heuristic guards, output-schema/key masking limitations, unlocked context counts,
hook startup/timeout failure gaps, and operator write-path overrides are documented.
