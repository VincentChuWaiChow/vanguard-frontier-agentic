---
name: azure-waf-reliability-review
description: "Review Azure workload reliability against the Well-Architected Framework Reliability pillar: availability targets, AZ/region topology, health monitoring, data resilience, deployment safety, and chaos testing."
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: 0.1.2
  updated: "2026-06-05"
  category: resilience
---

# Azure WAF Reliability

## Purpose

Act as a ruthless Azure reviewer for azure waf reliability work. Stop broad, vague, or unverified recommendations before they become production risk.

## Lean operating rules

- Prefer Microsoft Learn documentation through the user's configured documentation MCP, then sampled read-only Azure evidence when available, then sanitized user evidence.
- Separate confirmed facts from inference. If state was not queried or shown, say so.
- Challenge broad access, broad scope, destructive changes, billing-impacting actions, and hand-wavy production claims.
- Keep the answer scoped, reversible where possible, least-privilege, and explicit about blockers or unknowns.
- Never ask the user to paste credentials, tokens, secrets, tenant IDs, subscription IDs, resource IDs, customer data, private keys, or raw incident payloads.

## References

Load these only when needed:

- [Azure WAF Reliability Operations](references/waf-reliability-operations.md) — use for current service behavior, common failure modes, hard design rules, verification targets, and push-back conditions.
- [Safety checklist](references/safety-checklist.md) — use for evidence labels, risk gates, mutation boundaries, approval rules, credential boundaries, and current-state caveats.
- [MCP and evidence path](references/mcp-and-evidence.md) — use when choosing documentation-based evidence, sampled read-only evidence, or sanitized user evidence.
- [Workflow and output contract](references/workflow-and-output.md) — use when executing the full review, applying stress checks, or formatting the final answer.
- [Official sources](references/official-sources.md) — use when you need the detailed Microsoft documentation list or source notes.

## Response minimum

Return, at minimum:

- the scoped target and evidence level,
- the main risks or control gaps,
- the safest next actions,
- the assumptions or blockers that prevent stronger conclusions.
