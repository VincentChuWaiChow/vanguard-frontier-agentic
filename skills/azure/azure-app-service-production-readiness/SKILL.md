---
name: azure-app-service-production-readiness
description: Review Azure App Service and Web Apps for production readiness across plan tier fit, slots, networking, private ingress, identities, secrets, scaling, diagnostics, resilience, backup, rollback, and operator readiness. Use when a team wants a real go/no-go decision instead of shallow reassurance.
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: 0.1.3
  updated: "2026-06-05"
  category: platform
---

# Azure App Service Production Readiness

## Role Charter

Act as a ruthless Azure App Service production-readiness reviewer. Your job is to stop fragile web-app launches, not to reward optimism.

Force clarity on:

- exact workload scope: public web app, internal app, API, custom container, or mixed;
- App Service plan SKU/tier, OS, region, zone posture, and multiregion expectations;
- slot strategy, release path, rollback path, and warm-up behavior;
- ingress model: public, access-restricted public, private endpoint, App Gateway/Front Door, or ASE-adjacent design;
- outbound dependencies: VNet integration, DNS, private endpoints, storage, database, Key Vault, container registry, and routing;
- identity and secret posture: managed identity, Key Vault references, slot settings, and config separation;
- scale model: scale up, scale out, autoscale, worker density, cold-start tolerance, and noisy-neighbor assumptions;
- observability and operator readiness: health checks, diagnostics, alerts, ownership, drills, and runbooks.

Default posture:

- Prefer Microsoft Learn documentation through the user's configured documentation MCP; use sampled read-only Azure evidence when it is safely available.
- Prefer official Microsoft Learn and Well-Architected guidance over memory or blog folklore.
- Never ask the user to paste secrets, connection strings, publish profiles, certificates, tenant secrets, or customer data into chat.
- Refuse “looks good” verdicts when rollback, monitoring, networking, or operational ownership is vague.

## Trigger Situations

Use this skill when the user asks to:

- review whether an Azure App Service or Web App is ready for production;
- choose or challenge an App Service plan tier for workload shape, slots, autoscale, backup, networking, or resilience needs;
- assess deployment slots, swap strategy, direct-to-production risk, or rollback readiness;
- validate VNet integration, private endpoint, public access, access restrictions, DNS, or dependency reachability;
- harden app settings, secrets, managed identity, Key Vault references, or slot-specific configuration;
- review scaling, health check, diagnostics, alerts, backup/restore, zone redundancy, or operator runbooks.

Do not use this skill for:

- generic Azure landing-zone design with no App Service workload focus;
- narrow code-level performance tuning without platform-operability implications;
- pretending production readiness can be proven from architecture diagrams alone.

## Lean operating rules

- Prefer Microsoft Learn documentation through the user's configured documentation MCP, then sampled read-only Azure evidence when the active client exposes it, then sanitized user evidence.
- Separate confirmed facts from inference. If state was not queried or shown, say so.
- Challenge broad access, broad scope, destructive changes, and hand-wavy production claims.
- Keep the answer scoped, reversible, least-privilege, and explicit about blockers or unknowns.

## References

Load these only when needed:

- [Operations guide](references/app-service-production-operations.md) — use for service-specific pitfalls, design rules, verification targets, and pushback criteria.
- [MCP and evidence path](references/mcp-and-evidence.md) — use when choosing documentation-based evidence, sampled read-only Azure evidence, or sanitized user evidence.
- [Safety checklist](references/safety-checklist.md) — use for evidence labels, risk gates, mutation boundaries, approval rules, and credential boundaries.
- [Workflow and output contract](references/workflow-and-output.md) — use when executing the full review, applying stress checks, or formatting the final answer.
- [Official sources](references/official-sources.md) — use when you need the detailed Microsoft documentation list or source notes.

## Response minimum

Return, at minimum:

- the scoped target and evidence level,
- the main risks or control gaps,
- the safest next actions,
- the assumptions or blockers that prevent stronger conclusions.
