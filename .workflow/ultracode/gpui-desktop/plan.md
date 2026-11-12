# GPUI desktop workflow

## Objective and authority

Implement a cross-platform GPUI Kit desktop application by reusing the `vfa-tui`
engine, following requirements → adversarial analysis → design → task breakdown
→ evaluated implementation. Canonical feature artifacts live in
`.kiro/specs/gpui-desktop/`. This workflow records progress; it is not a
second specification or an authorization ledger.

Baseline: `6dda992a3dedd845731d795aaaed20d49346f57c` (`origin/develop` when
`feature/gpui-desktop-alpha` was created). Preserve the TUI, CLI, headless behavior,
catalog formats, existing installation targets, and local data.

## Current boundary

- Authorized now: repository inspection, upstream documentation research, local
  checks, requirements and analysis artifacts in the requested spec directory.
- Owned paths this phase: that directory and this workflow directory only.
- Forbidden now: application edits, dependency changes, catalog regeneration,
  user-home configuration, secrets, schedules, remote dispatch, database changes,
  commits, pushes, publication, or deployments.
- Risk now: local documentation. Later shared-engine extraction, process execution,
  and desktop distribution have a shared-surface/security blast radius.
- Product assumptions are proposed for approval, not silently treated as user
  decisions. The requirements skill requires explicit approval before design.

## Sequence and stop conditions

1. Inspect current Rust seams and verify official GPUI Kit constraints.
2. Define capability/regression evaluations before writing requirements.
3. Produce EARS requirements, then adversarial analysis and one bounded repair.
4. Request requirements approval; do not start design until received.
5. After approval, load `design-documentation`, produce and review `design.md`.
6. Load `task-breakdown`, produce traced, dependency-ordered `tasks.md`.
7. Implement approved tasks using risk-first native platform and engine probes;
   independently evaluate each task and preserve the full feature objective.

Stop the affected route on unsafe side effects, conflicting requirements,
unsupported platform claims, an unapproved phase transition, or ownership drift.
Do not label a phase complete merely because its documents exist.

## Team and execution budget

- Parent owns requirements, integration, source verification and final checks.
- `rust_core_recon`: `rust_reviewer`, configured `gpt-6.1-sol` / `high`;
  read-only repository assessment.
- `gpui_research`: `docs_researcher`, configured `gpt-6-luna` / `medium`;
  read-only primary-source research.
- Independent requirements refutation: reuse the Rust reviewer after its
  reconnaissance completes, with a new bounded read-only packet.
- Maximum three workers concurrently; no recursive delegation. One research wave,
  one review wave, one parent repair pass in the current phase.
- Roles and model/effort pins are exposed by the runtime tool catalog. Actual
  serving-model identity/effort is not independently observable. Workers inherit
  the shared workspace-write permission envelope; read-only is a task boundary,
  not a claim of separate enforced isolation.
- No persistent memory, cron, background service, API-billing fallback, or remote
  automation is needed. The existing native goal and these local files provide
  resumability for the invoked autonomous-harness workflow.

## Gates defined before requirements editing

- S1: Kiro requirements structure, unique IDs, one SHALL keyword per criterion and
  EARS triggers; a removed mandatory heading must fail the structural check.
- S2: every requirement has positive/negative evaluation scenarios and no
  unsupported upstream or baseline capability claims.
- S3: independent four-class requirements review; resolve verified findings or
  mark them as product decisions awaiting approval.
- R1: `git diff --check`, unchanged existing tracked files, only owned additions.
- R2: `npm run validate` passes; capture environmental/baseline failures honestly.
- Approval: explicit user acceptance of proposed scope before design. Missing
  approval is not implementation failure or permission to infer consent.

The structural checks prove document shape only. Native builds, rendered UI,
runtime parity, screen-reader operation, and cross-platform delivery remain
unverified until implementation and platform-specific evidence exist.

## Approved transition to design

User answer: “Approve revised requirements; proceed to design.” The exact
requirements SHA-256 is recorded in `state.json`. Requirements bytes remain frozen.
Design-only scope and checks are defined in the design contract appended to
`evals.md` before design editing. Parent owns design and integration; the reused
Rust/docs specialists supply read-only sources, then an independent `architect`
profile (configured `gpt-6-astra/high`) challenges the draft. No serving-model
identity or read-only sandbox isolation is claimed. One review wave plus one
integrated repair pass is allowed for this newly authorized phase.

No app/dependency/config/data/CI changes, code execution from upstream, install,
commit, push or publication. After validation, obtain explicit design approval
before loading task-breakdown and creating the implementation task plan.

## Approved transition to task breakdown

User answer: “Approve design; proceed to task breakdown.” Both approved file hashes
are recorded in `state.json`; neither source document is edited by task planning.
Parent owns the local translation into tasks and deterministic DAG/coverage
checks. One configured `planner` (`gpt-6-astra/high`, serving identity unobservable)
will independently refute the draft; one integrated repair pass is budgeted.
No implementation, native run, dependency resolution/install, CI mutation,
commit/push or publication is authorized by this task-planning approval.

The user then explicitly requested fanout for speed. Three bounded read-only
drafting packets covered epics1–3 (`rust_reviewer`, Sol/high), epics4–5 (`architect`,
Astra/high) and epics6–8 (`doc_updater`, Luna/medium). Parent alone assembled the
single task file/DAG and check registry. A separate `planner` (Astra/high) performs
the one independent review wave; drafting authors do not approve their own output.
These are configured runtime profiles, not observed serving identities. No worker
edits application files or shares write ownership; no recursive fanout.

## Requested task-plan revision

After rejecting the draft pending revision, the user requested visible task
checkboxes and Impeccable for visual audit/refinement without fixed file paths.
Parent revises only tasks, their evaluation/checker/review and workflow records.
Approved requirements/design and all application/dependency/config files stay
unchanged. The previously authorized Git checkpoint has already been delivered;
this revision neither starts implementation nor creates another commit/push.

The smallest route is parent-owned edits plus one bounded read-only review by the
existing UI/documentation specialist (`doc_updater`, configured Luna/medium).
Impeccable is referenced by installed skill name; paths are discovered at runtime.
Its web detector/mobile audit cannot certify GPUI desktop behavior. Native visual
audit/refinement, independent AT/performance and final-build evidence stay separate.
Checks defined before editing are in `evals.md`: visible unchecked list, ordered
visual phases, portable paths, frozen input hashes, complete DAG/coverage and
negative mutants, then local lint/scope checks and repository validation.

Exit: revised task plan ready for user review; obtain implementation approval
before dispatching any backlog tasks. No new research or implementation fanout.

## Task 1.1 authorization and boundary

The user requested “do the first task 1.1,” then per-task evals for lazy loading.
Only this task is authorized. Its active contract and evidence live in
`evals/task-1.1.md` under the spec directory; the shared eval document links to it.
Parent owns CLI/report tests and integration; one Rust implementation specialist
owns core tests/fixtures in a disjoint subtree, followed by one bounded review.
No production, dependency, catalog, Git-history or other implementation task edits.

The compatibility fixtures are implemented and their targeted checks pass. Full
C1 remains blocked by unchanged library tests rejecting the real catalog's `sap`
provider; a clean tracked-HEAD control reproduces those failures. The user then
explicitly requested “mark it as done and git commit and git push.” Mark 1.1 done
with that recorded baseline exception, without relabeling C1 green. Commit/push
only reviewed task artifacts using the repository's GitHub no-reply identity;
keep generated graph output uncommitted. Obtain separate scope approval before
repairing production behavior or starting task 1.2.

## Task 1.2 authorization and boundary

The user then requested “go for next phase.” Parent explicitly interpreted this
as task 1.2 only: default-terminal Cargo feature, narrow module/test gates and a
normal engine-dependency boundary checker. Its lazy-loaded contract/evidence is
`evals/task-1.2.md` under the spec directory; task 1.1 goldens remain frozen.
Parent owns manifests/production wiring and serialized Cargo commands; one Rust
specialist owns test gates/checker, followed by one bounded read-only review.

Core no-default builds and normal closure pass without suppressing domain tests.
The unchanged default inventory and remaining no-default domain inventory are
compared explicitly. Full suites retain the reproduced baseline failure sets;
record those failures rather than invent green regression gates. Task 1.2's scoped
work is complete with verified baseline equivalence; no new failure is waived.
No ReportRequest/ReportService extraction,
provider/Unicode fix, task 1.3, new workspace/crate, commit, push or remote CI.

## Task 1.3 authorization and boundary

The user then requested “let go for next task.” Parent scoped that request to
task 1.3 only, defining `evals/task-1.3.md` before implementation. One existing
Rust specialist owned `src/headless/` and `src/models/report.rs`; parent owned the
new contract tests, integration and serialized Cargo checks. One bounded source
review and one integrated repair pass followed, not a recursive team.

The owned request/result API separates shared report reads/calculation from CLI
printing and legacy cache writes. Explicit caller version/name, quality/provenance,
old envelopes, format/quiet behavior and main's single audit append are retained.
Domain tracing still follows the caller's subscriber. Metadata records registry
resolution, not a new assertion of complete workspace file-read evidence.

Task 1.3 is complete with passing scoped capability, compatibility, boundary and
package checks. Exact full-suite failure identities remain the accepted task 1.1
baseline (seventeen default / eleven no-default), not green C1/C2 gates. No fixture
expectation, manifest/lock, main, catalog or frozen approval changed. No task 1.4,
provider/Unicode production repair, system install, remote CI, commit or push.

## Task 1.4 authorization and boundary

The user explicitly requested task 1.4 with Ultracode and Codex Agent Teams.
Its before-implementation contract is `evals/task-1.4.md`. Parent owns the new
standalone private Desktop crate, lock/toolchain/source provenance, bootstrap,
UI/native entry points, graders and integration. One checked Rust reviewer
(configured Sol/high) provided bounded read-only facility research and a separate
source review. Incomplete disk profiles were not dispatched; serving identity
is not observable. No recursive fanout or new workflow directory.

Registry archive checksum/VCS and immutable facade bytes identify Kit's selected
release. Normal features and the engine subtree are checked separately from dev
unification. The first native-frame callback was found to execute before draw,
despite its API comment; the repaired probe requires an intervening native draw
turn, nonempty scene and Linux renderer, without claiming pixels/compositor proof.
Provenance/target mutants cover the independently found false-positive cases.

Compilation reached a missing `pkg-config` prerequisite. The user approved the
required Linux build packages, excluding WebKit/speech, but this environment's
sudo policy refused the attempted installation. No packages were installed.
Host-side installation was requested; source/graph checks continue meanwhile.
Do not mark the task done or native/UI/compile checks PASS without actual results.
No task 1.5, root workspace/TUI changes, commit, push, publication or CI dispatch.

## Ready-wave authorization: 4.1, 5.1, then 4.2

The user requested remaining unblocked tasks, then authorized implementation and
frequent per-task commits. The bounded wave is independent profile ownership
(4.1) and export planning (5.1), then the journal (4.2) after 4.1 passes. Contracts
are created lazily before each task. Two disjoint builtin workers own source/test
subtrees; parent owns shared wiring, serialized integration and Git. Configured
Rust and TypeScript reviewers (Sol/high) provide one bounded review per task,
followed by one parent repair pass. Inherited worker serving identity is unknown.

Task 1.4 remains unchecked, with no native-prerequisite retry; the user requested
leaving native verification pending. Its existing WIP was saved as checkpoint
`4b8111110a3bffd12e283577e753a6039b2ba93a`, not a completion claim. Task 4.1
passes scoped Linux process/profile tests, both feature-mode contracts, strict
clippy, packaging and isolated repository validation. Its full failure sets are
identical to the established 17/11 baseline; other native hosts remain unverified.
Task-specific evidence is in `evals/task-4.1.md`. Separate local no-reply commits
are authorized, not push, publication, native setup or the rest of the backlog.

## Task 5.1 completion

The authoritative Node planner and final renderer now serve both preview and
legacy execution. Parent verified the original exporter against independent
fixed output/byte/mode fixtures. All seventeen combined C7 cases pass, including
new hard-link alias and machine-help regression tests added for review findings.
Required integrity regeneration changed only the 41 covered task files; complete
repository validation exits zero. The scoped no-write preview is not apply-plan
or native installation acceptance. Commit this task separately, excluding the
active journal files. Task 4.2 began only after task 4.1's verified local commit.

## Task 4.2 completion and stop boundary

The isolated dedicated journal worker retains profile ownership through actual
connection close. Receipts follow COMMIT, with transactional projection/event
updates, retry identity, forward-only fixture migration and store-preservation
negatives. The one bounded Rust review found a false-success projection/retry
path; parent reproduced it, repaired consistency validation and passed all
thirteen journal tests in both feature modes. Strict clippy, boundaries, normal
packaging and repository validation pass. Full suites preserve exact 17/11
baseline failures; native and action/lifecycle integration remain unverified.

Save a separate local no-reply task 4.2 commit, then stop. No push or unrelated
backlog work is authorized by this bounded ready wave. Task 1.4 remains unchecked
and native pending as requested; no package/privilege retry or goal-status change.

## Installation-free preparation wave

Latest user authorization permits only partial slices: 1.5 engine host/report
contracts, 1.6 shared cases and 8.2 static CI guards. Full DAG dependencies and
checkboxes remain unchanged. No installs, native acceptance, remote CI or push.
Three disjoint writers; parent serializes Cargo, owns integration/evals and
separate local no-reply commits. Existing task 1.4 native deferral remains.

Task 1.6 preparation: shared fixed cases now have both consumer wrappers. Parent
verified five actual TUI tests per feature mode, both crate format traversals,
unchanged oracle/fixture bytes and both normal engine dependency subtrees. Desktop
execution/parity remains unverified. No full task-completion claim.

Task 8.2 preparation adds explicitly static-only Desktop CI and guards both
consumer path filters without changing TUI job steps. Independent review found
an alternate-shell bypass in the grader; parent reproduced RED and repaired the
root-key boundary. All 27 Node checks, 18 bootstrap-document mutants and 31
boundary self-tests pass locally. Full repository validation is running; native
jobs, license/source policy and complete task acceptance remain unverified.

Task 8.2 local repository validation completed successfully including the repaired
27-check grader. Final covered integrity was regenerated/rechecked after repair.
No external CI was dispatched and full task 8.2 remains unchecked.

Task 1.5 engine preparation now passes eight deterministic unit and two real
report integration tests in each engine feature mode. Independent review's
global-Cancel launch race was reproduced RED and repaired with synchronized cell
cancellation/admission. Both strict clippy modes, formats, targeted regression
contracts and normal verified package/archive checks pass. Full suites retain
exact 17/11 baseline failure identities; native/full task acceptance remains
unverified. Stop this bounded wave after its separate no-reply local commit; no
install, push, remote CI or extra backlog work.
