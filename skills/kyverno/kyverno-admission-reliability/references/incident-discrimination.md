# Which observation separates a correct policy denial from a webhook call error, an upstream API rejection, or stale reports?

## Question

Given an observed admission failure, which hypothesis explains it, and which single observation would separate it from the others?

## Release scope

- Applies to policies.kyverno.io at v1.20.0 [KYV-E01]; failurePolicy semantics are in [KYV-E04] and webhook defaults in [KYV-E08].
- Legacy ClusterPolicy before v1.20 has the same failurePolicy default of Fail [KYV-E28].
- Kyverno releases other than v1.20.0 are unverified here.

## Assumptions

- The API response text is supplied verbatim, with timestamp.
- Reports are read as describing existing resources only.
- Metric label names are those documented in [KYV-E12]. How that counter records webhook call errors or timeouts is unverified, so a flat or non-rising request_allowed=false series does not exclude a call error.

## Evidence to request

- `admission-trace`: verbatim API server error for the failed write, and an audit event if available.
- `runtime-metric`: kyverno_admission_requests_total by request_allowed for the window [KYV-E12]. Any webhook error or timeout series is caller-supplied; name its source.
- `installed-config`: webhook entries and the policy's failurePolicy.
- `population-report`: PolicyReports with capture times, if staleness is suspected.
- Kubernetes Events for the denied request [KYV-E09].

## Interpretation

Rank by the discriminating evidence in the table, not by words such as likely.

| Hypothesis | Predicts | Discriminating evidence | Contradicted by |
| --- | --- | --- | --- |
| Correct policy denial | Response carries the policy message; the counter with request_allowed=false increments for policy denials [KYV-E12] | A policy name and message in the API response; denied counter rises | An error that names the webhook call and has no policy message |
| Webhook call error or timeout | API server error naming the webhook, no policy message; with Ignore the write is admitted unevaluated [KYV-E04] [KYV-E06] | Error text naming the webhook failure; webhook latency near timeoutSeconds | A policy message in the response; a policy message in the response with request_allowed=false rising for the same request class (how call errors are counted is unverified) |
| Upstream API rejection | Schema, quota or RBAC rejection without any Kyverno message | Rejection text from the API server's own validation, with no webhook named | A response naming the Kyverno webhook or carrying a policy message |
| Stale or incomplete reports | Reports lag or omit resources; denied requests absent from reports [KYV-E09]; an `error` result means evaluation error [KYV-E10] | Report capture times older than the as_of; denied requests found in metrics or events but not reports | A report result timestamped inside the incident window that matches the observed resource |

1. Read the response text first; the presence of a policy message is the strongest single discriminator.
2. If the response names a webhook failure, treat it as a call error regardless of the rule's correctness; failurePolicy then decides the effect [KYV-E04].
3. Reports never contain denied creates because those objects do not exist; use metrics and events for them [KYV-E09].
4. An `error` report result signals an evaluation error, not a violation [KYV-E10].
5. The image verification functions return counts [KYV-E18]. Whether a given count makes the validation false depends on the policy's expression, which must be read from the supplied manifest; a validation that evaluates false is a denial that carries the policy message [KYV-E04]. A registry dependency error is different and its surfacing is unverified; it needs an admission-trace.

## Worked example (synthetic)

**Requester supplied**: during the invoice release in `payments`, the registry `registry.example.invalid` shows degradation. Two Pod create failures: one with a policy message about unverified image; one with a webhook call error. Metrics show request_allowed=false rising.

**Agent derived**: the first is a denial: the policy message being present shows the validation evaluated false [KYV-E04]; which verification count produced that is unverified until the policy expression is read [KYV-E18]. The rising request_allowed=false series is consistent with policy denials [KYV-E12]. The second matches a webhook call error; whether the registry caused it, and whether the counter records it, is unverified and needs an admission-trace.

**Still needed**: verbatim second error, webhook entry used, controller health at that minute, registry availability during the window.

**Decision today**: `indeterminate`, verdict `needs-review`; the denial is explained, the call error is not.

**Cheapest discriminating check**: an admission-trace for the second failure showing error class and webhook latency against timeoutSeconds.

**Developer-facing denial message**: for the first, "Image registry.example.invalid/payments/invoice is not verified; rebuild and sign it." For the second: not applicable, there is no policy message.

**Escalation owner**: SRE or on-call engineer accountable for API write availability.

**Keep out of evidence**: registry credentials, tokens, customer identifiers.

## Counterexample

The PolicyReport for the `payments` namespace shows all pass during the incident, so the failure is dismissed as upstream. Reports describe only resources that exist [KYV-E09]; the failed creates never produced objects, so the report cannot contradict a denial.

## Escalation threshold

Hand to the on-call owner when writes are failing now. Whether the denial was correct goes to `kyverno-policy-review-agent`; signer or registry trust design goes to `sigstore-cosign-supply-chain-review-agent`; live changes go to `kubernetes-live-admission-policy-guard-agent`.

## What would change the verdict

- A policy message appearing in the failing response, moving it to denial.
- An admission-trace tying the registry dependency error to a specific response.
- Report capture times or events showing the report was current.
- An API response naming a non-Kyverno validation source.
