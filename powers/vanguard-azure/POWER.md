---
name: "vanguard-azure"
displayName: "Vanguard Frontier — Azure"
description: "Curated Azure agents for Entra ID, AKS, App Service, Key Vault, Cosmos DB, and ARM/Bicep with live-mutation guards. Routes via azure-maestro to specialist or live-guard agents. Mutations on real Azure environments require subscription ID, tenant ID, resource group, and approval confirmation."
keywords: ["azure", "entra-id", "aks", "app-service", "key-vault", "cosmos-db", "bicep", "live-guard"]
author: "VincentChuWaiChow"
---
# Vanguard Frontier — Azure

Curated Azure agents for Entra ID, AKS, App Service, Key Vault, Cosmos DB, and ARM/Bicep with live-mutation guards. Routes via azure-maestro to specialist or live-guard agents. Mutations on real Azure environments require subscription ID, tenant ID, resource group, and approval confirmation.

## When to engage this Power

Activate when the task references Azure services, resources, or operations. Do not activate on unrelated requests — narrow keyword matching is required to avoid false activations (Kiro Powers convention).

## Routing pattern

- **`azure-maestro-agent`** — classifies and routes the task to the right specialist

Use the maestro as the entry point: classify the task, then dispatch to one specialist or a parallel team of specialists. Never have the maestro itself execute a live mutation.

## Live-guard agents (gate_mode only)

- `azure-live-aks-rollout-guard-agent` — never auto-dispatched; gate_mode only
- `azure-live-app-service-slot-swap-guard-agent` — never auto-dispatched; gate_mode only
- `azure-live-arm-deployment-stack-guard-agent` — never auto-dispatched; gate_mode only
- `azure-live-cost-budget-action-guard-agent` — never auto-dispatched; gate_mode only
- `azure-live-entra-role-assignment-guard-agent` — never auto-dispatched; gate_mode only
- `azure-live-keyvault-rotation-purge-guard-agent` — never auto-dispatched; gate_mode only
- `azure-live-pim-jit-activation-guard-agent` — never auto-dispatched; gate_mode only

Live-guard agents enforce approval, target confirmation, evidence capture, and rollback plans before executing a mutation. They are never auto-dispatched — the maestro must place them in `live-guard-gate` or `runtime-evidence-gate` mode.

## Invariants

- Confirm Azure subscription ID, tenant ID, and resource group before any live mutation.
- Live-guard agents (azure-live-*) must never be auto-dispatched; require approval and rollback plan.
- PIM (Privileged Identity Management) elevation is a separate decision from RBAC role assignment.
- Management group SCP-equivalent policies cascade — review blast radius before org-level changes.

## Where the agents live

Agent specs and adapters are part of the [Vanguard Frontier Agentic](https://github.com/VincentChuWaiChow/vanguard-frontier-agentic) marketplace. For this provider, see `agents/azure/` in that repository. All 37 agents in this provider ship a Kiro adapter (`harnesses/kiro-ide.agent.md`, `kiro-cli.agent.json`).

## Companion install paths

- **Claude Code:** `/plugin marketplace add VincentChuWaiChow/vanguard-frontier-agentic` then `/plugin install vanguard-frontier-agentic@vanguard-frontier-agentic`
- **Codex / Copilot / Cursor / Gemini CLI / Kiro (file export):** `npx vfa-export-agents --platform <harness> --provider azure --repo .`
