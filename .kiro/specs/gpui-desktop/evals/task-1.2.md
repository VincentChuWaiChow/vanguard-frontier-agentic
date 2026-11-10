# Task 1.2 evaluation contract

## Scope and route

- User request: “go for next phase,” interpreted explicitly as the next unchecked
  task, 1.2 only. No task 1.3 service extraction or GPUI implementation, provider
  fixes, catalog changes, system/global installs, commits, pushes or remote CI.
- Parent owns Cargo manifest/lockfile and production feature/module wiring.
  One existing Rust implementation specialist owns test module gates and the
  feature-boundary checker. Shared Cargo executions are parent-owned/serialized.
  One bounded independent review and one integrated repair pass are allowed.
- Complexity: initially B because CLI/report/domain code shares a module; bounded
  source inspection resolves the seam without changing domain behavior or public
  default imports. Existing configured Sol specialists are adequate; no architecture
  escalation or new abstraction is needed. Serving identity remains unobservable.
- Preserve approved requirements/design and task 1.1 fixed input/expected fixtures.
  Gate presentation-dependent adapters only, not whole domain test suites.

## Capability and regression checks

1. Default `terminal` feature enables optional ratatui, crossterm, terminal-light,
   clap and tracing-subscriber dependencies. Binary requires `terminal`; default
   CLI/TUI imports and task 1.1 executable report contracts stay unchanged.
2. `cargo check --no-default-features --lib --locked` succeeds. Normal engine
   dependency closure contains no terminal/GPUI presentation packages. Determine
   manifests from explicit arguments/Cargo metadata, never machine-specific paths.
3. `cargo test --no-default-features --lib --test core_contracts --locked` executes
   domain tests, rather than hiding failures with broad feature gates. Pure report
   severity/status/audit helpers and tests remain compiled without `terminal`.
   Private report projections remain available in test builds; their CLI caller
   alone is optional. Full pure service extraction remains task 1.3.
4. Run no-default integration/property suites as well as defaults. Compare test
   inventories to ensure only presentation/CLI adapter cases disappear. No empty
   suite, missing-fixture return, assertion weakening or catch-all lint suppression.
5. Boundary checker has deterministic negative fixtures: terminal/GPUI leakage,
   malformed/missing engine roots and incorrect bin/default feature declarations
   fail. Scope closure to the shared engine subtree, not the entire Desktop graph.
   A second consumer is checked when explicitly supplied; it does not exist yet.
6. Default fmt/clippy/locked tests, task 1.1 golden contracts and repository
   validation remain required. Distinguish the accepted seventeen existing SAP
   library failures from new failures; never relabel the full regression gate green.
7. Package source layout stays valid: optional binary dependency wiring cannot
   refer to excluded test helpers. No new published crate, workspace or dependency
   version updates. Record package assembly/verification limits honestly.

## Graders

Use the selected TUI manifest through Cargo arguments and execution-specific
temporary Cargo cache/target paths; no persistent developer path constants.

- `cargo fmt --manifest-path <manifest> --check`
- `cargo clippy --manifest-path <manifest> --all-targets --locked -- -D warnings`
- `cargo clippy --manifest-path <manifest> --no-default-features --all-targets --locked -- -D warnings`
- `cargo check --manifest-path <manifest> --no-default-features --lib --locked`
- `cargo test --manifest-path <manifest> --no-default-features --lib --test core_contracts --locked`
- Default and no-default locked tests with `--no-fail-fast`, actual task 1.1 CLI
  tests under defaults, and before/after `-- --list` inventory comparisons.
- `python3 <package>/tests/check_feature_boundary.py --manifest-path <manifest> --self-test`
- Package assembly checks against the selected source tree, `verify.py --tasks
  --self-test`, local links/JSON/scope, spelling/Markdown and `npm run validate`.

Context7 verified Cargo's optional `dep:` features, binary `required-features` and
normal dependency-tree edge selection against primary
[Cargo documentation](https://github.com/rust-lang/cargo/blob/master/doc/book/src/reference/features.md).
Unknown native platforms, downstream Desktop lock graph and runtime/UI feasibility
remain UNVERIFIED; task 1.2 does not claim those later gates passed.

## Evidence and disposition

Status: TASK 1.2 DONE; FULL REGRESSION GATES RETAIN KNOWN BASELINE FAILURES.
The authorized boundary work meets its scoped capability/compatibility criteria;
comparison proves no new failures against the recorded, previously accepted
baseline. Completion does not mark C1/C2's full suites green, waive new failures,
authorize production repairs or approve any later implementation task.

### Changed files

- `tools/vfa-tui/Cargo.toml`: default `terminal`, optional adapter dependencies,
  and explicit existing binary requiring the feature. `Cargo.lock` is unchanged.
- `tools/vfa-tui/src/lib.rs`: feature gates for app/cli/ui/logging adapters only.
- `tools/vfa-tui/src/headless/reporter.rs`: CLI-taking method is optional; pure
  severity/status/audit/render contracts stay available, and private projections
  remain compiled in test builds without terminal. No pure-service extraction.
- `tools/vfa-tui/tests/integration/mod.rs`: gate only the five CLI/TUI adapter
  modules. `tests/property/mod.rs` gates only UI properties. Domain/audit/headless
  properties and all task 1.1 golden fixtures remain unchanged.
- `tools/vfa-tui/tests/check_feature_boundary.py`: manifest/Cargo-identity checks,
  normal engine-subtree validation and deterministic negative fixtures.
- This task eval, its shared index, task status and workflow progress records.
  Generated AST graph output is separate local tooling data, not feature code.

### Checks

| Criterion | Status | Evidence |
| --- | --- | --- |
| No-default library builds | PASS | Locked/offline no-default `cargo check --lib` succeeds |
| Engine dependency boundary | PASS | Live checker sees one engine subtree with no normal presentation dependency; 31 deterministic positive/negative cases pass |
| Default behavior inventory | PASS | Exact same 1032 test identities before/after, including binary and CLI golden tests |
| No-default domain coverage | PASS inventory | 559 library, 5 core, 77 integration and 171 property tests remain; every removed test is in the explicit presentation/CLI adapter allowlist |
| Default full regression | MISS existing baseline | Exact same seventeen failing library test names as task 1.1; all remaining default targets pass |
| No-default full regression | MISS existing baseline | Eleven failures are exactly the prior non-presentation domain subset; remaining tests pass, with no new failures |
| Fmt and strict clippy | PASS | Both default and no-default all-targets clippy pass with warnings denied; fmt check passes |
| Package assembly/verification | PASS candidate snapshot | Clean temporary tracked-source snapshot overlaid with candidate manifest/lib/reporter packages and verifies; no dirty/no-verify bypass or publication. Packaged source bytes match the candidate |
| Packaged no-default build | PASS | Extracted verified package builds its no-default library; explicit no-default binary request correctly fails because terminal is required |
| Repository validation | PASS | `npm run validate` exits zero, QA cluster green |
| Extra strict documentation probe | MISS existing baseline | Broken/private intra-doc links also occur on unmodified HEAD; current no-default error kinds are a subset, not a new docs regression |
| Independent review | PASS process | Reviewer identified undiscovered-bin oracle gap; parent now validates Cargo metadata targets and adds a real auto-bin mutation rejection |
| Final scope/lint | PASS | Frozen approvals/task1.1 fixtures/lockfile unchanged; dependency pins and package metadata preserved; local links/JSON/scope, spelling and Markdown checked |
| Desktop graph/native platforms | UNVERIFIED | Consumer parsing has synthetic coverage, not a real Desktop lock graph or cross-platform runtime/build result |

### Regression and provenance notes

- Full suite failures remain attributable to the existing real-catalog `sap`
  provider mismatch. Tests are not skipped or assertions weakened to claim a
  green gate. Existing ignored doctest behavior is unchanged.
- The retained no-default inventory includes pure headless unit tests, exit/status
  properties and real audit persistence tests. Optional CLI run/orchestration is
  not a completed `ReportService`; task 1.3 still owns that public pure contract.
- Plain packaging from the dirty working tree was correctly refused. Verification
  used a controlled source snapshot with candidate bytes, not an unapproved commit,
  `--allow-dirty`, `--no-verify` or a claim of release/VCS provenance.
- No provider/Unicode behavior changes, dependency version updates, new published
  crate/workspace, commits, pushes or remote CI. No task 1.3 or GPUI work started.
- Configured implementation specialist: `rust_beast_mode`, `gpt-6.1-sol/medium`;
  parent owns source integration/Cargo commands. Review: existing `rust_reviewer`,
  `gpt-6.1-sol/high`. Serving identity is unobservable; read-only review is a task
  boundary within shared permissions, not an isolation guarantee.
- The single integrated review repair covers automatically discovered binaries,
  not only explicit manifest entries. A temporary candidate with an ungated
  `src/bin/probe.rs` is rejected from actual Cargo metadata; the working tree is
  never mutated for this adversarial probe. No consumer GUI binaries are falsely
  required to enable the engine's terminal feature.

### Subsequent delivery authorization

After the implementation report, the user explicitly requested “git commit and
git push.” This separately authorizes delivery of task 1.2 artifacts on the current
feature branch with the repository's GitHub no-reply identity. Generated graph
output, later implementation, production repairs and remote CI dispatch remain
excluded. Existing baseline failures remain recorded, not converted to PASS.
