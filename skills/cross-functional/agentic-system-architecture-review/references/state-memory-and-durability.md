# State, memory and durability

Use this reference to review what an agentic system remembers, what survives a failure, and what happens when a step runs twice. Pattern selection decides who does what; this decides what persists and what is safe to repeat.

Memory poisoning (an attacker writing into long-term memory or context) is a security threat owned by the security agent. This reference covers memory as a state-lifecycle concern only; for the threat, see [tools, memory and data reach](../../agentic-security-review/references/tools-memory-and-data-reach.md).

## State scope

Every piece of state belongs to one scope. Ask for an inventory that names, for each scope, the owner, the schema and its version, the store, the retention, the deletion path, the isolation key, and who may read and write.

| Scope | Typically holds | Lifetime | Isolation key | Review questions |
|---|---|---|---|---|
| Per task | Working state, intermediate results, pending approvals, tool-call results | Until the task ends plus an audit retention period | Task identifier | Is it checkpointed? Who can read it after the task ends? Does it copy restricted data into a lower-classified store? |
| Per conversation or session | Message history, summaries | The session | Session identifier within a tenant | What bounds its growth? Is a summary treated as a trusted source later? |
| Per user or tenant | Long-term memory, preferences, cached retrievals | Until deleted or expired | Tenant and user keys enforced by the store | Is isolation enforced by the store or only by a prompt? Does it expire? |
| Per system | Configuration, prompts, tool registries, caches | Until redeployed | Environment | Is a change versioned and attributable? |

Isolation that exists only in a prompt ("only use this tenant's data") is not isolation (`inference`). Request evidence that the store enforces the partition key on reads and writes, and a plan-only cross-tenant retrieval test.

## Memory isolation, retention and deletion

- Retention. Every memory class needs an explicit expiry or a documented reason to have none. Long-term memory with no expiry and no review is a finding.
- Deletion. A deletion request must reach every copy: the memory store, checkpoints, replicas, caches, vector indexes, derived summaries, and any recorded execution history that captured the data. Deleting the memory store while checkpoints still hold the same content is a failed deletion.
- Replay history. Where a system replays from recorded history, payloads in that history fall inside the deletion scope. Ask how the engine supports redaction or expiry of recorded payloads, and verify against its docs for the installed version. Prefer recording references to data over recording the data itself (`inference`).
- Classification travels with state. Restricted data must not be checkpointed or summarized into a store with a lower classification.
- Backups. State the backup policy for deleted data; "deleted" and "removed from backups after N days" are different claims and should be stated separately.

## Checkpointing

A checkpoint is what lets a task resume after a pause or a failure. Review:

- What is captured: state after each side-effecting step, the pending approval, the tool results already obtained, and the position in the flow.
- Granularity. Checkpointing only at the end of a task defeats the purpose; checkpointing every token is wasteful. Ask what the design loses on a crash at its worst moment.
- Resume semantics. Resuming from the last checkpoint re-executes whatever came after it. Any side effect in that window must be safe to repeat; see idempotency below.
- Schema versioning. What happens to in-flight tasks when the state schema or the flow definition changes on deploy?
- Retention and size. Checkpoints grow and may hold sensitive data; they need the same retention and deletion treatment as other state.

Framework checkpoint primitives (for example LangGraph checkpointers keyed by thread identifier, and Microsoft Agent Framework superstep checkpoints, per the sources in [pattern selection](pattern-selection.md)) are documentation-based leads. Verify the backend and its restart behavior for the installed version.

## Finding: agent session storage is not durable execution

State this finding whenever a design calls session or history storage "durable":

- Per https://openai.github.io/openai-agents-python/running_agents/, OpenAI Agents SDK sessions automatically retrieve conversation history before each run and store new messages after each run. The same page lists separate integrations (Dapr, Temporal, Restate and DBOS) as "durable orchestration when runs may span long waits, retries, or process restarts". The page does not say whether sessions are or are not durable execution.
- Per https://adk.dev/sessions/state, the in-memory session service is lost when the process restarts.
- Both facts are `documentation-based` and were not verified against an installed version. The conclusion drawn from them is `inference`: a store of conversation history does not by itself record which side effects a run already performed, or resume a half-finished run at the step that failed. The generalization that durable execution is not native in most agent SDKs is also `inference`. Verify the installed version.

The review test: kill the process mid-task. Where does the task resume, what does it repeat, what is lost, and who finds out? A design that cannot answer has no durable execution regardless of how much history it stores.

Durable execution is needed when a task waits for a human or an external event, runs longer than a request, has side effects that must take effect once, or carries a completion commitment. It is not needed for a short single-request flow whose steps are read-only or idempotent; there, retrying the whole request is a valid design. Severity guidance: block when a multi-step process with side effects, long waits or approvals has no durable mechanism and a failure scenario is reachable; otherwise make it a condition.

## Idempotency keys for side-effecting tool calls

A tool call that changes state needs a way to be repeated safely, because retries, resumes and replays will repeat it.

- Generate the key before the call and store it with the result. A fresh random key per attempt defeats the purpose: derive it from the stable identity of the business operation and the step.
- Bind the key to the recorded decision, not to regenerated text. A model may produce different arguments on a second attempt; the key and the arguments must be recorded together and reused.
- The receiving system must deduplicate on the key and return the original result on a repeat. If the tool does not support keys, wrap it with an outbox or a check-then-act under a uniqueness constraint, or do not retry it automatically.
- Define the key's scope and lifetime. A key that expires before the longest retry or resume window protects nothing.
- The repository's own live-guard contract requires an idempotency key generated before the write and recorded in the audit log (docs/execution-tiers.md, `repo-evidence`). That contract governs this repository's gated guards, not your system, but it is a usable reference shape.

For money-moving and account-creating seams see [idempotency and safe retries](../../revenue-critical-journey-integrity-review/references/idempotency-and-safe-retries.md).

## Timeouts, retries and backoff

- Every external call, including model calls, has a timeout.
- Retries are bounded: a cap on attempts, exponential backoff with jitter, and a cap on delay. Retry only errors classified as transient. A side-effecting call without an idempotency key is not retried automatically.
- Retries share a task-level budget. Layered retries (client, gateway, agent, tool) multiply; count them across layers and charge them to the task.
- Separate transient failures (rate limits, timeouts) from content failures. A model output that fails schema validation gets a bounded number of repair attempts and then stops and escalates; it does not loop.
- Retrying a model call changes the output. If the first output was already acted on, a retry is a different decision, not a repeat.

## Compensation (saga) design

When a task spans systems with no shared transaction, plan the undo as part of the design.

- Each step has a compensating action, or is explicitly marked non-compensable.
- Identify the pivot step, the point after which the task can only move forward, and put the human approval and the highest scrutiny immediately before it.
- Compensations are idempotent and retried like any other step.
- A failed compensation has a defined owner and path, normally a human queue, not silence.
- Partial completion is visible to users and operators; a half-done task must not look finished.
- A compensating action is itself a mutation and needs the same authorization as the original. Under the board contract, agents recommend compensations; they never execute them.

## Replay safety

Where a runtime recovers by replaying recorded history, the replayed code must reach the same decisions every time.

- Durable-workflow engines such as Temporal are reported to recover by replaying recorded event history, which requires deterministic workflow code. This reference did not corroborate that against a primary page, so treat it as a lead and verify against the Temporal documentation for the installed version.
- Nondeterministic work (model calls, tool calls, clock reads, random numbers) must run as recorded activity or step results, so a replay reuses the recorded result instead of calling again.
- Review: is the model call inside the replayed workflow function, or in a recorded step? A model call in the replayed path will return something different on replay.
- Are the prompt version, model identifier and tool schema version recorded beside each result, so an investigator can see which version produced it?
- What happens to in-flight tasks on a deploy that changes the flow? Ask for the versioning strategy.
- A low sampling temperature is not determinism (`inference`). Do not accept it as a substitute for recorded results.

## Plan-only probes to request

This skill never runs them. Ask the owner for the results, or for a plan to obtain them:

- kill the process between a side effect and its checkpoint, then resume,
- deliver the same input or event twice,
- let an approval expire and confirm nothing executes,
- delete one user's data and check checkpoints, indexes and summaries,
- deploy a state schema change while a task is paused.
