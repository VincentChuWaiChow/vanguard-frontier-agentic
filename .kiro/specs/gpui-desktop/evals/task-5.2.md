# Task 5.2: installation-free Node apply preparation

## Goal and authorization

Implement only the Node producer portion after verified planner 5.1. Full Rust
framing/redaction, lifecycle, journal and native dependencies remain pending;
full task 5.2 stays unchecked. No installs, real operator targets, remote CI, push,
new dependencies or legacy rendering/policy changes. Tests use disposable fixtures.

## Ownership and downstream contract

One worker owns scripts/export-marketplace-agents.mjs, tests/export-apply.test.mjs
and narrowly required test fixtures. Parent owns shared metadata/integrity, package
validation, evals and local Git. Future Rust export_protocol consumes this exact
script-owned vfa-export/1 contract, never recomputes destinations or rendering.

Invocation is solely --apply-plan <file>; reject extra modes/selectors/options,
including help. No operation-ID flag: external engine operation binding is deferred.
Read only a bounded regular non-symlink/non-aliased plan (8 MiB maximum). Validate
shape/digest/confirmable, reconstruct original selection/options, run the SAME
planner and compare the entire canonical current plan before creating target dirs.
Never replan under changed approval. Plan digests are integrity, not authorization.

### Versioned JSONL contract fixed before implementation

All records have protocol=vfa-export/1, plan_digest=approved digest and one-based
seq. Types are apply-start, file-outcome and exactly one terminal apply-summary.
Start: file_count. Successful file record: zero-based file_index, exact planned
relative path, status=WRITTEN and final={sha256,size,mode}. Emit success only after
actual write/mode/close success, using the existing final renderer/writer.
Summary: status=SUCCEEDED or FAILED, planned, written (known emitted successes),
failed_index (null or attempted current index), unattempted, uncertain and code
(null or fixed bounded diagnostic code). No raw file contents or arbitrary error
messages in protocol. SUCCEEDED means all planned files written, uncertain=false,
failed_index=null, unattempted=0. On write failure, current effects are conservatively
uncertain, previous successful outcomes remain retained and remainder is explicitly
unattempted. No file outcome is invented for a failed current attempt.

Limit each UTF-8 record including newline to 64 KiB and total Node protocol to
8 MiB. Preflight size all success records plus reserve 64 KiB terminal capacity
before any effect; do not buffer all records. This producer limit is not proof of
the future engine's shared in-flight raw-output budget. PLAN_LIMIT/PROTOCOL_LIMIT
are explicit failures, never truncation. Await each stdout write callback and
backpressure before the next target write. Failure/EPIPE stops further writes;
failed summary is attempted only while output remains usable. Exit zero requires
a fully flushed valid success summary. Invalid preflight may emit no stdout and
nonzero diagnostic; absence/truncation is not success. No rollback/retry/deletion.

## Executable positive and negative checks

Actual CLI fixture tests compare independent expected Codex/companion bytes/modes,
force/no-skills/provider/role behavior, stable indices/sequence/counts and final
summary. Do not derive expected rendered bytes solely from production plan fields.
Changed source/catalog/script/destination/ancestor or recomputed-digest semantic
tampering rejects before writes. Invalid/oversized/nonregular/link plans and
incompatible flags fail. Deterministic disposable script-copy instrumentation may
cause real per-write failure after one success; no production fault switches.
Assert known previous effects, uncertain current file and unattempted remainder.
Closed/slow stdout, bounds and incomplete summary must never report verified
success or silently continue. Native same-user filesystem race protection and
multi-file atomicity are NOT promised; preflight/per-write checks detect changes.

## Regression, handoff and stopping

Run new apply tests, all14 planner tests, four Cursor/Kiro notice groups and actual
root validation (including coverage). Opt-in Codex marketplace installation is
UNVERIFIED, not an aggregate process-pass claim. Parent regenerates covered hashes
LAST, preserves versions/frozen approvals and commits this task separately with
both GitHub no-reply identities. Independent TypeScript/security source review;
only one parent-integrated repair pass in this shared wave. Stop if reliability
requires Rust/journal/native work, altered legacy behavior or production test hooks.

Initial status: contract fixed before apply implementation; full task pending.

### Parent-identity requirement resolved before implementation completes

Parent independently reproduced a planner gap using committed pre-apply code:
replacing the empty nested .codex/agents directory changed its inode while the
entire plan/digest remained identical. Disposable evidence: ancestor-red.json.
Authorize only the necessary additive destination.parent_chain metadata: all
existing directory ancestors, stable device/inode/type+mode identities, excluding
mutable directory timestamps/size. Full replan must detect replacement. Apply-only
parent preparation validates original bindings and identities of its own newly
created directories, with exclusive component mkdir; missing/replaced originally
bound directories and unexpected external new parents reject rather than being
adopted. Keep legacy rendering/writing unchanged. This detects changes, not
malicious same-user races or multi-file atomicity. Required regressions exercise
both stale preflight and actual per-write replacement after one known success.

Parent integration registers the planner/apply fixture tests in a new root
validate:export-contracts gate, chained into npm run validate. Generated gate
counts and covered integrity are regenerated only after worker bytes settle.
Directory root/anchor fingerprints use the same stable directory identity subset
as parent_chain; file/source fingerprints retain their existing full data.
Sibling-directory timestamp changes alone are not native identity replacement.

## Independent review and single integrated repair

The configured TypeScript reviewer (gpt-6.1-sol/high, read-only default) found two
concrete blockers in the frozen candidate. Parent reproduced both with actual
Node CLI fixtures before repair: fully resigned array-valued platforms were
accepted and bypassed strict Codex transformation; a source of exactly64MiB
rendered to67108993bytes in the disposable path and produced a confirmable plan
but apply rejected it using the source-size rather than renderer-size bound.

Parent added two focused regression tests; both failed against the candidate.
The single integrated production repair requires a string platform before object
key lookup and validates final size against MAX_RENDER_BYTES. The permanent
render-bound regression uses a copied-script-only reduced source bound that still
fits the actual exporter, while parent separately exercises the actual64MiB
boundary. No renderer, selector or legacy-write semantics were changed to fix it.
Raw RED evidence: node-findings-red.json and review-regressions-red.log.

## Final scoped producer results

| Criterion | Status | Evidence |
| --- | --- | --- |
| Strict sole-mode/current-plan apply | PASS | Fifteen actual apply tests; malformed/stale/recomputed-digest mutations and selector/options mixing reject before effects |
| Parent identity and owned directories | PASS | Parent RED old-plan gap; new replacement/missing/mode, unexpected external and apply-created parent faults reject; unrelated sibling timestamp changes stay valid |
| Exact final/legacy behavior | PASS | All14 planner fixed-oracle cases plus15 apply cases; combined notice-file run has30 passing Node tests and four notice assertion groups, zero skips |
| Outcomes and partial effects | PASS | Real first-success/second-failure fixture retains known writes, marks current effects uncertain and remainder unattempted; no rollback/retry |
| Output reliability/bounds | PASS | Real closed stdout, delayed1800-file pipe, callback/error ordering, terminal-fd failure and copied-limit boundary fixtures; nonzero/no terminal success on loss |
| Review regressions | PASS repaired | Both new focused oracles RED first; array platforms now reject with zero target effects; actual64MiB source renders67108993bytes and applies with independently expected bytes |
| Repository integration | PASS | New validate:export-contracts runs29 apply/planner cases inside full npm run validate; complete repository gate exits0 and final covered integrity matches |
| Engine regressions/dependency pins | MISS baseline only | Both full Rust runs exit101 for exact known17/11 names, solely --lib; manifests/locks/compiler unchanged, both normal engine metadata subtrees pass |
| Full/native acceptance | UNVERIFIED | Rust parser/raw budget/redaction, single-use operation authorization, journal receipts, lifecycle/native/Desktop parity and full5.2 remain pending |

### Commands and artifact binding

```bash
node --check scripts/export-marketplace-agents.mjs
node --check tests/export-apply.test.mjs
node --test --test-reporter=tap tests/export-apply.test.mjs tests/export-plan.test.mjs tests/export-cursor-kiro-skill-notice.test.mjs
npm run manifest:write:all
npm run readme-counts:write
npm run docs-data:write
python3 tests/validate-asset-integrity.py --write
npm run validate
python3 tests/validate-asset-integrity.py
```

Raw evidence in the discovered ignored target wave directory: ancestor-red.json,
node-findings-red.json, review-regressions-red.log, node-final.log,
node-findings-green.json, node-actual-boundary-green.log, regenerate.log,
npm-validate.log and final-wave-results.json. Parent's independent boundary tool
uses disposable targets and literal expected Codex bytes, not production hashes
as its only oracle. The existing fixed compatibility fixtures remain unchanged.

An initial parent combined run exposed two grader/environment issues, not more
production defects: the NEW size-bound oracle mistakenly used .agents/skills
instead of the immutable legacy .codex/skills destination, and the pre-existing
7000-file overflow fixture exhausted the nearly-full host tmpfs. Parent corrected
only the new oracle against the fixed legacy contract and reran Node temporary
fixtures under ignored target storage. No existing assertion or legacy byte/mode
oracle changed; the unchanged renderer was not adapted to a mistaken expectation.

Context7 /nodejs/node primary stream docs confirmed callback/error ordering and
drain/backpressure obligations; [Node Writable documentation](https://github.com/nodejs/node/blob/main/doc/api/stream.md)
is source guidance, not runtime proof. Actual stdout tests provide local evidence.
Pipe acceptance/flush is NOT a consumer-journal durability receipt.

One reused built-in writer (serving model/effort unobservable) implemented the
producer/tests. Configured TypeScript reviewer is gpt-6.1-sol/high, read-only
default, serving identity/effective isolation unobservable. Parent independently
reproduced all blockers, integrated one two-line production repair, added two
regressions and owns validation/generation/Git. The process and producer tracks
had disjoint files; no parallel Cargo or shared-target writes occurred.

Existing destination inspection retains its legacy64MiB bound; the actual large
rendered creation test is not a claim of arbitrary-large forced re-exports. No
malicious same-user race protection, multi-file atomicity, redacted protocol,
consumer parser correctness, automatic recovery or full-task completion claimed.
Opt-in Codex marketplace installation remains UNVERIFIED, not an aggregate pass.

Final npm pack --dry-run --json confirms the actual exporter is included and
private build/eval/workflow evidence is excluded; no publication occurred. The
real slow-pipe test uses bounded timing observations of stalled writes, not a
formal all-schedules/fairness proof. Closed-pipe/fault tests and source review are
separate evidence. Final AST update completed; HTML exceeds visualization limits
and is not native visual evidence.
