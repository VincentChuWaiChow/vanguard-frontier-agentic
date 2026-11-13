# Fictional example: proposed asynchronous receipt delivery

This example is invented for training. All names, requirements, and observations below belong to the fictional brief. No live system was inspected, no test was run, no Jira issue was created, and no decision was accepted. Source identifiers refer only to the self-contained brief below; there are no external links.

## Fictional brief and evidence register

| Source ID | Supplied fictional information | What it establishes | What it does not establish |
| --- | --- | --- | --- |
| F1 | The team describes a checkout handler that calls an email provider before returning the receipt response. | The reported current sequence. | Actual latency, provider reliability, or verified production behavior. |
| F2 | The product lead asks that an email-provider outage not prevent a completed purchase from being acknowledged to the buyer. | A requested business outcome. | An approved latency target or delivery-time commitment. |
| F3 | The platform owner says the team can operate a background worker and a durable database table. | A stated operational capability. | Measured capacity, safe retention limits, or an implemented design. |
| F4 | The operations lead reports that buyers sometimes contact support about missing receipts. | A reported support concern. | Frequency, cause, or proof that the proposed design will reduce contacts. |
| F5 | The architecture review chair has been asked to decide this change; no review result has been supplied. | The intended decision authority and missing outcome. | Acceptance, rejection, or authorization to deploy. |
| F6 | The receipt-service team is preparing this proposal; Mira is the service owner in this fictional brief. | The draft authoring team and recorded service ownership. | Authority to accept the architectural decision. |

---

# ADR-004: Decouple receipt email delivery from checkout acknowledgement

| Metadata | Value |
| --- | --- |
| Status | Proposed |
| Author / owner | Fictional receipt-service team / Mira, service owner |
| Document version | 0.1 — initial proposal |
| Decision authority | Architecture review chair, as stated in F5; authority confirmation pending |
| Decision date | Not decided |
| Jira reference | Not linked; no issue exists in this example |
| Supersedes | None |
| Superseded by | None |

## Decision to make

Should checkout record a receipt-delivery request for a worker to process after the purchase is committed, instead of waiting for the email provider before acknowledging checkout?

## Context and drivers

F1 reports that checkout currently waits for the email provider. F2 requests that a provider outage not block acknowledgement of a completed purchase. These statements motivate investigating separation of the two operations; they do not establish an incident rate or a performance improvement.

F3 suggests the team could operate a worker and a database-backed queue. That capability is a team report and still needs a concrete capacity and failure-recovery check. F4 makes delivery visibility relevant, but the cause of reported missing receipts remains unknown.

## Options considered

| Option | Expected benefit | Cost / risk | Evidence or uncertainty |
| --- | --- | --- | --- |
| Keep synchronous delivery | Preserves the reported sequence and avoids a new worker. | Checkout acknowledgement remains coupled to the provider call. | Sequence is reported in F1; actual failure behavior is unverified. |
| Store a delivery request with the purchase, then process it asynchronously | Could separate purchase acknowledgement from provider availability and retain delivery work. | Requires worker ownership, retry rules, backlog monitoring, and handling of repeated delivery attempts. | F3 reports relevant capability; transaction boundaries and capacity are unverified. |
| Commit the purchase, then send a transient event | Could separate response handling from delivery with less persistent state. | An event publication failure after commit could leave delivery work unrecorded unless another mechanism closes that gap. | This failure window is design reasoning; no implementation or test result is supplied. |

## Proposed decision

Propose recording a receipt-delivery request in the same transaction as the completed purchase, with a worker processing pending requests. This is a recommendation for review, not an accepted decision or a deployment instruction.

The proposal prioritizes retaining delivery intent when the provider is unavailable. Its main unresolved tradeoff is the balance between retrying uncertain deliveries and avoiding duplicate emails. A worker may lose its response after the provider accepts a message; retrying could then send another copy. The provider's deduplication capabilities have not been established.

## Consequences

| Kind | Expected consequence | Required follow-up |
| --- | --- | --- |
| Positive | Checkout could acknowledge a committed purchase independently of the provider call. | Verify the transaction and response paths against F2. |
| Negative | The service gains background work, a backlog, and a new operational responsibility. | Define ownership, capacity checks, alert signals, and a recovery runbook. |
| Negative | An uncertain provider outcome may lead to duplicate receipt attempts. | Verify provider behavior and agree the business tolerance before acceptance. |
| Follow-on | Support needs a way to distinguish pending, delivered, and unresolved receipts. | Define observable delivery states without equating an attempted send with confirmed receipt. |

## Decision authority and evidence

No acceptance evidence is available. F5 identifies the intended review chair but supplies no decision record. The status remains **Proposed**. No acceptance timestamp or accepted version is recorded.

## Validation and follow-up

| Question / action | Proposed owner | Evidence needed | Due / tracking |
| --- | --- | --- | --- |
| Can purchase and delivery intent commit atomically? | Receipt-service owner | Reviewed transaction design and a failure-focused test. | Not set / not linked |
| Can the provider deduplicate repeated attempts? | Integration owner, assignment pending | Provider contract and a controlled test of uncertain responses. | Not set / not linked |
| What delivery delay and duplicate rate are acceptable? | Product lead | Explicit business decision with rationale. | Not set / not linked |
| Can the worker recover an accumulated backlog safely? | Platform owner | Capacity experiment, monitoring proposal, and recovery walkthrough. | Not set / not linked |

## Uncertainty and lifecycle

No production measurements, provider contract, schema, endpoint specification, or completed tests were supplied. No quantitative benefit is claimed. The proposal may change if atomic persistence is infeasible or the retry tradeoff cannot meet an agreed business requirement.

If this ADR is later accepted, record the actual authority, evidence, date, and accepted version. A later substantive reversal should use a new ADR with reciprocal superseding references; retain this record's original rationale.
