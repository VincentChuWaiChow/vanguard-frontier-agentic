---
name: aws-dynamodb-patterns
description: "Design, implement, refactor, audit and debug Amazon DynamoDB access patterns in any language. Use for keys/indexes, single/multi-table models, IAM/tags/resource policies, encryption/KMS, TTL/PITR, backup/restore, replicas, Streams/DAX, native vectors/embeddings, Bedrock/S3 integrations, CloudFormation/CLI, bounded performance-test plans and content-freshness reviews. Selective command/feature routes load on demand. Not permission to access AWS, change features or run load."
allowed-tools: Read Grep Glob Bash(python3 scripts/select_mode.py:*) Bash(python3 scripts/review_due.py:*)
metadata:
  author: "github: VincentChuWaiChow"
  version: "1.8.0"
  updated: "2026-11-08"
  category: data
---

# AWS DynamoDB Patterns

Select the smallest relevant route. Do not recursively load linked guides, siblings,
examples or evals. Instructions/examples are not proof of runtime correctness.

## Always-on boundaries

- Before live reads confirm account/role, Region, table/index and allowed evidence;
  require exact approval for writes, deployment, backfill, deletion or load tests.
- Never inspect credentials, print customer items or execute examples on skill load.
  Preserve the current IaC owner/data; no reset/recreation or hidden hot-path scans.
- Use the existing language/SDK; do not default to an ORM, Toolbox or single-table.
- Separate facts, assumptions, local checks and runtime evidence. Return selected
  scope, findings and PASS/MISS/UNVERIFIED proportionately; never claim bulletproof.

## Command routes

Recognize `/aws-dynamodb-patterns <command>` when host discovery supports it;
Codex may use `$aws-dynamodb-patterns`, or plain language. No host registration is
performed. Real selector: `python3 scripts/select_mode.py <command>` (paths only).

| Command | Read only this route |
|---|---|
| help | [Help](references/commands/help.md); show commands and stop |
| select | [Selected features](references/commands/select.md) |
| design | [Table/access-pattern design](references/commands/design.md) |
| refactor | [Existing-model/code updates](references/commands/refactor.md) |
| backup | [PITR/on-demand protection](references/commands/backup.md), then only the selected method |
| restore | [New-table recovery](references/commands/restore.md), then only the selected method |
| audit | [Audit](references/commands/audit.md), then only the chosen offline/live variant |
| performance-test | [Performance test](references/commands/performance-test.md) |
| feature-plan | [Feature plan](references/commands/feature-plan.md) |

Audit defaults offline. Live mode is an authorization-gated metadata workflow;
selection is neither approval nor execution. Unknown commands fail; no fallback
from help into source/environment inspection. Combinations are manual, not chains.

## Task routes

| Task | Load only when needed |
|---|---|
| Broad data-access design/implementation and validation sequence | [Workflow](references/workflow.md), then selected domain detail |
| Keys, relationships, access patterns, single/multi-table choice | [Modeling](references/modeling.md) |
| LSI/GSI selection and constraints | [Indexes](references/lsi-vs-gsi.md) |
| Query/write correctness, cursors, retries, transactions, migration | [Runtime contract](references/runtime-contract.md) |
| Attribute types, native vector schemas/search | [Data types/vectors](references/data-types-and-vectors.md) |
| Embedding meaning/precision, Bedrock/S3/KB architecture | [Embedding integrations](references/embedding-integrations.md) |
| IAM, tags/ABAC, resource policies | [Access](references/iam-and-access.md) |
| Key ownership, encryption selection/lifecycle | [Encryption](references/encryption.md) |
| TTL/PITR, recovery, replicas | [Runtime TTL](references/runtime-contract.md) or [recovery](references/recovery-and-replicas.md), as selected |
| Stream consumers or DAX cache | [Streams/DAX](references/streams-and-dax.md) |
| Metrics/alarms | [Monitoring](references/monitoring.md) |
| Table CloudFormation/CLI provisioning | [Provisioning](references/provisioning.md), then selected asset |
| Language-specific SDK or optional Toolbox | [Adapters](references/language-adapters.md), matching section only |
| Core AWS or embedding CLI examples | [Core CLI](references/commands/aws-cli.md) or [embedding CLI](references/commands/embedding.md) |
| Current claims/source provenance | [Sources](references/sources.md) |
| Due review or skill edits | [Maintenance](references/maintenance.md) or [eval contract](evals/eval.md) |

At substantive use (not help), run `scripts/review_due.py --as-of YYYY-MM-DD` with
the actual client-local date. Review every June/December; if due/overdue load
Maintenance and remind the user. Never advance the ledger on load or claim a full
review from a targeted update. This is not a background/calendar notification.

## Repository references

Load these only when needed:

- [Operations reference](references/operations.md) — one-page backup and restore, disaster recovery, billing dimensions, monitoring, performance, IAM, tagging, encryption, network isolation and audit logging.
- [Workflow and output contract](references/workflow-and-output.md) — which workflow to load and the answer shape.
- [Official sources](references/official-sources.md) — evidence ledger, port record and primary AWS pages.
- [Safety checklist](references/safety-checklist.md) — before any live read or proposed change.
