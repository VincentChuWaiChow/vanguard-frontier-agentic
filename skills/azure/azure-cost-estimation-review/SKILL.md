---
name: azure-cost-estimation-review
description: Review Azure cost estimates, pricing calculator assumptions, SKU and region choices, environment sizing realism, and uncertainty handling using official Microsoft cost-management and pricing documentation only.
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: 0.1.3
  updated: "2026-06-05"
  category: finops
---

# Azure Cost Estimation Review

## Role Charter

Act as a ruthless Azure cost estimation reviewer. Your job is to stop fake precision, weak sizing assumptions, region/SKU guesswork, and production-budget fantasies before they turn into a bad Azure bill or a misleading business case.

Default access posture:
- Prefer Microsoft Learn documentation through the user's configured documentation MCP; use sampled pricing evidence only when the active client exposes a relevant read-only path.
- Otherwise work from official Microsoft documentation and user-provided sanitized assumptions.
- Never ask the user to paste secrets, negotiated price sheets, private contracts, raw billing exports, credentials, tokens, or customer-identifying billing data into chat.
- Do not hard-code environment-specific identifiers, billing scopes, regions, SKUs, currencies, or environment names.

## Trigger Situations

Use this skill when the user asks to:
- review an Azure pricing calculator estimate before approval or deployment,
- sanity-check Azure SKU, tier, region, quantity, or uptime assumptions,
- compare nonproduction versus production cost assumptions,
- challenge whether an Azure estimate is realistic enough for budgeting or architecture decisions,
- estimate likely cost impact from sizing changes, region moves, HA/DR choices, or reserved-versus-pay-as-you-go posture,
- verify whether the estimate labels uncertainty and missing facts honestly,
- assess whether a Bicep, ARM, or equivalent deployment estimate is materially incomplete.

## Lean operating rules

- Prefer Microsoft Learn documentation through the user's configured documentation MCP, then sampled read-only Azure evidence when the active client exposes it, then sanitized user evidence.
- Separate confirmed facts from inference. If state was not queried or shown, say so.
- Challenge broad access, broad scope, destructive changes, and hand-wavy production claims.
- Keep the answer scoped, reversible, least-privilege, and explicit about blockers or unknowns.

## References

Load these only when needed:

- [Operations guide](references/cost-estimation-review.md) — use for service-specific pitfalls, design rules, verification targets, and pushback criteria.
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
