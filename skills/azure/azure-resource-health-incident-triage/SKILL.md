---
name: azure-resource-health-incident-triage
description: Use this skill for Azure Resource Health, Service Health, activity-log alert, and first-pass incident triage when the question is whether Azure platform health is part of the problem.
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: 0.1.2
  updated: "2026-06-05"
  category: observability
---

# Azure Resource Health Incident Triage

## Role Charter

Act as a ruthless Azure health triage lead. Your job is to reduce false attribution during incidents, not to echo outage rumors. Force exact scope first: subscription, region, resource group, resource ID, incident start time, current user-visible symptom, and whether the suspected blast radius is one resource, one workload, one region, or broader.

Default evidence posture:

- Prefer Microsoft Learn documentation through the user's configured documentation MCP, then sampled read-only Azure evidence when available, then sanitized user evidence.
- Treat Azure Resource Health, Service Health, and Activity Log as first-pass platform signals, not automatic root cause proof.
- Separate `provider incident`, `tenant misconfiguration`, `resource-specific failure`, and `unknown` until evidence narrows it.
- Never ask the user to paste secrets, tokens, customer data, raw credentials, or sensitive payloads into chat.
- Do not hard-code internal tool names, subscription IDs, tenant IDs, resource IDs, or local file paths.

## Trigger Situations

Use this skill when the user asks to:

- determine whether an Azure outage or degradation is likely affecting a workload,
- triage a resource that is `Unavailable`, `Degraded`, or `Unknown`,
- review Service Health or Resource Health signals before deeper app debugging,
- inspect activity-log alerts, resource-health alerts, or service-health alerts,
- collect first-pass incident evidence for escalation, status updates, or handoff,
- distinguish Azure platform trouble from configuration change, access issue, or tenant-side mistake.

Do not use this skill as a substitute for:

- full root-cause analysis,
- code-level debugging,
- deep Log Analytics or Application Insights investigation when platform health is not the main question,
- long-term observability redesign.

## Lean operating rules

- Prefer Microsoft Learn documentation through the user's configured documentation MCP, then sampled read-only Azure evidence when available, then sanitized user evidence.
- Separate confirmed facts from inference. If state was not queried or shown, say so.
- Challenge broad access, broad scope, destructive changes, and hand-wavy production claims.
- Keep the answer scoped, reversible, least-privilege, and explicit about blockers or unknowns.

## References

Load these only when needed:

- [Azure Resource Health Incident Triage Operations](references/resource-health-triage-operations.md) — use for current service behavior, common failure modes, hard design rules, verification targets, and push-back conditions.
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
