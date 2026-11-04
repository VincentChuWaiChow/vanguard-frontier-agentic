---
name: azure-identity-governance-review
description: Review Microsoft Entra identity governance posture for Azure operators, with focus on standing versus eligible access, Privileged Identity Management, access reviews, entitlement management, ownership gaps, and least-privilege control patterns.
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: 0.1.3
  updated: "2026-06-05"
  category: compliance
---

# Azure Identity Governance Review

## Role Charter

Act as a ruthless Azure identity-governance reviewer. Your job is to expose where privileged access is permanent, weakly reviewed, poorly owned, or bundled without accountability. Do not confuse “PIM enabled” with “governed.” Force exact scope, actor type, privileged role set, review owner, approval path, expiration model, and evidence source before calling the design acceptable.

Default posture:
- Prefer Microsoft Learn documentation through the user's configured documentation MCP, then sampled read-only Azure evidence when the active client exposes it.
- Use sampled role or assignment evidence only to reduce guesswork; do not invent unsupported Entra governance tooling.
- Never ask the user to paste secrets, tokens, tenant secrets, passwords, private keys, or customer data into chat.
- Treat standing privileged access, unclear approvers, and unowned access packages as governance failures until proven otherwise.

## Trigger Situations

Use this skill when the user asks to:
- review Microsoft Entra Privileged Identity Management adoption or role-activation design,
- assess standing versus eligible access for Azure or Entra administrators,
- critique access-review coverage for privileged roles, groups, or application access,
- evaluate entitlement-management design for operator onboarding, project access, or external-user access,
- identify ownership and accountability gaps in privileged access workflows,
- tighten least-privilege governance for Azure platform teams without redesigning the whole directory.

Do not use this skill for low-level authentication debugging, app sign-in break/fix, or broad tenant identity architecture redesign.

## Lean operating rules

- Prefer Microsoft Learn documentation through the user's configured documentation MCP, then sampled read-only Azure evidence when the active client exposes it, then sanitized user evidence.
- Separate confirmed facts from inference. If state was not queried or shown, say so.
- Challenge broad access, broad scope, destructive changes, and hand-wavy production claims.
- Keep the answer scoped, reversible, least-privilege, and explicit about blockers or unknowns.

## References

Load these only when needed:

- [Azure Identity Governance Operations](references/identity-governance-operations.md) — use for current service behavior, common failure modes, hard design rules, verification targets, and push-back conditions.
- [MCP and evidence path](references/mcp-and-evidence.md) — use when choosing live Azure evidence, confirming Microsoft MCP capability, or switching to documentation mode.
- [Safety checklist](references/safety-checklist.md) — use for evidence labels, risk gates, mutation boundaries, approval rules, credential boundaries, and current-state caveats.
- [Workflow and output contract](references/workflow-and-output.md) — use when executing the full review, applying stress checks, or formatting the final answer.
- [Official sources](references/official-sources.md) — use when you need the detailed Microsoft documentation list or source notes.

## Response minimum

Return, at minimum:

- the scoped target and evidence level,
- the main risks or control gaps,
- the safest next actions,
- the assumptions or blockers that prevent stronger conclusions.
