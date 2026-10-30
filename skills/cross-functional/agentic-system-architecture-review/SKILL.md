---
name: agentic-system-architecture-review
description: Use this skill for a vendor-neutral or cross-vendor design review of an agentic system — when the question is "should this be multi-agent at all", which orchestration pattern is the lowest sufficient one (deterministic workflow, single agent with tools, or multi-agent), whether delegation and handoff boundaries hold, whether state, memory and durable execution are designed (checkpoints, idempotency, retries, compensation, replay), and whether cascading failures are contained (budgets, fan-out caps, circuit breakers, kill switch). Static review only; it reads supplied designs, code, configuration and traces and never executes, deploys or approves anything. Verdicts are proposals that must survive a designated security challenge and a human decision owner under the agentic-board-protocol skill.
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-10-30"
  category: architecture
  lifecycle: experimental
---

# Agentic System Architecture Review

## Purpose

Review the form and control flow of an agentic system so it uses the lowest sufficient pattern and survives its own failure modes. The dominant failures are multi-agent topologies adopted without a demonstrated need, agent SDK sessions mistaken for durable execution, handoffs with no contract and no bounds, side-effecting tool calls with no idempotency, and cascading failures with no budget, breaker, or kill switch.

This skill proposes; it does not decide. This repository ships role contracts, skills, a JSON Schema and offline gates; it is not an orchestration runtime, and nothing here authenticates senders, enforces authorization at runtime, or stores audit records. A verdict from this skill is a proposal that a designated challenger attacks and a human decision owner resolves, per the [agentic board protocol](../agentic-board-protocol/SKILL.md).

## When to use

Use this skill when the user asks to:

- decide whether a design should be a deterministic workflow, a single agent with tools, or a multi-agent system ("should this be multi-agent at all"),
- review an orchestration topology: supervisor, handoff, fan-out, group chat, planner-executor-verifier, event-driven, or graph designs,
- review delegation boundaries, routing fallback, handoff contracts, privilege attenuation down a chain, loop bounds, or where humans approve,
- review state scope, memory isolation and retention, checkpointing, idempotency, retries, compensation, or replay safety,
- review cascading-failure containment: budgets, fan-out caps, circuit breakers, bulkheads, degraded modes, kill switch, SLOs, and observability hooks,
- compare agent frameworks for a design that spans more than one vendor or uses a custom framework.

## When not to use

Do not use this skill for:

- single-vendor platform specifics — use that vendor's agent: `aws-agentcore-agent`, `databricks-genai-agent-engineering-agent`, `nvidia-agentic-ai-platform-review-agent`, `snowflake-cortex-ai-agent-security-governor-agent`, `azure-ai-foundry-ops-governor-agent`, `salesforce-agentforce-ai-agent`, `copilot-studio-agent-governance-alm-agent` (start at `copilot-governance-maestro-agent` for Microsoft Copilot governance requests), or `sap-joule-governance-adoption-agent`. When one system composes several vendors, or a custom framework plus vendor tools, review the seams here and hand each vendor interior to its agent.
- MCP tool-schema fidelity — use `typescript-mcp-tool-contract-agent`.
- evaluation design — use `llm-ai-pipeline-test-review-agent`.
- the security attack surface (identity, delegated authorization, tool and memory poisoning, supply chain) — use `agentic-security-review-agent`.
- the business case for automating at all — use the value-case procedure in the [value case reference](../agentic-board-protocol/references/value-case.md).
- executing, deploying, load-testing, or approving anything. This skill is static review only.

## Lean operating rules

- Decide the lowest sufficient pattern first: deterministic workflow, then single agent with tools, then multi-agent. Every step up needs evidence; record each rejected rung with a reason. The conventional-automation alternative sits between the first two rungs and belongs to the protocol's [value case reference](../agentic-board-protocol/references/value-case.md); check that it was considered before accepting a single agent.
- Demand evidence for each claimed need for more than one agent: separation of duties, parallelism, context-window limits, or distinct tool permission sets. Role names alone (planner, coder, reviewer) are not evidence.
- Run the check "an agent SDK is not a durable workflow engine": for any process that outlives one request, ask what survives a restart, what is replayed, and what makes each side effect safe to repeat.
- Ground framework APIs in Context7 for the installed version (`resolve-library-id`, then `query-docs`; label `context7-grounded`) when the invoking harness provides it. Absence from a Context7 result is uncorroborated, not disproven. Otherwise write "verify against the official docs for the installed version" instead of asserting behavior from memory.
- Label every claim with the envelope evidence vocabulary: `live-evidence`, `repo-evidence`, `user-provided-sanitized-evidence`, `context7-grounded`, `documentation-based`, `inference`.
- Own pattern selection, durable execution and state lifecycle, and cascading-failure containment. Hand identity, delegated authorization, tool and memory poisoning, and provenance to the security agent; do not adjudicate them.
- Emit handoffs per agentic-board-protocol: objectives phrased as review questions, one return envelope carrying the verdict, the designated challenger named. Never approve; a decision request always goes to a human principal.
- State the tool-grant boundary plainly: claude-code `tools: "Read, Grep, Glob"` and gemini `tools:` are documented allowlists (an omitted list inherits every parent tool), cursor `readonly: true` blocks file edits and state-changing shell but not every tool, and copilot `tools:` and codex `sandbox_mode = "read-only"` follow this repo's own gates; kiro-ide and kiro-cli carry no verified restricting field, so their limits are prompt-level only. The per-harness table in [docs/execution-tiers.md](../../../docs/execution-tiers.md) is the single source for these claims. A SKILL.md `allowed-tools` line pre-approves tools; it does not restrict them (per https://code.claude.com/docs/en/skills).
- Never request, echo, or store credentials, tokens, tenant identifiers, or customer data; redact and flag any that appear. Never claim certification or that a framework mapping is satisfied.

## References

Load these only when needed:

- [Pattern selection](references/pattern-selection.md) — the technical rungs, the four needs that can justify multiple agents, orchestration patterns as vendor-neutral concepts, and a framework primitives snapshot.
- [Delegation and handoff review](references/delegation-and-handoff-review.md) — topology, routing fallback, handoff contracts, attenuation, loop bounds, human-in-the-loop placement, and the A2A task lifecycle as a resumability boundary.
- [State, memory and durability](references/state-memory-and-durability.md) — state scope, memory isolation and deletion, checkpointing, idempotency, retries, compensation, and replay safety.
- [Reliability and containment](references/reliability-and-containment.md) — budgets, fan-out caps, circuit breakers, bulkheads, degraded modes, kill switch, SLOs, and observability hooks.
- [Workflow and output](references/workflow-and-output.md) — the step-by-step review, the output template, and a synthetic example verdict.

## Response minimum

Return, at minimum:

- the verdict outcome, one of pass, pass-with-conditions, block, or simpler-pattern-sufficient,
- the lowest sufficient pattern and the rungs rejected, each with a reason,
- findings, each with a failure scenario, an evidence label, a fix direction, and a verification step,
- missing evidence, marked blocking or not, and what could not be verified,
- handoffs: which agent receives what, and why,
- the designated challenger who must review the verdict, and the statement that the verdict is a proposal until a human decision owner decides.
