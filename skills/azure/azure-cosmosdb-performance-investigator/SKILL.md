---
name: azure-cosmosdb-performance-investigator
description: Use this skill for Azure Cosmos DB performance investigation, especially RU spikes, query latency, throttling, hot partitions, indexing inefficiency, partition-skew analysis, request-charge profiling, diagnostic-log review, and evidence-driven remediation planning.
allowed-tools: Read Grep Glob WebFetch
metadata:
  author: "github: VincentChuWaiChow"
  version: 0.1.3
  updated: "2026-06-05"
  category: data
---

# Azure Cosmos DB Performance Investigator

## Purpose

Investigate Azure Cosmos DB performance pathologies with evidence-first profiling instead of lazy “add more RUs” advice.

This skill is for deep performance work across:

- RU inefficiency and unexpected request-charge spikes,
- query latency and scan-heavy query behavior,
- hot partitions and partition-key skew,
- throttling, retry inflation, and client-perceived latency,
- indexing gaps and poor query/index alignment,
- container, partition, and workload-level profiling,
- diagnostic-log and metrics-backed remediation planning.

## When to use

Use this skill when the user asks for:

- slow Azure Cosmos DB queries or workload latency,
- high RU cost or suspicious request-charge behavior,
- 429 throttling analysis,
- hot partition or partition-skew investigation,
- indexing or query-performance tuning,
- a step-by-step Cosmos DB profiling plan.

Do not use this skill as a substitute for:

- initial data-model design when the main problem is greenfield schema modeling,
- pure account/platform governance review when performance is incidental,
- generic application debugging unrelated to Cosmos DB workload behavior,
- vector-search-specific Mongo vCore tuning unless the user explicitly asks for that API surface.

## Lean operating rules

- Prefer Microsoft Learn documentation through the user's configured documentation MCP, then sampled read-only Azure evidence when the active client exposes it, then sanitized user evidence.
- Separate confirmed facts from inference. If state was not queried or shown, say so.
- Challenge throughput-first fixes that ignore partition skew, query scans, indexing, or client retry inflation.
- Keep the answer scoped, reversible, least-privilege, and explicit about blockers or unknowns.

## References

Load these only when needed:

- [Operations guide](references/cosmosdb-performance-investigation.md) — use for service-specific pitfalls, design rules, verification targets, and pushback criteria.
- [MCP and evidence path](references/mcp-and-evidence.md) — use when choosing documentation-based evidence, sampled read-only Azure evidence, or sanitized user evidence.
- [Safety checklist](references/safety-checklist.md) — use for evidence labels, risk gates, mutation boundaries, approval rules, and credential boundaries.
- [Workflow and output contract](references/workflow-and-output.md) — use when executing the full investigation, applying stress checks, or formatting the final answer.
- [Data profiling playbook](references/data-profiling-playbook.md) — use when you need the detailed step-by-step profiling sequence.
- [Official sources](references/official-sources.md) — use when you need the detailed Microsoft documentation list or source notes.

## Response minimum

Return, at minimum:

- the scoped target and evidence level,
- the main performance pathologies observed or still unproven,
- the safest next profiling or remediation steps,
- the assumptions or blockers that prevent stronger conclusions.
