# GPUI desktop evaluation contract

Defined before requirements editing. Status: requirements phase, not feature
acceptance. Baseline: `6dda992a3dedd845731d795aaaed20d49346f57c`.

## Specification graders

| ID | Grader and pass signal | Owner |
| --- | --- | --- |
| S1 | `python3 .kiro/specs/gpui-desktop/verify.py --self-test`: valid document accepted; missing heading, duplicate ID, missing evaluation and invalid EARS obligation rejected | Parent |
| S2 | Each requirement maps to the positive and negative scenario below; source facts have citations in `research.md` and assumptions are labeled | Parent review |
| S3 | Independent review for wrong detail level, ambiguity, inconsistency and incompleteness; dispositions in `requirements-analysis.md` | Reviewer, parent resolves |
| S4 | Explicit user requirements approval before `design.md`; explicit design review before final task breakdown | User / parent |
| S5 | After those phases, all requirement IDs map to design sections, task IDs and actual acceptance evidence | Parent; pending |
| R1 | `git diff --check` exits 0; `git diff --name-only` empty during this additive spec-only phase; untracked additions restricted to the two owned directories | Parent |
| R2 | `npm run validate` exits 0; no catalog/integrity regeneration or existing file edits needed for spec additions | Parent |

S1 is a structural check, not natural-language logical proof. Kiro's proprietary
Analyze Requirements / SMT service is not exposed in this Codex session. S3 uses
the installed analysis skill's manual-review fallback; no solver result will be
claimed. No reliability `pass@k` metric is inferred from one run.

## Feature acceptance scenarios

These are required future tests, not tests already implemented or passed.
Implementation tasks must bind each row to exact executable tests and native
evidence artifacts. Fixture expectations must be independent of the production
function under test. Use temporary directories and fake subprocesses, never real
operator repositories for automated mutation tests.

| Requirement | Positive scenario and expected result | Negative / regression scenario and expected result |
| --- | --- | --- |
| R1 | Select valid checkout from unrelated CWD; show canonical source and current snapshot | Switch while queued/cancelling is blocked; obsolete presentation result discarded but original terminal/audit evidence retained |
| R2 | Load all entity types; compare IDs, counts, optional metadata and relationships with fixed fixture expectations | One malformed catalog; other valid collections remain usable and failure is not represented as zero healthy results |
| R3 | Combined query/provider/harness filter returns known intersected IDs in stable order; clearing restores all | Empty query, Unicode, no matches and deleted selection do not panic or select a different record silently |
| R4 | Valid reload updates selected record; unchanged bytes preserve state | Malformed reload preserves labeled last-good data; deletion makes stale status visible; no mutation allowed from stale preview |
| R5 | Dependency edges and one changed asset produce fixture-expected graph and integrity findings | Cycle, missing file and missing manifest are represented as distinct findings, not success |
| R6 | Two fixture workspaces show populated coverage/drift/policy results and correctly scoped details | Mixed-success scans show PARTIAL, absent policy NOT_CONFIGURED, invalid policy UNAVAILABLE; stale dependencies and excluded denominators remain explicit |
| R7 | Approved gate runs exact existing script; stream output and terminal status reflect helper exit code | Untrusted checkout, missing runtime, invalid gate or failing process does not produce a successful result or dependency bypass |
| R8 | Authoritative plan exposes paths and existing-file conflicts; confirmed unchanged request produces exact expected fixture delta | Pre-launch symlink swap is rejected; concurrent edit invalidates assurance; cancellation after first write shows partial/unknown effects without replay or rollback |
| R9 | Desktop event loop drives real engine runtime/process services through completion and cancellation; helper descendants are reaped | Missing runtime context, no-newline oversized stream, slow UI, repeated cancellation and application close cannot masquerade as successful cleanup |
| R10 | Operation records and reports match fixture results; terminal commit is acknowledged independently of execution outcome | Inject SQL failure before launch (no action), after child success (SUCCEEDED plus audit FAILED), then restore store (persist evidence once; never rerun action) |
| R11 | TUI/headless/desktop adapters call one engine; shared fixture results match baseline where unchanged | Core dependency graph contains terminal/GPUI UI dependencies, or catalog/CLI/output compatibility changes; fail regression gate |
| R12 | Build and native smoke on every required OS/architecture; Linux checked on X11 and Wayland | Cross-compilation alone, WSL, absent GPU driver or one host's test run cannot mark another target verified |
| R13 | Complete catalog → search → details → gate and preview flows using keyboard/pointer; assert actual application effects and real AT behavior | Disconnect callback while preserving label/value metadata: test must fail; focus trap, unnamed input, inaccessible dialog and hidden scaled controls fail acceptance |
| R14 | Release build meets declared budgets using the committed 10,000-entity fixture, fixed corpus/pacing, raw samples and nearest-rank p95 protocol | Reject empty fixture, missing pressure overlap or excluded slow samples; oversized unterminated output respects bounds before newline |
| R15 | Explicitly trust current execution-content revision; only approved executable/argv and destination are used | Edit trusted scripts/package config before a gate/preview launch: invalidate trust; catalog payloads never become commands and known credential patterns remain redacted |
| R16 | Reproducible locked builds, notices/checksums, release-specific component/test matrix and native unpacked-app smoke | Missing runtime/asset/license or developer CWD dependency fails; unavailable pixel renderer is not PASS, and component support cannot be inferred from general framework claims |

## Implementation and release gates

- Preserve existing `cargo fmt --manifest-path tools/vfa-tui/Cargo.toml --check`,
  `cargo clippy --manifest-path tools/vfa-tui/Cargo.toml --all-targets --locked --
  -D warnings`, and `cargo test --manifest-path tools/vfa-tui/Cargo.toml --locked`.
  Rebaseline only with an explicit, reviewed reason; current manifest MSRV text is
  not proof of the locked dependency floor.
- Design must add exact commands for shared-engine, desktop UI and native matrix
  gates after crate layout is selected. Do not invent nonexistent passing tests.
- Before any push: repository validation, spell-check, markdown lint and all
  affected Rust gates, plus targeted positive/negative runtime probes.
- Native evidence records commit, lockfile hash, OS/build/architecture, backend,
  toolchain, command, exit status, screenshots and scenario results. CI build,
  headless semantic tests and real rendered-window checks remain separate fields.
- Platform tests and performance/accessibility runs are `UNVERIFIED` until that
  evidence exists. A disabled feature is not a substitute for a required scenario.

## Current report

See [requirements-analysis.md](requirements-analysis.md) for review dispositions
and the workflow [phase report](../../../.workflow/ultracode/gpui-desktop/final-report.md)
for commands actually executed. Feature rows R1–R16 are all `UNVERIFIED` in this
spec-only phase.

## Design phase contract

Goal: a reviewable, implementation-ready GPUI desktop design for the exact
user-approved requirements; no application implementation in this phase.

- Success: standard design sections, source-backed toolkit/runtime choices,
  concrete adapter/API and data contracts, failure/recovery semantics, native
  acceptance strategy, and a design/test mapping for every approved criterion.
- Integration surfaces: existing `vfa_tui` library/CLI/report behavior, planned
  GPUI adapter and runtime bridge, authoritative exporter protocol, process
  backend and operation evidence store. Downstream consumers are both adapters,
  their test suites and the subsequent task breakdown.
- Deliverables: `design.md`, `design-review.md`, and `verify.py --design` structural
  checks with negative mutants. Extend existing evidence; no parallel spec system.
- Required checks: requirements approval hash unchanged; complete criterion-range
  traceability with valid design/test anchors; missing trace row/section rejected;
  bounded independent design refutation; `npm run validate`, local links and scope.
- Blocking conditions: unapproved requirement changes, unresolved high-severity
  architecture contradictions, hidden runtime/dependency assumptions or a design
  that cannot preserve existing package/CLI/data contracts. Uncompiled prototype
  gates must be explicit, not reported as proven support.
- Owned paths: this spec directory and its workflow directory only; requirements
  content is frozen at its approved SHA-256. Forbidden: app/dependency/runtime,
  catalog, database, CI mutation, commit, push, release or installation actions.
- Team: parent designs/integrates; existing read-only Rust and docs specialists
  provide independent source evidence; a separate configured architect reviews
  the draft. One design/review wave and one integrated repair pass, no recursive
  agents. Actual serving identities are unobservable; inherited shared sandbox
  controls remain unchanged.
- Human gate: design review/approval before task-breakdown; requirements approval
  is not approval of unseen design decisions. Implementation/runtime gates below
  the design remain unverified until executed.

## Task-breakdown phase contract

- Goal: actionable implementation tasks for the frozen user-approved requirements
  and design. Task planning is authorized; application implementation is not.
- Deliverables: `tasks.md`, `tasks-review.md`, and a `verify.py --tasks` check.
- Success: two-level unchecked implementation tasks, concrete owned paths,
  explicit dependencies, design/requirement links, test commands, observable
  positive/negative completion signals and timebox estimates. Every approved
  criterion and every design/verification anchor is covered.
- Graders: enforce both approval hashes, task/check IDs, acyclic dependencies in
  listed execution order, known references and complete criterion coverage.
  Negative mutants remove coverage, create a dependency cycle, use an unknown
  check or remove owned paths; they must fail. Structural coverage does not prove
  task granularity, feasibility or implemented behavior.
- Review: one independent configured planner challenges sequencing, scope,
  feasibility barriers and missing integration; parent alone writes and resolves
  findings in one integrated repair pass. No further research/team is needed for
  a purely local task translation unless a concrete source contradiction appears.
- Ownership: only this spec/workflow directory; approved requirements/design
  bytes, application/dependencies/CI/catalog/data and Git history remain unchanged.
- Final checks: spec grader with negative mutants, local links/JSON/scope and
  `npm run validate`. Missing lint tools stay UNVERIFIED, not silently substituted.
- Exit gate: present the task plan for review and obtain explicit implementation
  authorization; no task is complete merely because its checkbox/test name exists.

## Task-plan revision checks

- Capability: the live, unchecked Markdown task list precedes the long check
  registry. Missing or code-fenced checkboxes must fail the structural grader.
- Capability: explicit dependent tasks use the installed `impeccable` skill for
  native visual critique/audit and scoped refinement, with one initial batched
  inspection and at most one confirmation round. Final readiness requires both.
- Portability: discover skill base, selected checkout, package manifests and
  evidence destination at execution; never bake in a machine-specific location
  or silently fall back to CWD. Relative source ownership hints are not runtime
  path constants. Relocated checkout and Unicode/space-containing evidence paths
  must work; unavailable skills or unwritable output must fail explicitly.
- Native limits: Impeccable's web detector and mobile-specific guidance do not
  validate GPUI desktop. Review actual native windows against the approved
  desktop criteria; preserve independent AT/performance proof and freshness after
  refinements. A score, mock screenshot or unsupported PASS label cannot pass.
- Regression: preserve approved requirements/design hashes, existing task IDs and
  coverage, acyclic dependencies, unstarted implementation and the prior Git
  checkpoint. Only task/checker/evaluation/review/workflow artifacts may change.
- Graders: `verify.py --tasks --self-test`, including negative fixtures for hidden
  checkboxes, missing Impeccable phases, fixed machine paths and missing final
  visual dependency; bounded read-only specialist review; scope/hash checks,
  Markdown/spelling checks and `npm run validate`. Native visual quality remains
  UNVERIFIED until the future implementation and observations actually exist.

## Task-specific evaluations

Load only the evaluation file for the task being executed, not this directory as
a whole. Create later task contracts when their work is authorized.

- [Task 1.1: compatibility fixtures](evals/task-1.1.md)
- [Task 1.2: default-terminal feature boundary](evals/task-1.2.md)
