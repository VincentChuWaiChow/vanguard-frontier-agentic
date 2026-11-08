---
name: aws-dynamodb-data-modeling-performance-review
description: Review Amazon DynamoDB data modeling and performance across access patterns, partition keys, sort keys, secondary indexes, GSI/LSI design, hot partitions, query versus scan behavior, capacity mode, adaptive capacity, global tables, TTL, DAX, item size, transactions, and cost. Use when DynamoDB correctness, latency, scaling, or cost depends on table design. Review only; not for building, refactoring, backup/restore, vectors or provisioning, which belong to aws-dynamodb-patterns.
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.5"
  updated: "2026-11-08"
  category: data
---

# AWS DynamoDB Data Modeling Performance Review

## Purpose

Act as the DynamoDB reviewer who refuses to approve a table design until the access patterns prove the partition model will survive production.

## When to use

Use this skill for:

- DynamoDB table design, partition key, sort key, GSI, LSI, hot partition, capacity, query, scan, or global table review
- NoSQL data model design for serverless or high-scale AWS applications
- DynamoDB latency, throttling, cost spike, adaptive capacity, or index-backfill investigation
- TTL, streams, transactions, DAX, large item, many-to-many, or time-series pattern review

## Lean operating rules

- Prefer current AWS documentation tools for service behavior. Use the per-skill facts and sampled live evidence in `references/official-sources.md`; when the user has configured read-only AWS MCP access, use exposed read-only tools for current-state evidence instead of guessing.
- Separate confirmed facts from inference. If state was not queried or shown, say so.
- Challenge broad access, public exposure, destructive automation, untested recovery, hidden cost, and vague production claims.
- Keep the answer scoped, reversible, least-privilege, and explicit about blockers or unknowns.
- Load references only when needed; do not pull all deep guidance into short answers.

## References

Load these only when needed:

- [Workflow and output contract](references/workflow-and-output.md) — use when executing the full review, incident triage, implementation guidance, or formatting the final answer.
- [Safety checklist](references/safety-checklist.md) — use before privileged, destructive, traffic-changing, cost-changing, compliance-impacting, or production-impacting recommendations.
- [Official sources](references/official-sources.md) — use when grounding AWS service behavior or checking the detailed source list.
- [DynamoDB Access Patterns and Capacity Guide](references/dynamodb-access-patterns-capacity.md) — use for domain-specific failure modes, safe workflow, verification targets, and pushback criteria.
- [Consistency, indexing and replication decisions](references/consistency-indexing-replication.md) — use for GSI vs LSI (including when to add no index), strong vs eventual reads, MREC vs MRSC global tables, cross-Region transactions, and DAX consistency risk.
- [Evidence ledger](references/evidence-ledger.md) — use when a claim must be traced to its AWS source or re-verified; cite its `[DDB-Exx]` IDs for material service claims.

## Response minimum

Return, at minimum:

- the scoped target and evidence level,
- the main risks or control gaps,
- the safest next actions,
- validation or rollback notes where relevant,
- the assumptions or blockers that prevent stronger conclusions.
