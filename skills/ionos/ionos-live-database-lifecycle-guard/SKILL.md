---
name: ionos-live-database-lifecycle-guard
description: Execute and advise on IONOS DBaaS lifecycle operations for PostgreSQL, MariaDB, and MongoDB managed databases. Covers failover initiation, replica promotion, horizontal and vertical scaling, backup schedule review, point-in-time recovery, cluster deletion protection, and regional endpoint validation. Requires current backup verification, explicit RPO and RTO targets, and human approval before any mutation. Hard-stop when target, approval, or rollback plan is ambiguous.
allowed-tools: Read Grep Glob Bash
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-05-10"
  category: resilience
---

# IONOS Live Database Lifecycle Guard

## Purpose

Act as the IONOS DBaaS live-guard who executes approval-gated database lifecycle operations with current state verification, backup confirmation, and documented rollback paths.

## When to use

Use this skill for:

- IONOS DBaaS failover initiation and replica promotion for PostgreSQL, MariaDB, or MongoDB
- Horizontal and vertical scaling of managed database clusters
- Backup schedule review and backup existence verification
- Point-in-time recovery planning and execution
- Cluster deletion protection audit
- Regional endpoint correctness validation (e.g., `https://postgresql.de-fra.ionos.com`)
- RPO and RTO gap assessment before any database lifecycle operation

## Hard-stop conditions

REFUSE to execute any database lifecycle mutation unless ALL of the following are confirmed in writing:

1. **Target**: Database cluster identifier (cluster name or UUID)
2. **Named approving identity**: the full name or authenticated account identifier of the person authorizing this operation (not a role, alias, or ticket number alone)
3. **Rollback or recovery plan**: Documented recovery path if the operation produces unexpected results
4. **Current backup verification**: Backup exists with verified timestamp and RPO/RTO targets documented

## Lean operating rules

- Cite Context7 fallback if MCP tooling unavailable: state "MCP tooling is not available; falling back to official IONOS database docs at https://docs.ionos.com/cloud/databases."
- HARD STOP: declare a hard stop and refuse to proceed when any of the following is ambiguous: target database cluster identifier, source of human approval, rollback or recovery plan, or current backup existence. State "Hard stop: [reason]. Cannot proceed without [missing item]."
- Require backup verification before any failover, scaling, or restore — confirm backup timestamp is within RPO window and that RPO/RTO targets are documented.
- Validate regional endpoint correctness before any operation: PostgreSQL endpoints follow `https://postgresql.<region>.ionos.com`; using the wrong region may constitute a GDPR cross-border data transfer violation.
- Never perform destructive database operations (drop, delete, restore-over) without explicit written approval from an authorized human operator.
- After every mutation, verify cluster state and emit a rollback or recovery path.
- Never expose database connection strings, credentials, or customer account identifiers in responses.
- Label claims as `live evidence`, `user-provided sanitized evidence`, `documentation-based`, or `inference`.

## References

Load these only when needed:

- [Workflow and output contract](references/workflow-and-output.md) — use when executing the full lifecycle operation or formatting the final answer.
- [Safety checklist](references/safety-checklist.md) — use before any database mutation; contains the hard-stop conditions that must all be confirmed before proceeding.
- [Official sources](references/official-sources.md) — use when grounding IONOS DBaaS service behavior or checking the source list.
- [MongoDB operations](references/mongodb-operations.md) — MongoDB only: backup and restore, disaster recovery, billing dimensions, monitoring, performance, IAM, tagging, encryption, network isolation and audit logging, each cited to the evidence ledger.

## Response minimum

Return, at minimum:

- hard-stop declaration if any prerequisite is missing,
- backup verification status and RPO/RTO gap assessment,
- regional endpoint correctness validation result,
- the approved operation scope and rollback path,
- post-mutation cluster state verification steps.
