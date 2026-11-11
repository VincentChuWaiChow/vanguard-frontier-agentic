# aws-dynamodb-patterns

Language-neutral DynamoDB access-pattern design and implementation guidance.
Entry point: [SKILL.md](SKILL.md).

Includes a worked customer-orders/sparse-index model, complete CloudFormation and
AWS CLI table assets, JS/TS pagination, Java/Rust/PHP/Kotlin query excerpts, and a
recipe for other SDKs. DynamoDB Toolbox is optional and JS/TS-only.

## Command triggers

Type these as agent requests when the host exposes the skill. They are not host
commands installed by this package. Instructions live separately under
`references/commands/`; only the selected command/features and audit variant load.

| I want to… | Trigger to type | Outcome / boundary |
|---|---|---|
| Discover commands/features | `/aws-dynamodb-patterns help` | Show help and stop; no AWS inspection |
| Focus on selected features | `/aws-dynamodb-patterns select --features encryption,vectors` | Focused explanation/design; no persistent activation |
| Design a new table/access pattern | `/aws-dynamodb-patterns design --features core,indexes,encryption` | Access-pattern matrix, keys/indexes, alternatives and requested IaC/SDK code; no creation |
| Refactor an existing model or access layer | `/aws-dynamodb-patterns refactor --features core,indexes` | Classify change; compatibility, backfill/cutover/rollback plan and small code/IaC diff |
| Improve item-update safety | `/aws-dynamodb-patterns refactor --features core` | Conditional/versioned updates, upsert and retry semantics; no unnecessary migration |
| Check/plan continuous recovery | `/aws-dynamodb-patterns backup --method pitr` | Actual PITR status/window, retention and RPO/RTO gaps; no automatic enablement |
| Plan a native named backup | `/aws-dynamodb-patterns backup --method on-demand --features pitr,encryption` | Approved creation, exact ARN/source and AVAILABLE checks; asynchronous, not an exact-instant snapshot |
| Recover before an incident | `/aws-dynamodb-patterns restore --method pitr` | Approved point inside observed window; NEW table only; validation before cutover |
| Recover a named backup | `/aws-dynamodb-patterns restore --method on-demand --features pitr,encryption` | Exact source/available backup ARN, new target, dependency and application recovery checks |
| Audit code/templates offline | `/aws-dynamodb-patterns audit --mode offline --features iam,encryption,pitr` | Local/sanitized evidence; no AWS or automatic external lookup |
| Audit live settings | `/aws-dynamodb-patterns audit --mode live --features pitr,monitoring` | Scoped metadata review after read authorization; no scans/remediation |
| Plan a local performance test | `/aws-dynamodb-patterns performance-test --features core,indexes --environment local` | Bounded plan; local results cannot prove AWS performance |
| Plan a nonproduction vector test | `/aws-dynamodb-patterns performance-test --features vectors --environment nonproduction --duration-seconds 30 --max-requests 500 --max-concurrency 2` | Requested caps; target/rate/attempt/cost/stop bounds and run approval still required |
| Plan feature enablement | `/aws-dynamodb-patterns feature-plan --features pitr,streams --action enable` | Proposed desired-state changes, not application |
| Plan a supported disable | `/aws-dynamodb-patterns feature-plan --features ttl --action disable` | Review retention/expiry implications; no automatic change |
| Plan encryption configuration | `/aws-dynamodb-patterns feature-plan --features encryption --action configure` | Key ownership/settings; not a generic on/off toggle |

The real **offline selector** accepts the same syntax from the skill directory:

```bash
python3 scripts/select_mode.py /aws-dynamodb-patterns audit --mode offline --features iam,encryption
```

It emits JSON routes only; it does not audit, benchmark, enable features or grant
authorization. Follow the returned [command procedure](references/commands/README.md)
to do the selected work. Audit defaults offline; omitted features default to core,
except backup/restore which default to pitr. Their default method is PITR.

## Feature combinations

Use a bundle with `--features`; it selects scope, not AWS activation. Not every
action works with every bundle: incompatible feature-plan actions fail together.
Unselected features are not implicitly healthy or disabled.

| Focus | Feature bundle | Useful for |
|---|---|---|
| Security/access | `iam,encryption,tags,resource-policy` | Identity/resource/key boundaries and governance |
| Recovery/lifecycle | `ttl,pitr,replicas` | Expiry, recoverability and regional topology |
| Latency/capacity | `core,indexes,dax,monitoring` | Access patterns, hot keys, cache and metrics |
| Change processing | `streams,monitoring` | Consumer lag, retries and observability |
| Semantic retrieval | `vectors,embeddings,iam` | Model/precision/source mapping and authorized retrieval |

## Workflow combinations

Run **one command at a time**. These are manual steps, **not an automatic pipeline**
or a new `combo` command. Each step retains its own scope and approval gate; a
feature plan is not applied merely because a later audit is selected.

| Goal | Suggested sequence | Gate between steps |
|---|---|---|
| Design then review | `/aws-dynamodb-patterns design --features core,indexes,encryption` → `/aws-dynamodb-patterns audit --mode offline --features core,indexes,encryption` | Review actual generated artifacts; no implicit table creation or deployment |
| Refactor then validate | `/aws-dynamodb-patterns audit --mode offline --features core,indexes` → `/aws-dynamodb-patterns refactor --features core,indexes` → `/aws-dynamodb-patterns performance-test --features core,indexes --environment nonproduction` | Protect existing data/contracts; execution and test targets/bounds require separate approval |
| Business-continuity drill | `/aws-dynamodb-patterns backup --method pitr` → `/aws-dynamodb-patterns restore --method pitr` → `/aws-dynamodb-patterns audit --mode live --features pitr,encryption,monitoring` | Approve recovery point/new target and drill; test data/ACL/dependencies and measured RPO/RTO before separately approved cutover |
| Review and verify encryption | `/aws-dynamodb-patterns audit --mode offline --features iam,encryption` → `/aws-dynamodb-patterns feature-plan --features encryption --action configure` → `/aws-dynamodb-patterns audit --mode live --features iam,encryption` | Apply changes separately only if approved; live audit needs authorized read scope |
| Investigate latency | `/aws-dynamodb-patterns audit --mode offline --features core,indexes,monitoring` → `/aws-dynamodb-patterns performance-test --features core,indexes --environment nonproduction` | Review target, dataset, enforced limits, cost and stop thresholds before any run |
| Evaluate semantic search | `/aws-dynamodb-patterns select --features vectors,embeddings,iam` → `/aws-dynamodb-patterns audit --mode offline --features vectors,embeddings,iam` → `/aws-dynamodb-patterns performance-test --features vectors --environment nonproduction` | Fix model/ground truth and authorization; evaluate recall/freshness, not just self-match |

## Commands and decision guides

- [Backup](references/commands/backup.md) / [restore](references/commands/restore.md):
  lazy PITR/on-demand procedures, real CLI examples and new-table recovery gates.
- [Embedding meaning, precision and integrations](references/embedding-integrations.md):
  float32 vs base Number, coordinates vs source references, app-managed Bedrock /
  DynamoDB example and distinct S3 Vectors / Knowledge Bases flows.
- [Embedding CLI companion](references/commands/embedding.md): separately loaded
  Titan generation, native/S3 vector search and existing-KB sync/status/retrieval.
- [Data types and native vectors](references/data-types-and-vectors.md): all ten
  AttributeValue tags, ordered numeric embeddings, native vector indexes and
  SearchVectors, with optional CLI/CloudFormation examples and IAM limitations.
- [AWS CLI commands](references/commands/aws-cli.md): inspect, get/query, service pagination,
  conditional put/update/delete, optional LSI creation, online GSI and TTL/PITR.
- [LSI vs GSI](references/lsi-vs-gsi.md): choose by partition scope, consistency,
  lifecycle, size, capacity coupling and projection requirements.
- [Encryption](references/encryption.md): AWS-owned vs AWS-managed `aws/dynamodb`
  vs customer-managed KMS keys; CloudFormation/CLI selection and lifecycle risks.
- [IAM, tags and access](references/iam-and-access.md): actual IAM actions and
  table/index/stream ARNs, ABAC, resource policies and cross-account safeguards.
- [Streams and DAX](references/streams-and-dax.md): delivery/checkpoint semantics,
  consumers, cache correctness, client support, networking and service-role rights.
- [PITR and replicas](references/recovery-and-replicas.md): restore to new tables,
  current-version MREC commands and distinct MRSC requirements.
- [CloudWatch](references/monitoring.md): correct per-domain metrics/dimensions,
  statistics, alarm missing-data policy and notification verification.

The CLI cookbook's domain index groups these commands; it is not an executable
deployment script. Research uses exposed Context7 and AWS documentation MCPs,
without installing agents, scheduling background automation or making live changes.

## June / December content reminder

Review this skill every June and December; the next window starts **1 December
2026**. [Maintenance](references/maintenance.md) documents the checklist and a
read-only due-status helper. Missed reviews remain overdue; targeted edits do not
pretend to complete the full review. This does not register external notifications.

```bash
# From the skill directory; replace with the actual user/client-local date.
python3 scripts/review_due.py --as-of 2026-11-08
```

Example **agent prompts**, not shell/slash commands (host installation is separate):

```text
Use aws-dynamodb-patterns to choose LSI vs GSI for this access-pattern matrix.
Use aws-dynamodb-patterns to generate approved-target CLI query/conditional-write examples; do not execute them.
Use aws-dynamodb-patterns to compare AWS-managed and customer-managed encryption and show the CloudFormation delta.
```

No AWS operation runs by loading this skill. Examples require explicit target and
change approval. The skill lives in this repository; installation into any host's
global skill directory is a separate action, not implied by creation here.

See [provisioning](references/provisioning.md), [sources](references/sources.md),
[eval contract](evals/eval.md), and [changelog](CHANGELOG.md). Local tests do not
prove AWS runtime acceptance or universal SDK compilation.
