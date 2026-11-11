---
name: "vanguard-huawei"
displayName: "Vanguard Frontier — Huawei Cloud"
description: "Curated Huawei Cloud agents for IAM, CCE, GaussDB, OBS, DEW (KMS+CSMS), and ModelArts with live-mutation guards and MLPS 2.0 compliance. Routes via huawei-maestro to specialist or live-guard agents. Enterprise Projects are billing constructs, not security boundaries — verify IAM and SCP scope independently."
keywords: ["huawei-cloud", "iam", "cce", "gaussdb", "obs", "dew", "modelarts", "live-guard"]
author: "VincentChuWaiChow"
---
# Vanguard Frontier — Huawei Cloud

Curated Huawei Cloud agents for IAM, CCE, GaussDB, OBS, DEW (KMS+CSMS), and ModelArts with live-mutation guards and MLPS 2.0 compliance. Routes via huawei-maestro to specialist or live-guard agents. Enterprise Projects are billing constructs, not security boundaries — verify IAM and SCP scope independently.

## When to engage this Power

Activate when the task references Huawei Cloud services, resources, or operations. Do not activate on unrelated requests — narrow keyword matching is required to avoid false activations (Kiro Powers convention).

## Routing pattern

- **`huawei-maestro-agent`** — classifies and routes the task to the right specialist

Use the maestro as the entry point: classify the task, then dispatch to one specialist or a parallel team of specialists. Never have the maestro itself execute a live mutation.

## Live-guard agents (gate_mode only)

- `huawei-live-cce-rollout-guard-agent` — never auto-dispatched; gate_mode only
- `huawei-live-cost-budget-action-guard-agent` — never auto-dispatched; gate_mode only
- `huawei-live-gaussdb-mutation-guard-agent` — never auto-dispatched; gate_mode only
- `huawei-live-iam-policy-change-guard-agent` — never auto-dispatched; gate_mode only
- `huawei-live-kms-key-destruction-guard-agent` — never auto-dispatched; gate_mode only
- `huawei-live-obs-bucket-policy-guard-agent` — never auto-dispatched; gate_mode only

Live-guard agents enforce approval, target confirmation, evidence capture, and rollback plans before executing a mutation. They are never auto-dispatched — the maestro must place them in `live-guard-gate` or `runtime-evidence-gate` mode.

## Invariants

- Confirm Huawei Cloud account ID, region, and Enterprise Project before any live mutation.
- Enterprise Projects are billing/attribution constructs, NOT security boundaries — verify IAM policy and SCP scope independently.
- MLPS 2.0 Level 3 (GB/T 22239-2019) requires specific service configurations — huawei-compliance-sovereignty-agent flags gaps.
- Live-guard agents (huawei-live-*) must never be auto-dispatched; require approval and rollback plan.

## Where the agents live

Agent specs and adapters are part of the [Vanguard Frontier Agentic](https://github.com/VincentChuWaiChow/vanguard-frontier-agentic) marketplace. For this provider, see `agents/huawei/` in that repository. All 44 agents in this provider ship a Kiro adapter (`harnesses/kiro-ide.agent.md`, `kiro-cli.agent.json`).

## Companion install paths

- **Claude Code:** `/plugin marketplace add VincentChuWaiChow/vanguard-frontier-agentic` then `/plugin install vanguard-frontier-agentic@vanguard-frontier-agentic`
- **Codex / Copilot / Cursor / Gemini CLI / Kiro (file export):** `npx vfa-export-agents --platform <harness> --provider huawei --repo .`
