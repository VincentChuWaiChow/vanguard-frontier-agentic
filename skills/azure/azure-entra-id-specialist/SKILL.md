---
name: azure-entra-id-specialist
description: Use this skill for Microsoft Entra ID specialist work, especially Conditional Access, authentication methods, MFA and SSPR registration, identity protection, workload identities, app registrations, external identities, agent identities, break-glass posture, and tenant identity control reviews.
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: 0.1.5
  updated: "2026-06-05"
  category: security
---

# Azure Entra ID Specialist

## Purpose

Review and guide Microsoft Entra ID posture beyond governance-only workflows. Use this skill when the user needs broader Entra identity administration, access-control hardening, sign-in control critique, identity-risk handling, workload identity review, or app-registration security guidance.

This skill is for Entra-focused work across:

- Conditional Access design and lockout-risk review,
- MFA, SSPR, authentication methods, and registration protection,
- identity protection and risky-user or risky-sign-in handling,
- workload identities, managed identities, and service-principal access posture,
- agent identities, AI-agent Conditional Access, and agent-governance control boundaries,
- application registrations and enterprise-app access shape,
- external identities and B2B/B2C-style tenant access boundaries,
- break-glass and emergency-access account safety,
- tenant-level identity control review that is broader than governance alone.

## When to use

Use this skill when the user asks for:

- Microsoft Entra ID security or tenant identity review,
- Conditional Access design or exclusions critique,
- MFA, SSPR, or authentication-method posture review,
- identity protection or risky sign-in/risky user response guidance,
- workload identity or service principal risk review,
- AI agent identity, agent blueprint, or agent-governance posture review,
- app-registration or enterprise-app access hardening,
- external identity or guest access posture review.

Do not use this skill as a substitute for:

- a narrower PIM / access review / entitlement-management governance review when that is the whole problem,
- low-level authentication bug fixing inside application code,
- generic Azure RBAC review when the real problem is Azure resource authorization rather than Entra identity control,
- full network or landing-zone design where identity is only incidental.

If the problem narrows mainly to PIM, access reviews, entitlement management, or standing-versus-eligible access, use **Azure Identity Governance Review** instead of stretching this skill.

## Lean operating rules

- Prefer Microsoft Learn documentation through the user's configured documentation MCP, then sampled read-only Azure evidence when the active client exposes it, then sanitized user evidence.
- Separate confirmed facts from inference. If state was not queried or shown, say so.
- Treat Microsoft licensing and service entitlement as a first-class constraint; do not imply a control exists for the tenant if the required license or product entitlement is unproven.
- If the user mentions an adjacent Microsoft service that is not explicitly covered in the current examples, consult official references before concluding feature rights, identity scope, or licensing behavior. The examples in this skill are anchors, not the limit of the role.
- Challenge broad exclusions, permanent privileged access, vague emergency-access stories, weak app-registration ownership, and hand-wavy production claims.
- Keep the answer scoped, reversible, least-privilege, and explicit about blockers or unknowns.

## References

Load these only when needed:

- [Operations guide](references/entra-id-identity-operations.md) — use for service-specific pitfalls, design rules, verification targets, and pushback criteria.
- [MCP and evidence path](references/mcp-and-evidence.md) — use when choosing documentation-based evidence, sampled read-only Azure evidence, or sanitized user evidence.
- [Safety checklist](references/safety-checklist.md) — use for evidence labels, risk gates, mutation boundaries, approval rules, credential boundaries, and current-state caveats.
- [Workflow and output contract](references/workflow-and-output.md) — use when executing the full review, applying stress checks, or formatting the final answer.
- [Licensing and service entitlements](references/licensing-and-service-entitlements.md) — use when Conditional Access, PIM, ID Protection, Workload ID, Microsoft 365 bundles, Microsoft Fabric examples, or cross-service feature rights are in scope.
- [Adjacent Microsoft service expansion](references/adjacent-service-expansion.md) — use when the user brings up another Microsoft service and you need to learn the identity, entitlement, or licensing relationship before answering.
- [Official sources](references/official-sources.md) — use when you need the detailed Microsoft documentation list or source notes.

## Response minimum

Return, at minimum:

- the scoped target and evidence level,
- the main Entra control gaps or safety risks,
- the safest next actions,
- the assumptions or blockers that prevent stronger conclusions.
