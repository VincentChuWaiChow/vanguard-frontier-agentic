# Task 1.3 evaluation contract

## Scope and route

- User requested the next task; scope is 1.3 only. Preserve the default CLI's
  actual data envelopes, formatting/quiet behavior, producer version, exit codes
  and best-effort persistence. No task 1.4, provider/Unicode fixes, dependency
  version changes, catalog edits, commits, pushes or remote CI.
- Parent owns new service-contract tests and spec/workflow evidence. One configured
  Rust implementation specialist owns `src/headless/` and `src/models/report.rs`.
  Shared Cargo execution is parent-owned/serialized. One implementation pass,
  one bounded independent review and one integrated repair pass; no recursive team.
- Complexity B: shared report orchestration mixes filesystem reads, calculations,
  presentation and SQLite writes. Existing Sol specialists are adequate once the
  API/effect boundary below is explicit; no new architecture or dependency needed.
- Freeze task 1.1 fixtures/expected values and task 1.2 terminal boundary. Do not
  switch to the unused flattened `HeadlessOutput` as the legacy schema.

## Explicit boundary and API

- Engine-owned `ReportProducer { name, version }` and `ReportRequest` contain
  owned catalog root, registry/policy/index paths, report types, output format,
  workspace filter and producer identity. No Cli/theme/GPUI types in these DTOs.
- `ReportService::generate(&ReportRequest)` returns owned `ReportResult` with
  `payload`, `exit_code`, `producer`, `quality`, `provenance` and `diagnostics`.
  Payload retains the existing report schema; producer version comes from the
  request, not a service-side package-version constant. Producer name is retained
  in result metadata without adding fields to the legacy JSON envelope.
- Quality distinguishes current, not-configured, unavailable and partial inputs;
  at minimum catalog/registry/policy states are explicit. Provenance identifies
  selected paths/workspaces and observation time. These annotations are separate
  from legacy payload/exit status; missing policy must not imply healthy policy
  data merely because the old envelope says zero violations.
- Generation may read explicit source/config/workspace files, but never creates
  or mutates the supplied index, prints, initializes a global tracing subscriber,
  installs helpers or replays an action. “Pure” means presentation/write-effect
  separation here, not a claim of a filesystem-free mathematical function.
  Existing domain tracing events follow the caller's subscriber; generation does
  not own console logging. The stream probe checks direct silence without an
  installed subscriber, not unconditional silence under arbitrary caller logging.
- Preserve `HeadlessReporter::new/run/render` default imports and behavior.
  Its terminal-only wrapper builds the request using its existing format/quiet
  settings and `vfa-tui` producer identity, emits collected diagnostics in existing
  order, performs the existing best-effort cache writes and prints existing output.
  Keep main's existing audit append unchanged. A private owned persistence snapshot
  may carry calculation results to the adapter; do not recalculate expected values
  or silently change cache/audit policy.

## Observable success and negative guards

1. Both feature modes compile and expose the core request/service/result API.
   Normal no-default closure still contains no terminal/GPUI dependency.
2. Pure-service payloads match reviewed task 1.1 JSON fixtures; custom caller name
   and version remain explicit. Single/combined reports, formats and filters retain
   calculation semantics; stable nested payload data is not normalized away.
3. A re-executed test-process probe captures the service call's stdout/stderr
   between sentinels and verifies they are empty, then installs a test subscriber
   to prove the service did not take the global tracing slot. No new published
   helper binary, unsafe descriptor redirection or process-global env mutation.
4. Missing and pre-existing temporary index destinations remain unchanged after
   generation, including sidecar files. Legacy CLI tests must still observe cache
   writes, retained rows and one main-level audit append per invocation.
5. Wrong producer/envelope/exit/nested-data mutations fail the oracle. Actual
   unavailable/malformed inputs receive honest quality metadata, not fabricated
   healthy zeros. Known legacy payload quirks are not “fixed” by this extraction.
6. All original domain and CLI tests remain enabled in their proper feature modes;
   existing failure identities stay the same apart from justified new tests.
   Keep the recorded seventeen default / eleven no-default SAP baseline failures
   separate from any newly introduced failure.

## Graders and evidence

- Default and no-default `cargo test --test report_service_contracts --locked`.
- Task 1.1 `core_contracts` and default actual CLI/headless contracts.
- Default/no-default fmt/clippy/all-target builds and full locked test suites,
  baseline-failure comparison, boundary checker and deterministic negative cases.
- Package assembly/verification from controlled candidate bytes without a dirty
  or no-verify bypass; frozen dependency/fixture/approval hashes; scoped diff,
  local links/JSON, spelling/Markdown, spec checker and `npm run validate`.
- Use execution-specific Cargo caches/targets and explicit manifests. Resolve test
  binaries through Cargo/current test executable, never fixed machine paths.
- Later native GPUI, Desktop's separate lock graph and complete governance quality
  integration remain UNVERIFIED; they are not created or approved by this task.

## Review and source boundaries

One existing `rust_beast_mode` specialist, configured `gpt-6.1-sol/medium`, owned
only the three production source files below. Parent owned tests and integration,
then ran the graders independently. One existing `rust_reviewer`, configured
`gpt-6.1-sol/high`, performed a bounded read-only source review and found no
blocking extraction defect. It identified the caller-owned tracing distinction
now documented above and in the API. Read-only was a task restriction inside a
shared workspace-write sandbox, not enforced file isolation. Serving model/effort
and token/cost savings are unobservable; configured profiles are not runtime proof.

The parent repair closes the no-default unused producer import and a reproduced
non-UTF8 catalog quality bug: all failed catalogs were falsely PARTIAL because
lossy error paths did not match native hash-map paths. The Unix regression first
failed in `vfa-task-1-3-native-path-red.log`, then passed after matching the fixed
catalog filenames. Missing/invalid registry quality also propagates to workspace
resolution quality, rather than labeling zero resolved workspaces current. This
does not fix the separately recorded legacy registry Unicode expansion defect.

Context7 MCP was used to verify the one-global-subscriber contract against
[the primary tracing documentation](https://docs.rs/tracing/latest/tracing/subscriber/fn.set_global_default.html).
The process probe reserves its subscriber only after both successful and invalid
input generation; taking that global slot inside the service would fail the test.
No new dependency, helper executable or global environment mutation is needed.

Quality/provenance describes catalog/config observations and registry resolution.
It does not claim every workspace file was readable: the existing scanner does
not expose complete read-error evidence. Full governance-quality integration is
a later task. A valid registry with no selected workspaces is current only as a
resolution observation; it is not successful scan or compliance evidence.

## Changed-file ownership

- `tools/vfa-tui/src/headless/mod.rs`: shared API re-exports.
- `tools/vfa-tui/src/headless/reporter.rs`: service extraction and CLI effects.
- `tools/vfa-tui/src/models/report.rs`: owned request/result and evidence models.
- `tools/vfa-tui/tests/report_service_contracts.rs`: fixed oracles and effect probes.
- This task eval, shared eval index, task checkbox and workflow progress records.

No manifest, lockfile, main, old golden fixture, catalog, approved requirements
or design edit is required. One implementation specialist plus one source review
was the smallest useful route: tests and source were independent write sets,
while Cargo execution and final integration stayed serialized with the parent.

## Results

Scoped task 1.3 is complete. Full-suite baseline failures remain non-green;
completion neither waives new failures nor establishes release readiness.

| Criterion | Result | Evidence |
| --- | --- | --- |
| Owned API and explicit caller | PASS | Default eleven / no-default ten new integration tests; fixed payloads, custom producer and results surviving dropped requests/fixtures |
| Legacy payload/format/persistence | PASS | Existing fourteen headless-report and two persistence cases pass, plus five core contracts in both modes; frozen goldens unchanged; actual CLI still creates the index |
| Negative oracles | PASS | Six payload mutations, flattened envelope, wrong returned exit and wrong caller all rejected against independent fixed expectations |
| Stream/subscriber separation | PASS scoped | Re-executed test process captures empty direct stdout/stderr for All with valid and invalid inputs, then successfully installs the global test subscriber |
| No index writes | PASS | Missing parent remains absent; pre-existing SQLite index plus live WAL/SHM sidecars remain byte-identical after All; preserved row and absence of cache migration verified |
| Honest input evidence | PASS scoped | Missing configs NOT_CONFIGURED, invalid configs UNAVAILABLE, mixed catalog/policy/workspace resolution PARTIAL, unavailable workspace diagnostics/counts and native non-UTF8 provenance retained |
| Original tests remain enabled | PASS | Default 1032 and no-default 812 original identities unchanged; only the new service target added (eleven / ten cases) |
| Full default regression | MISS baseline | Library 725 pass / seventeen fail; binary ten, core five, integration 102, property 173 and service eleven pass; exact failing names equal task 1.2 baseline |
| Full no-default regression | MISS baseline | Library 548 pass / eleven fail; core five, integration 77, property 171 and service ten pass; exact failing names equal task 1.2 baseline |
| Dependency boundary | PASS | Actual engine closure plus 31 deterministic boundary cases; no normal terminal/GPUI dependency |
| Formatting and linted builds | PASS | Fmt and default/no-default all-target clippy with warnings denied |
| Package | PASS candidate bytes | Normal package assembly and verification from tracked HEAD plus the three candidate source files; archive source bytes match, then packaged no-default library builds; no dirty/no-verify bypass or publication |
| Repository/spec/lint/scope | PASS | Repository validation, structural spec mutants, whitespace, local links, JSON, frozen artifacts, spelling and Markdown checked |
| AST graph maintenance | PASS AST only | `graphify update .` succeeds; HTML visualization exceeds the existing size limit, not native/visual acceptance evidence; generated files stay uncommitted |
| Desktop/native/full scan evidence | UNVERIFIED | No real Desktop lock graph, GPUI/runtime/native host or complete scan-read integration exists in this task; one existing ignored doctest remains ignored |
| Git delivery | UNVERIFIED pending delivery | Subsequently authorized by the user; pre-delivery HEAD is `58cca6f28c6aaa40168ea3c4c01b09565c63aa45`; actual commit and remote SHA are reported after execution |

### Reproducing the executed graders

Commands ran from the discovered repository root with session-selected temporary
Cargo home/target locations. Temporary logs named below are this run's evidence,
not hardcoded application or skill locations.

```bash
manifest=tools/vfa-tui/Cargo.toml
cargo fmt --manifest-path "$manifest" --check
# Run each Cargo command once normally and once with --no-default-features.
cargo test --manifest-path "$manifest" --locked --offline --test report_service_contracts
cargo clippy --manifest-path "$manifest" --locked --offline --all-targets -- -D warnings
cargo test --manifest-path "$manifest" --locked --offline --no-fail-fast
cargo test --manifest-path "$manifest" --locked --offline -- --list
CARGO_NET_OFFLINE=true python3 tools/vfa-tui/tests/check_feature_boundary.py \
  --manifest-path "$manifest" --self-test
python3 .kiro/specs/gpui-desktop/verify.py --tasks --self-test
npm run validate
git diff --check
graphify update .
```

All direct service, fmt, clippy, boundary, spec, repository, scope and package
commands exit zero. Both full test commands exit 101, solely for the recorded
library failures. A temporary comparison driver excludes the separate doctest
group, asserts exact old test identity sets in each mode and exact failing names
against task 1.2, then allows only the new service target. Its result is
`vfa-task-1-3-regression-comparison.json`. Full/default and no-default logs are
`vfa-task-1-3-{default,no-default}-{full,list,contracts,clippy}.log` in the temporary
run directory. After strengthening the WAL fixture, both direct service targets
and strict clippy were rerun; production source bytes did not change.

The temporary packaging driver extracts the tracked crate into a newly selected
temporary root, overlays only the three changed production files and invokes
`cargo package --manifest-path <candidate-manifest> --locked --offline` with its
normal verification. It compares archive source bytes to the working candidate,
checks that the existing test exclusion is preserved, then checks the extracted
no-default library. `vfa-task-1-3-package.log` and
`vfa-task-1-3-package-evidence.json` record the result. This is package/build
evidence, not publication, VCS provenance or desktop-consumer parity.

Separate scoped codespell and offline markdownlint commands cover the changed
Rust and hidden task/workflow documents; no tool/dependency or lint configuration
is installed or changed. Approved requirement/design hashes, all old fixtures,
Cargo manifest/lockfile, main and Git identity/HEAD are checked separately.

### Subsequent delivery authorization

The user requested “git commit and git push” for the completed task 1.3 changes.
This authorizes a normal commit/push on the existing feature branch, using the
repository's GitHub no-reply author and committer identity. It does not authorize
history rewriting, changes to develop, task 1.4, publication or remote CI dispatch.
Generated graph output and unrelated files remain excluded.

Delivery is parent-owned inline work: the source was already independently
reviewed, and deterministic scope/identity/hash/readback guards are sufficient;
another agent would not improve this sequential Git operation. Before commit,
refresh repository validation, spelling/Markdown, scoped Rust checks and exact
candidate bytes. Require both Git identities to match the approved no-reply
identity, inspect the committed file set and verify the remote feature SHA after
push. Stop on branch/source/index drift, identity mismatch or a non-fast-forward
remote change. Actual delivery success is not preclaimed in this checkpoint.
