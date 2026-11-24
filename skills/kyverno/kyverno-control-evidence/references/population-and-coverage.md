# What numerator, denominator and window support a coverage claim, and what must not be counted?

## Question

For a statement such as "the control covers N percent of workloads", which items form the numerator and denominator, over what window, and which tempting items must be left out?

## Release scope

- Applies to PolicyReports and exceptions for policies.kyverno.io at v1.20.0 [KYV-E01]. Result values include pass, fail, warn and error [KYV-E10]; excepted objects report skip by default [KYV-E13].
- The full value list used here is pass, fail, warn, error and skip. An exception's reportResult may be set to skip or pass, so confirm it per exception [KYV-E13].
- Legacy ClusterPolicy before v1.20 is out of scope here because its engine is removed at v1.20.0 [KYV-E01] [KYV-E28].

## Assumptions

- The inventory and its scope rule are supplied by the control owner, with a capture time.
- as_of and window are supplied by the caller; this reference sets no freshness threshold.
- Reports describe resources that exist; blocked requests are seen in metrics and events [KYV-E09].

## Evidence to request

- `population-report`: PolicyReports with per-result values and capture times, keyed to the inventory.
- `manifest` or `installed-config`: the inventory of in-scope workload objects and the scope rule.
- `installed-config`: the exception list with `policyRefs`, scope and `expiresAt` [KYV-E13].
- `runtime-metric`: kyverno_admission_requests_total by `request_allowed` for the same window [KYV-E12].
- The caller's `as_of` time and window.

## Interpretation

1. Fix the object denominator from the inventory, not from the reports. Objects with no report belong in the denominator and are blind spots.
2. Count the numerator as in-scope objects with a fresh result inside the supplied window. Freshness means a result timestamp within window of as_of; both come from the caller.
3. Do not add blocked creates to the object denominator: a denied create never became an object, and requests are visible in metrics and events [KYV-E09] [KYV-E12].
4. Do not count excepted objects as demonstrated prevention; they report skip by default [KYV-E13]. Take the excepted count from the skip results in the report, then filter the exception list by expiresAt against as_of: an exception past expiresAt is no longer applied [KYV-E13], so a skip it produced earlier may be stale.
5. A compliance percentage needs the pass, fail, warn, error and skip breakdown; error means an evaluation error, not a pass [KYV-E10]. A fresh result is not a passing result: fail results sit inside the fresh count.
6. Record lineage: each number cites the evidence item it came from.

## Worked example (synthetic)

**Requester supplied**: inventory I-1 lists 120 in-scope workload objects in the `payments` estate; report set R-1 has fresh results for 100 of them (that count includes the 8 skip results and any fail results); exception list X-1 excepts 8 of those 100, matching the 8 skip results in R-1; event stream M-1 records 6 blocked creates. as_of and window supplied by the caller.

**Agent derived**:

| Figure | Value | Lineage |
| --- | --- | --- |
| Object denominator | 120 | I-1 |
| Objects with fresh results | 100 | R-1 |
| Fresh-result coverage (includes fail and skip; not enforcement) | 100/120 = 83.3% | R-1 over I-1 |
| Excepted objects within the 100 | 8 | skip results in R-1 (excepted objects report skip by default [KYV-E13]); X-1 filtered by expiresAt against as_of |
| Objects with no fresh result | 20 | I-1 minus R-1 |
| Blocked creates (not in the denominator) | 6 | M-1 [KYV-E12] |

The 6 blocked creates are not added to 120 [KYV-E09]. The 8 exceptions are not demonstrated prevention [KYV-E13]. The 100 includes the 8 skip results and any fail results, so 83.3% is not an enforcement or pass rate. No compliance percentage is supportable without the pass, fail, warn and error breakdown for the other 92 observed objects [KYV-E10]. Filter X-1 by expiresAt against as_of before relying on the 8: an expired exception is no longer applied [KYV-E13]. The 20 without fresh results are a blind spot whose cause is still unknown.

**Still needed**: the result breakdown for the 92, the cause for the 20, and the as_of and window.

**Decision today**: `conditional`, verdict `needs-review`; 83.3% is a fresh-result coverage figure (includes fail and skip; not enforcement), not a compliance rate.

**Cheapest discriminating check**: the per-result breakdown from R-1 for the 92 objects.

**Developer-facing denial message**: not applicable; this is a measurement, not a rule.

**Escalation owner**: assurance analyst accountable for control coverage claims.

**Keep out of evidence**: customer identifiers, object names tied to tenants, report fields containing secrets.

## Counterexample

All 92 non-skip fresh results are pass, so the team states 100 percent compliance. The 20 objects with no fresh result and the 8 excepted (skip) objects are outside that claim; the honest statement is limited to the 92 observed, non-excepted objects.

## Escalation threshold

Hand to the control owner when the unknown 20 exceed what the caller will accept as a blind spot. Why a cohort has no results goes to `kyverno-policy-review-agent` for matching or `kyverno-admission-reliability-agent` for controller health. Exception approval goes to `kyverno-exception-governance-agent`.

## What would change the verdict

- The pass, fail, warn and error breakdown for the 92 objects, and confirmation that the other 8 results are skip.
- A stated cause and remediation for the 20 objects without fresh results.
- A caller-supplied window and as_of that make the 100 results fresh or stale.
- An exception list showing different counts than the report, or exceptions whose expiresAt is before as_of.
