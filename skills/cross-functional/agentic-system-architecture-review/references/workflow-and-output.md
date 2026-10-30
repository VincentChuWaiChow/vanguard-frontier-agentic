# Workflow and output contract

Use this reference for an end-to-end architecture review of an agentic system. It fixes the order of the steps and the shape of the result so two reviews of the same design converge on the same findings. The review is static: it reads what was supplied and executes nothing.

## Workflow

1. **Classify the scope.**
   - Identify what was supplied (design document, code, configuration, trace export) and whether the system is single-vendor. A single-vendor system goes to that vendor's agent; a system that composes vendors or a custom framework stays here for the seams. The routing list is in the skill.
   - Name the decision under review and its canonical risk tier, R0 to R5, using the [risk tier crosswalk](../../agentic-board-protocol/references/risk-tier-crosswalk.md). The envelope's tier is the tier of the decision under review; the board's own work is static review. Never invent or redefine an R-label.
   - Set materiality flags from the schema's list: financial, regulatory, security-critical, external-communication, personal-data, irreversible. A flag forces full-depth review and escalation to a principal. It never changes the tier.
   - The decision class is architecture.
2. **Gather evidence.**
   - List every evidence reference with its label: `live-evidence`, `repo-evidence`, `user-provided-sanitized-evidence`, `context7-grounded`, `documentation-based`, or `inference`.
   - List missing evidence and mark each item blocking or not. Blocking missing evidence means stop and escalate to a principal; do not guess. See the [stop conditions reference](../../agentic-board-protocol/references/stop-conditions.md).
   - Ground framework API claims in Context7 for the installed version when the harness provides it. Otherwise write "verify against the official docs for the installed version".
   - Redact and flag any credential-shaped string that appears in the material. Do not echo it.
3. **Find the lowest sufficient pattern.** Walk the rungs in [pattern selection](pattern-selection.md) from the bottom. Record each rejected rung with the missing evidence that rejects it. Before accepting the single-agent rung, check that the conventional-automation alternative was considered (see the [value case reference](../../agentic-board-protocol/references/value-case.md)); if it was not, say so and route that question to the protocol instead of answering it here.
4. **Review delegation and handoff.** Use [delegation and handoff review](delegation-and-handoff-review.md): topology, routing fallback, handoff contract, attenuation, loop bounds, human placement, and the A2A lifecycle if A2A is in use.
5. **Review state and durability.** Use [state, memory and durability](state-memory-and-durability.md): scope, isolation and deletion, checkpointing, idempotency, retries, compensation, replay.
6. **Review reliability and containment.** Use [reliability and containment](reliability-and-containment.md): budgets, caps, breakers, bulkheads, fallbacks, kill switch, SLOs, observability.
7. **Reach the verdict.** Pick exactly one outcome.
   - pass: the pattern is the lowest sufficient one and no finding stands.
   - pass-with-conditions: the pattern is acceptable and each condition is specific and verifiable.
   - block: a finding with a reachable failure scenario in the design as documented has no accepted fix.
   - simpler-pattern-sufficient: a lower rung meets the stated need and the proposed topology lacks a demonstrated need. Return this even when the proposed topology also has blocking findings, and list them so they are not reintroduced.
   Block only on a failure scenario you can describe from the evidence. A general unease is not a block.
8. **Hand the verdict to the designated challenger.** For an architecture decision the designated challenger is `agentic-security-review-agent`. Send it a delegate envelope whose objective is a review question ("Assess whether ..."), not an instruction, with attenuated actions, a budget, a deadline and a do-not-do list, per the [handoff envelope reference](../../agentic-board-protocol/references/handoff-envelope.md). At R0 and R1 without materiality flags the challenge is optional under the protocol, but still name the designated challenger in the result. Its challenge references your verdict; the challenge round limit is in the stop conditions reference. Record the challenge outcome as it comes; never delete an unresolved one. Independence here is procedural, not epistemic: a separate contract and context, possibly the same underlying model family.
9. **Raise the decision request.** For R2 and above, with any materiality flag, or for an automation-investment decision, the protocol requires an attached challenge and at least one materially different alternative, and a decision request addressed to a human principal with dissent preserved. For R0 and R1 without materiality flags the challenge is optional and a return envelope with the verdict is enough. See the [decision record reference](../../agentic-board-protocol/references/decision-record.md). Agents never decide and never record an approval.

## Output template

```markdown
# Architecture review: <system name>

Verdict: <pass | pass-with-conditions | block | simpler-pattern-sufficient> - one sentence.
Status: proposal. Not approved. Designated challenger: <agent id>. Decision owner: a human principal.

## Scope
- Supplied: <artifacts and their evidence labels>
- Decision class: architecture. Canonical risk tier of the decision under review: <R0-R5>.
- Materiality flags: <list or none>. Single-vendor or cross-vendor: <which>.

## Lowest sufficient pattern
<rung> - why it is sufficient, and the evidence.

### Rungs considered and rejected
| Rung | Rejected because | Missing evidence |
|---|---|---|

## Findings
| ID | Severity | Area | Failure scenario | Evidence label | Fix direction | Verification step |
|---|---|---|---|---|---|---|

Severity is blocking (reachable failure scenario, no accepted fix), condition (acceptable only with a named, verifiable change), or advisory.

## Evidence and gaps
- Evidence used, each with its label.
- Missing evidence, each marked blocking or not.
- Not reviewed here or runtime-only: <list>

## Handoffs
- <agent id>: <what it receives and why>

## Decision request
<For R2 and above, with any flag, or for automation investment: alternatives, challenges, dissent, recommended outcome. Otherwise state that the challenge is optional per the protocol.>
```

Rules for the result:

- Every finding carries a failure scenario that says how the design reaches a wrong outcome. A finding without one is an opinion; drop it or mark it advisory.
- Never present a framework fact without its label or a "verify against the official docs for the installed version" note.
- Never write that a design satisfies a framework or is certified or compliant. A mapping is a candidate that an accountable owner confirms.
- Never include credentials, tokens, tenant identifiers, names or email addresses.

## Example verdict (synthetic)

This example is invented for illustration. No real system, team or data is described.

Scenario: a fictional team proposes a five-agent group-chat design (intake, classifier, researcher, resolver, reviewer) to triage invoice disputes for a non-production pilot. A sanitized design document was supplied.

```markdown
# Architecture review: invoice dispute triage (synthetic)

Verdict: simpler-pattern-sufficient - a deterministic workflow with two model steps and a human approval meets the stated need.
Status: proposal. Not approved. Designated challenger: agentic-security-review-agent. Decision owner: a human principal.

## Scope
- Supplied: design document (user-provided-sanitized-evidence).
- Decision class: architecture. Canonical risk tier of the decision under review: R2 (pilot scoped to a non-production environment).
- Materiality flags: financial. The flag forces a designated challenge and a principal decision; it does not change the tier.

## Lowest sufficient pattern
Deterministic workflow: classify the dispute into the fixed taxonomy (one model call), fetch facts with fixed read-only tools, draft a reply (one model call), require a human approval before any credit request, then create the request with an idempotency key.

### Rungs considered and rejected
| Rung | Rejected because | Missing evidence |
|---|---|---|
| Single agent with tools | The path does not depend on open-ended tool results; the taxonomy is fixed and enumerable (inference). Revisit if disputes need open-ended investigation. | A sample of disputes the fixed branches miss |
| Multi-agent | None of the four needs is demonstrated: all five agents hold the same tools and context, the steps are sequential, and no context-size measurement was supplied. | Any evidence for separation of duties, parallelism, context limits, or distinct permissions |

## Findings
| ID | Severity | Area | Failure scenario | Evidence label | Fix direction | Verification step |
|---|---|---|---|---|---|---|
| F1 | blocking (as proposed) | Separation of duties | The reviewer agent holds the resolver's tools and context, so it agrees with the resolver's mistaken credit request and nothing independent stops it. | user-provided-sanitized-evidence | Replace the reviewer agent with a human approval before any credit request. | Confirm no credit request path exists without an approval step. |
| F2 | condition | Durability | A dispute waits days for a customer reply while state sits in the framework's default in-memory session; a restart loses the task and the late reply has nothing to resume. | inference (verify the installed version's restart behavior) | Persist pending disputes in a durable store keyed by dispute identifier. | Kill the process mid-wait in a pilot environment and resume. |
| F3 | condition | Loop bounds | The group chat has no round cap; two agents that disagree on classification alternate until the budget of the whole pilot is spent. | user-provided-sanitized-evidence | Drop the group chat; cap any critic step at a fixed round count. | Review the configured cap and where it is enforced. |
| F4 | condition | Idempotency | The credit-request call has no idempotency key, so a retry after a timeout can create two requests. | user-provided-sanitized-evidence | Derive a key from the dispute identifier and the step. | Replay the same dispute twice in a pilot environment. |

## Evidence and gaps
- Design document: user-provided-sanitized-evidence.
- Missing, not blocking: measured token counts for real disputes; a sample of disputes the fixed taxonomy misses.
- Not reviewed here: prompt-injection paths through dispute text and the credit-request tool's authorization (security agent).

## Handoffs
- agentic-security-review-agent: designated challenger; review the recommended workflow's data reach and the credit-request path.

## Decision request
R2 with a financial flag: attach the challenge, record the deterministic workflow as the proposal and the five-agent design as the rejected alternative with its reason, preserve any dissent, and address the request to a human principal.
```

In an envelope, the verdict above is the `verdict` payload of a return message:

```json
{
  "outcome": "simpler-pattern-sufficient",
  "summary": "The five-agent group-chat design has no demonstrated need for multiple agents. A deterministic workflow with two model steps and a human approval before any credit request meets the stated need.",
  "conditions": [
    "Replace the reviewer agent with a human approval step before any credit request is created.",
    "Persist pending disputes in a durable store so a restart does not lose a waiting task.",
    "Derive an idempotency key from the dispute identifier and step for every credit request."
  ]
}
```

The handoff to the designated challenger is a separate delegate envelope. A valid objective for it is a review question, for example: "Assess whether text in a dispute record can steer the proposed workflow into creating a credit request without human approval." An imperative such as "Create the credit request" is a stop condition under the protocol, not an objective. Neither this example nor the schema proves that any approval happened; sender authentication and approval enforcement belong to the deploying runtime.
