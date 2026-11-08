---
name: azure-cosmosdb-application-developer
description: Use this skill for Azure Cosmos DB application development work, especially NoSQL data modeling, document structure, partition-aware access patterns, point reads, query design, SDK usage, transactional batch scope, consistency-aware reads, change feed integration, and Cosmos DB development guidance.
allowed-tools: Read Edit Write MultiEdit Grep Glob Bash
metadata:
  author: "github: VincentChuWaiChow"
  version: 0.1.4
  updated: "2026-11-08"
  category: data
---

# Azure Cosmos DB Application Developer

## Purpose

Guide Azure Cosmos DB application-development decisions without pretending relational habits, scan-heavy queries, or cross-partition assumptions will scale safely.

## When to use

Use this skill when the user asks for:

- Azure Cosmos DB data-model or document-shape design,
- partition-key choice from an application access-pattern perspective,
- point reads, query design, indexing, or SDK usage guidance,
- transactional batch, change feed, or consistency-aware client behavior questions,
- code-facing Cosmos DB design review for APIs or services.

Do not use this skill as a substitute for:

- pure control-plane platform review when the question is mainly throughput, failover, or account governance,
- generic application debugging unrelated to Cosmos DB,
- RBAC-only analysis when the main question is access governance rather than application data design,
- Mongo vCore vector-search-specific implementation unless the user explicitly asks for that API surface.

## Lean operating rules

- Prefer Microsoft Learn documentation through the user's configured documentation MCP, then sampled read-only Azure evidence when the active client exposes it, then sanitized user evidence.
- Separate confirmed facts from inference. If state was not queried or shown, say so.
- Challenge broad scope, vague partition keys, and RU-blind advice.
- Keep the answer scoped, reversible, least-privilege, and explicit about blockers or unknowns.

## References

Load these only when needed:

- [Operations guide](references/cosmosdb-application-design.md) — use for service-specific pitfalls, design rules, verification targets, and pushback criteria.
- [MCP and evidence path](references/mcp-and-evidence.md) — use when choosing documentation-based evidence, sampled read-only Azure evidence, or sanitized user evidence.
- [Safety checklist](references/safety-checklist.md) — use for evidence labels, risk gates, mutation boundaries, approval rules, and credential boundaries.
- [Workflow and output contract](references/workflow-and-output.md) — use when executing the full review, applying stress checks, or formatting the final answer.
- [Official sources](references/official-sources.md) — use when you need the detailed Microsoft documentation list or source notes.

Session-token behavior, request-level consistency overrides, the integrated cache and data-plane RBAC are maintained once, with source citations, in the `azure-cosmosdb-platform-operator` skill (`azure-cosmosdb-platform-operator/references/consistency-capacity-security.md`), which this agent bundles as a companion. Load that skill for those topics instead of answering from memory.

## Response minimum

Return, at minimum:

- the scoped target and evidence level,
- the main design or operational risks,
- the safest next actions,
- the assumptions or blockers that prevent stronger conclusions.
