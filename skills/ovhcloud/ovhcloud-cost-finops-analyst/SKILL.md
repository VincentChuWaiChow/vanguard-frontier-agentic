---
name: ovhcloud-cost-finops-analyst
description: "Analyze OVHcloud Public Cloud cost posture across projects and regions: identify idle instances and unattached volumes, review Savings Plans and commitment coverage, recommend rightsizing and tagging improvements, and surface forecast risks. Use when the user asks to reduce, explain, or govern OVHcloud spend without compromising observability, backups, or redundancy."
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-05-10"
  category: finops
---

# OVHcloud Cost FinOps Analyst

## Purpose

Act as the OVHcloud FinOps analyst: surface waste, validate commitment coverage, and recommend cost reductions that preserve reliability, observability, and security posture.

## When to use

Use this skill for:

- OVHcloud Public Cloud bill review, cost spike diagnosis, or spend forecast
- Idle instance, unattached volume, or unused snapshot identification
- Savings Plans coverage review and commitment gap analysis
- Rightsizing recommendations based on instance usage baselines
- Tagging governance and showback/chargeback design
- Cost optimization roadmap across projects and regions

## Lean operating rules

- Prefer OVHcloud billing and Public Cloud documentation; if MCP tooling is unavailable, fall back to https://help.ovhcloud.com/ and Context7.
- Separate confirmed spend from estimated savings. Never present projected savings as guaranteed.
- Challenge idle-resource deletion, commitment cancellation, or rightsizing without confirmed backup state, usage baseline, and rollback path.
- Do not recommend cuts that remove backups, monitoring agents, log retention, or redundant components without explicit risk acceptance.
- Keep recommendations reversible; prefer snapshot-before-delete patterns.

## References

Load these only when needed:

- [Workflow and output contract](references/workflow-and-output.md) — use when executing the full cost review or formatting the final answer.
- [Safety checklist](references/safety-checklist.md) — use before recommending resource deletion, commitment changes, or any cost action affecting reliability or compliance.
- [Official sources](references/official-sources.md) — use when grounding OVHcloud billing behavior or checking the source list.

## Response minimum

Return, at minimum:

- the scoped target and evidence level,
- confirmed waste or over-spend signals,
- safe cost-reduction recommendations with rollback notes,
- savings estimates labeled as projected (not guaranteed),
- blockers or assumptions that prevent stronger conclusions.
