---
name: oci-nosql-pattern
description: "Design, implement, refactor and audit Oracle NoSQL Database Cloud Service access patterns in any language. Use for shard/primary keys, SQL/secondary indexes, continuation, same-shard transactions, consistency/version/TTL, OCI IAM/encryption, replicas, metrics, Terraform/DDL/OCI CLI, embeddings boundaries, bounded performance plans and customer-owned export/import recovery. Lazy command/feature routes; not Oracle SQL Database, native vector/PITR enablement or permission for live access."
allowed-tools: Read Grep Glob Bash(python3 scripts/select_mode.py:*) Bash(python3 scripts/review_due.py:*)
metadata:
  author: "github: VincentChuWaiChow"
  version: "1.0.0"
  updated: "2026-11-09"
  category: data
---

# OCI NoSQL Pattern

| Always-on rule | Required behavior |
|---|---|
| Scope | Oracle NoSQL Database Cloud Service (NDCS), not Oracle Database/Autonomous SQL, on-prem snapshots or DynamoDB |
| Safety | No cloud, credentials, installs, data reads/writes, export/import or load on skill load; preserve current IaC owner/data |
| Authorization | Before live metadata reads confirm principal, tenancy, region, compartment, table and permitted evidence; writes/load/recovery need exact separate approval |
| Evidence | Separate request, proposed plan, local checks, control-plane readback and application proof; label facts, assumptions and UNVERIFIED |
| Loading | Read only selected command/features and selected audit/recovery method; do not recursively read links or sibling guides |
| Language | Use existing SDK/language; no forced ORM or native capability invented to match another provider |

Recognize `/oci-nosql-pattern <command>` only when the host supports skill discovery;
`$oci-nosql-pattern` or plain language may also select it. No host registration.
The real local selector emits paths, not execution:
`python3 scripts/select_mode.py /oci-nosql-pattern <command> [options]`.

| Command | Load only selected route |
|---|---|
| help | [Help](references/commands/help.md); show commands and stop |
| select | [Feature selection](references/commands/select.md) |
| design | [Access-pattern design](references/commands/design.md) |
| refactor | [Compatibility and migration](references/commands/refactor.md) |
| audit | [Audit](references/commands/audit.md), then selected offline/live variant |
| performance-test | [Bounded plan](references/commands/performance-test.md), not a run |
| feature-plan | [Lifecycle plan](references/commands/feature-plan.md), not application |
| backup | [Backup](references/commands/backup.md), then selected migrator/sdk method |
| restore | [Restore](references/commands/restore.md), then selected migrator/sdk method; NEW target only |

| Task/feature | Read only when selected |
|---|---|
| core | [Model](references/modeling.md) and [runtime](references/runtime-contract.md) |
| indexes | [Secondary indexes](references/indexes.md); no LSI/GSI classification |
| iam | [OCI access](references/access.md) |
| encryption | [Managed versus dedicated CMEK](references/encryption.md) |
| ttl | [Expiration](references/ttl.md) |
| replicas | [Global Active Tables](references/replicas.md), not backup |
| monitoring | [Metrics](references/monitoring.md) |
| vectors | [Types/native capability boundary](references/types-and-vectors.md) |
| embeddings | [Generation/source/retrieval integration](references/embedding-integrations.md) |
| backup | [Customer recovery boundary](references/recovery-boundary.md), not every method |
| Provisioning requested | [Terraform/DDL/CLI](references/provisioning.md), then selected asset |
| Language requested | [Adapters](references/language-adapters.md), matching section only |
| Current capability/source question | [Dated sources](references/sources.md) |
| Due review or skill edits | [Maintenance](references/maintenance.md) or [eval contract](evals/eval.md) |

Audit defaults offline; recovery defaults migrator. Unknown/unsupported options fail
closed. `vectors` is informational; native ANN activation and `--method pitr` fail.
Bundles/manual sequences select scope only, never persistent activation or a chain.
At substantive use run `scripts/review_due.py --as-of YYYY-MM-DD` with the actual
client-local date. Review every June/December; next **2026-12-01**. Due/overdue:
load Maintenance and remind the user; never roll forward or schedule notifications.
