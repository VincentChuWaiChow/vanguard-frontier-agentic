---
name: azure-governance-policy-guardrails
description: Use this skill for Azure Policy guardrails, initiatives, assignment scope, management-group inheritance, exclusions, remediation risk, tag governance, allowed regions or SKUs, and staged governance rollout reviews.
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: 0.1.3
  updated: "2026-06-05"
  category: compliance
---

# Azure Governance Policy Guardrails

## Purpose

Design or review Azure governance guardrails with Azure Policy in a way that is enforceable, scope-aware, and safe to roll out.

## When to use

Use this skill when the user asks for:

- Azure Policy design or review,
- initiatives versus single policy choices,
- management-group or subscription assignment placement,
- exclusions, exemptions, or inheritance concerns,
- tag governance,
- allowed locations, resource types, or SKU restrictions,
- brownfield governance hardening,
- compliance enforcement rollout safety.

Do not use this as a substitute for full regulatory interpretation, SOC operations, or writing full organization-specific policy JSON unless the user asks for that next.

## Lean operating rules

- Prefer Microsoft Learn documentation through the user's configured documentation MCP, then sampled read-only Azure evidence when the active client exposes it, then sanitized user evidence.
- Separate confirmed facts from inference. If state was not queried or shown, say so.
- Challenge broad access, broad scope, destructive changes, and hand-wavy production claims.
- Keep the answer scoped, reversible, least-privilege, and explicit about blockers or unknowns.

## References

Load these only when needed:

- [Operations guide](references/policy-guardrail-operations.md) — use for service-specific pitfalls, design rules, verification targets, and pushback criteria.
- [MCP and evidence path](references/mcp-and-evidence.md) — use when choosing documentation-based evidence, sampled read-only Azure evidence, or sanitized user evidence.
- [Safety checklist](references/safety-checklist.md) — use for evidence labels, risk gates, mutation boundaries, approval rules, credential boundaries, and current-state caveats.
- [Workflow and output contract](references/workflow-and-output.md) — use when executing the full review, applying stress checks, or formatting the final answer.
- [Official sources](references/official-sources.md) — use when you need the detailed Microsoft documentation list or source notes.

## Response minimum

Return, at minimum:

- the scoped target and evidence level,
- the main risks or control gaps,
- the safest next actions,
- the assumptions or blockers that prevent stronger conclusions.
