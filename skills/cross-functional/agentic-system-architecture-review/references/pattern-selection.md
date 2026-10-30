# Pattern selection

Use this reference to decide the lowest sufficient pattern for an agentic system and to read an orchestration design for its failure modes. It covers the technical rungs only. The business rungs eliminate, redesign and keep the existing process come before them, and the conventional-automation rung (rules engines, ERP-native matching, scripts, RPA) sits between the deterministic-workflow rung and the single-agent rung. The [value case reference](../../agentic-board-protocol/references/value-case.md) owns those business rungs and the value case. If the design under review never faced them, say so and route the question there instead of answering it here.

Everything below is design judgment, labeled `inference`, unless a cell names a source. Framework facts come only from the delegate research snapshot further down this file, are labeled `documentation-based`, and carry the source lists stated there.

## The technical rungs

| Rung | Enter when | Evidence demanded | Move up only when |
|---|---|---|---|
| Deterministic workflow | Steps and branches are known at design time, inputs are structured or can be normalized, and each step can be tested alone. A step may call a model (classify, extract, draft) without making the flow agentic. | The step list or graph; the branch conditions; which steps call a model and why rules cannot replace that call; behavior on each step's failure; where state lives between steps. | The set of paths cannot be enumerated because the next step depends on what a tool or document returns, and a sample of real cases shows the workflow's branches miss some of them. |
| Single agent with tools | The path depends on observed results, the tool set is small and bounded, one permission set and one trust domain cover every tool, and the conventional-automation alternative was considered and rejected with a reason (see the value case reference). | A tool list with the permission each tool needs; turn, tool-call and cost caps; an evaluation showing the agent reaches the outcome (design it with `llm-ai-pipeline-test-review-agent`); why a workflow with a model step was insufficient. | One of the four needs below is demonstrated and a single agent with a better prompt or more tools does not meet it. |
| Multi-agent | At least one of the four needs below is demonstrated with evidence rather than asserted. | Evidence for each need claimed; the extra cost (model calls, latency, handoff surfaces, failure modes) accepted by the owner; the controls listed for the pattern in use. | Top rung. Re-justify it whenever the need changes. |

### The four needs that can justify more than one agent

| Need | Counts as evidence | Does not count |
|---|---|---|
| Separation of duties | A requirement or control that names the separation; the agents hold different permission sets and, ideally, different contexts, so the producer cannot verify or approve its own output. | Two prompts on the same tools with the same context. |
| Parallelism | Independent subtasks (no shared writes) and a latency or throughput target that a sequential run misses, with measured or modeled timings. | "It might be faster." |
| Context-window limits | Token measurements on real inputs showing the task does not fit one context, and an evaluation showing that summarizing or retrieving on demand loses decisive information. | An assumption with no measurement; retrieval or chunking never tried. |
| Distinct tool permission sets | A tool-to-permission map showing the sets differ and cannot be merged without widening any principal's access. | One permission set split into prompt personas. |

A claim of multi-agent need that supplies none of this evidence is `inference`. The verdict is simpler-pattern-sufficient, and the finding names the lower rung and what it would look like. Every extra agent adds model calls, a handoff surface and failure modes; the owner should accept that cost knowingly.

## Orchestration patterns as vendor-neutral concepts

A pattern is a shape of control flow, not a product. Most frameworks can express several of these, and a graph framework can express a deterministic workflow. Review the pattern first and the framework second.

| Pattern | Use when | Failure modes | Control required |
|---|---|---|---|
| Supervisor or manager-worker | Subtasks differ in skill or permission and one coordinator must integrate the results. | Coordinator becomes a bottleneck and a single point of failure; worker output enters the coordinator's context as data that may carry instructions; unbounded re-delegation. | Depth and re-delegation cap; per-worker budget carved from the coordinator's; worker output treated as data; the coordinator can never widen a worker's permissions. |
| Sequential pipeline | Stages are known, ordered and dependent. | An early mistake is laundered by later stages; no stage owns the end-to-end result. | Schema-validated stage boundaries; per-stage idempotency; stop on a failed validation instead of repairing silently. |
| Concurrent fan-out and fan-in | Subtasks are independent and the merge rule is defined. | Fan-out explosion; partial failure; merge conflicts; cost spikes; inconsistent reads. | Fan-out cap; per-branch timeout; an explicit partial-result policy; a deterministic merge rule. |
| Handoff or routing | A specialist should own the next step, or a router classifies the task. | Misrouting; routing loops; lost context; privilege carried across the handoff; guessing at low confidence. | A confidence threshold with an explicit unclassified outcome; hop cap; loop detection; minimal context transfer; attenuation of authority. |
| Group chat or debate | Adversarial challenge adds value and its token cost is accepted. | Runaway conversation; agreement that comes from a shared model family; no deterministic termination; token blowup. | Round cap; a terminator condition outside the model; a transcript budget. Independence between same-model participants is procedural, not epistemic. |
| Planner-executor-verifier | A plan can be validated before execution and verification costs less than the error it prevents. | Plan and execution drift; the verifier shares the executor's blind spots; a poisoned plan. | Executor limited to the plan's steps; a verifier with its own context and permissions; a digest binding approval to the exact plan. |
| Generator-critic loop | Output quality improves under critique and acceptance criteria are objective. | Oscillation; endless refinement; the generator gaming the critic. | Iteration cap; objective acceptance criteria; stop when an iteration produces no improvement. |
| Event-driven | Work is asynchronous, long-lived, or integrates with existing queues and systems. | Duplicate and out-of-order delivery; poison messages; event loops and storms; hidden coupling. | Idempotent consumers; a dead-letter path; versioned event schemas; a loop breaker; a stated ordering assumption. See [webhook delivery: dedup and ordering](../../revenue-critical-journey-integrity-review/references/webhook-delivery-dedup-ordering.md) for the consumer-side checks. |
| Graph or stateful workflow | Control flow branches or cycles, needs pause and resume, or needs human approval mid-run. | State schema drift between versions; checkpoint bloat; cycles that never terminate; conflicting writes to shared state. | State schema versioning; checkpoint retention; a cycle bound; defined merge rules for shared state. |

## Framework primitives (research snapshot)

Versions drift. Verify against Context7 or the official docs for the installed version before asserting any cell about the system under review. No version numbers are recorded here on purpose. The cells come from a delegate research pass on 2026-10-30 and are labeled `documentation-based`. Not every cell maps to a page listed under the table, and none was re-verified against an installed version. Treat the table as a lead for the review, not as a finding.

| Framework | Orchestration primitive | Human-approval primitive | Persistence and replay | Caveat |
|---|---|---|---|---|
| LangGraph | StateGraph, subgraphs, Functional API | interrupt() resumed with Command(resume=); needs a checkpointer | Checkpointers keyed by thread_id | The langgraph-supervisor library is archived; its README, as researched, recommends building the supervisor from tools. Verify which checkpointer backend is configured and whether it survives a restart. |
| OpenAI Agents SDK | Handoffs, Agent.as_tool(), guardrails | needs_approval produces interruptions; RunState approve or reject | Sessions retrieve and store conversation history; the docs list Dapr, Temporal, Restate and DBOS integrations separately, for durable orchestration of long waits, retries and process restarts | Pre-stable release line; the research noted breaking changes between minor releases, so pin and re-verify. |
| Microsoft Agent Framework | Sequential, concurrent, handoff, group chat and magentic orchestrations | RequestPort and request_info; tool approval; plan review in magentic | Superstep checkpoints; a Durable Task extension exists but its status was not verified | The research characterized the durability options as Azure-oriented; verify the extension's status and scope. |
| Google ADK | Sequential, Parallel and Loop agents plus LLM delegation; the newer major line adds graph and dynamic workflows (whether they supersede the first three is not stated on adk.dev/workflows; unverified) | Graph RequestInput; Tool Confirmation (marked Experimental) | SessionService; the in-memory implementation is lost on restart | A2A support was marked Experimental. Check which workflow model the installed version offers. |
| CrewAI | Crews (sequential or hierarchical) and Flows | The @human_feedback decorator | @persist and checkpointing | The default A2A protocol revision was noted as a pre-stable one; compare it with the A2A release you need to interoperate with. |
| LlamaIndex Workflows | Event-driven steps (@step) | InputRequiredEvent and HumanResponseEvent | Ephemeral by default; Context.to_dict for serialization; a DBOS-backed runtime for durability | A2A support was not established by the research. |
| Bedrock AgentCore | None. It hosts any framework and is not an orchestrator | Gateway elicitation and Policy | Memory service and Runtime sessions; no workflow replay | Review the hosted framework's orchestration and durability separately. Use `aws-agentcore-agent` for AgentCore specifics. |
| Temporal | Workflows, activities and timers | Signals and updates with wait_condition | Event-history replay with deterministic workflow code, as reported by the research pass; the page listed below does not show this, so verify against the Temporal documentation for the installed version | The research pass noted official integrations for the OpenAI Agents SDK, LangGraph (public preview) and ADK (pre-release). The listed page is an index and gives no statuses, so verify each integration's status on its own page before relying on it. |

Pages the research pass consulted (page-to-claim mapping within each list was not recorded, and a listed page may not support every cell):

- LangGraph: https://docs.langchain.com/oss/python/langgraph/persistence and https://docs.langchain.com/oss/python/langgraph/interrupts
- OpenAI Agents SDK: https://openai.github.io/openai-agents-python/multi_agent/, https://openai.github.io/openai-agents-python/handoffs/, https://openai.github.io/openai-agents-python/human_in_the_loop/ and https://openai.github.io/openai-agents-python/running_agents/
- Microsoft Agent Framework: https://learn.microsoft.com/agent-framework/workflows/orchestrations/, https://learn.microsoft.com/agent-framework/workflows/checkpoints/ and https://learn.microsoft.com/agent-framework/workflows/human-in-the-loop/
- Google ADK: https://adk.dev/workflows/, https://adk.dev/sessions/state and https://adk.dev/a2a
- CrewAI: https://docs.crewai.com/en/concepts/flows
- LlamaIndex Workflows: https://developers.llamaindex.ai/python/llamaagents/workflows/durable_workflows/
- Bedrock AgentCore: https://docs.aws.amazon.com/bedrock-agentcore/latest/devguide/release-notes.html
- Temporal: https://docs.temporal.io/develop/python/integrations (an integrations index; it does not by itself support the replay, determinism or integration-status cells above)

Reading the table:

- Durable execution is not native in most agent SDKs. The generalization is `inference`, drawn from the OpenAI Agents SDK row (the docs present sessions and the durable-orchestration integrations as separate things) and the ADK row (in-memory sessions are lost on restart). A multi-agent framework is not a durable workflow, so the review must ask where durability comes from. See [state, memory and durability](state-memory-and-durability.md).
- AgentCore hosts frameworks rather than orchestrating them. A design that says "AgentCore orchestrates our agents" has an unreviewed orchestration layer.
- An archived library, an experimental feature, or a preview integration is a finding when the design depends on it for a control (approval, persistence, A2A). Condition the verdict on a verified replacement or an accepted risk.

## Recording the result

Write down, for the verdict:

- the lowest sufficient rung and the evidence that supports stopping there,
- each higher rung rejected, with the missing evidence named,
- for a multi-agent verdict, the demonstrated need from the four above and the controls required for each pattern used,
- the framework facts relied on, each labeled `context7-grounded` or `documentation-based`, or marked "verify against the official docs for the installed version".
