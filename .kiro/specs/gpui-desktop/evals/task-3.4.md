# Task 3.4: governance input-quality preparation

Authority/scope: the approved three-slice continuation in
[the shared contract](no-install-continuation.md). Full task dependencies remain
unchanged; no native integration or full-task completion is claimed.

Before implementation, require `governance_contracts` to prove:

- Exact coverage/drift/policy/suppression results over two fixed workspace inputs.
- Additive structured scan observations capture named directory/entry/file errors;
  existing vector API and legacy report payload bytes/semantics remain compatible.
- Successful empty is not unavailable, and absent policy/registry is explicitly
  not configured; invalid/unreadable/mixed inputs retain counts and exclusions.
- Reports and quality use the same scan observations, not a second independent
  scan that could describe different bytes or hide failed input.
- Source/request changes invalidate prior results; failed refresh retains only
  appropriately identified last-good data with original provenance, marked stale.

Parent owns integrated C1/C2/C3, formatting, strict lint, baseline comparison,
packaging and frozen/scope checks.

## Observed results

- PASS: initial runtime RED reproduced Current instead of Unavailable for a failed
  harness. Missing-API compile RED is separate, not the behavioral proof.
- PASS: default twelve and engine-only eleven governance contracts; unchanged
  report/core golden contracts and both full regression suites pass.
- PASS: fixed two-workspace coverage, drift and suppression oracles; named scan
  errors/counts; report-specific policy quality and bounded stale provenance.
- PASS after one parent review repair: existing empty registry is NotConfigured,
  unlike a no-match filter; traversable/unlistable workspace roots retain readable
  assets and legacy payload while quality remains partial. Both new regressions
  first failed at runtime; the permission case ran as an unprivileged process.
- PASS: strict clippy, format and package/source parity through parent graders.
- UNVERIFIED: atomic filesystem observations, native adapter/second-lock behavior
  and full task dependencies. No measured coverage threshold is claimed.

Raw logs are under ignored `tools/vfa-tui/target/no-install-continuation/`, notably
`governance-runtime-red.log`, `review-runtime-red.log`,
`review-repair-green-engine.log` and `postreview-{default,engine}.log`.
Status: READY FOR REVIEW for this engine-preparation slice, not the full task.
