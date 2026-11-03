# ADK Runtime Controls

Answers one question: for an Agent Development Kit (ADK) agent bound for Agent Runtime, which framework-level controls exist, which are experimental, and what evidence a reviewer must see in the code before accepting each one. Platform identity, IAM and network controls are in [identity-and-network-boundaries.md](identity-and-network-boundaries.md); deployment mechanics are in [deployment-and-lifecycle.md](deployment-and-lifecycle.md).

## Version baseline (verified 2026-11-01)

- Latest Python release: `google-adk` 2.13.0, released 2026-10-23 (https://pypi.org/project/google-adk/). Python 3.11+. Release cadence is "roughly bi-weekly", so every claim below is pinned to 2.13.0 and must be rechecked against the version the project pins.
- A 1.x line is still maintained (1.39.1, 2026-08-27). Release 1.27.0 was yanked with the reason "Broke Agent Engine Deployments" (PyPI). A project pinned to 1.27.0 is a blocking finding.
- ADK 2.0 builds applications from two classes, `Agent` and `Workflow` (https://github.com/google/adk-python README). Templated workflow agents (`SequentialAgent`, `LoopAgent`, `ParallelAgent`) are, per https://adk.dev/agents/workflow-agents/loop-agents/, "superseded" for Python and Go by graph-based and dynamic workflows. They still exist; using them in a 2.x project is a maintenance note, not a defect.
- Context7 (`/google/adk-python`) listed versions up to v2.6.1 on 2026-11-01. Use it to corroborate call shapes, never to pin a version.

## Review table

Each row is a control the reviewer either finds in the code or reports as missing. Stage labels are ADK's own.

| Control | ADK surface | Stage | Evidence to require | Finding when absent or wrong |
|---|---|---|---|---|
| Tool guardrail | `before_tool_callback(tool, args, tool_context)` on the agent; returning a dict skips the tool and hands that dict back to the model, returning `None` lets the call run | Documented | The callback keys on `tool.name` (tool identity), not on free text in `args`; mutating tools are denied by default | High: a mutating tool reachable with no deterministic gate |
| Cross-agent policy | Plugins, applied at the runner level; "plugins are the recommended approach for implementing policies that are not specific to a single agent" (https://adk.dev/safety/) | Documented | One plugin enforces the same policy on every agent in the app | Medium: per-agent callbacks that drift apart |
| Human approval | `require_confirmation=True` on a `FunctionTool`, or `tool_context.request_confirmation(...)` | Experimental (ADK Python v1.14.0+) | See the next section before accepting it | Blocking when it is the only approval control on Agent Runtime |
| Loop bound | `LoopAgent(..., max_iterations=N)`; an exit tool sets `tool_context.actions.escalate = True` | Documented | An explicit `max_iterations` (no default is stated in the docs) | High: "the `LoopAgent` itself does not inherently decide when to stop looping" |
| Delegation | `sub_agents` tree (an agent can have one parent; a second raises `ValueError`); LLM-driven `transfer_to_agent`; explicit `AgentTool`; 2.x modes `chat`, `task` (leaf only, returns via `finish_task`), `single_turn` | Documented | Mutating sub-agents are not reachable by model-chosen transfer from an agent that reads untrusted input | High: model-chosen routing into a privileged agent |
| State scope | Prefixes: none (this session), `user:` (all sessions of one user), `app:` (all users), `temp:` (this invocation, not persisted) | Documented | Per-user or per-tenant data never written under `app:`; updates go through `append_event` | High: `app:` holding one user's data leaks it to every user |
| Session store | `InMemorySessionService`, `DatabaseSessionService`, `VertexAiSessionService` | Documented | Not `InMemorySessionService` in production ("All conversation data is lost if the application restarts.") | Medium: silent state loss on restart |
| Long-term memory | `VertexAiMemoryBankService`, `VertexAiRagMemoryService`, `InMemoryMemoryService`; `load_memory` and `preload_memory` tools | Documented | Who writes memory, from which session content, scoped to which user | Route poisoning analysis to agentic-security-review-agent |
| Code execution | Vertex code execution, ADK code executor or a custom executor; "sandboxing must be used" | Documented | Hermetic executor with no network access | High: model-generated code with network or host access |
| Tool identity | Agent-Auth (the agent's service account; "appropriate for agents where all users share the same level of access") versus User-Auth (OAuth as the controlling user) | Documented | Agent-Auth only where every user has equal access, with per-user attribution logged | High: Agent-Auth on a tool whose users have different entitlements |
| A2A exposure | `to_a2a(root_agent, port=...)` publishes a card at `/.well-known/agent-card.json`; `RemoteA2aAgent` consumes one | Experimental in Python | Who may call the exposed endpoint and what the card advertises | Route A2A trust analysis to agentic-security-review-agent |
| Telemetry schema | ADK emits OpenTelemetry GenAI spans (`invoke_agent`, `invoke_workflow`, `execute_tool`, `generate_content`) | Conventions are Development status | Dashboards and alerts name the schema version they parse | Medium: 2.13.0 "Defaults to OpenTelemetry GenAI v2 (`otel_semconv_1_44`, removing `call_llm`)"; revert with `ADK_TELEMETRY_SCHEMA_VERSION_OPT_IN=otel_semconv_1_36` |

## Human approval on Agent Runtime: the confirmation gap

The ADK docs source (https://github.com/google/adk-docs/blob/main/docs/tools-custom/confirmation.md, section "Known limitations") states: "The tool confirmation feature has the following limitations: DatabaseSessionService is not supported by this feature. VertexAiSessionService is not supported by this feature."

`VertexAiSessionService` is the managed session service an agent on Agent Runtime uses. So an approval design that relies on `require_confirmation` or `request_confirmation` in a deployed agent with managed sessions rests on an unsupported combination. The reviewer:

1. Records the session service the deployment actually uses (code, `--session_service_uri`, or the deployment config).
2. If it is `VertexAiSessionService` or `DatabaseSessionService` and confirmation is the approval control, raises a blocking finding.
3. Requires instead: a deny-by-default tool guardrail for every mutating tool, approval taken outside the agent by an accountable human, and IAM that does not let the agent's identity perform the action at all until approval is granted (see [identity-and-network-boundaries.md](identity-and-network-boundaries.md)).

## Minimal guardrail example

The shape follows the verified callback pattern (Context7 `/google/adk-python`, `src/google/adk/integrations/mongodb/README.md`, retrieved 2026-11-01). Python parameter names must match the documented names exactly (https://adk.dev/callbacks/types-of-callbacks/). Model and tool names are placeholders.

```python
from google.adk import Agent

MUTATING_TOOLS = frozenset({"issue_refund", "update_order"})


def deny_mutating_tools(tool, args, tool_context):
    if tool.name in MUTATING_TOOLS:
        return {
            "status": "ERROR",
            "error_details": f"{tool.name} requires an out-of-band human approval.",
        }
    return None  # read-only tools proceed


agent = Agent(
    name="order_assistant",
    model="MODEL_ID",
    instruction="Answer order-status questions.",
    tools=[lookup_order, issue_refund, update_order],
    before_tool_callback=deny_mutating_tools,
)
```

Limits the reviewer states with it: the denial returns to the model as tool output, so the model may try another tool or rephrase. The guardrail therefore keys on tool identity, and the privileged action must also be impossible for the runtime identity through IAM. This example was not executed in this repository; it shows the control shape, not a tested deployment.

## Common implementation errors

- Guarding on `args` text ("if 'delete' in args") instead of `tool.name`: bypassed by rephrasing.
- A policy flag the model can set: the guardrail must read state that only the developer's code writes.
- One callback per agent with different lists: use a plugin for app-wide policy.
- `max_iterations` omitted because an exit tool exists: the exit tool is model-invoked and may never fire.
- Reusing one agent instance under two parents: ADK raises `ValueError`; reviewers see it as a refactor that copied the instance.
