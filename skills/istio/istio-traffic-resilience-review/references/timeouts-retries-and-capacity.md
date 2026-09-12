# Timeouts, retries and capacity

## TRAFFIC-04: budget the whole call path

Inventory deadline and retry ownership at caller, proxy and downstream SDK/service. Count attempts versus retries explicitly for each API. Multiplying per-layer maximum attempts is a pessimistic bound for repeated nested calls, not a prediction: deadlines, cancellation and error eligibility change actual behavior.

Record request budget, per-try timeout, backoff, retryable errors, total budget, idempotency guarantees and downstream capacity. When one number is absent, show the missing term instead of substituting a generic recommended setting. For streams and long-lived connections, confirm timeout semantics rather than using HTTP request assumptions.

Counterexample: a caller already retries a payment write and the mesh adds retries without a proven deduplication contract. The change is not safe merely because the retry count is small. Test: TRAFFIC-04.

## TRAFFIC-05: shedding and ejection

Review connection/request limits, pending requests, outlier detection, healthy capacity after ejection, locality/failover, gateway/waypoint sharing and retry load together. A high ejection percentage can remove the capacity needed to recover. Do not assert a universal circuit-breaker number without measured load and failure objectives.

Separate TCP connection counts, HTTP concurrency and queued requests. Identify whether long-lived connections delay redistribution or rollback. Tie proposed thresholds to a named service objective and measurement window.

## Canary decision

Require user-defined abort thresholds for errors, latency, saturation and downstream side effects, plus sample window and rollback owner. Canary success for reads does not establish write correctness. Tests: TRAFFIC-04, TRAFFIC-05.

Sources: VS and NETWORK for API behavior; the budget worksheet and risk rules are suite design requirements.
