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
