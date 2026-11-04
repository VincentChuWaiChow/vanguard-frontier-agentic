---
name: kubernetes-rbac-review
description: Use this skill for Kubernetes RBAC, Role, ClusterRole, RoleBinding, ClusterRoleBinding, ServiceAccount, workload identity, or least-privilege review tasks. Trigger when the user asks whether cluster access is too broad, how to grant workload permissions safely, or how to audit RBAC state.
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: 0.1.0
  updated: "2026-05-05"
  category: security
---

# Kubernetes RBAC Review

## Purpose

Review Kubernetes RBAC objects — Roles, ClusterRoles, RoleBindings, ClusterRoleBindings, and ServiceAccounts — against least privilege, namespace scope minimization, and operational safety.

## Lean operating rules

- Prefer live cluster evidence (`kubectl auth can-i`, `kubectl get rolebinding`, audit logs) when the active client exposes it; otherwise fall back to official Kubernetes documentation and sanitized user evidence.
- Separate confirmed facts from inference. If state was not queried or shown, say so.
- Challenge cluster-scoped access granted to workloads that only need namespace-scoped access.
- Challenge wildcard verbs (`*`), wildcard resources (`*`), and wildcard API groups (`*`) unless explicitly justified.
- Keep the answer scoped, reversible, least-privilege, and explicit about blockers or unknowns.

## References

Load these only when needed:

- [Evidence path and tooling](references/mcp-and-evidence.md) — use when choosing live cluster evidence, confirming MCP capability, or switching to documentation mode.
- [Workflow and output contract](references/workflow-and-output.md) — use when executing the full review, applying stress checks, or formatting the final answer.
- [Official sources](references/official-sources.md) — use when you need the detailed Kubernetes documentation list or source notes.

## Response minimum

Return, at minimum:

- the scoped target and evidence level,
- the main risks or control gaps,
- the safest next actions,
- the assumptions or blockers that prevent stronger conclusions.
