---
name: azure-security-posture-hardening
description: Use this skill for Azure security posture review, baseline hardening, managed identity adoption, Key Vault posture, private access decisions, Azure Policy guardrails, and logging or audit gap analysis. Trigger when the user asks how to harden an Azure workload or platform without defaulting to broad access or public exposure.
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: 0.1.2
  updated: "2026-06-05"
  category: security
---

# Azure Security Posture Hardening

## Purpose

Review and harden Azure platform or workload posture using operator-grade controls:

- least privilege,
- managed identities over stored secrets,
- private access where justified,
- Key Vault hardening,
- policy-enforced controls,
- audit and diagnostic coverage,
- staged remediation with rollout safety.

## When to use

Use this skill when the user asks for:

- Azure security baseline or posture review,
- managed identity migration guidance,
- Key Vault hardening or secret-handling critique,
- private endpoint or public exposure decisions for sensitive services,
- Azure Policy or Defender-backed hardening recommendations,
- logging, diagnostics, or auditability expectations for Azure security controls,
- zero-trust-oriented review of platform or workload controls.

Do not use this skill as a full compliance audit, incident forensics runbook, or a substitute for deep service-specific implementation docs.

## Lean operating rules

- Prefer Microsoft Learn documentation through the user's configured documentation MCP, then sampled read-only Azure evidence when available, then sanitized user evidence.
- Separate confirmed facts from inference. If state was not queried or shown, say so.
- Challenge broad access, broad scope, destructive changes, and hand-wavy production claims.
- Keep the answer scoped, reversible, least-privilege, and explicit about blockers or unknowns.

## References

Load these only when needed:

- [Azure Security Posture Hardening Operations](references/security-posture-hardening-operations.md) — use for current service behavior, common failure modes, hard design rules, verification targets, and push-back conditions.
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
