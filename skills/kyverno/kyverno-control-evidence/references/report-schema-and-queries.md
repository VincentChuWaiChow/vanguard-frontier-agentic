# What does a PolicyReport record, and which questions can its fields answer?

## Question

Which fields does a PolicyReport result carry, and which coverage, attribution, freshness and exception questions can those fields answer, with which other evidence?

## Release scope

- PolicyReport and ClusterPolicyReport at Kyverno v1.20.0 are served and stored as `wgpolicyk8s.io/v1alpha2` [KYV-E47]. Legacy ClusterPolicy results before v1.20 are out of scope because that engine is removed at v1.20.0 [KYV-E01].
- The v1.20.0 Helm chart leaves Reports Server and OpenReports disabled by default [KYV-E42]; confirm which report producer the environment runs before reading a report as complete.

## Assumptions

- The inventory, as_of time and window are supplied by the caller; this reference sets no freshness threshold.
- The reports supplied are sanitized exports with capture times.
- Retention and any report producer settings are environment-specific and requested, not assumed.

## Evidence to request

- `population-report`: PolicyReport or ClusterPolicyReport exports with `results[]` and `summary`, with capture time [KYV-E47].
- `manifest` or `installed-config`: the in-scope inventory and its scope rule, with capture time.
- `installed-config`: the chart values for `reportsServer.enabled` and `openreports.enabled`, and report retention [KYV-E42].
- `installed-config`: PolicyExceptions with `policyRefs`, `reportResult` and `expiresAt` [KYV-E13].
- `runtime-metric`: `kyverno_admission_requests_total` by `request_allowed` for the same window [KYV-E12].
- `manifest`: the policy's `auditAnnotations`, if per-result properties are relied on [KYV-E48].

## Interpretation

1. A result carries `policy`, `rule`, `result` (pass, fail, warn, error or skip), `source`, `timestamp`, `scored`, `severity`, `category`, `properties` and `resources`; the report carries a summary count for each result value [KYV-E47]. Read `result` from the report, not from the summary alone: the summary is a count, not a list of objects.
2. Question: how many in-scope objects have a result? The summary counts results, so coverage needs the summary plus an inventory; an object with no report is only visible by comparing against the inventory [KYV-E47]. Counts in the summary can exceed the object count when one object has several results.
3. Question: which policy or rule produced a result? Attribution needs `policy` and `rule`; a blocked-request counter has no such label [KYV-E12], so a denial cannot be attributed to a policy from that counter alone.
4. Question: is the result fresh? Freshness compares the result `timestamp` to a caller-supplied as_of and window [KYV-E47]. The agent never sets the threshold.
5. Question: was it a violation? `fail` is a violation, `error` is an evaluation error, `warn` appears for example when `policies.kyverno.io/scored` is false [KYV-E10]. A fresh result is not a passing result.
6. Question: why did it fail or what should the owner do? `properties` hold extra key-value data, and auditAnnotations attach key-value data to results [KYV-E48]. Treat the property keys as policy-author conventions; they carry no meaning unless the policy defines them.
7. Question: was the object excepted? Excepted objects report `skip` by default; an exception's `reportResult` may be `pass` [KYV-E13]. A skip alone cannot distinguish an exception from an out-of-scope exclusion; join to the exception list, then check `expiresAt` against as_of, since an expired exception is no longer applied [KYV-E13].
8. Question: were requests blocked? Reports describe existing resources; blocked requests are investigated through metrics and Events [KYV-E09]. They never enter a report or the object denominator.
9. An image policy `pass` can mean nothing failed rather than an image was verified. Observed with the pinned kyverno CLI v1.20.0 in this repository's starter kits: an image outside `matchImageReferences` reports `pass`, so a `pass` on an image policy is not verification.
10. An empty report is not a pass: report presence depends on the producer being enabled and running [KYV-E42], and the cause of absence is unknown until checked.

## Worked example (synthetic)

**Requester supplied**: decision KYV-DEC-0001 for the invoice service in `payments`: an inventory of 120 in-scope objects, a 7-day report set, sanitized exceptions and request counters, with as_of 2026-11-20T00:00:00Z.

**Agent derived**: tracing each number to the field that supports it.

| Number | Meaning | Field or source |
| --- | --- | --- |
| 120 | in-scope objects | inventory, not reports |
| 100 | objects with a fresh result | result `timestamp` against as_of and window, per object [KYV-E47] |
| 8 | excepted objects | results with `result: skip`, joined to the exception list [KYV-E13] |
| 20 | objects with no fresh result | inventory minus objects with a fresh result; cause unknown |
| 6 | blocked creates | request counter with `request_allowed` false [KYV-E12]; not objects [KYV-E09] |

The remaining 92 observed objects have no supplied pass, fail, warn or error breakdown, so `100/120` is fresh-result coverage, not a compliance rate.

**Still needed**: the per-result breakdown for the 92, the cause for the 20, retention and the report producer settings [KYV-E42].

**Decision today**: `conditional` / `needs-review`.

**Cheapest discriminating check**: the per-object result list for the 92, grouped by `result`.

**Developer-facing denial message**: not applicable; this is a measurement, not a rule.

**Escalation owner**: assurance analyst.

**Keep out of evidence**: object names tied to customers, request payloads, user identities in Events, tokens.

## Counterexample

A namespace summary shows `fail: 0` and the team reports no violations. The inventory is unknown, so objects with no result are invisible, and denied creates are not in reports at all [KYV-E09]. A summary with zero fail and an unknown inventory supports no claim about the population.

## Escalation threshold

Hand to `kyverno-policy-review-agent` when objects have no result and matching is suspected; to `kyverno-admission-reliability-agent` when controller or report producer health is suspected; to the environment owner when retention is shorter than the window.

## What would change the verdict

- An inventory with capture time and scope rule for the full window.
- The pass, fail, warn and error breakdown per object [KYV-E10].
- Chart values showing the report producer enabled [KYV-E42].
- Exceptions with `expiresAt` that confirm which skips were still applied at as_of [KYV-E13].
