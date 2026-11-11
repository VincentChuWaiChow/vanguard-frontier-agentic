---
name: "vanguard-oci"
displayName: "Vanguard Frontier — OCI"
description: "Curated Oracle Cloud agents for IAM, OKE, Autonomous Database, Vault, and Resource Manager with live-mutation guards. Routes via oci-maestro to specialist or live-guard agents. Distinguishes commercial vs gov-cloud realm; mutations require tenancy, compartment, and region confirmation."
keywords: ["oci", "oracle-cloud", "iam", "oke", "autonomous-database", "vault", "resource-manager", "live-guard"]
author: "VincentChuWaiChow"
---
# Vanguard Frontier — OCI

Curated Oracle Cloud agents for IAM, OKE, Autonomous Database, Vault, and Resource Manager with live-mutation guards. Routes via oci-maestro to specialist or live-guard agents. Distinguishes commercial vs gov-cloud realm; mutations require tenancy, compartment, and region confirmation.

## When to engage this Power

Activate when the task references OCI services, resources, or operations. Do not activate on unrelated requests — narrow keyword matching is required to avoid false activations (Kiro Powers convention).

## Routing pattern

- **`oci-maestro-agent`** — classifies and routes the task to the right specialist

Use the maestro as the entry point: classify the task, then dispatch to one specialist or a parallel team of specialists. Never have the maestro itself execute a live mutation.

## Live-guard agents (gate_mode only)

- `oci-live-autonomous-db-lifecycle-guard-agent` — never auto-dispatched; gate_mode only
- `oci-live-cost-budget-runaway-guard-agent` — never auto-dispatched; gate_mode only
- `oci-live-iam-policy-compartment-guard-agent` — never auto-dispatched; gate_mode only
- `oci-live-network-security-rule-guard-agent` — never auto-dispatched; gate_mode only
- `oci-live-oke-rollout-guard-agent` — never auto-dispatched; gate_mode only
- `oci-live-resource-manager-stack-guard-agent` — never auto-dispatched; gate_mode only
- `oci-live-vault-key-destruction-guard-agent` — never auto-dispatched; gate_mode only

Live-guard agents enforce approval, target confirmation, evidence capture, and rollback plans before executing a mutation. They are never auto-dispatched — the maestro must place them in `live-guard-gate` or `runtime-evidence-gate` mode.

## Invariants

- Confirm OCI tenancy OCID, compartment, and region before any live mutation.
- Live-guard agents (oci-live-*) must never be auto-dispatched; require approval and rollback plan.
- Commercial and government cloud realms have separate identity domains — verify realm before action.
- Compartment hierarchy enforces policy scope; review parent-compartment grants before sub-compartment changes.

## Where the agents live

Agent specs and adapters are part of the [Vanguard Frontier Agentic](https://github.com/VincentChuWaiChow/vanguard-frontier-agentic) marketplace. For this provider, see `agents/oci/` in that repository. All 41 agents in this provider ship a Kiro adapter (`harnesses/kiro-ide.agent.md`, `kiro-cli.agent.json`).

## Companion install paths

- **Claude Code:** `/plugin marketplace add VincentChuWaiChow/vanguard-frontier-agentic` then `/plugin install vanguard-frontier-agentic@vanguard-frontier-agentic`
- **Codex / Copilot / Cursor / Gemini CLI / Kiro (file export):** `npx vfa-export-agents --platform <harness> --provider oci --repo .`
