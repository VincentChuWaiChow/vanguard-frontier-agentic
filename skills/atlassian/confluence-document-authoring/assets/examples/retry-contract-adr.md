# SYNTHETIC ADR: bounded retries for job submission

**SYNTHETIC EXAMPLE — NOT EXECUTED.** This is an invented engineering scenario, not a decision about an existing product. Names, requirements, limits, and proposed behavior below are illustrative. No implementation, approval, security review, or test result is asserted.

| Metadata | Value |
|---|---|
| Decision lifecycle | proposed |
| Decision owner | unknown — a service owner has not been assigned |
| Decision authority / delegated scope | unknown |
| Reviewer | unknown; security and API contract reviews pending |
| Revision / date | synthetic revision 0.1 / unknown |
| Source | Invented scenario premises SYN-01 through SYN-04 below |
| Jira / related documentation | unknown / unknown |
| Supersedes | none in this synthetic example |
| Superseded by | none in this synthetic example |
| Review trigger | Contract review, security review, changed retry window, or evidence of duplicate job creation |
| Verification | NOT RUN |

The lifecycle is a decision state, not a Confluence publication or deployment state.

## Question and scenario premises

Should a hypothetical client automatically retry a job-submission request when it cannot tell whether the server accepted the request?

| ID | Synthetic premise | Why it matters | Real-world evidence |
|---|---|---|---|
| SYN-01 | The example service creates asynchronous work through a proposed `POST /v1/jobs` operation. | A lost response can leave job creation ambiguous to the caller. | unknown; no real API contract supplied |
| SYN-02 | Duplicate jobs can lead to duplicate notifications and operator work. | Blind retries have a material cost in this scenario. | unknown; no incident or business-rule source supplied |
| SYN-03 | A candidate client policy has at most three total submission attempts within a 15-minute overall retry budget. | Deduplication retention must cover the allowed retry window and operational delays. | proposed scenario constraint, not an approved service limit |
| SYN-04 | The proposed design could place a job record, idempotency record, and outbox event in one transactional database. | That atomic boundary could prevent two accepted job records for one key while allowing recoverable queue delivery. | unknown; no database capability or implementation verified |

The external worker's effect contract is **unknown**. This decision cannot establish exactly-once execution or downstream side effects without that contract.

## Criteria and viable alternatives

The priorities below are proposed for this example. No authorized stakeholder has accepted them.

| Criterion | A: no automatic retry after ambiguous completion | B: durable idempotency key with bounded retries | C: client-assigned job ID with idempotent `PUT` |
|---|---|---|---|
| Avoid duplicate accepted jobs | Avoids retry-induced duplicates, but callers still need reconciliation for an uncertain result. | Proposed key uniqueness and atomic persistence suppress duplicate acceptance within the retention window. | A stable resource ID can express deduplication directly if the contract and authorization rules support it. |
| Recover from transient response loss | Requires a documented lookup or operator workflow; caller progress may be slow. | Repeating the same key can return the original job identity after a lost response. | Repeating the same resource creation can recover the identity; conflict semantics must be designed. |
| Implementation and support cost | Lowest write-path change; shifts ambiguity and support work to consumers. | Adds persistent state, concurrency handling, retention, and recovery obligations. | Changes resource naming, ownership, and API semantics; existing clients may need a larger migration. |
| Contract compatibility | Could preserve the current interface if one existed. | Potentially additive, but actual compatibility remains unknown until the real contract is reviewed. | Likely a larger interface change; its actual impact requires a contract diff. |
| Security and abuse exposure | No new key store, but existing authorization still applies. | Needs tenant/principal scoping, payload comparison, data minimization, and bounded resource use. | Needs equivalent scope and anti-enumeration controls for caller-selected identifiers. |

## Proposed decision

Select **B for further design and verification**, because it can make ambiguous submission recoverable while keeping a server-assigned job identity. A is reasonable when ambiguity is rare and a reliable reconciliation workflow exists. C is reasonable for a new API that can expose a caller-selected resource identity. This is a qualitative comparison, not a benchmark.

### Candidate contract — illustrative and unverified

1. A logical submission has one client-generated `Idempotency-Key`. Retries reuse the same key and the same validated payload. A client must not generate a new key simply because the response was lost.
2. The proposed uniqueness scope is authenticated tenant, authenticated principal, operation version, and key. Principal changes during a retry are outside this proposed contract and require reconciliation.
3. The service atomically records the key, a fingerprint of the validated request, one job ID, and an outbox event. Database support for this transaction is a prerequisite, not an established fact.
4. First acceptance returns proposed status `202` and the job ID. A repeat with the same fingerprint returns that job ID and acceptance status. Reuse with a different fingerprint returns proposed status `409` and does not create a second job.
5. A concurrent request for the same key must resolve through the same uniqueness constraint. The precise behavior while the first transaction is unresolved remains a contract-review item; it must not create an independent job.
6. Proposed key retention is 24 hours from acceptance. This is an illustrative design value requiring capacity and policy review. Automatic retries stop after the 15-minute budget; retries after expiry have no deduplication guarantee and require reconciliation rather than a fresh submission.
7. The proposed client retries transport failures or responses explicitly classified as transient by the final contract, up to three total attempts. Candidate statuses are `408`, `429`, `502`, `503`, and `504`; this list is not yet a deployed guarantee. Use bounded backoff with jitter and honor a valid `Retry-After` only within the remaining retry budget. The backoff parameters and per-attempt timeout remain unknown.
8. Do not automatically retry validation, authorization, or key/payload conflicts. Report the unresolved outcome when the retry budget expires. The job lookup and reconciliation interface must be specified before implementation.

An outbox can publish more than once after recovery. **This proposal deduplicates job acceptance only.** Worker execution and external effects need their own stable job identity, duplicate handling, and verified downstream contract. Acknowledging a queued job is not evidence that its effects completed.

## Authority and acceptance conditions

| Required evidence | State | Consequence |
|---|---|---|
| Named decision authority and delegated scope | unknown | Decision remains proposed. |
| Explicit choice and approved scope | unknown | This document provides no implementation or deployment approval. |
| Security review of key scope, retention, abuse controls, and information disclosure | pending | Production acceptance is not established. |
| Explicit authority for acceptance while security review remains pending | unknown | Do not relabel the ADR accepted with conditions. |
| API contract and downstream effect review | pending | Status codes, example path, fields, and retry semantics remain illustrative. |

If an actual authorized decision later permits conditional acceptance, record the evidence, permitted work, remaining deployment restrictions, condition owner, and exit criteria. Do not infer that exception from this example.

## Consequences and obligations

| Consequence | Implication / mitigation | Owner / Jira |
|---|---|---|
| Persistent deduplication state | Capacity, retention, cleanup, and availability affect submission behavior. Do not expire records before the guaranteed window. | unknown / unknown |
| Fingerprint stability | Define normalization and versioning from the authoritative schema; avoid storing unnecessary payload data. | unknown / unknown |
| Outbox delivery may repeat | Consumers must tolerate repeated job IDs; downstream side-effect behavior is still a blocker. | unknown / unknown |
| Cross-principal retries are outside scope | Document reconciliation and credential-rotation behavior before client adoption. | unknown / unknown |
| Retention limits the guarantee | Communicate the retry window explicitly; the client must retain its key and known job ID. | unknown / unknown |
| Rollback may strand accepted jobs or keys | Design compatibility for existing records and queued work before any rollout. | unknown / unknown |

## Verification plan

| Claim | Verification method | Required observation | Evidence |
|---|---|---|---|
| Same key and payload produce one accepted job | Concurrent requests and lost-response simulation against the actual contract | One job record; responses reconcile to the same ID | NOT RUN |
| Different payload under one key is rejected | Submit two validated but different payloads with one key | Documented conflict; no second job | NOT RUN |
| Crash recovery does not lose accepted work | Fault injection around transaction commit and outbox publication | Accepted job remains discoverable; outbox resumes without creating a new job | NOT RUN |
| Tenant/principal boundary is enforced | Authorization review and cross-scope contract tests | No unintended data disclosure or collision across authorized scopes | NOT RUN |
| Retry policy respects all budgets | Simulated transient failures, long `Retry-After`, and timeouts | At most three attempts, no retry outside overall budget, explicit unresolved outcome | NOT RUN |
| Worker effects tolerate duplicate delivery | Review and test against the material downstream schema | Defined duplicate outcome without an unsupported exactly-once claim | BLOCKED — schema unknown; NOT RUN |

## Reconsideration

Revisit the proposal if the actual API already has a stable operation ID, the database cannot provide the required atomic boundary, retry windows change, the downstream contract invalidates duplicate handling, or measured support cost favors another alternative. Supersession must link a new authorized decision; no such decision exists here.
