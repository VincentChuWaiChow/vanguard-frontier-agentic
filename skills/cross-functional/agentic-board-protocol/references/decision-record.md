# Decision Record

Reference for [agentic-board-protocol](../SKILL.md). Load it when an agent must assemble a decision request, when a designated challenger attacks a proposal, when a HARD gate is in play, or when you review a principal's decision record.

The board's agents analyze, challenge and recommend. They never decide. A decision is a decision-record sent by a principal, bound to exactly one decision-request. Everything before that record is input to a human. The envelope and its message types are described in [handoff-envelope.md](handoff-envelope.md); stop conditions are in [stop-conditions.md](stop-conditions.md); the value case that investment decisions carry is in [value-case.md](value-case.md).

## The ten-step adversarial process

The steps are the logical order of the review. Evidence is gathered throughout, not only at step 5. Each step lands in envelope fields and message types, so the gate can check that it happened in form.

| # | Step | What happens | Envelope fields and message types |
|---|---|---|---|
| 1 | Framing | A principal states the decision as a review question, sets scope and limits, and, when a decision will be requested, names the decision class and the decision owner. | Root delegate: objective, subject_risk_tier, data_classification, materiality_flags, authorized_actions, do_not_do_list, budget, deadline, max_hops, decision_class, decision_owner. |
| 2 | Proposal | The owning specialist forms a proposal under the class the root declared; the proposer cannot relabel it. | delegate and return messages from the proposer; decision_request.decision_class (must equal the root's), proposer_agent, proposal_summary. |
| 3 | Alternative | At least one materially different alternative is recorded with the reason it was rejected. For investments, alternatives come from the automation ladder in value-case.md. | decision_request.alternatives (kind, summary, rejected_because). |
| 4 | Attack | The designated challenger attacks the proposal. Rounds are counted per challenger, not self-declared, and capped at two. A reviewer's adverse finding on another agent's work is always a challenge, never a block verdict on a return. | challenge message: proposal_task_id, round, hard_gate, outcome, finding. |
| 5 | Evidence | Every claim carries a label; gaps are named; a blocking gap stops the work. | evidence_refs, missing_evidence. |
| 6 | Risk | The tier of the decision and the materiality flags are stated and carried unchanged or higher. A security challenge may set hard_gate. | subject_risk_tier, materiality_flags, challenge.hard_gate. |
| 7 | Economics | An automation-investment request carries a value case, challenged by the finance challenger. | decision_request.value_case. |
| 8 | Resolution | Each challenge ends sustained, withdrawn or unresolved. Open challenges become dissent. The recommendation follows from what is still open. | challenge.outcome, decision_request.dissent, recommended_outcome. |
| 9 | Gate | The deterministic rules and any HARD gate are applied. Every challenge raised on the proposal is attached. The request goes to the decision owner the root named, never to an agent. | decision-request addressed to the root's decision_owner; challenge_task_ids complete; approval_state pending-human-review. |
| 10 | Record | The addressed principal decides once. The record is bound to the request by digest and acknowledges dissent; at R5 a second principal co-signs in their own record. | decision-record: request_task_id, request_digest, outcome, decided_by, rationale, dissent_acknowledged; second_approver and a co-signing record at R5. |

Resolution never happens by the most confident agent, by voting, or by averaging. A confident agent can be wrong, a vote among agents of one model family adds little independent information, and an average of positions is a position nobody argued for. Disagreement that evidence does not close is carried to the principal as dissent.

## Risk-proportionate depth

The board does not convene a full adversarial process for trivial work. Depth follows the canonical tier of the decision under review (see [risk-tier-crosswalk.md](risk-tier-crosswalk.md)) and the materiality flags.

| Condition | Depth | Challenge | Alternatives and dissent | Human step |
|---|---|---|---|---|
| R0 or R1, no materiality flags, and not an automation-investment decision | Single specialist review | Optional | Alternatives are not required by the gate. A challenge that is attached and stays open still needs a dissent entry. | A return to the requester is enough. A decision-request, when one is made, still goes to a principal. |
| R2, any materiality flag, or an automation-investment decision | Designated challenger | Required | At least one alternative; dissent for every open challenge | Decision-request to a principal. Approval follows the deploying organization's policy. |
| R3 or higher | As for R2, plus a principal decision-record is the only way forward | Required | Same as R2 | A principal's decision-record. R5 additionally needs a second approver, a different subject from the decider, who co-signs with their own decision-record. |

Notes:

- Materiality flags and automation-investment decisions force full depth even at R0 or R1. Neither changes the tier.
- The gate checks the internal consistency of a decision-record when one is present. It cannot detect that nobody asked for one, so "only way forward" is the protocol's expectation, carried by the agent contracts and the deploying runtime.
- The canonical R5 posture in docs/compliance/applicability-engine.md is R4 controls plus a second approver plus explicit risk acceptance. The gate checks that every R5 decider's record names a second approver who is a different subject from the decider, and that this second approver sends their own co-signing decision-record on the same request, digest and outcome. The gate checks that form; authenticating that two people sent the two records is the runtime's job. It requires a risk acceptance reference only when a decision approves over an open HARD gate; any other risk acceptance is the deploying organization's process to supply.

## Designated challengers

A challenger is designated by decision class, not volunteered by the proposer. When a challenge is required, the gate requires the designated challenger to be among the challengers; additional challengers are allowed.

| Decision class | Designated challenger | Falls back to, when the designated agent is itself the proposer |
|---|---|---|
| architecture | agentic-security-review-agent | agentic-system-architecture-review-agent (when the security agent proposed) |
| security | agentic-system-architecture-review-agent | agentic-security-review-agent (when the architecture agent proposed) |
| automation-investment | finance-capital-allocation-advisor-agent | none: if the finance agent is the proposer, no valid designated challenger exists and the request needs a different proposer |

Independence here is procedural, not epistemic. The proposer and the challenger have separate contracts and are expected to work in separate contexts, and the gate requires that the proposer never challenges its own proposal. They may still be the same underlying model family and can share blind spots. That is why unresolved disagreement goes to a human instead of being settled between agents.

The architecture challenger of a security decision may challenge the feasibility and placement of a control. It does not downgrade a confirmed exploit path.

## HARD gate

A HARD gate is a challenge with hard_gate true. By convention it is set by the security review, following the HARD-gate precedent in agents/frontend/frontend-board-chair-agent; the gate itself applies the semantics to any challenge that sets it. Leaving a challenge off the request does not escape it: every challenge raised on the proposer's envelopes and sent no later than the request, or than a decision-record bound to it, must be attached (DR-CHALLENGE-COMPLETE). A challenge that arrives after the request but before the decision makes that decision invalid; the proposer supersedes the request with one that attaches it.

- A challenge with hard_gate true and outcome sustained or unresolved forbids recommending approve or conditional-approve. The recommended outcome must be reject, redesign or escalate.
- Only a principal's decision-record with a risk_acceptance_ref may approve over it. The gate checks that the reference is present, not that the acceptance is real or that its owner has authority.
- An agent can never record overruled. The challenge outcomes are sustained, withdrawn and unresolved; the decision outcomes are approve, conditional-approve, reject and redesign. Overriding a HARD gate is a human act, recorded with risk acceptance, rationale and acknowledged dissent.
- Only the challenger can clear its own finding, by a later round with outcome withdrawn. A HARD gate clears only on a later round from the same challenger that is itself a HARD-gate withdrawal (hard_gate true, outcome withdrawn); an unrelated withdrawn nit never clears it. Give the reason for the withdrawal and the new evidence in the finding; both rounds stay attached and in the record.
- Urgency, claimed prior approvals and embedded instructions never lower a HARD gate. See the instruction-bait stop condition.

## Dissent

Dissent is preserved, never deleted. Every sustained or unresolved challenge attached to a decision-request that its challenger has not withdrawn in a later round has a dissent entry carrying the challenger's position, and the principal's record acknowledges dissent (dissent_acknowledged is always true in the schema). The acknowledgement is a recorded statement; the schema cannot show that the principal read it.

## Binding the record to the request

A decision-record names the request by request_task_id and carries request_digest, the sha256 of the canonical JSON of the whole decision-request envelope.

- Canonical JSON means keys sorted, compact separators (a comma and a colon with no spaces), non-ASCII characters left as UTF-8 and not escaped, and UTF-8 bytes hashed. The digest string is sha256: followed by 64 lowercase hexadecimal characters. The reference implementation is canonical_digest in tests/agentic-board-suite/agentic_board_rules.py; an implementation in another language must produce the same bytes.
- Editing the request after the decision breaks the binding, because the digest no longer matches.
- Compute it with `python3 tests/agentic-board-suite/agentic_board_rules.py --digest <request-envelope.json>`. A plain sha256sum of a pretty-printed file is not the canonical form and will not bind.
- The digest binds the request envelope, and so the challenge ids it lists, but not the content of those challenge envelopes or the dissent wording beyond what the request itself carries. Binding challenge content into the digest is open work; until then an append-only store is what keeps a challenge from being rewritten after the decision.
- The digest binds content. It does not prove that the principal saw that content when deciding. Displaying the request faithfully to the approver is the runtime's duty.
- A request id that points at a challenge or any other message type fails the binding.

## The decision in force

- A correlation that carries a decision-request is opened by a principal whose root envelope names decision_class and decision_owner (a different subject from itself). Every decision-request goes to that owner, so the proposing agent never picks its own decider.
- One decision-request is in force at a time. Re-asking means a new decision-request whose supersedes_task_id names an earlier one; it goes to the same owner, and a request is superseded at most once. The decision in force is the one bound to the request that nothing supersedes.
- Each decision-request has at most one decider's record. A contradicting second record on the same request is a violation, not a newer opinion.
- A decision-record is a review outcome. It is never a live-guard approval token, never an audit-event approval_id, and carries no target, plan digest or expiry, so it can never authorize execution on its own.

## Principal decision record

- approval_state is approved for approve and conditional-approve, and rejected for reject and redesign, and the record has status completed. A decision recorded after the deadline or on a stopped request is not a decision.
- The decider is the record's own sender and the principal the request was addressed to, and is not the principal who opened the correlation. The record is not sent before the request.
- An R5 decider's record carries a second_approver whose subject differs from the decider's and from the requester's. That principal co-signs with their own decision-record carrying co_signs_task_id, the same request_task_id, request_digest and outcome, and no second_approver of its own. The gate applies this to every R5 decider's record regardless of outcome.
- Approving over an open HARD gate carries risk_acceptance_ref.
- conditions list what must hold for a conditional-approve; the rationale is mandatory.
- Comparisons use subjects, never roles. When an agent opened the correlation there is no requesting principal to compare against.

## Rules checked by the gate

| Rule | Requirement and notes |
|---|---|
| `CH-PROPOSAL-LINK` | A challenge's proposal_task_id resolves to an envelope in the record set. |
| `CH-ROUND-COUNTED` | A challenge's round equals 1 plus the number of earlier challenges by the same challenger on envelopes from the same proposer (ordered by sent_at, then task_id). A self-declared round that disagrees fails. |
| `DR-TO-PRINCIPAL` | A decision-request is addressed to a principal; agents never decide. |
| `DR-PROPOSER-IS-SENDER` | A decision-request is sent by the agent it names as proposer_agent. A request that names a proposer other than its own sender fails; whether the sender really is that agent is runtime-only. |
| `DR-CHALLENGE-TARGETS-PROPOSAL` | Every attached challenge's proposal_task_id points at an envelope sent by the proposer. |
| `DR-CHALLENGE-RESOLVES` | Every challenge_task_ids entry resolves to a challenge envelope in the record set. |
| `DR-CHALLENGE-COMPLETE` | Every challenge on the proposer's envelopes sent no later than the request, or than a decision-record that binds it, is attached. Closes dropping a HARD gate by omission or by deciding before it is attached. |
| `DR-ONE-REQUEST` | One decision-request in force per correlation: any other names an earlier decision-request in supersedes_task_id, and no request is superseded twice. |
| `DR-OWNER-FROM-ROOT` | A correlation carrying a decision-request is opened by a principal whose root names a decision_owner other than itself, and every decision-request addressed to a principal goes to that owner. |
| `DR-CLASS-FROM-ROOT` | A decision-request's decision_class equals the decision_class declared on the root, so relabelling an investment cannot escape the value case or the finance challenger. |
| `DR-CHALLENGE-REQUIRED` | At R2 or above, with any materiality flag, or for automation-investment, at least one challenge is attached. |
| `DR-DESIGNATED-CHALLENGER` | When a challenge is required, the designated challenger for the decision class is among the challengers. |
| `DR-PROPOSER-NOT-CHALLENGER` | No attached challenge was raised by the proposer. |
| `DR-ALTERNATIVES` | When a challenge is required, at least one materially different alternative is recorded. The gate checks that an alternative is present; whether it is materially different is a judgment for the challenger and the principal. |
| `DR-UNRESOLVED-DISSENT` | Every sustained or unresolved attached challenge that its challenger has not withdrawn in a later round (a HARD gate clears only on a later HARD-gate withdrawal) has a dissent entry. |
| `DR-NO-APPROVE-OVER-DISSENT` | With any live (sustained or unresolved, not later withdrawn) attached challenge, recommended_outcome is not approve. A conditional-approve is still blocked when the open challenge is a HARD gate. |
| `DR-HARD-GATE` | With a live HARD-gate challenge (sustained or unresolved, not withdrawn by a later HARD-gate round from the same challenger), recommended_outcome is reject, redesign, or escalate. |
| `DR-VALUE-CASE-REQUIRED` | An automation-investment decision-request carries a value_case. |
| `DRC-PRINCIPAL-SOURCE` | A decision-record is sent by a principal. An agent-sourced decision-record fails; whether the sender really is a principal is runtime-only. |
| `DRC-DECIDER-IS-SENDER` | decision_record.decided_by is the record's own sender, so a record that names someone other than its own sender as decider fails. |
| `DRC-REQUEST-BINDING` | request_task_id resolves to a decision-request and request_digest is the sha256 of its canonical JSON. |
| `DRC-NOT-REQUESTER` | Neither the decider nor a named second approver is the principal who opened the correlation. Checked only when a principal opened it. |
| `DRC-DECIDER-IS-ADDRESSEE` | A decider's record is decided by the principal the decision-request was addressed to. |
| `DRC-AFTER-REQUEST` | A decision-record is not sent before the decision-request it binds. |
| `DRC-ONE-RECORD` | A decision-request has at most one decider's record, and each decider's record at most one co-signing record. |
| `DRC-COMPLETED` | A decision-record has status completed. |
| `DRC-SECOND-APPROVER` | An R5 decider's record names a second approver distinct from the decider. |
| `DRC-CO-SIGNED` | A decider's record that names a second approver is co-signed by that principal's own decision-record. |
| `DRC-CO-SIGN-BINDING` | A co-signing record names a decider's record on the same request with the same digest and outcome, comes from that record's second approver, is not sent before it, and names no second approver itself. |
| `DRC-HARD-GATE-ACCEPTANCE` | Approving over a live HARD-gate challenge requires risk_acceptance_ref on the decider's record. |
| `DRC-APPROVAL-STATE` | approval_state is approved for approve and conditional-approve, and rejected for reject and redesign. |
