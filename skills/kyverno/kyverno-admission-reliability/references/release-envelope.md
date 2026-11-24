# Can the admission path absorb this change within its latency and error budget, and what aborts the release?

## Question

Given a proposed policy or configuration change, can the admission path carry it within a latency and error budget, which writes are affected if it degrades, what signal aborts the release, and what is the recovery path?

## Release scope

- Applies to policies.kyverno.io at v1.20.0 [KYV-E01]. ImageValidatingPolicy is Stable in v1.20.0 and needs registry credentials listed in its own credentials field [KYV-E19].
- For legacy ClusterPolicy on releases before v1.20, the same timeout and failurePolicy defaults apply but the engine is removed at v1.20.0 [KYV-E28] [KYV-E01].
- The webhook timeout default is 10 seconds within 1 to 30 for ValidatingPolicy [KYV-E06] and for ImageValidatingPolicy webhookConfiguration.timeoutSeconds [KYV-E36]; Kubernetes webhooks share the range [KYV-E08].
- ValidatingPolicy failurePolicy defaults to Fail [KYV-E04]. The ImageValidatingPolicy CRD states no failurePolicy default [KYV-E36], so the installed value must be read from installed-config, not assumed.

## Assumptions

- The budget (latency and error) is supplied by the service owner; this reference does not invent one.
- The baseline is unknown until measured from the supplied series.
- Degradation window length is a stated planning input, not an observation.
- Request classes are grouped by resource kind and operation, the labels the metrics carry [KYV-E12].
- A rollback that needs the same degraded dependency is not a recovery path.

## Evidence to request

- `installed-config`: admission controller replica count and placement; a highly available controller needs at least three replicas [KYV-E11].
- `runtime-metric`: kyverno_admission_review_duration_seconds histogram and kyverno_admission_requests_total counter with resource_kind, resource_request_operation and request_allowed labels [KYV-E12].
- `manifest` or `rendered-gitops`: the proposed change, including any new image policy and its `timeoutSeconds`.
- `installed-config`: the dependencies image policies call (registry names) and their stated availability objective.
- `admission-trace`: any prior timeout or error samples for the affected request classes.

## Interpretation

1. Confirm HA minimum: three admission replicas, with leader election meaning one replica manages certificates and webhooks at a time [KYV-E11]. Fewer replicas means a single failure removes the path.
2. Compute p95 and p99 per request class (resource kind and operation) from the histogram [KYV-E12]. A cluster-wide average hides the slow class.
3. Estimate affected writes as matching write rate times the degradation window, with the rate taken from the request counter [KYV-E12]. Never guess the rate; if the series is missing, the estimate is unknown.
4. Image verification contacts the registry during admission — inference; confirm with an admission-trace. The ImageValidatingPolicy needs registry credentials in its own credentials field [KYV-E19]. The added latency is unknown until measured; a timeout chosen above the measured p99 plus registry variance is a judgment, not a fact.
5. Under Fail, an exhausted timeout rejects matching writes; under Ignore it admits them unevaluated [KYV-E04] [KYV-E06]. For an ImageValidatingPolicy the failurePolicy (Ignore or Fail) and timeout are its own fields [KYV-E36] and the failurePolicy has no CRD default, so read the installed value. State both consequences for the affected operations.
6. Define abort signals from measured series (for example sustained p99 above the supplied budget, a rising request_allowed=false rate [KYV-E12], or a caller-supplied webhook error series, which has no KYV citation) with the owner who decides to abort. Thresholds are caller-supplied.
7. Define the recovery path: the rollback action, the dependencies it needs (registry, GitOps controller, API access), and whether recovery workloads are themselves matched by a Fail webhook.

## Worked example (synthetic)

**Requester supplied**: the invoice release in namespace `payments` adds an ImageValidatingPolicy with `timeoutSeconds: 15`, images from `registry.example.invalid`. A 7 day metric export for Deployment and Pod CREATE. Controller runs two replicas. Budget supplied by the service owner.

**Agent derived**: two replicas is below the HA minimum of three [KYV-E11]. Timeout 15 is above the 10 second default and within 1 to 30 [KYV-E36]; the policy's failurePolicy is not stated and the CRD gives no default [KYV-E36], so the installed value is read from installed-config. A longer timeout also lengthens how long a degraded registry can hold Pod writes. p95 and p99 for Pod CREATE come from the histogram [KYV-E12]; the added registry latency is unknown. Affected writes if the registry degrades for the stated window = Pod CREATE rate times that window.

**Still needed**: measured baseline for Pod CREATE, the registry availability objective, the installed failurePolicy and the webhook entry (`-fail` or `-ignore`) the policy lands on [KYV-E36] [KYV-E07], and recovery-workload selectors.

**Decision today**: `conditional`, verdict `needs-review`; conditions are a third replica, a measured latency baseline and a named registry fallback owner.

**Cheapest discriminating check**: a controlled-test in a disposable cluster measuring Pod CREATE latency with the image policy under registry slowness, run by an authorized operator.

**Developer-facing denial message**: not applicable; this is an availability envelope, not a rule. Developers see only the policy's own denial text when an image fails verification.

**Escalation owner**: SRE or on-call engineer accountable for API write availability.

**Keep out of evidence**: registry credentials, pull secrets, internal hostnames beyond the fictional registry, customer data.

## Counterexample

Average latency is well inside budget, so the release looks safe. The p99 for Pod CREATE, the only class the new image policy touches, is the class that breaches the timeout. A cluster-wide mean hides it, which is why the envelope is per request class.

## Escalation threshold

Hand to the on-call owner when the budget is breached during rollout. Any webhook or policy change in a live cluster goes to `kubernetes-live-admission-policy-guard-agent`; registry trust design goes to `sigstore-cosign-supply-chain-review-agent`.

## What would change the verdict

- A measured baseline and p99 for each affected request class.
- Three or more admission replicas with placement across failure domains.
- A named, tested recovery path that does not depend on the failing registry.
- Evidence that the policy lands on an Ignore entry, which changes the risk from rejected writes to unevaluated writes.
