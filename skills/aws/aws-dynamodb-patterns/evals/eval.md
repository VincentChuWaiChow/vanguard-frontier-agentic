# Eval contract: aws-dynamodb-patterns 1.0.0

## Extension contract: 1.8.0 (defined before implementation)

Add real router design/refactor subcommands and separate lazy command files, with
README triggers/combos, help/index and two compact SKILL table rows. Design must
start from access patterns and produce key/index alternatives plus requested IaC/
SDK examples; refactor must distinguish application item updates, mutable config,
index lifecycle and key/model migration, with preservation/backfill/cutover tests.
No new migration runner, flags, AWS action, installs, data/schema mutation or host
registration. Defaults stay core and routing-only, no eager sibling loading.

Parent owns scoped skill edits/report. One reused read-only domain specialist,
research then review (two turns, no fan-out), one bounded parent repair. Graders:
extend actual mode tests for new slash/CLI routes and rejection of execution flags;
README trigger parser must cover both; all previous 90 tests and new checks,
metadata/links/whitespace/compactness and independent current-source review.
Versioned defaults and all existing commands/assets must remain compatible.
Live migration, AWS acceptance and host loading remain UNVERIFIED. Scoped git
publication follows standing user request only after staged checks pass.

User added backup/restore before publication: include separate lazy backup and
restore commands, PITR default and explicit pitr/on-demand method selection.
One additional exact read-only backup steward researches/reviews this added scope
in at most two turns; no parallel writers or nested delegation.
Both default to the pitr feature route, remain nonexecuting selectors and reject
execution flags/unknown methods. Workflows need current recovery-window/backup
evidence, new-target-only restore, actual CLI examples, exact authorization,
RPO/RTO, KMS/dependency checks, restored-settings validation and controlled cutover.
Use fake AWS to test the new command examples and failure guards; never execute
backup/restore or alter real continuity posture during skill creation.
Recovery modes must not implicitly pull the combined PITR/replica procedure via
the generic pitr feature mapping; selected method/target pages supply that scope.
Other modes retain their existing feature mapping. Test both method isolation and
absence of this broader indirect eager load.

## Documentation contract: 1.7.1 (defined before implementation)

Add README human trigger/feature-bundle/manual workflow-combination tables; every
literal trigger must parse through the unchanged router. Make SKILL.md a <=90-line,
<=900-word tabular dispatcher; move its detailed workflow intact to a lazy
reference with rebased links. Preserve always-on authorization/data/privacy gates,
help terminal behavior, June/December reminder and per-command selective loading.
No new runtime flags, command chaining, cloud work or claimed measured savings.
Parent owns README/entrypoint, new workflow reference, docs tests/version/changelog;
one bounded read-only preservation review. Graders: new README/loading tests,
existing suites/links/metadata, exact pre/post deep-workflow comparison and diff
check. No host/runtime or cost proof is inferred from literal word reduction.

## Extension contract: 1.7.0 (defined before implementation)

Baseline 7e3f593. User explicitly requests Brownfield-style references/commands/
organization, separate command files, /aws-dynamodb-patterns audit and offline/live
audit paths with lazy loading. Parent owns relocation/router/nav/test edits in this
skill and one workspace report. No unrelated cleanup, host registration/install,
live AWS audit/load/mutation, credential or runtime config/memory changes.

Required evidence: all command instructions live under references/commands/, one
file per command plus separate audit/offline.md and audit/live.md; compact entry
dispatch table; no combined all-command workflow. Slash-prefix syntax is accepted
by the real local selector and documented as host-dependent skill invocation,
not falsely registered. Audit defaults offline; --mode live selects only live
transport, --evidence repo/live-metadata stays compatible, conflicting flags fail.
Neither selector performs AWS work. Live audit workflow must gather authorized
metadata, not scan items or mutate; offline must use only local/sanitized evidence.

Grader: extend actual modes CLI tests for slash/default/aliases/conflicts and exact
non-eager read sets; fake-AWS execute live metadata snippet including missing-target
and wrong-account guards; existing 79 checks, metadata/version/links/JSON/whitespace,
preservation checks and independent review. Content moves must preserve working
AWS examples and relative links. Historical reports remain historical snapshots.
Host discovery, actual live results and measured token savings remain UNVERIFIED.

One reused exact specialist review, no nested agents; parent performs integration
and one bounded repair if needed. Scoped publication follows the user's standing
request after validation. Completion requires inspecting the final tree and actual
selector output, not merely changing tests to match moved paths.

## Extension contract: 1.6.0 (defined before implementation)

Baseline b3c09ce. Add a small offline command selector for select, audit,
performance-test and feature-plan, plus lazy mode/audit/performance/change guides.
Feature selection is per invocation, not persistent configuration or an AWS change.
Reuse existing feature CLI guides; preserve all previous schemas, helper behavior
and tests. Parent owns edits in this skill and one workspace report. One reused
read-only specialist researches/reviews in at most two turns, no fan-out.

Acceptance: actual Python CLI help/JSON output; selected-only, deduplicated existing
reference paths; clear repo-audit and bounded performance-plan defaults; unknown
features/actions and invalid budgets fail closed; incompatible feature changes
(including encryption disable) reject. Selector performs no cloud/subprocess,
reference-content reads or writes. Guides separate configuration evidence from
runtime correctness, client from service latency, throttling/retries/skew, vector
recall/freshness and stop/cost bounds. No invented native slash registration or
claim that selecting a mode audits/benchmarks/enables anything by itself.

Grader: new evals/modes.test.py (actual CLI plus import-level filesystem guards),
existing 71 checks, metadata/JSON/links, source-backed review and whitespace.
Local routing is not measured host loading/token savings or AWS proof. Existing
optional vector strict lint E3039 remains an explicit compatibility MISS. No live
audit/load/feature change, package install, scheduler, config or memory edits.
Scoped git commit/push follows the user's continuing publication request only
after validation. Parent resolves any defect in one bounded repair wave.

## Extension contract: 1.5.0 (defined before implementation)

Baseline 7e5c9e7. Preserve the already-published embedding lessons and injected
example; add one lazy CLI companion, one Titan request asset and offline tests.
Parent owns this skill and one workspace report. Reuse one read-only DynamoDB
specialist for bounded source research/review (two turns, no fan-out); simplify
new code inline, without speculative abstractions. No cloud calls, installation,
resource creation/deletion, scheduling, memory or runtime config changes. Scoped
git commit/push is explicitly authorized after checks.

Pass signals: entry point routes theory and commands separately, with no command
body or eager include; CLI cookbook and theory link only to the selected companion.
Examples cover Titan embedding output, distinct native DynamoDB/S3 Vectors request
shapes, and KB ingestion/status/retrieval with explicit targets and mutation labels.
Execute the actual Bash examples with a fake AWS binary; assert request payloads,
fixed model/dimensions, missing-target and invalid-vector failures, and no resource
creation/deletion. Never substitute JSON fixtures for embedding generation.
Run all existing 65 tests, metadata/JSON/links and whitespace checks; preserve old
table/vector/encryption/SDK assets. Strict optional vector lint E3039 stays an
explicit compatibility MISS. Offline checks do not prove effective IAM, SDK/service
acceptance, RAG quality, host selective loading or measured token savings.

Parent resolves test/review failures in one bounded repair wave. Graders: new
evals/embedding-commands.test.py, existing Python/Node suites and static lazy-route
checks. Report PASS/MISS/UNVERIFIED; commands do not execute on skill load.

## Extension contract: 1.4.0 (defined before implementation)

Baseline: b059949. Scope: embedding meaning/float precision and coherent AWS
integration guidance, one injected-client Bedrock-to-DynamoDB example, local tests
and a workspace report. Parent owns edits; one existing read-only DynamoDB
specialist supplies current-source research and a bounded review. No cloud calls,
new services, installations, config/memory edits or background scheduling. Preserve
default table, vector fixtures, encryption and prior language examples. The user's
requested scoped git commit/push is a separate delivery step after verification.

Completion checks (parent fixes failures):

1. Explain coordinates vs identifiers/source references; dimensions vs precision;
   base Number vs f32 index; finite range/significant digits, rounding, overflow,
   underflow/zero-vector policy and float16/binary compatibility limits.
2. Distinguish ordinary S3 objects, S3 Vectors and native DynamoDB indexes. Show
   app-managed Bedrock+DynamoDB and documented Knowledge Bases/S3 Vectors flows;
   do not invent DynamoDB as a supported managed Knowledge Bases backend.
3. Executable injected-client example uses the same fixed embedding configuration
   for writes/searches, validates dimensions/numbers, preserves explicit source
   references and conditional writes, and performs no work at import time.
4. Deterministic tests exercise f32 boundary/rounding and invalid inputs, actual
   Bedrock request/response and DynamoDB wire contracts using fakes, and fail
   before item writes/searches on invalid embeddings. Guidance explicitly covers
   authorization, versioned source/chunk lineage, model migrations, async event
   deduplication/deletions and retrieval evaluation. No runtime quality claims.
5. Existing 55 checks, metadata/links/JSON/whitespace pass; strict optional vector
   lint E3039 remains an explicit compatibility MISS, not repaired or hidden.

Graders: new evals/embedding-flow.test.py, existing package/domain/Python/Node
suites, source-backed specialist challenge and git diff --check. New adversarial
prompt is a saved regression case, not a generated-response reliability result.
At most two specialist turns, depth one, one bounded repair wave; no broad pilot.

## Extension contract: 1.3.0 (defined before implementation)

Scope: a June/December content-review reminder and current DynamoDB data-type /
native-vector guidance with verified examples. Preserve all pre-existing 1.2.0
working-tree edits and reports (baseline HEAD da6e500 is not a clean snapshot).
Parent owns new/targeted skill edits and one report in the existing workspace.
One fresh-context DynamoDB specialist researches/reviews; no external calendar,
scheduler, live AWS calls, package install, commit/push or memory/config changes.

Completion requires:

1. Entry-point reminder and deterministic local due-status command use June and
   December calendar windows; missed deadlines stay overdue until a completed
   review is recorded. Initial next review 2026-12-01, not six months after November.
2. Dates and reviewer/source checks are explicit; a targeted update is not falsely
   recorded as a full skill audit. No background notification is claimed.
3. All current AttributeValue families are distinguished from vector representation;
   native vector indexes/SearchVectors are not confused with ordinary Query/GSI or
   an external OpenSearch integration. Primary docs and current SDK/CLI gaps recorded.
4. Verified JSON/CLI examples, limitations, IAM/tenant controls, cost/accuracy and
   consistency are covered without adding a vector index to the default table.
5. Maintenance date boundaries, vector dimensions/types/request shapes, bad cases
   and mocked CLI calls pass deterministic tests; existing suites still pass.
6. Version/changelog, source links/navigation and preserved 1.2.0 files agree.

Graders: new `evals/data-vectors-maintenance.test.py`, existing package/command/
operations/create/JS suites, skill metadata, JSON/link/whitespace and applicable
local schema checks. Parent resolves failures; unrun SDK compilation, IAM/runtime
search, recall/load, and external reminders remain UNVERIFIED. Maximum three
delegated turns, depth one and one bounded review repair; no broad paired pilot.

## Extension contract: 1.2.0 (defined before implementation)

Scope: research through **Context7 and AWS documentation MCP**, then add grouped
guidance and example commands for IAM action/resource scopes, table/index/stream
tags and ABAC, resource policies, Streams, DAX, PITR, multi-Region replicas and
CloudWatch. Clean baseline `da6e500`. No live AWS calls, auth/config changes,
installations, schedules, persistent-memory updates or remote agent dispatch.
The invoked autonomous-harness skill does not authorize those unrelated actions.

Parent owns all edits within this skill and one report in the existing workspace.
Two fresh-context read-only specialists research security and operations; bounded
review follows, at most six delegated turns total, depth one, one repair wave.
Do not treat offline TOML model/sandbox settings as observed runtime identity.

### Required completion evidence

1. Both MCPs actually return relevant documentation; source record identifies
   exact URLs and significant current-service findings, not invented lookup claims.
2. IAM matrix maps API use to actual IAM actions and correct table/index/stream/
   cluster/other resources, including transaction permission subtleties and
   account-level actions; policy examples are narrowly scoped and parseable.
3. Tagging and resource-policy guidance covers actual supported resources,
   propagation/ABAC boundaries, cross-account rules, revision preconditions and
   no implicit lockout bypass; commands are grouped separately.
4. Streams and DAX explain delivery/order/retention and cache consistency, client
   support and separate identity/network/service-role requirements, with commands.
5. PITR and replicas include restore-to-new-table, actual recovery window,
   per-Region/configuration verification, MREC/MRSC constraints and safe examples.
6. CloudWatch commands use documented metric names/dimensions/statistics; alarm
   target, missing-data and notification wiring are explicit, not claimed live.
7. Domain navigation, version/changelog and source links agree; existing default
   table/encryption/SDK assets and earlier runtime contracts remain unchanged.
8. Existing regression suite plus new fake-CLI and policy/permission negative
   fixtures pass. All command groups are exercised offline; specialist review has
   no unresolved high-impact defect. Live authorization/restore/replication/cache/
   alarm delivery and new response-level reliability remain UNVERIFIED.

### Graders and failure owners

Parent runs `python3 evals/operations.test.py`, existing package/command/create/JS
suites, metadata/JSON/link checks and `git diff --check`; expected exit 0. Every
new domain must have a substantive guide plus matching captured fake-CLI action,
not only a heading keyword. IAM negative fixtures reject table/index scope errors,
wildcard principals/actions and invalid transaction IAM names. Primary docs and
domain reviewers grade semantics; static tests do not prove IAM permission success.
Stop claims on missing documentation, unknown mutable outcomes or failed checks.

## Extension contract: 1.1.0 (defined before edits)

User scope: add practical commands, an LSI/GSI decision guide, and AWS-managed vs
customer-managed encryption guidance. Baseline: clean worktree at `e950ceb`.
Parent owns changes inside this skill plus one extension report under the existing
skill workspace. One fresh-context domain reviewer; at most one repair follow-up.
No live AWS requests, new installs or runtime changes. The user subsequently
authorized git commit and git push after completion, scoped to this extension;
other publication or host installation is not authorized.

- PASS requires linked CLI examples for inspection, key reads, base/GSI/LSI queries,
  service pagination, conditional put/update/delete, index creation/readiness and
  backup settings, with explicit target and read/write boundaries.
- PASS requires an LSI/GSI comparison and concrete decision cases: same vs different
  partition keys, consistency, create-only LSI, 10 GB collection constraint,
  throughput coupling, projections, quotas and neither-index redesign cases.
- PASS requires all three encryption choices (including AWS-owned default), CFN/CLI
  examples, key identity/readback, KMS permissions/cost/lifecycle and old-backup risk.
- Default table/CLI asset and SDK examples must remain unchanged. Version/changelog,
  source links, local resource links and regression expectations must agree.
- Graders: `python3 evals/commands.test.py` (fake CLI only, parse actual arguments,
  JSON and examples), existing package/JS/fake-create suites, `cfn-lint` on actual
  template and generated CLI/CFN variants, metadata/JSON/whitespace checks, and a
  bounded specialist challenge. All must exit 0; cloud acceptance stays unverified.

The original creation contract below is retained as historical context.

Defined before implementation. Local, additive skill work; no AWS execution,
installation, runtime configuration, publication, commit, or push is authorized.
Parent owns integration and all writes. Initial working tree was clean.

## Capability gates

1. Valid skill metadata, concise workflow, resolving local resource links, no
   personal paths or host-specific tool assumptions.
2. Access-pattern-first design compares single-table and multiple tables; explains
   keys, sparse indexes, consistency, pagination, skew, write correctness, capacity,
   operational security, and forward-only migration.
3. CloudFormation and AWS CLI examples describe the same composite-key table/GSI.
   CLI separately enables TTL/PITR and waits for table creation. Examples do not
   execute automatically or silently adopt existing resources.
4. Language-neutral request contract plus JS, TS, Java, Rust, PHP, Kotlin examples
   and an extension recipe for other languages. Toolbox stays optional and
   version-scoped. No claim of compiling every language.
5. Primary-source documentation and Context7 retrieval are recorded; inaccessible
   inspiration is identified, not fabricated. Original synthesis, not copying.
6. Adversarial answers reject empty-page pagination termination, strong GSI reads,
   unrestricted scan/filter design, unconditional overwrites, cross-tenant cursor
   reuse, all-language Toolbox claims, and unapproved deployment.

## Graders and expected pass signals

- `python3 .../skill-creator/scripts/quick_validate.py skills/aws-dynamodb-patterns`:
  valid metadata, exit 0. Parent fixes failures.
- `python3 skills/aws-dynamodb-patterns/evals/verify.py`: JSON/YAML and table/index
  equivalence, local links, examples and negative fixtures, exit 0. Static/local
  evidence only; does not certify AWS acceptance or design quality.
- `cfn-lint skills/aws-dynamodb-patterns/assets/table.cfn.yaml`: exit 0, no findings.
- `bash -n skills/aws-dynamodb-patterns/assets/create-table.sh`: exit 0.
- `python3 skills/aws-dynamodb-patterns/evals/create-table.test.py`: fake-CLI tests
  prove target guards, sequencing and fail-closed behavior, exit 0; never AWS calls.
- `node --test skills/aws-dynamodb-patterns/evals/query-pages.test.mjs`: adversarial
  mocked pagination and request invariants pass without AWS credentials/network.
- `git diff --check -- skills/aws-dynamodb-patterns skill-workspaces/aws-dynamodb-patterns`:
  exit 0. Inspect new files too, because untracked files are absent from git diff.
- Paired pilot: three prompts in `evals.json`, with-skill and no-skill answers from
  fresh-context agents; explicit assertion grading and generated review viewer.
  Grouping cases by arm is correlated, not an independent reliability experiment.
- Independent domain review: no unresolved high-impact correctness defect.

## Regression guards and failure conditions

- Only the new skill and its `skill-workspaces/aws-dynamodb-patterns/` review
  artifacts may change; generated Graphify refresh is permitted by root guidance.
- No live cloud command, package installation, or existing skill rewrite.
- Stop claims at observed evidence: uncompiled SDK snippets and undeployed AWS
  templates remain unverified at runtime. Do not invent pass@3, pass^3, telemetry,
  causal improvement, or universal SDK compatibility from one paired pilot.

## Bounded execution plan

Research two read-only tracks (domain traps; upstream inspiration) while parent
retrieves Context7 docs and authors the package. Then two grouped pilot arms and
one domain-review follow-up. At most six delegated turns total, three workers at
once, depth one, one bounded integration repair. No competing orchestration root;
use the repository's existing skill-workspaces convention for the handoff.
