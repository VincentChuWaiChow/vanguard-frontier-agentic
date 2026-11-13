# Worked example: a sparse duplicate-invoice report

## Supplied brief — source S1

> “Retry causes duplicate invoices.”

This is the only supplied information. It is a user report, not a verified reproduction. The ticket below is an original drafting example; no system was inspected, no implementation was performed, and all verification is **NOT RUN**.

## Triage result

Draft a bounded discovery spike. An implementation bug is **not ready**: the report does not identify the system, the operation's business identity, the tenant boundary, the intended retry contract, or proof of the failure. Do not import a 24-hour window, an Idempotency-Key, HTTP 409, a database design, or other decisions from a different example.

## Ticket: establish the failure and intended retry behavior for invoice creation

- **Type:** Spike
- **Readiness assessment (not Jira status):** Draft — the research question is clear; exact target, evidence access, investigation budget, and decision ownership remain unresolved. Initial clarification can proceed; system investigation awaits a confirmed boundary.
- **Owner:** TBD
- **Target context / revision / environment:** Unknown — not supplied.
- **Decision recipient:** TBD

### Business outcome and research question

Establish enough evidence and an agreed contract to scope a change that prevents unintended additional invoices during retries.

**Question:** Under which conditions does a retry create an unintended additional invoice, and which observable behavior should the identified operation provide instead?

### Evidence and limits

| Anchor | Evidence | Established / unestablished |
| --- | --- | --- |
| S1 | The exact supplied brief above | Establishes that a user reports duplication associated with retrying. It does not prove cause, occurrence rate, financial impact, or the intended contract. |

- **Confirmed impact:** The report exists; no actual affected invoice, customer, or financial effect is confirmed.
- **Suspected impact:** Unintended invoice records may confuse users or downstream processing. Whether money moved is Unknown and must not be inferred.
- **Observed versus expected:** Observed behavior is reported only. Expected retry behavior and whether the two invoices represent the same business operation are Unknown.
- **Root cause:** Unknown. Missing idempotency, client behavior, concurrent processing, and repeated distinct business requests are possibilities to distinguish, not findings.

### Questions that determine implementation scope

| Question | Why it matters | Decision / evidence owner |
| --- | --- | --- |
| Which product/service, operation, environment, and revision are involved? | Establishes the exact target and applicable behavior. | TBD |
| What sanitized sequence reproduces the issue, including the first response or timeout and both resulting records? | Separates a reported symptom from a demonstrated failure. | TBD |
| What identifies one business operation, and what should happen for a repeat, a changed payload, and simultaneous requests? | Defines success and conflict behavior without inventing rules. | TBD |
| How is tenant/account identity established, and where must duplicate detection be scoped? | Prevents cross-tenant collisions or disclosure. | TBD |
| Is reuse ever allowed, and is there an agreed retention/expiry rule? | Determines which repeats are intentional new work. | TBD |
| What survives a timeout or crash, and what are the confirmed side effects? | Establishes transaction and recovery boundaries, including any downstream effects. | TBD |
| Who can confirm the business contract, and what investigation budget is available? | Provides a bounded decision path without a fabricated deadline. | TBD |

### Bounded investigation plan

- **In:** Obtain the missing target and sanitized report; inspect permitted existing contracts and evidence; attempt a safe reproduction if an approved environment is available; locate the failure boundary; document options and their limits.
- **Out:** Code changes, live operations, historical data repair, selecting an unconfirmed idempotency contract, and implementing a proposed fix.
- **Budget:** Unknown — agree an effort cap before investigation. No deadline or estimate has been supplied.
- **Stopping rule:** Stop once the agreed question is answered or the agreed cap is reached. Deliver an evidence-backed conclusion or an explicit inconclusive result, with unresolved questions and their impact on implementation readiness.
- **Dependencies / blockers:** Exact target and permitted evidence access; safe reproduction context; budget; a recipient for business decisions. Owners: TBD.

### Discovery acceptance and verification

These criteria accept an investigation deliverable. They do not silently accept the eventual implementation.

| AC ID | Observable discovery deliverable | Verification method and expected evidence | Actual result |
| --- | --- | --- | --- |
| AC-01 | The report is reproduced in the confirmed safe context, or the bounded attempt is documented as inconclusive with its limits. | Review sanitized preconditions, exact revision/environment, action order, observed results, expected-behavior source, and evidence locations. If not attempted, state why. | NOT RUN |
| AC-02 | A decision record defines the operation identity, tenant boundary, repeated/changed/concurrent-request behavior, and any retention rule, or marks each unresolved rule as an implementation blocker. | Review confirmed decisions with their sources and decision owners; unresolved answers must remain Unknown rather than assumed defaults. | NOT RUN |
| AC-03 | Findings distinguish demonstrated failure mechanisms from hypotheses and compare bounded options against the confirmed contract. | Review source-linked observations, contrary evidence, limitations, and checks still needed. Do not accept a code-location guess as root-cause proof. | NOT RUN |
| AC-04 | The handoff recommends separately scoped follow-up work and explicitly assesses its readiness. | Review the discovery artifact and proposed child scopes. A ready implementation ticket requires the gate below; otherwise hand off unresolved questions or another bounded discovery step. | NOT RUN |

### Gate for a separate implementation ticket

An implementation ticket may be drafted after discovery, but assess it **Ready for implementation** only when all material items are resolved:

- The exact target/revision and the actual defect are supported by evidence.
- The business operation, tenant boundary, retry/concurrency/expiry behavior, and relevant side effects have an agreed contract.
- The proposed work has bounded in/out scope, known dependencies, and stable implementation AC IDs mapped to feasible verification methods and expected proof.
- Required decisions and target access are resolved; ownership and review arrangements are recorded honestly.

If discovery is inconclusive, it may still produce a useful completed decision brief. The unresolved implementation remains **Needs discovery**. Do not combine “investigate and implement the correct fix” into one ready ticket.

### Handoff and definition of done

Deliver the sourced investigation record, the decision table, the consumed budget if known, and proposed follow-up tickets to the confirmed human reviewer (currently TBD). Record actual evidence against AC-01 through AC-04. Unknown answers may be legitimate discovery results, but they stay visible as blockers where material.

**Current handoff:** Draft discovery scope only. Artifact/revision for an investigation result: Unknown. Human review: PENDING. Implementation: not performed. All checks: **NOT RUN**.
