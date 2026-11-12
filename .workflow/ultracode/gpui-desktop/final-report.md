# GPUI Desktop progress report

## Current ready-wave status

Tasks 4.1, 5.1 and 4.2 are complete in scoped checks. See the lazy-loaded task evals for current
commands/evidence; the older phase reports below are historical. Full Rust
regression gates remain non-green with unchanged 17/11 baseline failures.
Task 1.4 remains unchecked and native verification pending, as requested.

The user authorized frequent separate local commits using Raishin and GitHub
no-reply author/committer identity; no push. A clearly labelled task 1.4 WIP
checkpoint was saved first without claiming native acceptance. Task 4.1 changed
only profile ownership, required engine/consumer dependency wiring and evidence.
Two disjoint builtin workers plus bounded configured Rust/TypeScript reviews
(`gpt-6.1-sol/high`) are the smallest useful parallel route; actual worker model/effort
are inherited and unobservable. Parent owns final checks, integration and Git.


## Current task 1.4 implementation

Status: BLOCKED ON HOST PREREQUISITES; NATIVE BUILD/UI/EXECUTION INCOMPLETE.
The user authorized task 1.4 only with Ultracode and Codex Agent Teams. Read the
lazy-loaded [task 1.4 eval](../../../.kiro/specs/gpui-desktop/evals/task-1.4.md)
for the original criteria, source evidence, exact checks and outstanding work.

The new private Desktop crate selects the exact Kit release, separate lockfile,
numeric compiler pin, shared no-default engine, Root/assets bootstrap, an actual
callback-state UI test and explicit main-thread native smoke target. Provenance
records registry checksums, source identities and facility limits. Deterministic
pin/target mutants and actual normal dependency checks pass; these are not compile
or launch evidence. The native smoke source review caught an early-frame false
positive, repaired without claiming rendered pixels or native AT/IME/clipboard.

Compilation stopped in Fontconfig's build script because `pkg-config` is absent.
The user approved the required build packages, but sudo refused installation.
No package mutation occurred; host-side installation is requested. UI callbacks,
strict Desktop clippy/all-target build and real X11/Wayland launch remain owed.
The task stays unchecked. Three consecutive goal turns confirm the same missing
prerequisite and refused install; no worker/build remains live. The goal is blocked
until host-side installation permits the remaining checks. No host is certified.
No later task, frozen approval/TUI change, Git delivery or publication is implied.

## Previous task 1.3 implementation

Status: TASK 1.3 DONE; KNOWN FULL-SUITE BASELINE FAILURES REMAIN. The user's “let go
for next task” was scoped to task 1.3 only. Read the active
[task 1.3 evaluation](../../../.kiro/specs/gpui-desktop/evals/task-1.3.md) for the
owned files, exact commands, independent review and remaining limits; load later
task evals only when their tasks start.

`ReportRequest` and `ReportService` now return owned payload, explicit producer,
quality/provenance and diagnostics without owning printing, global logging or
index writes. The existing CLI wrapper retains rendering, best-effort persistence
and its original public entry point; main's audit append is unchanged. Existing
domain tracing follows the caller's subscriber, not a service-installed console
logger. Workspace quality is registry resolution, not complete scan-read proof.

Both feature modes' new contracts pass (eleven default, ten no-default), including
fixed goldens, nine oracle mutations, invalid/mixed inputs, non-UTF8 catalog paths,
owned lifetime, all report routes, real SQLite WAL sidecars and subprocess stream/
subscriber probes. All original test identities remain enabled in the correct
feature modes. Full suites retain exactly seventeen default / eleven no-default
baseline failures; they are still MISS, not new failures or green release gates.
Strict clippy, package verification and packaged no-default build pass.

Parent integrated one Rust implementation specialist (configured Sol/medium) and
one source reviewer (configured Sol/high), with no recursive fanout or serving-
identity claim. No new manifest/dependency, catalog, approved requirement/design,
old fixture, global config or main edit. No task 1.4 or native proof is implied.
The user subsequently authorized normal commit/push of task 1.3 using the GitHub
no-reply author/committer identity. Actual commit and remote-SHA verification are
reported after delivery, not preclaimed here. Develop, generated graph output,
history rewrites, later implementation and remote CI dispatch remain excluded.

## Previous task 1.2 implementation

Status: TASK 1.2 DONE; KNOWN FULL-SUITE BASELINE FAILURES REMAIN. The user's “go for
next phase” was scoped to task 1.2 only. Read the active
[task 1.2 evaluation](../../../.kiro/specs/gpui-desktop/evals/task-1.2.md) for the
exact changed files, default/no-default checks and limits; do not load every task
eval as a single document.

The default terminal feature preserves the existing CLI/TUI surface; opting out
builds a presentation-free library. Pure domain tests remain active, and the
default test inventory is unchanged. Package verification from a controlled
candidate-source snapshot and both feature modes' strict clippy pass. The full
default suite retains the same seventeen baseline failures; no-default retains
the eleven non-presentation failures, not new failures. The checkbox records
completion of scoped boundary work with that evidence, not green full regression
gates. No task 1.3, provider repair, Git delivery or native/GPUI feasibility success
is implied.

## Previous task 1.1 implementation

Status: TASK 1.1 DONE BY EXPLICIT USER ACCEPTANCE. The user requested completion
and commit/push after the reproduced baseline failures were reported. The checked
box accepts the scoped fixture work; the full C1 gate still has seventeen existing
failures and is not marked green. The rest of the backlog remains unauthorized.

Read the active [task 1.1 evaluation](../../../.kiro/specs/gpui-desktop/evals/task-1.1.md)
for exact files, commands, results, review disposition and limitations. The shared
eval document is an index rather than an inline copy of task-specific details.

Library contracts and actual CLI/report/SQLite fixtures pass, as do fmt, strict
clippy and repository validation. The unchanged library suite has seventeen
failures caused by catalog `sap` entries rejected by the existing provider enum;
a clean HEAD archive reproduces the identical failure set. Production Rust,
manifests, lockfile, catalog and approved requirements/design remain untouched.
The known non-ASCII registry-expansion bug is also documented, not silently fixed.
No production repair or next-task implementation occurred. Commit/push is now
explicitly authorized; actual commit/remote verification is reported in chat after
delivery, not preclaimed here. Generated AST graph artifacts stay uncommitted as
separate local tooling output, not feature code or visual acceptance evidence.

## Historical task-planning report

The following records describe the earlier planning-only phase, not the current
task's authorization or completion status.

Status: REVISED TASK PLAN READY FOR REVIEW; IMPLEMENTATION NOT STARTED. Requirements
and design remain approved. The requested checkbox visibility, Impeccable visual
audit/refinement and portable execution-path revision is incorporated. The complete
cross-platform feature remains unfinished; approval to implement is still needed.

## Deliverables and scope

- [Task plan](../../../.kiro/specs/gpui-desktop/tasks.md): dependency-ordered,
  owned-path implementation records with positive/negative checks, native admission,
  both lock graphs, honest evidence/approval gates and timebox estimates.
- [Task review](../../../.kiro/specs/gpui-desktop/tasks-review.md): four independent
  findings and parent repairs; no independent rereview or runtime proof claimed.
- Spec checker validates task dependencies/coverage/check references and rejects
  hidden/missing boxes, missing visual phases, fixed machine/install paths and a
  bypassed final visual gate. Both approved source documents remain unchanged.
- Only spec/workflow artifacts changed. No application, dependency, catalog,
  database, CI, installation or Git-history changes occurred in this revision.
  The earlier authorized planning checkpoint was committed/pushed separately.

## Checks

| Criterion | Status | Evidence |
| --- | --- | --- |
| Frozen approved requirements/design | PASS | Recorded user answers and SHA-256 match both exact files |
| Task DAG/coverage | PASS structural | `rtk proxy python3 .kiro/specs/gpui-desktop/verify.py --tasks --self-test`: 44 tasks, all 121 criteria, D/V/C anchors and acyclic topological dependencies |
| Checkbox visibility | PASS source structure | All 52 epic/task boxes unchecked; live checklist is the first main section, outside code fences; no renderer-specific pixel claim |
| Negative spec tests | PASS | Five requirements, five design and fourteen task mutants rejected; each task mutant must change its input |
| Independent review | PASS process | Initial planner findings repaired; bounded UI specialist review identified mobile-only Impeccable scope, now explicitly excluded as desktop proof |
| Scope/whitespace/JSON | PASS | Owned spec/workflow changes only; approval files byte-identical to checkpoint; no implementation or checked task |
| Path portability | PASS spec only | Spec grader passes from unrelated CWD after copying to a Unicode/space-containing temporary root; future app/helper/evidence relocation remains UNVERIFIED |
| Repository validation | PASS | `npm run validate` through `rtk proxy bash` exits 0; revision log ends with `OK: QA cluster eval green` |
| Spell/Markdown lint | PASS revision scope | Temporary codespell passes owned changed text/code; offline markdownlint reports zero issues across all ten owned Markdown files |
| Feature/runtime/native proof | UNVERIFIED | Every implementation task remains unchecked; no native UI/AT/performance/package proof fabricated |
| Implementation authorization | UNVERIFIED | Latest authorization was task breakdown plus planning fanout, not code/dependency/remote side effects |

## Fanout and integration

At the user's explicit request three independent bounded read-only tracks ran:

- Engine/runtime/domain: rust_reviewer, configured `gpt-6.1-sol/high`.
- Journal/trust/export safety: architect, configured `gpt-6-astra/high`.
- Native UI/delivery translation: doc_updater, configured `gpt-6-luna/medium`.

Parent alone integrated paths, IDs, dependency/check scopes and the final write
stream. A separate planner (`gpt-6-astra/high`) challenged the combined plan. There
were no parallel file writers or recursive agents. Configured profile identities
are known from the tool catalog; serving identity/effort is unobservable, shared
workspace-write is not read-only isolation, and cost/speed savings are not measured.
One integrated repair pass was used. Concrete task/role packets are selected anew
at implementation dispatch, never inferred permission to launch the whole backlog.

## Impeccable and portable paths

Tasks 7.4/7.5 use the installed skill by name for native visual critique/audit and
scoped polish, followed by at most one confirmation round. C13 defines scene,
theme/window/scale/state evidence, impact-ranked findings and final-build freshness.
Final readiness depends on the visual cycle; unresolved P0/P1 blocks acceptance.
Refinement cannot silently reuse old AT, performance or package observations.

The installed context launcher ran successfully and found no product/visual
implementation. This planning revision creates no visual authority documents or
UI code. The skill's mobile native references provide applicable heuristics only;
its web detector/overlays do not validate GPUI. Desktop conformance, real native
AT, performance and rendered evidence remain separate future obligations.

Skill base, implementation/data checkout, Cargo manifests and evidence destination
are discovered or explicitly selected at execution. Existing relative source paths
remain task ownership hints, not runtime constants. Missing/ambiguous locations,
unwritable output and relocation/Unicode failures cannot silently fall back.

Parent owned edits and final checks; one existing `doc_updater`, configured
`gpt-6-luna/medium`, provided a bounded read-only review. Actual serving identity
is unobservable; read-only was a task boundary within the shared workspace sandbox.
This was the smallest useful route: one narrow specialist challenge, no extra
research, implementation workers, recursive fanout or repeated polish loop.

## Changed files in this revision

- `.kiro/specs/gpui-desktop/tasks.md`
- `.kiro/specs/gpui-desktop/tasks-review.md`
- `.kiro/specs/gpui-desktop/evals.md`
- `.kiro/specs/gpui-desktop/verify.py`
- `.workflow/ultracode/gpui-desktop/plan.md`
- `.workflow/ultracode/gpui-desktop/state.json`
- `.workflow/ultracode/gpui-desktop/final-report.md`

## Next action

Review the revised task plan and explicitly authorize implementation if accepted.
Requirements/design remain approved and frozen; implementation is unstarted.
The previously proposed start was compatibility baseline, feature boundary, pure report
service and pinned bootstrap, shared-contract second-graph runner, runtime/native
feasibility; independent exporter planning can run where write sets permit.
No required platform or workflow is removed to get a passing smaller product.

## Authorized Git checkpoint

The user subsequently requested `git commit and git push` for the current planning
artifacts. This publishes the checkpoint on `feature/gpui-desktop-alpha`, not
`develop`, and does not approve implementation or resolve the requested task-plan
revision. Actual commit/remote SHA verification is reported in chat after delivery;
this record does not preclaim a Git outcome before its own commit exists.

Pre-push `npm run validate` and the spec checker passed. Temporary codespell 2.4.3
and markdownlint-cli2 0.24.0 were downloaded into `/tmp` with tool approval; no
repository/system dependency files or global tool installs changed. Codespell
passed both default repository scope and explicitly named hidden owned files.
The literal Markdown command reported 13 issues only in generated CHANGELOG.md;
existing CI/npm lint configuration already excludes that file. The unchanged
CI exclusions plus the new hidden specs/workflow documents passed across 6198
Markdown files with zero issues. No lint configuration or changelog was edited.

## Ready-wave export planner evidence

Task 5.1: PASS canonical protocol/final renderer, no-write/invalid-input cases,
fixed legacy bytes/modes and all seventeen C7 CLI regressions. Parent reproduced
and repaired the review's unplanned hard-link overwrite and machine-help stdout
findings. Required asset integrity and full repository validation pass. Actual
apply-plan/native execution remain UNVERIFIED and task 1.4 is still pending.
The task 4.1 local commit is `5bbe9bf921f9799e9e6a62fa2403a1a45845de91`,
with both no-reply identities verified. No push has occurred in this ready wave.

## Ready-wave journal evidence and final boundary

Task 4.2: PASS thirteen journal tests per feature mode, profile/core/report
contracts, real SQL and COMMIT rollback, retry/readback/reopen, isolated legacy-v4
WAL sidecar preservation and real forward migration rollback. Parent reproduced
and repaired the independent Rust review's inconsistent projection/retry success
path; eight actual SQL mutation classes now fail closed without history repair.
Both strict clippy modes, normal engine closures, package bytes and complete
repository validation pass. AST updated; HTML graph remains above its size limit.

Full Rust regression remains MISS with exact unchanged seventeen default and
eleven no-default failures. Native/ACL/MSRV, caller redaction, action/recovery and
real lifecycle integration remain UNVERIFIED. Details and exact commands are in
`evals/task-4.2.md`; no empty/missing fixture or native skip is treated as PASS.

The one implementation worker and reused configured Rust reviewer were sufficient
after task 4.1. Task 5.1 local commit is
`1c9331192d7294b62fae6435d5b57774e408bfbf`; both no-reply identities verified.
Commit the journal separately, then stop this bounded wave. No push, additional
backlog task or native setup. Task 1.4 remains unchecked/native pending.

## Installation-free preparation wave

Three disjoint implementation tracks delivered shared cases (local commit
3425d4f9), static CI guards (5f57ac4d) and the engine-only persistent report host.
Parent independently reproduced and fixed one CI grader bypass and one global
cancellation race. Local capability, strict lint, package and repository checks
pass; full Rust suites remain non-green with exact known 17/11 failures.

Lazy evidence: task-1.5.md, task-1.6.md and task-8.2.md in the spec's evals
directory. Those are partial preparations, not completion of the full tasks.
Native GPUI/foreground, Desktop behavioral execution, source/license policy,
other native platforms and full lifecycle/journal integration remain unverified.
Three built-in writers have unobservable serving model/effort; configured Rust
and TypeScript read-only reviewers use gpt-6.1-sol/high, with serving identities
unobservable. Parent serialized Cargo and owns integration and no-reply commits.
No installation, external mutation, push or additional backlog work was performed.

## Offline scenario follow-up

PASS: real poisoned-registry cleanup and concurrent admission/Cancel scenarios,
ten runtime tests per feature mode, all targeted contracts, both formats/strict
clippy modes, exporter14/static27/notice4 groups and relocated core5 per mode.
MISS: exact existing 17/11 full failures; no additional failed Cargo target.
UNVERIFIED: native/foreground/Desktop execution, old MSRV, other OS and opt-in
Codex marketplace installation E2E. Aggregate process passes cannot prove an
opt-in file actually executed; the older evidence row is clarified accordingly.

Only two appended Rust unit tests plus lazy eval/workflow records changed;
production, old tests, fixtures, manifests/locks and frozen approvals unchanged.
One built-in writer and one configured Sol/high Rust reviewer (serving identities
unobservable), parent actual graders/integration. No production repair needed.
No installs, push, remote CI or global settings changes; full task boxes unchanged.

## Task 2.1 process/helper preparation checkpoint

PASS Linux: actual typed argv/cwd/environment launches, finite std-only helper,
stdin gate, exits/streams, invalid inputs, redacted Debug and strict JSON encoding;
six process plus three legacy subprocess cases per feature mode. Both formats,
strict clippy, targeted contracts, normal package bytes and both normal engine
metadata subtrees pass. MISS: exact known17/11 full Rust failures, only --lib.
UNVERIFIED: native Windows/macOS, descendant ownership, safe reader lifecycle/output
budgets, trust grants, Desktop parity and full2.1 acceptance.

One reused built-in writer (serving identity/effort unobservable), configured
Sol/high read-only Rust review and parent integration/actual graders. Process
source, helper/tests and lazy eval/workflow records only in this separate commit;
Node5.2 code/covered metadata are excluded. Full task checkboxes remain unchanged.

## Task5.2 Node apply preparation

PASS: fifteen apply scenarios, all fourteen planner oracles, four Cursor/Kiro
notice groups, one thirty-test combined run, new twenty-nine-case root export
gate and full repository validation. Reproduced/repaired strict platform coercion
and renderer-size validation; actual64MiB source boundary verifies final bytes.
Ordered bounded outcomes await stdout delivery; actual partial writes/pipe failure
never become verified full success. Existing legacy oracles unchanged.

MISS: exact known17/11 Rust full failures, only --lib. UNVERIFIED: native task1.4,
full2.1/5.2 dependency gates, Rust protocol/framing/redaction, operation/journal
authorization and Desktop/native acceptance. Full task checkboxes unchanged.

One additional reused writer plus configured Sol/high TypeScript source review
(serving identities unobservable), parent independent RED/GREEN and integrated
two-line repair. Combined with the disjoint process track this is two writers
and one review wave, no recursive fanout or extra implementation wave.
No system/dependency installs, real operator targets, push or external CI.

### Current wave changed paths

- tools/vfa-tui/src/subprocess/mod.rs
- tools/vfa-tui/src/subprocess/executor.rs
- tools/vfa-tui/src/subprocess/program.rs
- tools/vfa-tui/tests/process_contracts.rs
- tools/vfa-tui/tests/support/process_helper.rs
- scripts/export-marketplace-agents.mjs
- tests/export-apply.test.mjs
- package.json
- catalog/asset-integrity.json
- docs/_data/catalog.yml
- .kiro/specs/gpui-desktop/evals/task-2.1.md
- .kiro/specs/gpui-desktop/evals/task-5.2.md
- .kiro/specs/gpui-desktop/evals.md
- .kiro/specs/gpui-desktop/tasks.md
- .workflow/ultracode/gpui-desktop/plan.md
- .workflow/ultracode/gpui-desktop/state.json
- .workflow/ultracode/gpui-desktop/final-report.md

Exact grader commands/results are in the two lazy task evaluations. Final npm
dry-run includes the exporter and excludes private build/evidence artifacts.
Slow-pipe observations are local bounded timing evidence, not every interleaving.
Scope is READY FOR REVIEW; the overall native feature remains unverified.
