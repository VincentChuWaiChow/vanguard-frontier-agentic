---
name: azure-observability-investigator
description: Use this skill for Azure Monitor, Log Analytics, Application Insights, alerting, KQL triage, telemetry-gap analysis, workbooks, or operator-grade incident and posture investigations.
allowed-tools: Read Grep Glob WebFetch
metadata:
  author: "github: VincentChuWaiChow"
  version: 0.1.2
  updated: "2026-06-05"
  category: observability
---

# Azure Observability Investigator

## Purpose

Investigate Azure operational health using evidence from metrics, logs, traces, alerts, and observability configuration before jumping to root-cause claims.

This skill is for operator-grade Azure monitoring work across:

- Azure Monitor metrics and logs,
- Log Analytics workspace design and query posture,
- Application Insights telemetry and dependency signals,
- alert rules, action groups, and alert processing rules,
- workbook or Grafana-backed operational visibility,
- KQL-based triage,
- telemetry blind spots, noisy alerts, and missing-signal investigations.

## When to use

Use this skill when the user asks for:

- Azure Monitor or Application Insights incident investigation,
- noisy, duplicate, stale, or low-value alert review,
- Log Analytics or KQL triage help,
- missing telemetry or observability-gap analysis,
- workspace or signal-placement review,
- dashboard, workbook, or operational reporting critique,
- recommended next diagnostic steps for a recent failure.

Do not use this skill as a substitute for:

- full application debugging with code changes,
- SIEM engineering or Microsoft Sentinel content design,
- resource-health-first outage triage when the main question is whether Azure itself is degraded,
- instrumentation implementation details unless the user asks for that next.

## Lean operating rules

- Prefer Microsoft Learn documentation through the user's configured documentation MCP, then sampled read-only Azure evidence when available, then sanitized user evidence.
- Separate confirmed facts from inference. If state was not queried or shown, say so.
- Challenge broad access, broad scope, destructive changes, and hand-wavy production claims.
- Keep the answer scoped, reversible, least-privilege, and explicit about blockers or unknowns.

## References

Load these only when needed:

- [Azure Observability Investigation Operations](references/observability-investigation-operations.md) — use for current service behavior, common failure modes, hard design rules, verification targets, and push-back conditions.
- [Safety checklist](references/safety-checklist.md) — use for evidence labels, risk gates, mutation boundaries, approval rules, credential boundaries, and current-state caveats.
- [MCP and evidence path](references/mcp-and-evidence.md) — use when choosing live Azure evidence, confirming Microsoft MCP capability, or switching to documentation mode.
- [Workflow and output contract](references/workflow-and-output.md) — use when executing the full review, applying stress checks, or formatting the final answer.
- [Official sources](references/official-sources.md) — use when you need the detailed Microsoft documentation list or source notes.

## Response minimum

Return, at minimum:

- the scoped target and evidence level,
- the main risks or control gaps,
- the safest next actions,
- the assumptions or blockers that prevent stronger conclusions.
