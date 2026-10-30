# Handoff Envelope

Reference for [agentic-board-protocol](../SKILL.md). Load it when you build, review, or validate a message exchanged between board agents and accountable principals.

The envelope is the only channel between board agents and the humans who own a decision. Its shape is defined by [schemas/agentic-handoff.schema.json](../../../../schemas/agentic-handoff.schema.json), which checks shape only. Every cross-field and cross-message rule is checked by [tests/agentic-board-suite/agentic_board_rules.py](../../../../tests/agentic-board-suite/agentic_board_rules.py), so each defect fails with the id that names it.

This repository ships the contract and an offline gate. It is not an orchestration runtime: nothing here authenticates senders, enforces authorization at runtime, or stores audit records. A valid envelope authorizes nothing.

## Message types

Every envelope has one `message_type`. A type carries its own payload object (delegate carries none) and no payload that belongs to another type.

| message_type | Payload key | Purpose |
|---|---|---|
| delegate | none | Ask an agent to perform analysis. The objective is a review question. Sent by a principal or an agent. |
| return | verdict | Deliver a verdict to the requester: a return is addressed to the sender of its parent. verdict.outcome is one of pass, pass-with-conditions, block, simpler-pattern-sufficient, with a summary and optional conditions. |
| challenge | challenge | A designated challenger's adversarial finding on a proposal: proposal_task_id, round, hard_gate, outcome (sustained, withdrawn, unresolved) and finding. |
| escalate | escalation | Stop and hand the work to an accountable principal: a reason and a route_hint. The route_hint says where a human should take the work next; it is a hint, never a dispatch. Always addressed to a principal. |
| decision-request | decision_request | An agent assembles proposal, alternatives, challenges and dissent for a principal. It is never a decision. Addressed to a principal and sent by the agent it names as proposer. |
| decision-record | decision_record | The principal's decision, bound to one decision-request by digest. Sent by a principal. The protocol reserves approved and rejected for this message type. |

The decision-request and decision-record payloads are explained in [decision-record.md](decision-record.md). The value_case that an automation-investment decision-request carries is explained in [value-case.md](value-case.md).

## Field reference

All fields below are required by the schema unless marked optional. The schema rejects unknown fields, so a stray credential field fails shape validation. It cannot detect a secret pasted into a free-text field; redaction is the sender's duty.

### Identity and routing

| Field | Purpose and constraints |
|---|---|
| schema_version | Always 1.0.0. |
| message_type | One of the six types above. |
| task_id | Unique identifier of this message in the record set. Lowercase letters, digits and hyphens, 3 to 64 characters. |
| correlation_id | Groups every message of one piece of work. Same pattern as task_id. One correlation per record set. |
| parent_task_id | task_id of the message this one responds to. null only on the root message (hop_count 0). |
| hop_count | Position in the chain. The root is 0; a child is its parent plus one. The schema allows 0 to 32. |
| max_hops | Delegation depth limit for the chain, 1 to 16. It can only shrink down a chain. A delegate at or beyond it stops. |
| sent_at | RFC 3339 UTC with a literal Z and whole seconds, so string order equals time order. The gate never reads a clock; the value is whatever the sending runtime stamped. |
| source, destination | The sending and receiving party. A party is either an agent (kind agent, id ending in -agent) or a principal (kind principal, subject, role). They must differ. |

### The ask

| Field | Purpose and constraints |
|---|---|
| objective | The analysis question, 10 to 2000 characters, phrased as a review question ("Assess whether ..."). An imperative mutation request is a stop condition, not an objective. See [stop-conditions.md](stop-conditions.md). |
| input_schema_version | Semantic version of the input material the sender provided, so a receiver can refuse input in a shape it does not understand. The gate checks the pattern only. |
| expected_output_schema | Name and version of the output the requester expects, in the form name/major.minor.patch. The gate checks the pattern only. |

### Evidence

| Field | Purpose and constraints |
|---|---|
| evidence_refs | List of reference and label pairs. Labels are live-evidence, repo-evidence, user-provided-sanitized-evidence, context7-grounded, documentation-based, inference. The gate does not resolve references or judge labels. |
| missing_evidence | List of items with a blocking flag. A blocking gap on a delegate or decision-request is a stop condition; a non-blocking gap travels with the message and stays visible. |

### Scope and limits

| Field | Purpose and constraints |
|---|---|
| data_classification | public, internal, confidential or restricted, the same vocabulary as input_classification in schemas/audit-event.schema.json. It never decreases down a chain. |
| subject_risk_tier | Canonical R0 to R5 tier of the decision under review, from docs/compliance/applicability-engine.md. It is not the tier of the board's own static analysis. It never decreases down a chain. See [risk-tier-crosswalk.md](risk-tier-crosswalk.md). |
| materiality_flags | Any of financial, regulatory, security-critical, external-communication, personal-data, irreversible. Flags force full-depth review and principal escalation. They never remap subject_risk_tier and never disappear down a chain. |
| authorized_actions | One or more of the actions this message may be used for. Board agents may only be authorized read, analyze, recommend, challenge, escalate. The schema vocabulary also lists execute, mutate, deploy, approve, pay, communicate-externally, delete and grant-access, only so that a violation can be expressed and caught by its own rule. Actions never widen down a chain. |
| do_not_do_list | One or more explicit prohibitions that travel with the work, 3 to 300 characters each. Never empty; never loses an item down a chain. |
| budget | currency (three uppercase letters) plus limit and spent counters for tool calls, tokens and cost in integer minor currency units. Limits never increase down a chain and the currency never changes. Spent counters are reported by the sender. |
| deadline | UTC timestamp in the same format as sent_at. It never extends down a chain. |

### State

| Field | Purpose and constraints |
|---|---|
| approval_state | not-required, pending-human-review, approved or rejected. An agent-sourced message carries only the first two. A principal-sourced decision-record carries approved or rejected. |
| status | dispatched, completed, escalated or stopped. Escalated means the message hands the work to a principal, so it is valid only on an envelope addressed to a principal. |
| stop_reasons | Zero or more of the eight stop reason values, listed in [stop-conditions.md](stop-conditions.md). Any reason requires status stopped or escalated; status stopped requires at least one reason. |
| stop_note | Optional in general, 10 to 1000 characters. Required when conflicting-instructions is declared. |
| redactions | Optional list naming what was removed from the carried material, so a reader knows content is absent. It never lists the removed values. |
| synthetic | true for fixtures and examples. A synthetic record can never carry measured value evidence or promotion beyond pilot. |

## Principals

A principal is an accountable individual, identified by an opaque, pseudonymous subject issued by the deploying organization's identity provider, plus a role.

- The subject has the form `subject:<issuer>:<hex digest>`, for example `subject:corp-idp:7c1e9a40b2d35f68`. The schema pattern allows a lowercase issuer label and 16 to 64 hexadecimal characters.
- A subject is never a name, an email address, or a shared or group identity. This follows the stance of schemas/audit-event.schema.json, where caller_identity must be an identified individual principal and never a shared or unidentified identity for R3+ actions.
- The role is a separate, descriptive field. Self-approval and distinct-second-approver checks compare subjects, never roles: two people sharing a role are two approvers, and one person holding two roles is still one approver.
- Subjects are opaque on purpose: the record carries no names or emails, and the mapping from subject to person stays with the organization's identity provider. Opaque is not anonymous. A subject the deploying organization can map back to a person is pseudonymous personal data under the GDPR (Recital 26), so records need the organization's retention and data-protection handling like any other personal data.
- The schema cannot tell whether a subject really is an individual or whether the sender really holds it. Issuing individual subjects and authenticating the sender is the runtime's job.

## Reading a chain

By protocol the root delegate comes from a principal and has hop_count 0. The gate requires a principal root only for a correlation that carries a decision-request, whose root must also name the decision class and decision owner (`DR-OWNER-FROM-ROOT`, `DR-CLASS-FROM-ROOT`). A review-only correlation may be opened by an agent; nothing in it can be decided. Every other message points at its parent and has hop_count equal to the parent's plus one. A message is sent by its parent's destination (a reply), or by its parent's sender when the parent was stopped (escalating its own stopped delegate). A stopped message is terminal: its only permitted child is an escalate to a principal (`STOP-TERMINAL`).

Down the chain, limits only tighten: tier, classification, materiality flags and prohibitions never shrink; deadline and max_hops never extend; actions and budget limits never widen; spent counters are cumulative and never decrease. These hold per chain, parent to child. Siblings each inherit the parent's limits, so fan-out across siblings is not bounded by the gate; a runtime has to carve the budget across them. A record set in which a child asks for more than its parent granted fails that rule's own id, so any widening shows up when the record set is validated.

## What a valid envelope does NOT prove

A record set that passes the gate is well-formed and internally consistent. It does not prove:

- who sent a message. The schema and the gate do not authenticate senders; the deploying runtime must.
- that a human approval happened. An approved decision-record is data; whether the named subject really decided is the runtime's and the organization's to establish. The gate does confine approved and rejected to principal-sourced decision-records (see `HO-APPROVAL-ONLY-ON-RECORD` below), but it cannot check that the sender really is that principal.
- that sent_at is true, that spent counters are accurate, or that the data_classification and subject_risk_tier were assigned correctly. Whoever composes an envelope writes these fields; only a runtime that stamps them makes them evidence.
- that evidence_refs resolve, that evidence labels are honest, or that redactions were applied.
- that any control exists. A tool grant, sandbox or approval workflow named in a record is a claim about a runtime this repository does not contain.
- that records are stored append-only or are tamper-evident. An append log is not immutable by itself.

## Example: a synthetic delegate envelope

A principal asks the architecture agent a review question. Every value is synthetic.

```json
{
  "schema_version": "1.0.0",
  "message_type": "delegate",
  "task_id": "ex-delegate-1",
  "correlation_id": "ex-corr-1",
  "parent_task_id": null,
  "hop_count": 0,
  "max_hops": 6,
  "sent_at": "2026-11-02T09:00:00Z",
  "source": { "kind": "principal", "subject": "subject:corp-idp:7c1e9a40b2d35f68", "role": "platform-owner" },
  "destination": { "kind": "agent", "id": "agentic-system-architecture-review-agent" },
  "objective": "Assess whether the claims-triage design needs several agents or whether one agent with a fixed tool list suffices.",
  "input_schema_version": "1.0.0",
  "subject_risk_tier": "R2",
  "data_classification": "internal",
  "materiality_flags": [],
  "authorized_actions": ["read", "analyze", "recommend", "challenge", "escalate"],
  "do_not_do_list": [
    "Do not execute, deploy, or mutate any system",
    "Do not request, store, or echo credentials, tokens, or secrets",
    "Do not record an approval on behalf of a human"
  ],
  "evidence_refs": [
    { "ref": "adopter-repo:docs/design/claims-triage.md#architecture", "label": "repo-evidence" }
  ],
  "missing_evidence": [
    { "item": "Tool permission inventory for the triage agent", "blocking": false }
  ],
  "budget": {
    "currency": "USD",
    "limit_tool_calls": 40,
    "spent_tool_calls": 0,
    "limit_tokens": 400000,
    "spent_tokens": 0,
    "limit_cost_minor": 2000,
    "spent_cost_minor": 0
  },
  "deadline": "2026-11-06T17:00:00Z",
  "approval_state": "not-required",
  "expected_output_schema": "agentic-review-verdict/1.0.0",
  "status": "dispatched",
  "stop_reasons": [],
  "synthetic": true
}
```

What to notice:

- The objective is a review question and opens with "Assess", so it does not read as a mutation request.
- authorized_actions lists no execute verb, and the do_not_do_list travels with the work.
- subject_risk_tier is the requester's tier for the decision under review (R2 here, a reversible non-production change). The board's own work is static review at any tier.
- The evidence ref points into the adopter's own repository, as its adopter-repo prefix says. No such file exists in this repository, and the gate does not resolve evidence refs.
- The one missing_evidence item is non-blocking, so it does not stop the delegate, but the receiver sees the gap.
- To validate a record set, wrap envelopes as `{"messages": [ ... ]}` in a file and run `python3 tests/agentic-board-suite/agentic_board_rules.py <record-set.json>`. It prints one rule id per violation and exits 1 on any violation, and exits 2 when a path cannot be read (later files are still checked). Malformed input (invalid or non-strict JSON, duplicate keys, NaN or Infinity, nesting deeper than 32 levels, lone surrogates) reports `HO-SCHEMA`, never a traceback.
- The gate and the schema live in this repository (or its npm package). An install exported to another repository carries the agents and this protocol's prose, not the gate, so run it from a checkout of this repository.

## Rules checked by the gate

Each row names the rule id the offline gate reports. The requirement is the gate's own wording; the notes say what the rule does and does not establish.

| Rule | Scope | Requirement and notes |
|---|---|---|
| `HO-SCHEMA` | Envelope | Envelope matches schemas/agentic-handoff.schema.json (shape only), with patterns read as ECMA-262 as JSON Schema requires, so a trailing newline never passes an identifier pattern. A missing or empty messages array, a non-object document or message, invalid JSON, duplicate object keys and undigestable strings (a lone surrogate) also fail here, never as a traceback. |
| `HO-ROOT-PARENT` | Envelope | parent_task_id is null if and only if hop_count is 0. |
| `HO-NO-SELF-ADDRESS` | Envelope | source and destination are different parties, compared by agent id or principal subject. |
| `HO-NO-EXECUTION` | Envelope | authorized_actions is a subset of read, analyze, recommend, challenge, escalate. The board has no execute verb. |
| `HO-AGENT-NO-APPROVAL` | Envelope | An agent-sourced envelope carries approval_state not-required or pending-human-review. Agents cannot write approved or rejected. |
| `HO-APPROVAL-ONLY-ON-RECORD` | Envelope | A principal-sourced envelope other than a decision-record carries approval_state not-required or pending-human-review. Together with `HO-AGENT-NO-APPROVAL` and `DRC-APPROVAL-STATE`, approved and rejected can appear only on a principal-sourced decision-record. Whether the sender really is that principal is runtime-only. |
| `HO-ESCALATE-TO-PRINCIPAL` | Envelope | An escalate envelope, and any envelope with status escalated, is addressed to a principal, never to another agent. A stop marked escalated on a delegate to an agent would otherwise escape `STOP-TERMINAL`. |
| `HO-PAYLOAD-RETURN` | Envelope | A return envelope carries a verdict. |
| `HO-PAYLOAD-CHALLENGE` | Envelope | A challenge envelope carries a challenge. |
| `HO-PAYLOAD-ESCALATE` | Envelope | An escalate envelope carries an escalation. |
| `HO-PAYLOAD-DECISION-REQUEST` | Envelope | A decision-request envelope carries a decision_request. |
| `HO-PAYLOAD-DECISION-RECORD` | Envelope | A decision-record envelope carries a decision_record. |
| `HO-PAYLOAD-EXCLUSIVE` | Envelope | An envelope carries no payload belonging to another message_type. |
| `HO-STOPPED-HAS-REASON` | Envelope | status stopped requires at least one stop reason. |
| `HO-REASON-HAS-STATUS` | Envelope | Any stop reason requires status stopped or escalated. |
| `HO-STOP-NOTE` | Envelope | Declaring conflicting-instructions requires a stop_note. |
| `HO-BLOCK-AS-CHALLENGE` | Envelope | A return addressed to an agent never carries verdict block. An agent's adverse finding on another agent's work travels as a challenge, so the decision path has to carry it. |
| `HO-RETURN-EVIDENCE` | Envelope | A return whose verdict is not block cites at least one evidence_ref not labelled inference and lists no blocking missing evidence. A pass over a declared blocking gap is not a pass. |
| `HO-DESTINATION-TIER` | Envelope | Every agent destination is a catalog agent (catalog/agents.json) whose execution_tier is static-review or read-only-runtime. Untiered and mutating-runtime agents get an escalate route hint to a principal, never a delegate. |
| `HO-UNIQUE-TASK-ID` | Record set | task_id is unique within the record set. |
| `HO-ONE-CORRELATION` | Record set | All envelopes in the record set share one correlation_id. |
| `HO-ONE-ROOT` | Record set | Exactly one envelope in the record set has hop_count 0. |
| `HO-CHAIN-PARENT` | Chain | A non-root envelope's parent is present and hop_count equals the parent's plus one. A missing parent is a violation, never a pass. |
| `HO-CHAIN-TIME` | Chain | An envelope is not sent before its parent. |
| `HO-CHAIN-CUSTODY` | Chain | An envelope is sent by its parent's destination, or by its parent's sender when the parent was stopped. This is a consistency check on the record; proving the sender really is that party is runtime-only. |
| `HO-RETURN-TO-SENDER` | Chain | A return is addressed to the sender of its parent: a finding goes back to whoever asked for it. A reviewer's block verdict cannot skip the delegating agent and land with a principal outside the decision path. |
| `HO-ATTEN-ACTIONS` | Chain | authorized_actions never widen from parent to child. |
| `HO-ATTEN-BUDGET` | Chain | Budget limits never increase from parent to child, and the currency never changes. Limits are compared, not spent counters. |
| `HO-MONO-TIER` | Chain | subject_risk_tier never decreases from parent to child. |
| `HO-MONO-CLASS` | Chain | data_classification never decreases from parent to child. |
| `HO-MONO-MATERIALITY` | Chain | materiality_flags never lose a flag from parent to child. |
| `HO-MONO-PROHIBITIONS` | Chain | do_not_do_list never loses an item from parent to child. |
| `HO-MONO-DEADLINE` | Chain | deadline never extends from parent to child. |
| `HO-MONO-MAX-HOPS` | Chain | max_hops never increases from parent to child. |
| `HO-MONO-SPENT` | Chain | Budget spent counters are cumulative: they never decrease from parent to child. |
