## EVAL DEFINITION: HOL scanner high-finding remediation

### Assumptions

- The authoritative regression input is `plugin-scanner==2.0.864`, whose wheel
  hash matches the scanner action pinned by `.github/workflows/hol-plugin-scanner.yml`.
- The supplied external run used scanner 2.0.1116, so a clean local 2.0.864 scan
  is necessary but the external-version result remains unverified until CI runs.
- The seven high findings are the four `HARDCODED_SECRET` results, two duplicate
  `GITHUB_ACTIONS_UNTRUSTED_CHECKOUT` results, and one
  `DANGEROUS_DYNAMIC_EXECUTION` result reproduced from the repository root.

### Capability evals

- [x] An exact repository-root scanner run reports zero critical/high findings.
- [x] The asset-integrity repair workflow never checks out or executes a pull
  request head in a `pull_request_target` context.
- [x] Manual dispatch requires only the Dependabot branch; the workflow resolves
  the exact SHA and protects its push with a lease against concurrent updates.
- [x] Workflow catalog generation contains no dynamic evaluation and remains
  deterministic.
- [x] Redaction tests still prove GitHub and npm token-shaped values are removed,
  without embedding scanner-recognized credential literals in tracked files.

### Negative probes

- [x] A temporary copy containing a GitHub PAT-shaped literal triggers
  `HARDCODED_SECRET`.
- [x] A temporary workflow combining `pull_request_target` with a head-ref
  checkout triggers `GITHUB_ACTIONS_UNTRUSTED_CHECKOUT`.
- [x] A temporary JavaScript file containing an actual dynamic-function
  constructor triggers `DANGEROUS_DYNAMIC_EXECUTION`.

### Regression evals

- [x] `npm run validate` passes.
- [x] `npm run lint:spell` and `npm run lint:md` pass.
- [x] Rust formatting, clippy, and tests pass because Rust test fixtures change.
- [x] `git diff --check` passes and no scanner threshold is weakened.
- [x] Asset-integrity metadata is refreshed last for changed hashed files.

### Success metrics

- Capability checks: pass@1 = 4/4.
- Negative probes: pass@1 = 3/3.
- Release-critical regression checks: pass^1 = 1.00.

### Forbidden changes

- Do not suppress executable source, weaken scanner severity gates, or add broad
  ignore paths merely to hide findings.
- Do not commit, push, publish, or run external workflows without separate user
  authorization.

## EVAL REPORT

### Capability evals

- Root scan with pinned 2.0.864: **PASS** — score 96/A, 0 critical/high.
- Root scan with externally reported 2.0.1116: **PASS** — score 96/A,
  0 critical/high.
- Privileged checkout removed: **PASS** — the repair workflow is manual,
  validates and resolves the exact same-repository Dependabot branch tip,
  protects the push with a lease, and creates its write token only after
  regeneration. Operators provide one input instead of a branch/SHA pair.
- Static workflow metadata parser preserved: **PASS** — catalog check and the
  complete repository validation suite passed.
- Redaction behavior preserved: **PASS** — Rust unit, integration, property,
  and documentation tests passed.

### Negative probes

- Credential-shaped literal: **PASS** — `HARDCODED_SECRET` returned.
- Privileged PR-head checkout: **PASS** —
  `GITHUB_ACTIONS_UNTRUSTED_CHECKOUT` returned.
- Dynamic JavaScript construction: **PASS** —
  `DANGEROUS_DYNAMIC_EXECUTION` returned.

### Regression evals

- `npm run validate`: **PASS**.
- Codespell: **PASS** using an isolated temporary environment because the host
  did not have a `codespell` executable.
- Markdown lint: **PASS**, 8,226 files and zero errors using
  `markdownlint-cli2@0.17.2`; the unpinned host command selected a release that
  requires a newer Node runtime than the available Node 18.
- Rust gates: **PASS** with Rust 1.98.1 — formatting, clippy with warnings
  denied, 812 library tests, 10 binary tests, 93 integration tests, and 173
  property tests passed; one documentation test remained intentionally ignored.
- Asset integrity and diff whitespace: **PASS**.

### Metrics

- Capability pass@1: 4/4.
- Negative-probe pass@1: 3/3.
- Release-critical pass^1: 1.00.

### Status

READY FOR REVIEW. The GitHub-hosted workflow itself has not been dispatched;
runtime push behavior therefore remains **UNVERIFIED**.
