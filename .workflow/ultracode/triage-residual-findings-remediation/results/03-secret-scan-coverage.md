# Result 03 — secret scan coverage

## Verdict

Confirmed. Effectively nothing outside a three-file bundle was covered by a
build-failing secret scan.

## Evidence

- The blocking job scans `plugins/vanguard-frontier-agentic`
  (`.github/workflows/hol-plugin-scanner.yml:69`), which holds three tracked
  files.
- The repository-root job is explicitly advisory: `min_score: "0"` and
  `fail_on_severity: none` (`:141-142`), with a comment stating it is
  informational.
- `.plugin-scanner.toml:53-66` additionally suppresses roughly 6,500 tracked
  files, including 2,575 under `**/references/*.md`, 2,097 under `tests/**`,
  and 1,502 `**/metadata.json`.
- No gitleaks, trufflehog, detect-secrets, pre-commit config, or active git
  hook exists anywhere in the repository.

## Parent action

Added `validate:tracked-secrets`, a deterministic blocking gate over every
tracked text file, rather than widening the external scanner's exclusions —
each of which was added against a confirmed false positive and several of which
cover deliberate credential bait. Nine vendor-prefixed patterns; findings report
path, line and kind and never echo the matched value. Escape hatches are a
`<FAKE>` line marker, reusing the convention the routing fixtures already use,
and a per-path allowlist that cannot name a directory.

Baseline is clean at 13,925 files with zero false positives. A canary planted in
`docs/`, a path the external scanner excludes, fails the gate; marking the same
line `<FAKE>` passes it.
