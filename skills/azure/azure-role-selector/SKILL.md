---
name: azure-role-selector
description: Use this skill when the user asks which Azure role to assign, how to grant minimum access, whether a built-in role is sufficient, or when a custom role may be required.
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: 0.1.2
  updated: "2026-06-05"
  category: compliance
---

# Azure Role Selector

## Purpose

Select the narrowest Azure role and assignment scope that satisfies the requested access without defaulting to broad standing privilege.

## When to use

Use this skill when the user needs to:

- map requested Azure operations to a role,
- grant minimum access to a user, group, service principal, managed identity, or workload identity,
- decide whether a built-in role is enough,
- separate control-plane permissions from data-plane permissions,
- decide whether a custom role is justified,
- choose the safest assignment scope and validation path.

Do not use this skill for tenant-wide governance design, access review programs, or broad RBAC posture critique. Route those asks toward `azure-rbac-review` or a governance-focused skill.

## Lean operating rules

- Prefer Microsoft Learn documentation through the user's configured documentation MCP, then sampled read-only Azure evidence when available, then sanitized user evidence.
- Separate confirmed facts from inference. If state was not queried or shown, say so.
- Challenge broad access, broad scope, destructive changes, and hand-wavy production claims.
- Keep the answer scoped, reversible, least-privilege, and explicit about blockers or unknowns.

## References

Load these only when needed:

- [Azure Role Selection Operations](references/role-selection-operations.md) — use for current service behavior, common failure modes, hard design rules, verification targets, and push-back conditions.
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
