# Synthetic worked example: invoice retries within an agreed idempotency contract

> **EXPLICITLY SYNTHETIC.** Every product decision, interface, fixture, and failure scenario below is hypothetical and exists only to demonstrate a good ticket. No real system was inspected. No code was implemented. All tests are **NOT RUN**. The fictional mini brief is the sole requirements source; it is not evidence of actual behavior.

## Fictional mini brief — source S1

A hypothetical invoice API has a creation operation and an authenticated tenant identity. Its accepted creation request contains exactly `customer_ref` (a string), `amount_minor` (a positive integer), and `currency` (an uppercase three-letter string). Existing validation remains in force. A valid `Idempotency-Key` is an opaque, case-sensitive string.

For this illustration, the hypothetical product decisions are:

1. An idempotent operation is identified by **authenticated tenant identity + Idempotency-Key**. Tenants may reuse the same key independently. Client-supplied tenant fields cannot replace the authenticated identity.
2. The canonical fingerprint compares the validated tuple `(customer_ref, amount_minor, currency)`: string values compare exactly; the amount compares as an integer. JSON whitespace and field order do not affect equality. Changing any tuple value changes the fingerprint. This defines the behavior without choosing a storage hash or schema.
3. The first successful creation returns HTTP `201` with its invoice response. A completed retry with the same scoped key and fingerprint replays the original HTTP status, JSON response body, and Content-Type; it creates no additional invoice. Transport metadata such as Date and request IDs is outside the replay contract.
4. Reusing an unexpired scoped key with a different fingerprint returns HTTP `409` and changes neither the original invoice nor the stored result. No replacement invoice is created.
5. While one request owns an active claim for a scoped key, any concurrent request for that scoped key returns HTTP `409` without creating an invoice. After the successful commit, a matching retry follows the completed-replay rule. Different tenants remain independent.
6. The completed record remains valid for **24 hours from the successful atomic commit**, using the authoritative service time. A retry received before expiry replays; at or after expiry it may start a new operation and create another invoice. An active claim cannot expire into a second concurrent owner.
7. Claim ownership, invoice creation, and the replayable success result have one atomic success boundary: a successful commit contains the invoice and completed result together. Failure before commit leaves no committed invoice or stale claim that prevents a subsequent valid attempt. Failure after commit but before delivery leaves a replayable result. How the target stack enforces these properties requires repository evidence.
8. This example changes valid keyed creation requests only. In the fictional baseline, invalid bodies or invalid supplied keys are rejected before invoice creation or successful replay storage; preserve that behavior. Requests without a key follow the existing non-idempotent creation contract, which is also unchanged. Historical duplicate repair, payments, invoice cancellation, and a new client retry policy are excluded.

## Ticket: prevent additional invoices from valid retries of one scoped operation

- **Type:** Bug
- **Readiness assessment (not Jira status):** Draft — the hypothetical business contract is specified; real target context, reproduction evidence, implementation feasibility, and verification commands are unknown.
- **Owner:** TBD
- **Target context:** Fictional invoice service only. Real repository, revision, environment, and relevant files: **Unknown**.

### Business outcome and impact

Allow a tenant to retry a creation request after a lost response without creating an additional invoice within the agreed retention window.

- **Confirmed real impact:** None established; no actual evidence was supplied.
- **Hypothetical reported impact:** A request times out after the service may have committed an invoice; a retry reportedly creates another invoice.
- **Suspected mechanism:** Two attempts may cross the invoice-creation boundary without a shared enforced claim. This is an unverified hypothesis, not a root-cause finding.
- **Affected frequency, population, monetary impact, severity, and priority:** Unknown. Do not derive them from this example.

### Sources and evidence limits

| Anchor | Source | What it establishes / limits |
| --- | --- | --- |
| S1 | Fictional mini brief above, decisions 1–8 | Hypothetical desired behavior; no observed implementation evidence. |
| S2 | Hypothetical report in the impact section | Illustrative symptom only; not a reproduction or measured impact. |
| V1–V7 | Proposed verification plan below | Expected proof to collect; every check is NOT RUN. |

### Observed versus expected and sanitized reproduction plan

- **Reported observation:** In S2, a retry following an uncertain first response produces an additional invoice.
- **Expected:** S1 determines whether the request replays, conflicts, or becomes a new operation after expiry.
- **Reproduction state:** Not attempted. No real trace, log, environment, or failing test exists here.
- **Proposed safe setup:** An isolated fixture with two synthetic authenticated tenants, a valid request, a valid key, a controllable clock, a concurrency barrier, and controllable failures around the transaction boundary. Whether the real stack supports these mechanisms is Unknown.
- **Proposed steps:** Send one valid creation request; interrupt response delivery after its successful commit; retry the same request as the same tenant with the same key; compare responses and committed invoice count. This is a test plan, not an observed reproduction.
- **Expected reproduction evidence:** Sanitized request order, exact target/revision, captured first and replayed application responses, and a durable-state assertion. Actual evidence: **NOT RUN**.

### Scope

- **In:** Valid keyed creation requests, authenticated tenant isolation, canonical equivalence/conflict, concurrent claims, the retention boundary, and failure recovery around the atomic success boundary.
- **Out:** Missing/invalid-key behavior changes, new validation rules, client retry policy, payments, cancellation, historical repair, and unrelated cleanup.
- **Implementation constraints:** Preserve the S1 behavior. Do not prescribe a database, file, lock mechanism, or migration until the actual target and its guarantees are inspected.

### Acceptance criteria and expected verification

All criteria below express **hypothetical decisions from S1**, not claims about an existing API.

| AC ID | Observable pass condition | Method / expected proof | Actual result |
| --- | --- | --- | --- |
| AC-01 | Before expiry, a completed retry by the same tenant with the same key and canonical fingerprint returns the original 201 status, body, and Content-Type; the committed invoice count for that operation stays one. | **V1:** Sequential retry and lost-response replay tests capture the first result, compare the replay, and inspect durable state after each attempt. | NOT RUN |
| AC-02 | Two authenticated tenants using the same key remain independent. Neither receives the other's invoice or stored response. | **V2:** A two-tenant fixture proves independent creations and replays; inspect returned tenant ownership and stored scopes. | NOT RUN |
| AC-03 | JSON field order or whitespace changes replay the original result. Changing any fingerprint field under an unexpired scoped key returns 409 with no mutation to the invoice or stored result. | **V3:** Table-driven equivalent/conflicting-payload checks cover each tuple field and compare durable state before/after conflicts. | NOT RUN |
| AC-04 | With the first request held after acquiring its claim and before commit, concurrent requests for the same scoped key return 409 and create nothing. Releasing the first request produces one committed invoice; a subsequent matching retry replays it. | **V4:** A deterministic barrier holds the winner in flight. Contenders must complete with 409 before release. After release, assert exactly one invoice and one replayable result. | NOT RUN |
| AC-05 | A request immediately before the 24-hour expiry replays. At the expiry boundary a request can create a new operation. Concurrent attempts at expiry still permit only one active owner for that scoped key. | **V5:** Use a controlled authoritative clock around the precise boundary and a concurrency barrier during expired-record turnover; inspect responses and committed operations. | NOT RUN |
| AC-06 | Failure before commit leaves neither a committed invoice nor a stale blocking claim. Successful commit persists invoice and replayable result together. Losing delivery after commit does not permit a second invoice. | **V6:** Inject failure after claim acquisition, after invoice write but before commit, and after commit but before delivery. Restart/retry as supported by the confirmed stack; inspect durable state and replay behavior. These checks must establish the atomic claim/commit property, not only successful sequential retries. | NOT RUN |
| AC-07 | Existing validation and missing/invalid-key behavior are unchanged. Invalid requests do not create an invoice or a successful replay record. | **V7:** Run the confirmed existing contract checks plus targeted invalid-request cases, comparing against the evidenced baseline. | NOT RUN |

### Unknowns and blockers before implementation handoff

| Unknown | Why it blocks readiness | Required next evidence / decision |
| --- | --- | --- |
| Actual target, revision, interfaces, and authenticated tenant source | Prevents locating the change and verifying isolation. | Confirm the target and inspect its current contracts. Owner: TBD. |
| A real reproduction or other sufficient defect proof | S2 is fictional; it establishes no real bug. | Obtain a sanitized failing trace/test or other agreed evidence. Owner: TBD. |
| Transaction, claim, recovery, and persistence guarantees | A plausible idempotency design can still fail under concurrency or crashes. | Inspect the implementation boundary and demonstrate a feasible approach against V4–V6. Owner: TBD. |
| Existing validation baseline and test harness | Commands, fixtures, fault hooks, and expected unchanged behavior cannot be guessed. | Locate known checks and establish their applicability at the confirmed revision. Owner: TBD. |
| Review and execution ownership | No real team or reviewer was identified. | Confirm the appropriate owners before assignment. Owner: TBD. |

### Handoff and definition of done

The ticket remains **Draft / Needs discovery** until the real prerequisites above are resolved. The next handoff is for target confirmation and bounded evidence gathering; no implementation is authorized by this example.

For a subsequent implementation ticket, preserve AC-01 through AC-07, record the delivered artifact/revision, map actual results to V1–V7, and request human review of the tenant boundary and atomicity proof. A sequential happy-path test cannot satisfy AC-04 or AC-06. `NOT RUN` never becomes `PASS` because the design looks reasonable.
