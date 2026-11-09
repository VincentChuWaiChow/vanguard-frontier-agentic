---
name: azure-cosmosdb-pattern
description: "Design, implement, refactor, audit and debug Azure Cosmos DB for NoSQL access patterns in any language: partition/id routing, indexing/RUs, consistency/session tokens, transactions/ETags, TTL/change feed, Entra RBAC, service/customer-managed encryption, vectors/embeddings, ARM/CLI provisioning and continuous/periodic backup/restore. Select lazy command/feature routes and bounded performance plans. Other Cosmos APIs require their own workflow. Not authorization to access Azure, deploy, restore or run load."
allowed-tools: Read Grep Glob Bash(python3 scripts/select_mode.py:*) Bash(python3 scripts/review_due.py:*)
metadata:
  author: "github: VincentChuWaiChow"
  version: "1.0.0"
  updated: "2026-11-09"
  category: data
---

# Azure Cosmos DB Pattern

| Always-on contract | Required behavior |
|---|---|
| API scope | Default **Azure Cosmos DB for NoSQL** only. MongoDB/vCore, Cassandra, Table, Gremlin, PostgreSQL and Fabric aren't interchangeable; route to their own verified docs/skill. |
| Safety | Loading/selection never executes Azure work. Confirm tenant/subscription/principal, account/database/container, region and permitted evidence before live reads. Exact approval separately covers every mutation, restore, deployment, backfill or load run. |
| Preserve | Keep existing IaC ownership and data. No reset, implicit createIfNotExists/adoption, broad item export, credentials inspection, key listing, feature activation or hidden scans. |
| Evidence | Separate sourced facts, assumptions, local checks, metadata readback and runtime proof. Return proportionate PASS/MISS/UNVERIFIED; no universal SDK or performance claims. |
| Lazy loading | Read the selected command and selected feature paths only. Links are indexes, not recursive preload instructions. Never bulk-read siblings, examples, source ledger or evals on load. |

## Commands

When supported by the host, ask `/azure-cosmosdb-pattern <command>`; `$azure-cosmosdb-pattern`
or plain language may be used instead. This package registers no host command.
Real local selector: `python3 scripts/select_mode.py <command>` emits JSON paths only.

| Command | Load selected procedure |
|---|---|
| help | [Help](references/commands/help.md); show commands and stop |
| select | [Feature selection](references/commands/select.md) |
| design | [Design](references/commands/design.md) |
| refactor | [Refactor](references/commands/refactor.md) |
| audit | [Audit](references/commands/audit.md), then only offline or authorized live variant |
| performance-test | [Bounded performance plan](references/commands/performance-test.md) |
| feature-plan | [Feature lifecycle plan](references/commands/feature-plan.md) |
| backup | [Protection plan](references/commands/backup.md), then continuous or periodic |
| restore | [Recovery plan](references/commands/restore.md), then selected native recovery method |

Audit defaults offline; features default core except recovery defaults backup.
Help stops without source/environment inspection. Unknown inputs fail closed.
No combo command or automatic command chaining; each manual step retains its gates.

## Domain routes

| Selected task / feature | Read only when relevant |
|---|---|
| End-to-end implementation sequence | [Workflow](references/workflow.md) |
| core: partition/id, tenant/access-pattern model | [Modeling](references/modeling.md), [runtime](references/runtime-contract.md) |
| indexes / capacity: range/composite/spatial, RUs/hot keys | [Indexing and capacity](references/indexing-and-capacity.md) |
| iam: control/data plane, Entra identities, keys | [Access control](references/access-control.md) |
| encryption: service-managed vs Key Vault CMK | [Encryption](references/encryption.md) |
| ttl: expiry, concurrency, migration | [Runtime](references/runtime-contract.md) |
| backup: protection overview | [Recovery](references/recovery.md), then only chosen method |
| replication: regional/consistency resilience | [Replication](references/replication.md) |
| change-feed: consumers, Functions, deletion limits | [Change feed](references/change-feed-and-integrations.md) |
| vectors: JSON types/native policy/search | [Types and vectors](references/data-types-and-vectors.md) |
| embeddings: model/precision, source refs, Azure integrations | [Embeddings](references/embedding-integrations.md) |
| monitoring: selected-domain observability | [Metrics](references/monitoring.md) |
| provisioning: ARM and az CLI assets | [Provisioning](references/provisioning.md), selected asset only |
| Existing language/SDK or REST | [Adapters](references/language-adapters.md), matching section only |
| CLI cookbook | [Azure CLI](references/commands/azure-cli.md) |
| Time-sensitive claims / research | [Dated sources](references/sources.md) |
| Due review / edits | [Maintenance](references/maintenance.md) / [eval contract](evals/eval.md) |

At substantive use (not help), run `scripts/review_due.py --as-of YYYY-MM-DD`
using the actual client-local date. Review every June/December, initially due
**2026-12-01**. If due/overdue, load Maintenance and remind the user. Never advance
the ledger on load or from a partial update. No calendar/background notification.
