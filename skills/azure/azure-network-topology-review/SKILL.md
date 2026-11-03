---
name: azure-network-topology-review
description: Use this skill for Azure network architecture review, hub-spoke critique, routing and DNS dependency analysis, shared-services boundary decisions, firewall placement review, and landing-zone connectivity guidance.
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: 0.1.2
  updated: "2026-06-05"
  category: networking
---

# Azure Network Topology Review

## Purpose

Review Azure network topology with an operator-grade focus on connectivity boundaries, shared-services design, routing and DNS dependencies, and the separation between platform-owned and workload-owned controls.

This skill is for architecture and review work across:

- hub-spoke versus alternative connectivity models,
- shared hub services and spoke isolation,
- peering and subscription boundary decisions,
- route ownership and forced-tunneling implications,
- DNS and private name-resolution dependencies,
- centralized versus workload-local firewall, NVA, and private-endpoint patterns,
- platform-team versus workload-team control boundaries.

## When to use

Use this skill when the user asks for:

- an Azure hub-spoke review or critique,
- network topology advice for a landing zone or multi-subscription platform,
- shared-services placement guidance for DNS, egress, ingress, Bastion, or hybrid connectivity,
- routing, peering, UDR, gateway-transit, or forced-tunneling concerns,
- private connectivity architecture review when the main issue is topology and boundary design,
- clarification of who should own which network controls across platform and workload teams.

Do not use this skill for:

- packet-level troubleshooting,
- detailed firewall rule authoring,
- step-by-step ExpressRoute implementation runbooks,
- a Private Link placement question where private-endpoint design is the primary issue rather than the overall topology.

Route private-endpoint-first questions toward `azure-private-endpoint-adoption-planner` if that skill exists in the repo state the user is working with.

## Lean operating rules

- Prefer Microsoft Learn documentation through the user's configured documentation MCP, then sampled read-only Azure evidence when available, then sanitized user evidence.
- Separate confirmed facts from inference. If state was not queried or shown, say so.
- Challenge broad access, broad scope, destructive changes, and hand-wavy production claims.
- Keep the answer scoped, reversible, least-privilege, and explicit about blockers or unknowns.

## References

Load these only when needed:

- [Azure Network Topology Operations](references/network-topology-operations.md) — use for current service behavior, common failure modes, hard design rules, verification targets, and push-back conditions.
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
