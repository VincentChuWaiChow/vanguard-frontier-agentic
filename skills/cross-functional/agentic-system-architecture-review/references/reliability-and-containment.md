# Reliability and cascading-failure containment

Use this reference to review whether a failure in one agent, tool, model or tenant stays where it started. Cascading-failure containment is owned here: the board's one-owner table gives it to the architecture review, while attack-driven failures belong to the security agent and unit prices belong to `finops-ai-economist-agent`.

Containment is a runtime property. This reference reviews whether the design has it and whether the controls are enforced outside the model. This repository enforces none of it at runtime: its stop conditions are computed over handoff-envelope fields by an offline gate and are fixture checks, not containment (see the [stop conditions reference](../../agentic-board-protocol/references/stop-conditions.md)).

## What a cascade looks like

OWASP lists Cascading Failures as ASI08 in its Top 10 for Agentic Applications (per https://genai.owasp.org/2025/12/09/owasp-top-10-for-agentic-applications-the-benchmark-for-agentic-security-in-the-age-of-autonomous-ai/). In design terms the mechanisms to look for are:

- retry amplification, where layered retries multiply load on a dependency that is already struggling,
- fan-out explosion, where one request spawns many subtasks that each spawn more,
- agent loops that keep spending tokens and tool calls without progress,
- a slow or failing dependency that grows queues until other work starves,
- one tenant's workload consuming a shared model quota or worker pool,
- errors that one agent returns being consumed as facts by the next agent and compounded.

If a cascade starts from an attacker's input rather than a fault, hand it to `agentic-security-review-agent`.

## Controls to look for

| Control | What to look for | Failure if absent | Evidence to request |
|---|---|---|---|
| Budgets per task | Tokens, tool calls, cost, wall-clock and depth, enforced by the runtime and carved down the delegation tree (a child's budget comes out of its parent's). Exhaustion stops the task and escalates; it does not degrade to "best effort". | A single runaway task spends the quota for everyone. | Runtime configuration, not a prompt instruction; a trace of a task hitting its limit. |
| Fan-out caps | Maximum parallel branches and maximum total subtasks per task, a per-branch timeout, and a stated partial-result policy. | Exponential spawning; partial results presented as complete. | The cap values and where they are enforced. |
| Circuit breakers on tool and model calls | Per-dependency breakers that open on error rate or latency, probe before closing, and have a defined fallback while open. | Retries pile onto a failing dependency; the failure spreads upstream. | Breaker settings per dependency; the fallback each breaker triggers. |
| Bulkheads | Separate pools, queues and quotas between tenants and between workflows, with per-tenant rate and budget limits. | One tenant or one workflow starves the rest. | Quota and queue topology; the per-tenant limit values. |
| Loop and no-progress detection | Detection of repeated tool calls with the same arguments, repeated state, and iterations without improvement, with a hard cap. | Silent loops that look like long-running work. | The detector, its thresholds, and the action it takes. |
| Backpressure and load shedding | Bounded queues; early rejection with a clear status instead of unbounded buffering. | Latency grows without bound, then everything times out at once. | Queue bounds; the behavior at the bound. |
| Degraded modes and safe fallbacks | A defined behavior per dependency: serve cached or read-only results, route to a human queue, or return a partial result with an explicit flag. Fail closed for authorization and approval dependencies: when the approval service is down, nothing executes. Fail open only for non-safety features and only by explicit decision. | Ad hoc behavior during an outage, often fail-open. | The fallback table, and the owner's decision for each fail-open choice. |
| Kill switch and disengage path | The ability to stop one task, one tenant, one agent type, or the whole system without a deployment; named roles who may pull it; defined effect on in-flight tasks (pause, cancel, or compensate); tested. | A misbehaving system cannot be stopped quickly. | Runbook, the access list, and the date of the last drill. |

Treat a fallback model as a different dependency: it may have different data handling and different guardrails. A breaker that switches models must not change authorization or data residency (`inference`).

### Kill switch and the NIST AI RMF

NIST AI RMF MANAGE 2.4 addresses mechanisms to supersede, disengage, or deactivate AI systems (per https://airc.nist.gov/airmf-resources/airmf/5-sec-core/; read the subcategory text there for its exact wording). Use it as a candidate mapping aid when an owner asks which framework language a kill-switch finding relates to. It is not a claim that any design satisfies the subcategory; an accountable owner confirms any mapping. The same page carries the banner "The AI RMF 1.0 is being updated. A revised version is in progress." so re-check the subcategory text before citing it.

## SLOs for agent tasks

An agent system that has no service objectives cannot tell a regression from a bad day. Ask for:

- Task success rate, with success defined against a quality bar that an evaluation measures. Design the evaluation with `llm-ai-pipeline-test-review-agent`.
- Time to completion as a distribution, including the tail where retries and loops show up.
- Human intervention rate and escalation rate. A rising intervention rate is a cost the value case must carry.
- Cost per successful task, reported as a distribution and not as a mean.
- An error-budget policy: what changes when an objective is burned. A reasonable default is to freeze changes that expand autonomy until the budget recovers (`inference`).

On cost, report cost per successful task as a distribution rather than a mean, per https://www.finops.org/wg/finops-for-ai-overview/; the definition and the formula live in the [value case reference](../../agentic-board-protocol/references/value-case.md), and unit prices for models and tokens are not owned here.

This repository holds no baseline for any of these objectives. An adopter measures its own.

## Observability hooks

Caveat first. Per https://github.com/open-telemetry/semantic-conventions-genai, the OpenTelemetry GenAI semantic conventions are maintained in that repository and are at Development status, which means no stability guarantee. Never present them as a stable contract. Pin the convention version in the telemetry configuration and expect renames (`inference`).

Span names recorded in the agent-spans page (per https://raw.githubusercontent.com/open-telemetry/semantic-conventions-genai/main/docs/gen-ai/gen-ai-agent-spans.md):

- create_agent {gen_ai.agent.name}
- invoke_agent {gen_ai.agent.name}
- invoke_workflow, used as the operation name

Per https://github.com/open-telemetry/semantic-conventions-genai, an execute_tool {gen_ai.tool.name} span and these attributes are also listed: gen_ai.operation.name, gen_ai.agent.id, gen_ai.agent.name, gen_ai.agent.version, gen_ai.conversation.id, gen_ai.usage.input_tokens, gen_ai.usage.output_tokens, gen_ai.tool.name, gen_ai.tool.call.id, gen_ai.provider.name and gen_ai.request.model. That list was not independently re-verified; check the repository for the version in use.

What to review in the design:

- Can one task be reconstructed end to end: delegation hops, tool calls, retries, approvals, and breaker state changes? A single trace or correlation identifier must follow the task across handoffs. The board's handoff contract carries correlation and task identifiers for the same reason; see the [handoff envelope reference](../../agentic-board-protocol/references/handoff-envelope.md).
- Are budgets spent and limits hit visible as events or metrics?
- Do spans and attributes carry prompts, tool arguments or results? They may hold sensitive data. Apply the data classification and redaction before export.
- Is observability mistaken for audit? Traces are commonly sampled and mutable. Audit evidence needs an append-only store owned by the deploying runtime; this repository stores no audit records.

## What to hand off

- Attack-driven cascades, poisoned inputs, and abuse of the kill switch: `agentic-security-review-agent`.
- Evaluation design for success-rate objectives: `llm-ai-pipeline-test-review-agent`.
- Model and token unit prices: `finops-ai-economist-agent`.
- Value-case treatment of oversight and run cost: the protocol value case.

## Plan-only containment probes to request

This skill never runs them. Ask the owner for the results, or for a plan to obtain them:

- a tabletop drill of the kill switch against a task that holds a pending side effect,
- a scenario in which one dependency becomes slow, to see which breakers open and what fallback each triggers,
- a runaway-loop scenario that must hit the budget cap and stop,
- a noisy-tenant scenario, to see whether other tenants' latency and quota are protected.
