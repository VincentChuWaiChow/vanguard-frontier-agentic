# Which business measures can a pilot report honestly, and which counter-metric keeps each one honest?

## Question

Which business measures can a Kyverno pilot report from the supplied evidence, who owns each, and which paired measure stops it being gamed?

## Release scope

- Applies to policies.kyverno.io at v1.20.0 [KYV-E01]; admission metrics are those documented in [KYV-E12].
- Exceptions report as skip by default and expire per expiresAt [KYV-E13].
- Legacy ClusterPolicy before v1.20 is not covered [KYV-E01].

## Assumptions

- Every baseline is unknown until measured.
- Owners are roles, supplied by the requester.
- Windows are caller-supplied.

## Evidence to request

- `runtime-metric`: admission counters and latency histogram by request class [KYV-E12].
- `runtime-metric`: kyverno_admission_requests_total by request_allowed for blocked or denied requests [KYV-E12].
- `admission-trace`: Kubernetes Events or audit events for blocked and restored requests [KYV-E09].
- `population-report`: PolicyReports, only for results on objects that exist; they never show denied creates [KYV-E09].
- `installed-config`: the exception list with `expiresAt`; an exception past expiresAt is no longer applied [KYV-E13].
- A time-tracking record for analyst effort, supplied by the owner.
- A release-attempt log, to give legitimate blocks a denominator.

## Interpretation

| Measure | Human owner (role) | Data source (evidence type) | Denominator or unit | Window | Counter-metric |
| --- | --- | --- | --- | --- | --- |
| Legitimate releases blocked | Release manager | runtime-metric or admission-trace, plus the release log | Per observed release attempt | Caller-supplied | Unintended admissions |
| Median time to restore | Platform on-call lead | admission-trace and tickets | Minutes per blocked legitimate release | Caller-supplied | Recurrence of the same block |
| Expired exception objects not yet removed | Exception governance owner | installed-config | Count of exception objects with expiresAt before as_of [KYV-E13] | Caller-supplied | Exception volume |
| Admission p95 and p99 by request class | SRE owner | runtime-metric [KYV-E12] | Seconds per request class | Caller-supplied | Webhook errors (caller-supplied series, no KYV citation) |
| Analyst time per evidence pack | Assurance analyst | time record | Measured hours per pack | Caller-supplied | Evidence defects |

1. Baseline is unknown for every row until measured; report it as unknown, never as zero.
2. Each measure is reported with its counter-metric; one without the other is not reported.
3. Blocked requests are counted from request metrics and events, not from reports [KYV-E09] [KYV-E12].
4. Savings or incident-prevention claims are forbidden without an attributable comparison, such as a matched control group or a before and after with a stated baseline.
5. Exception counts include the skip results they produce; they are not prevention [KYV-E13]. An exception past expiresAt is no longer applied, so an expired object still present is housekeeping, not an active bypass [KYV-E13].

## Worked example (synthetic)

**Requester supplied**: a 30 day pilot on the invoice service in `payments`: 40 release attempts, 3 blocked, a metric export, and a claim that the pilot "prevented incidents".

**Agent derived**: blocked legitimate releases is 3 per 40 attempts only if the 3 are confirmed legitimate; unintended admissions are unknown, so the measure is incomplete. The prevention claim lacks an attributable comparison and is unsupported.

**Still needed**: confirmation of legitimacy for the 3, a count of unintended admissions, and a baseline.

**Decision today**: `conditional`, verdict `needs-review`; report the blocked rate with its counter-metric, withhold the prevention claim.

**Cheapest discriminating check**: the release owner's classification of the 3 blocks.

**Developer-facing denial message**: not applicable; this reference measures outcomes.

**Escalation owner**: assurance analyst accountable for control coverage claims.

**Keep out of evidence**: individual developer names, customer data, commercial figures.

## Counterexample

Time to restore falls after the pilot and is credited to better tooling. The decline is because the hardest blocks were bypassed by exceptions, so the exception volume counter-metric shows the true reason.

## Escalation threshold

Hand to the business sponsor when a measure has no owner or baseline. Exception approval goes to `kyverno-exception-governance-agent`.

## What would change the verdict

- A measured baseline for each row.
- An attributable comparison for any savings claim.
- Counter-metric data for each reported measure.
- A named owner per measure.
