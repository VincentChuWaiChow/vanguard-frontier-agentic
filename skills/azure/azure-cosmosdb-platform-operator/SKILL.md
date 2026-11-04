---
name: azure-cosmosdb-platform-operator
description: Use this skill for Azure Cosmos DB platform operations and design review, especially accounts, databases, containers, partition-key design, throughput and RU posture, consistency choices, indexing, throttling, multi-region replication, private connectivity, and Cosmos DB evidence-guided discovery.
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: 0.1.3
  updated: "2026-06-05"
  category: platform
---

# Azure Cosmos DB Platform Operator

## Purpose

Review and guide Azure Cosmos DB platform posture without hand-wavy assumptions about partitioning, consistency, throughput, or multi-region behavior.

## When to use

Use this skill when the user asks for:

- Azure Cosmos DB account, database, or container operational review,
- partition-key, consistency, throughput, or indexing decisions,
- RU cost, throttling, hot partition, or multi-region tradeoff analysis,
- private endpoint, failover, or platform control-plane posture questions,
- Cosmos DB documentation-grounded discovery or sampled read-only evidence gathering.

Do not use this skill as a substitute for:

- application code implementation details unless they directly affect platform posture,
- generic landing-zone or network architecture when Cosmos DB is incidental,
- RBAC-only analysis when the real problem is identity governance rather than database platform design,
- vector-search-specific Mongo vCore implementation unless the user explicitly asks for that API surface.

## Lean operating rules

- Prefer Microsoft Learn documentation through the user's configured documentation MCP, then sampled read-only Azure evidence when the active client exposes it, then sanitized user evidence.
- Separate confirmed facts from inference. If state was not queried or shown, say so.
- Challenge broad scope, vague partition keys, and RU-blind advice.
- Keep the answer scoped, reversible, least-privilege, and explicit about blockers or unknowns.

## References

Load these only when needed:

- [Operations guide](references/cosmosdb-platform-operations.md) — use for service-specific pitfalls, design rules, verification targets, and pushback criteria.
- [MCP and evidence path](references/mcp-and-evidence.md) — use when choosing documentation-based evidence, sampled read-only Azure evidence, or sanitized user evidence.
- [Safety checklist](references/safety-checklist.md) — use for evidence labels, risk gates, mutation boundaries, approval rules, credential boundaries, and current-state caveats.
- [Workflow and output contract](references/workflow-and-output.md) — use when executing the full review, applying stress checks, or formatting the final answer.
- [Official sources](references/official-sources.md) — use when you need the detailed Microsoft documentation list or source notes.

## Response minimum

Return, at minimum:

- the scoped target and evidence level,
- the main design or operational risks,
- the safest next actions,
- the assumptions or blockers that prevent stronger conclusions.
