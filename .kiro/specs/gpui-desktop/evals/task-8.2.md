# Task 8.2 evaluation: installation-free CI preparation

## Authorization and boundaries

Defined before implementation for the user-authorized installation-free wave.
This is preparation only: the full task checkbox and dependencies remain unchanged.
No installs, dependency/toolchain changes, remote CI, pushes, global configuration
or unrelated edits. Parent owns module/manifest wiring, evals, serialized Cargo
and separate no-reply local commits. Preserve other workers' files.

## Observable checks

- Preserve existing TUI jobs/steps, permissions and pinned actions; extend both
  push and pull_request filters to actual shared engine/exporter/schema inputs.
- Add Desktop static-contract workflow explicitly labeled not native acceptance.
  No install/setup steps, uploads, remote dispatch, invented runner labels or
  continue-on-error. Use existing Ubuntu/checkout convention and available tools.
- Run bootstrap documents-only self-test (18 negative fixtures), feature-boundary
  self-test (31 negative fixtures) and a dependency-free Node workflow grader.
- Grader reads actual workflow filters/steps. Removing required consumer triggers
  or static commands, adding continue-on-error or mislabeling static acceptance
  must fail mutation checks. Detached expected lists alone are insufficient.
- Register the local grader in repository validation without weakening existing
  checks; parent regenerates covered integrity after all covered bytes settle.
  Full npm validation, scoped docs/lint and exact scope/approval checks pass.

## Deferred full obligations

Native jobs/capability, both locked execution graphs, cargo-deny/source/license
policy and remote execution are UNVERIFIED. Static workflow checks do not complete
8.2. Status: checks defined; implementation not yet graded.

## Local results and bounded review

| Criterion | Result | Evidence |
| --- | --- | --- |
| Static workflow/mutants | PASS | node --test tests/gpui-ci-contracts.test.mjs exits 0: 27 tests, zero skips; actual event filters/commands inspected |
| Bootstrap documents | PASS static only | check_bootstrap.py --manifest-path tools/vfa-desktop/Cargo.toml --documents-only --self-test: 18 mutants reject; live compiler/archive/graph explicitly unverified by this command |
| Boundary self-test | PASS static only | check_feature_boundary.py --self-test: 31 mutants reject; no Cargo invocation |
| Existing TUI policy/steps | PASS | Parent comparison against pre-change source and grader hash preserves complete permissions/concurrency/env/jobs tail |
| Repository gate | PASS | npm run validate exits 0 including the new 27-check gate; final covered integrity write/readback also exits 0 |
| Native/both-lock/supply-chain | UNVERIFIED | No native CI execution, renderer, Desktop build, cargo-deny policy or full task 8.2 completion |

Independent TypeScript reviewer (configured gpt-6.1-sol/high, read-only) found
that an extra top-level TUI defaults/shell mapping could execute true instead of
the original scripts without changing the protected tail. Parent added the exact
mutation and reproduced RED: the old grader wrongly accepted it. The bounded
repair allowlists existing root keys and requires block mapping containers. The
new alternate-shell and inline-event mutations both reject; all 27 checks pass.

One built-in implementation worker owned only two workflows and the new Node
grader; its serving model/effort are unobservable. Parent owns package validation
wiring, generated gate count/integrity, tests, repair and local commit. No setup
actions were added to the Desktop workflow; existing TUI setup steps are preserved.

Full repository validation completed locally without installs. Covered test bytes
changed during the review repair after the first integrity gate, so parent also
regenerated and independently rechecked final integrity; the full run executed
the repaired Node grader. No intervening covered changes followed those checks.
