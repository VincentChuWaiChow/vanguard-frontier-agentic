# Task 1.1 evaluation contract

- Authorization: user requested only task 1.1. Remaining tasks are not authorized.
- Scope: compatibility tests/fixtures only, plus this evaluation and workflow/task
  evidence. Preserve production Rust, manifests/lockfile, catalog and approved
  requirements/design bytes. No system/global installation or remote CI.
  Subsequent commit/push authorization is recorded in the disposition below.
  Fetching existing locked dependencies into temporary caches was approved through
  the execution permission gate; no versions or lockfile bytes changed.
- Library checks: compile reviewed public imports and compare controlled catalog,
  search/relationship and serialization results with hand-reviewed fixed expected
  values. Missing fixtures fail, never return early or self-generate goldens.
- CLI checks: spawn the actual binary using Cargo's test-provided executable path;
  use temporary Unicode/space-containing checkout/config/index paths and isolated
  child HOME/XDG settings, never mutate process-global environment or user data.
  Assert real data envelopes, producer version, exact exit codes and stdout/stderr.
- Persistence: inspect real temporary SQLite schema/rows and existing audit data
  before/after headless runs; capture best-effort index failures without changing
  the legacy behavior. No assertions against a developer's real profile/database.
- Negative checks: wrong envelope, producer, payload, exit code and schema mutants
  must be rejected by the same golden/contract oracle. Normalize only explicitly
  named volatile timestamp/temporary-root/producer-version observation fields;
  preserve all stable nested content and validate normalized fields separately.
- Graders: targeted new contracts and existing headless/persistence integration
  tests, default Rust fmt/clippy/test with `--locked`, spec structure checks,
  scope/hash checks and `npm run validate`. Record baseline/environment failures
  separately; do not weaken assertions or production code to obtain a green gate.
- Route: bounded Grade A test implementation after source discovery; parent owns
  CLI/report fixtures and integration, one configured Rust implementation specialist
  owns library contracts in a disjoint fixture subtree. Shared Cargo builds are
  parent-owned/serialized. One implementation pass plus one bounded repair/review.

## Evidence and disposition

Status: DONE BY EXPLICIT USER ACCEPTANCE. After reviewing the reproduced baseline
failures, the user requested “mark it as done and git commit and git push.” The
checkbox records acceptance of the scoped fixture work, not a green full C1 gate.
The seventeen existing failures remain MISS; no other task or production repair
is authorized by this acceptance. Commit/push of the task artifacts is now
authorized separately, using the repository's GitHub no-reply identity.

### Files and independent oracles

- `tools/vfa-tui/tests/core_contracts.rs` and the fixed
  `tests/fixtures/compatibility/core/` input/expected JSON: public domain imports,
  collection/detail/default semantics, search/filter ordering, relationship edges,
  malformed reload retention and missing-fixture/error behavior.
- `tools/vfa-tui/tests/integration/headless_reports.rs` and
  `tests/fixtures/compatibility/reports/golden.json`: real binary envelopes/combined
  output, current formatting/quiet behavior, exact exits, temporary profile/argv
  paths, populated coverage-cache writes and filter retention, audit preservation,
  schema attributes and append-only triggers. Fixture paths above are relative to
  the TUI package; actual executable/source/temp paths are resolved at test time.
- The coordinator `evals.md` links to this file without inlining it. No future
  task eval files were pre-created. Dispatchers load this active file, not a glob.

### Required checks

Commands below use the existing TUI manifest relative to the selected source root,
with execution-specific temporary Cargo cache/target directories. No test depends
on a developer's machine path, global environment mutation or real database.

| Criterion | Status | Command / evidence |
| --- | --- | --- |
| Library fixtures | PASS | `cargo test --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --test core_contracts`: five tests passed |
| Real CLI/report fixtures | PASS | `cargo test --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --test integration_tests headless_reports`: fourteen tests passed |
| Formatting | PASS | `cargo fmt --manifest-path tools/vfa-tui/Cargo.toml --check` |
| Strict lint | PASS | `cargo clippy --manifest-path tools/vfa-tui/Cargo.toml --all-targets --locked --offline -- -D warnings` |
| Full C1 Rust regression | MISS baseline | Full locked/offline test run fails seventeen existing library tests; clean HEAD control reproduces the identical failure set |
| Repository gates | PASS | `npm run validate` exits zero; QA cluster green |
| Remaining Rust targets | PASS | Locked/offline `cargo test --no-fail-fast`: binary unit 10, core 5, integration 102 and property 173 passed; the one ignored doctest already exists in unchanged source |
| Golden mutation kill check | PASS | Corrupt fixed summary count; actual CLI assertion fails with exit 101. Restore exact bytes; same test passes again |
| Final scope/lint | PASS | Approved bytes and production/manifest/lock/catalog unchanged; local links/JSON/lazy index verified; spelling passes and Markdown lint reports zero issues |
| Native macOS/Windows or GPUI | UNVERIFIED | Not run or claimed by this Linux fixture-only task |

### Known limits, not concealed fixes

- The clean control extracts only tracked `HEAD` Rust/package/catalog inputs into
  a separate temporary tree and runs the unmodified library suite. Both control
  and working tree report the same seventeen failures: the real catalog's `sap`
  provider is not accepted by the existing `Provider` enum. No test is skipped,
  assertion loosened or provider/model/catalog changed to obtain a green result.
- Legacy registry expansion iterates UTF-8 bytes as individual characters:
  `WorkspaceRegistry::expand_env_with` changes non-ASCII text (for example `β`
  becomes `Î²`). The cache-side-effect fixture therefore uses an ASCII registered
  workspace; separate actual CLI argv/checkout/index tests use Unicode/spaces.
  This is not evidence of Unicode registry support and remains a production fix
  requiring separate scope approval.
- Quiet CLI still emits its JSON startup record. Markdown/table envelopes retain
  their current JSON fallback. These observations are characterized, not changed.
- The schema review found column names alone insufficient. The repaired oracle
  checks types/nullability/defaults/primary-key positions and exact audit triggers;
  real UPDATE/DELETE rejection and dropped-trigger mutants exercise enforcement.

### Review and route

One `rust_beast_mode` specialist, configured `gpt-6.1-sol/medium`, owned only core
contracts/fixtures. Parent owned CLI tests, integration and serialized Cargo runs.
One existing `rust_reviewer`, configured `gpt-6.1-sol/high`, performed a bounded
read-only review; its schema finding was repaired. Serving identity is unobservable;
read-only was a task boundary, not separate enforced sandbox isolation. This was
the smallest useful split: two independent fixture areas and one review, with no
architecture work, recursive fanout or speculative production repair.

Cargo executable-path behavior was verified through Context7 against the primary
[Cargo environment-variable reference](https://github.com/rust-lang/cargo/blob/master/doc/book/src/reference/environment-variables.md).
The repository-required AST-only `graphify update .` ran without an LLM/API. Its
generated graph output is separate from task code; HTML visualization was not
produced because of its graph-size limit, not a GPUI visual-audit result.
