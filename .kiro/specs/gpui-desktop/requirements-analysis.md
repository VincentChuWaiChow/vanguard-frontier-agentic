# Requirements analysis

Status: USER REQUESTED REQUIREMENTS REVISION; CONTEXT7 UPDATE READY FOR REVIEW.
Eight concrete findings in the initial draft were incorporated in one parent
repair pass. The subsequent Context7-grounded revision is recorded below.
Approval has not been granted;
this is not a claim of formal consistency or working application behavior.

The invoked `analyse-requirements` spelling resolves to the installed
`analyze-requirements` skill. Its structural preconditions are applied here.
Kiro's native Analyze Requirements / SMT facility is not exposed by the current
Codex tools; this is the skill's manual adversarial-review route, not solver proof.

## Method and convergence gate

Review [requirements.md](requirements.md) for wrong detail level, ambiguity,
inconsistency and incompleteness against the source evidence in
[research.md](research.md) and the scenarios in [evals.md](evals.md).

For each concrete finding, record requirement IDs, the conflicting/reachable
scenario, severity and disposition. Product choices remain proposed for explicit
approval; they are not resolved by an automated continuation or a reviewer vote.

Structural grading, semantic review, approval and native runtime acceptance are
separate gates. No design/task/implementation phase is started by this review.

## Findings and dispositions

| Finding | Class / severity | Evidence and reachable case | Parent disposition |
| --- | --- | --- | --- |
| F1: missing authoritative export plan | Completeness / high | R8.2 demanded paths/conflicts, but exporter dry-run returns selected names only (`scripts/export-marketplace-agents.mjs:623–659`) | R8.2 and scope now explicitly permit a backward-compatible authoritative plan contract; Rust must not duplicate exporter rules. R8 evaluation includes destination conflicts |
| F2: preflight mistaken for atomic writes | Ambiguity / high | Existing exporter documents residual symlink race exposure (`scripts/export-marketplace-agents.mjs:275–285`) | R8.5, R8.9–R8.10 and the proposed concurrency boundary distinguish observed preflight checks from atomicity. Detected mid-export changes stop further writes with partial/unknown effects; concurrent malicious same-user edits are not claimed safe |
| F3: stale session execution trust | Completeness / high | A trusted checkout's scripts/config may change at the same canonical path | Execution Manifest definition and R15.2/R15.9 bind reusable trust to executable content; changed/unestablished inventory requires new trust. Exact inventory remains a design deliverable |
| F4: cancellation hides partial writes | Completeness / high | Export may write files before `CANCELLED`, `TIMED_OUT` or `INTERRUPTED` | R8.7 now covers every non-verified-success outcome with known changes and explicit uncertainty, no replay or destructive rollback. Cancellation-after-first-write is a negative scenario |
| F5: audit failure after action success | Inconsistency / high | Actual completion can be known even if its persistence fails; current writer ignores SQL errors before flush acknowledgment (`tools/vfa-tui/src/persistence/writer.rs:199–230`) | R9.2 publishes observed outcome separately from Audit Persistence Status. R10.9–R10.11 preserve known result, expose evidence loss, block new actions and retry only deduplicated evidence persistence |
| F6: queued/cancelling source switch | Ambiguity / medium | Source could change while a job is nonterminal but not literally running; obsolete-result discard could erase terminal evidence | R1.5–R1.6 now distinguish presentation results from original operation/audit evidence and cover every nonterminal state |
| F7: partial governance looks healthy | Completeness / high | One workspace succeeds while another/policy input fails; stale data affects dependent scores | R6.8–R6.10 add absent/invalid policy states, dependent stale status and PARTIAL aggregate labeling with expected/observed counts and exclusions |
| F8: trivial performance workload | Wrong detail / medium | An empty fixture and cheap repeated queries could satisfy initial sample language | R14 references a fixed acceptance protocol: exact entity distribution, corpus, event pacing, concurrent pressure, sample boundaries, raw durations and nearest-rank percentile calculation |

These are specification repairs, not implementations of the new guarantees. The
independent reviewer inspected the initial draft; the parent checked the repaired
clauses against the findings. No second independent review or SMT run is claimed.

## Structural grader correction

The first negative test found that an evaluation-table checker could mistake the
specification regression row `R1` for the feature acceptance row `R1`. The checker
now limits matching to the feature acceptance section and requires exactly one row
per requirement. Removing only a feature evaluation row is rejected even when the
similarly named specification regression row remains. This is evidence about the
checker, not evidence that feature requirements are implemented.

## Criteria status

- **PASS — S1:** structural checker accepts the current document and rejects five
  deliberately malformed variants. See `verify.py --self-test`.
- **PASS — S2:** all requirement groups have positive/negative scenarios; baseline
  limitations, upstream facts and product assumptions are separated in research.
- **PASS — S3 process:** independent four-class review ran; all eight known findings
  have explicit parent dispositions above. This does not prove absence of all
  natural-language defects.
- **UNVERIFIED — S4:** explicit user approval of the requirements and proposed
  product decisions has not been received.
- **UNVERIFIED — S5:** design, task traceability and implementation acceptance are
  later gates and have not started.
- **UNVERIFIED — formal/native evidence:** no Kiro SMT result, desktop/core code,
  native matrix, AT/performance measurements or packaging proof exists yet.

## Decisions to approve before design

Approve or revise the scope/target matrix, checkout-backed delivery, full existing
engine workflows, reference-machine budgets, explicit concurrency/trust model and
private unsigned-alpha acceptance in [requirements.md](requirements.md).

Source revision/checksum pinning, exact execution manifest, exporter plan/result
compatibility, OS process-tree behavior and preservation of data/CLI contracts are
mandatory design/implementation obligations, not assumptions that they already work.

## User-requested Context7 revision

The user selected “Revise requirements before design” and requested Context7 MCP.
No platform/feature/delivery changes were specified, so those product decisions
remain proposed, not approved and not silently replaced.

- R9.9 adds runtime-context lifetime acceptance: GPUI executors are not a substitute
  for existing Tokio process/persistence service requirements.
- R13.9 rejects tests that only reproduce labels/metadata while the application
  handler is disconnected; both interaction and resulting state must be checked.
- R16.8–R16.9 separate framework support, component support, headless interaction,
  pixel rendering and native visual acceptance. A Windows headless-renderer skip
  cannot stand in for a successful Windows native visual check.
- The Context7 source record in research explicitly identifies indexed/live
  version drift. A stale `0.3.8` snippet does not override the live `0.3.12` guide
  or establish compatibility with an unchecked release revision.

This revision changes explicit acceptance/test constraints, not the proposed
feature inventory. It was parent-reviewed against retrieved primary sources; no
additional independent review or native execution is claimed. Design remains
unstarted until the revised requirements receive explicit approval.
