# Which evidence stream shows which outcome, and how do empty, stale, excluded and denied cases differ?

## Question

When an object or request is missing from the evidence, which stream should have shown it, and what does its absence mean in each case?

## Release scope

- Applies to PolicyReports, metrics and Kubernetes Events with policies.kyverno.io at v1.20.0 [KYV-E01].
- Background scans do not block or delete existing resources even with Deny, and reports describe existing resources [KYV-E09].
- Legacy ClusterPolicy behavior before v1.20 is not covered because its engine is removed at v1.20.0 [KYV-E01].

## Assumptions

- Retention of reports, metrics and events is environment-specific; it is asked for, not assumed.
- Staleness is judged against a caller-supplied as_of and window.
- A report row's presence for an excluded object in a live cluster is unverified.

## Evidence to request

- `population-report`: PolicyReports with timestamps for the cohort.
- `runtime-metric`: kyverno_admission_requests_total with `request_allowed`, `resource_kind` and `resource_request_operation` labels [KYV-E12].
- `admission-trace`: Kubernetes Events or audit events for denied requests [KYV-E09].
- `installed-config`: retention settings of reports, metrics storage and events.
- `manifest`: the scope rule (matchConditions) that defines exclusion.

## Interpretation

| Case | Object exists | Which stream shows it | What it means |
| --- | --- | --- | --- |
| Empty report | Yes | PolicyReport with no results | No evaluated result is recorded; cause unknown (matching, scan not run, controller health). Not a pass |
| Stale report | Yes | PolicyReport older than the window | Result predates the window; it may no longer hold. Needs caller as_of |
| Excluded workload | Yes | Offline: `skip` (fixture 01); in-cluster report row unverified | Out of scope by matchConditions; not evidence of enforcement, and not a failure |
| Denied create | No | request_allowed=false counter [KYV-E12]; Events [KYV-E09] | The request was blocked and no object exists; it cannot appear in a report |

1. Reports describe only resources that exist; denied requests are in metrics and events [KYV-E09].
2. A background finding for a pre-existing object is not a rejection; background scans do not block or delete existing resources [KYV-E09].
3. A `skip` for an excluded object, observed with the pinned kyverno CLI v1.20.0 in this repository's offline fixtures, means out of scope; it does not show the control worked.
4. `error` is an evaluation error, not a pass or fail [KYV-E10].
5. Retention may have removed old evidence; ask the environment owner before reading absence as absence of events.

## Worked example (synthetic)

**Requester supplied**: for the invoice service in `payments`, a PolicyReport for one Deployment with no results, another report dated before the window, a Job excluded by matchConditions, and a counter increment for a denied Pod create.

**Agent derived**: the empty report is an unexplained gap; the stale report is outside the window; the excluded Job is out of scope; the denied create is only in metrics and events [KYV-E12] [KYV-E09].

**Still needed**: as_of and window, retention settings, and the cause of the empty report.

**Decision today**: `conditional`, verdict `needs-review`.

**Cheapest discriminating check**: the report capture timestamp for the stale case against the supplied as_of.

**Developer-facing denial message**: not applicable; no developer-facing rule is evaluated here.

**Escalation owner**: assurance analyst accountable for control coverage claims.

**Keep out of evidence**: event payloads containing secrets, customer data, tenant identifiers.

## Counterexample

A namespace has zero fail results and the team reads that as no violations. The denied creates in the request counter never became objects [KYV-E09]; the fail count of zero says nothing about them.

## Escalation threshold

Hand to the environment owner when retention is shorter than the window. Why a cohort has no results goes to `kyverno-policy-review-agent` or `kyverno-admission-reliability-agent`.

## What would change the verdict

- Retention settings covering the whole window.
- Evidence of whether a report row exists for excluded objects in the cluster.
- A caller-supplied as_of and window.
- Events matching the counter increments.
