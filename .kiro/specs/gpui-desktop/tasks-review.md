# GPUI Desktop task-plan review

Status: REVIEW FINDINGS ADDRESSED; READY FOR TASK-PLAN REVIEW. Tasks are proposals,
not implemented work. The user has not authorized implementation yet.

The user explicitly requested fanout. Three read-only specialists drafted bounded
foundation/runtime, safety/export and UI/delivery records; the parent integrated
and normalized IDs, dependencies and check scopes. A separate configured planner
independently challenged the assembled plan. Both approved input documents
remain frozen. Structural DAG/coverage checks are not semantic/runtime proof.

## Findings and dispositions

| Finding | Source / affected obligations | Parent disposition |
| --- | --- | --- |
| Actual blocking workers arrive after early shutdown admission | Planner; tasks2.6/3.4/4.2, D3/V6, R9.4/R9.7/R9.9 | Added focused task4.7 after real scans/journal/evidence exist; it tests actual worker cancellation, SQLite interruption/busy handling, drain and shutdown acknowledgment. Native UI depends on it; early native process admission remains intact |
| Closure checks alone do not test both resolved graphs | Planner; C1–C4, R11.1/R11.6 | Added focused task1.6 and Desktop engine_contracts, reusing independent fixtures under Desktop's lockfile. Subsequent engine contracts require the second-graph behavior run; no separately published core added |
| Readiness wording rejected all manual evidence | Planner; task8.3, C8/C12, R13.7 | Corrected negative criterion to reject unsupported PASS-only/skip evidence while accepting provenance-bound, attributed native human observations. Missing required observations still block final acceptance |
| Packaging barrier prose exceeded its dependencies | Planner; G3/task8.1 | Narrowed G3 to task7.1 workflow acceptance before packaging; AT/performance remain final gates. This preserves the useful acyclic graph without adding a package/acceptance loop |

The structural grader separately caught absent R15.7 task coverage in the initial
draft; task6.1 now explicitly tests no implicit telemetry/download/helper/network/
credential/cloud behavior at launch/browse. Parent also clarified that GPUI is
forbidden in the engine subtree, not the entire Desktop dependency graph, and
stage-scoped future targets cannot create implicit acceptance cycles.

The reviewer verified frozen hashes and ran the initial grader; its disposition
was ready for task-plan review after these fixes. Parent incorporated one bounded
repair pass and independently reran the current graders. No second independent
review or implementation success is claimed.

## Team and checks

- Drafts: rust_reviewer (`gpt-6.1-sol/high`), architect (`gpt-6-astra/high`) and
  doc_updater (`gpt-6-luna/medium`); disjoint responsibilities, no worker writes.
- Review: separate planner (`gpt-6-astra/high`), not an author of the draft packets.
- Parent owns the only write stream, task/DAG integration, fixture/check scopes,
  and final verification. Those are configured profiles; serving identities are
  unobservable and shared workspace-write is not read-only isolation.
- **PASS, structural:** both approval hashes, complete criterion/design/test/check
  mapping and acyclic topological dependencies; negative mutants exercise missing
  coverage, cycles, unknown checks, missing ownership and altered design hash.
- **UNVERIFIED, runtime:** every implementation checkbox, native process/UI/AT/
  performance/package behavior and actual tests below the spec layer.
- Native hosts, graphics sessions, installed helpers, reference hardware and human
  AT availability are execution prerequisites, not invented passing evidence or
  grounds to delete required platforms. Obtain implementation authorization next.

## User-requested visual workflow revision

The existing checkbox syntax was present but buried beneath the check registry.
Parent moved the actual unchecked task list to the first main section, preserving
existing IDs, and added tasks 7.4/7.5 plus C13 for Impeccable-guided native visual
audit and bounded refinement. Final readiness now depends on the refinement gate.

The parent read the installed `impeccable` instructions and ran its context setup;
no product/visual implementation was found and no visual audit is claimed. A
bounded read-only `doc_updater` (`gpt-6-luna/medium`, configured; serving identity
unobservable) reviewed the proposed integration. It identified the key limitation:
Impeccable's native audit reference is mobile-oriented, not evidence of GPUI
desktop conformance. The plan now excludes mobile-only checks, prohibits the web
detector as native proof, and ties desktop assessments to approved criteria and
actual native observations. Independent AT/performance gates remain mandatory.

Skill/checkout/manifest/output locations are resolved at execution, not copied
from a developer machine. Existing relative source paths remain ownership hints;
the integration owner resolves concrete worker write sets at dispatch. Relocation,
spaces/Unicode and explicit failure cases are required future implementation
checks. Source/build freshness is rechecked after refinement and packaging.

This is one task-plan revision, not a new architecture or implementation phase.
The structural grader adds rejection fixtures for missing/fenced/buried boxes,
missing visual phases, fixed install/output locations and bypassed final visual
dependencies. Those fixtures cannot prove runtime relocation or visual quality.
