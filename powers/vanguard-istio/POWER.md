---
name: "vanguard-istio"
displayName: "Vanguard Frontier — Istio"
description: "Curated Istio agents for ambient mesh, authorization policy, dataplane diagnostics, gateway api. Routes via istio-maestro-agent to specialist agents based on task scope. Static review only; no live mutations."
keywords: ["istio", "service-mesh", "ambient-mesh", "mtls"]
author: "VincentChuWaiChow"
---
# Vanguard Frontier — Istio

Curated Istio agents for ambient mesh, authorization policy, dataplane diagnostics, gateway api. Routes via istio-maestro-agent to specialist agents based on task scope. Static review only; no live mutations.

## When to engage this Power

Activate when the task references Istio services, resources, or operations. Do not activate on unrelated requests — narrow keyword matching is required to avoid false activations (Kiro Powers convention).

## Routing pattern

- **`istio-maestro-agent`** — classifies and routes the task to the right specialist

Use the maestro as the entry point: classify the task, then dispatch to one specialist or a parallel team of specialists. Never have the maestro itself execute a live mutation.

## Live-guard agents (gate_mode only)

- *(none — this provider has no live-mutation guards in the catalog)*

Live-guard agents enforce approval, target confirmation, evidence capture, and rollback plans before executing a mutation. They are never auto-dispatched — the maestro must place them in `live-guard-gate` or `runtime-evidence-gate` mode.

## Invariants

- Route all tasks through istio-maestro-agent for proper classification and dispatch.
- Static review only -- agents analyze configuration and provide findings without mutating live systems.
- Service mesh policies affect traffic routing cluster-wide; review blast radius before changes.

## Where the agents live

Agent specs and adapters are part of the [Vanguard Frontier Agentic](https://github.com/VincentChuWaiChow/vanguard-frontier-agentic) marketplace. For this provider, see `agents/istio/` in that repository. All 7 agents in this provider ship a Kiro adapter (`harnesses/kiro-ide.agent.md`, `kiro-cli.agent.json`).

## Companion install paths

- **Claude Code:** `/plugin marketplace add VincentChuWaiChow/vanguard-frontier-agentic` then `/plugin install vanguard-frontier-agentic@vanguard-frontier-agentic`
- **Codex / Copilot / Cursor / Gemini CLI / Kiro (file export):** `npx vfa-export-agents --platform <harness> --provider istio --repo .`
