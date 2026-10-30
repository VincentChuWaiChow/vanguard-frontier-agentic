# Delegation and handoff review

Use this reference after the pattern is chosen, to review how work and authority move between agents, tools and humans. The review is static: it reads the design, the contracts and the configuration that were supplied. It does not exercise any handoff.

## Topology and delegation boundaries

Draw the delegation graph from the supplied evidence before judging it. Nodes are agents, workers, tools and humans. Edges are delegations and handoffs. Mark on each node its trust domain, permission set and the highest data classification it touches, and mark where a human gate sits.

For every edge, answer:

- Who may delegate to whom, and is that written down or only implied by a prompt?
- What travels with the delegation: the objective, the inputs, the permissions, the budget, the deadline, the prohibitions?
- What comes back, in what shape, and who validates it before the next step uses it?
- Who owns the outcome? Exactly one accountable owner per outcome; a delegating agent still owns what it delegates until it has verified the result.
- Is the hop justified? Each hop adds latency, cost and a failure surface. Prefer shallow topologies and demand a reason for each added level (`inference`).

Delegation is a privilege boundary. If a child needs authority its parent does not hold, that is an escalation to a human or to a different principal, not a delegation. Identity, token handling and confused-deputy paths belong to the security agent: see [identity and delegated authorization](../../agentic-security-review/references/identity-and-delegated-authorization.md) and hand the question over instead of answering it here.

## Routing confidence and fallback

A router that always picks something is guessing. Review these points:

- Does the router emit a score, a confidence, or an explicit unclassified outcome? Unclassified must stop the flow and ask or escalate, never fall through to a default specialist that can act.
- Where does the threshold come from: a labeled routing set that was measured, or a number someone chose? An unmeasured threshold is `inference`.
- What happens on a tie, or when two specialists both match? Serialize, run read-only first, or ask; do not pick at random.
- Is there loop detection (A routes to B routes to A) and a cap on re-routing?
- Is the fallback side-effect free? A human queue or a read-only answer is a safe fallback; a default action-taking agent is not.
- Is an action-taking specialist ever reachable by classification alone? Prefer a human or policy gate in front of any specialist whose tools mutate state.

Repository pattern (`repo-evidence`): this repository's maestro routing gate in `tests/validate-maestro-routing.py` models an unclassified mode on zero score, and its live-guard agents are never auto-dispatched. Adopters can mirror the shape; the gate itself only checks this repository's fixtures.

Plan-only scenarios to request from the owner: an ambiguous input, an input that names a specialist the user should not reach, an empty input, and an input that matches two specialists equally.

## Handoff contract review

A handoff is a contract between two parties that may fail independently. Review it as one. The board's own contract, [schemas/agentic-handoff.schema.json](../../../../schemas/agentic-handoff.schema.json), is a worked example. This reference does not restate its rules; the fields, the message types and the rule list live in the [handoff envelope reference](../../agentic-board-protocol/references/handoff-envelope.md).

Ask of the design under review:

- Is the handoff schema versioned, and is it validated at the receiving boundary rather than trusted because the sender is "one of ours"?
- Does it carry the objective as a question, evidence references with labels, missing evidence flagged as blocking or not, the budget, the deadline, the permitted actions, the prohibitions, the data classification and the risk tier?
- Is the channel closed? A side channel (shared scratchpad, free-form chat, a shared memory key) that bypasses the contract defeats it.
- What does the receiver do with a malformed handoff: reject, or repair silently? Reject, and report.
- Do handoffs carry secrets? They must not. Credentials belong to the runtime's identity layer, not the message.
- Does each handoff carry a correlation identifier so the whole task can be reconstructed later?

What a valid handoff does not prove: a schema checks shape only. Whether the sender is who it claims to be, whether it is authorized to send, and whether an approval really happened are runtime properties. The repository's rule module (`python3 tests/agentic-board-suite/agentic_board_rules.py <record-set.json>`) checks internal consistency of a record set offline; it authenticates nothing and enforces nothing at runtime. State this limit when the design relies on the contract for assurance.

## Privilege attenuation down chains

Authority should only narrow down a delegation chain, and sensitivity should only rise. Concretely, check that each child has no permitted action, budget, deadline, depth allowance or data reach larger than its parent's, and that the risk tier, data classification, materiality flags and prohibitions never shrink.

Review questions:

- Can a child be given a tool its parent lacks?
- Does a child inherit the parent's credentials wholesale, or is it issued its own narrower, shorter-lived grant? (Credential handling goes to the security agent.)
- Does a returned result carry authority back up the chain (an output that an upstream agent treats as an instruction)?
- Is attenuation enforced by the runtime, or only written in a prompt or schema?

The board encodes monotonic attenuation as checkable fields; see the [handoff envelope reference](../../agentic-board-protocol/references/handoff-envelope.md). Encoding is not enforcement. A schema rule that passes offline does not stop a runtime from granting more.

## Loop and recursion bounds

Every loop needs a bound that the runtime enforces, not one the model is asked to respect. Check for:

- a maximum delegation depth and a maximum number of iterations per agent,
- a round limit on critic, debate and challenge loops,
- a task-level budget (tokens, tool calls, cost, wall-clock) shared down the tree,
- repetition and no-progress detection (same tool, same arguments, same state),
- what happens at the bound: stop and escalate, not "best effort."

The board's computed stop conditions for depth, deadline, budget, missing evidence, routing loops and challenge rounds are defined in the [stop conditions reference](../../agentic-board-protocol/references/stop-conditions.md). They are fixture checks over envelope fields run by an offline gate. They are not runtime containment. Whether the design under review contains a loop at runtime is reviewed in [reliability and containment](reliability-and-containment.md).

## Human-in-the-loop placement

Place approvals where a wrong action is costly or irreversible, not on every step; constant prompts teach approvers to click through (`inference`). Usual placements: before an irreversible or external side effect, at any privilege escalation, at an unclassified or ambiguous routing result, and at a budget overrun.

Review questions:

- Can the system pause and resume? An approval that holds a connection open for hours is a durability defect. See [state, memory and durability](state-memory-and-durability.md).
- Is the approver an authenticated individual, and is the approval bound to the exact action proposed (for example by a digest of the request) rather than to the agent's summary of it? The board binds a decision to a request by a sha256 digest of its canonical form; see the [decision record reference](../../agentic-board-protocol/references/decision-record.md). Authenticating the approver is a runtime task.
- What happens on approval timeout or an unavailable approver? Fail closed: no action.
- Can the agent re-ask with different wording until it gets a yes? Cap and log repeated requests.
- Does the approval prompt show the real action and its arguments, or only a model-written description? A persuasive description can be an instance of Human-Agent Trust Exploitation (OWASP ASI09, per https://genai.owasp.org/2025/12/09/owasp-top-10-for-agentic-applications-the-benchmark-for-agentic-security-in-the-age-of-autonomous-ai/); hand it to the security agent.
- Does any agent approve its own or another agent's work? Agents never record approvals under the board contract.

## A2A task lifecycle as a resumability boundary

When agents communicate over the Agent2Agent protocol, the task lifecycle marks where work can pause and where it ends. From the specification at https://a2a-protocol.org/latest/specification/ (the latest released revision when this was verified on 2026-10-30; verify against the revision your implementation targets):

- Interrupted states are INPUT_REQUIRED and AUTH_REQUIRED. The task is waiting on something outside the agent.
- Terminal states are COMPLETED, FAILED, CANCELED and REJECTED.
- The full state list, per the same specification (https://a2a-protocol.org/latest/specification/), is SUBMITTED, WORKING, COMPLETED, FAILED, CANCELED, INPUT_REQUIRED, REJECTED and AUTH_REQUIRED, spelled with a TASK_STATE_ prefix in the enumeration. The full enumeration was not re-verified here; check it against the revision you target.

Design consequences to review:

- An interrupted state is a durable wait, not a blocked thread. The design must persist what it needs to resume: the task identifier, the context, the pending question, and who must answer. It needs a timeout for a task never resumed. See [state, memory and durability](state-memory-and-durability.md).
- A terminal state ends the task's lifecycle. Treat a retry as a new task unless the specification revision you target says otherwise.
- State updates can repeat. Handle a duplicate transition idempotently.
- Does the client treat REJECTED and FAILED differently? Verify what each means in the revision you target, and check that the follow-up behavior matches it.
- AUTH_REQUIRED is not authorization. The specification states that agents "MUST NOT treat the TASK_STATE_AUTH_REQUIRED state transition, by itself, as authorization for any particular operation", and that a credential obtained in that state "MUST NOT be assumed to authorize subsequent messages on the Task unless that behavior is explicitly defined by the implementation, credential issuer, or extension". Review the lifecycle here; hand the authorization aspect, including security schemes and Agent Card signatures, to `agentic-security-review-agent`.

## What to hand off

- Authentication, delegated authorization, token handling, Agent Card trust: `agentic-security-review-agent`.
- MCP tool-schema fidelity and protocol version for tools exposed over MCP: `typescript-mcp-tool-contract-agent`.
- How to measure routing accuracy and task completion: `llm-ai-pipeline-test-review-agent`.
