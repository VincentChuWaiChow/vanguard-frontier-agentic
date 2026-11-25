# Which writes never reach Kyverno, and what does a default install leave single-instance?

## Question

For a given write, did Kyverno never see it (filtered before evaluation), see it and error, or see it and deny it, and what does a default installation leave without redundancy when the admission path is disturbed?

## Release scope

- Applies to the Helm chart defaults at the v1.20.0 tag and policies.kyverno.io at v1.20.0, where the legacy engine is removed [KYV-E01]. These are chart defaults: an installed environment may differ, so ask for installed-config before applying any of them.
- Legacy ClusterPolicy on releases before v1.20 shares the failurePolicy default of Fail [KYV-E28]; the chart filtering defaults below are a v1.20.0 observation and are not claimed for earlier charts.
- The migration guide's wording lags the removal; trust the release notes [KYV-E02] [KYV-E01].

## Assumptions

- The requester can supply the `kyverno` ConfigMap and the webhook configurations as installed, not as templated.
- "Never saw the write" means the request was filtered before policy evaluation; it says nothing about whether the write was acceptable.
- Replica counts and PodDisruptionBudget values come from the installed Deployments, not from this reference.
- The consequence of failurePolicy Fail during a node drain is inference and is labelled so below.

## Evidence to request

- `installed-config`: the `kyverno` ConfigMap, specifically the resource filters, the excluded-namespaces setting and any webhook match conditions.
- `installed-config`: the MutatingWebhookConfiguration and ValidatingWebhookConfiguration objects, including `namespaceSelector`, `matchConditions` and the `-ignore` and `-fail` entries [KYV-E07].
- `installed-config`: the four controller Deployments with replica counts, and the admission controller PodDisruptionBudget if any.
- `manifest` or `rendered-gitops`: the Helm values that produced the install, to separate a chart default from a deliberate override.
- `admission-trace`: for the write in question, whether the API server logged a webhook call at all.
- `runtime-metric`: kyverno_admission_requests_total by resource_kind and resource_request_operation, to see whether the kind appears [KYV-E12].

## Interpretation

1. Chart resource filters skip the kinds Event, Node, ReplicaSet, APIService, TokenReview and SubjectAccessReview, and every object in kube-system, kube-public and kube-node-lease [KYV-E41]. A policy matching those kinds or namespaces is accepted by the cluster and then never runs.
2. The webhook `namespaceSelector` excludes kube-system by default, the Kyverno namespace is excluded by default (`excludeKyvernoNamespace` is true), and the webhook match conditions are empty by default [KYV-E41]. Empty match conditions means no extra pre-filter was installed; any conditions found in installed-config were added by someone.
3. Classify every observation into one of three buckets before reasoning:

| Bucket | What it means | What shows it |
| --- | --- | --- |
| Filtered | Kyverno never saw the write | The kind or namespace is in the resource filters or the webhook selector; the kind is absent from the request counter [KYV-E12] |
| Saw and errored | The webhook was called and evaluation or the call failed | An API error naming the webhook; under Fail the write is rejected, under Ignore it is admitted unevaluated [KYV-E04] [KYV-E06] |
| Denied | The webhook was reached and the validation evaluated to false | The policy's own message; unaffected by failurePolicy [KYV-E04] |

4. Absence of denials in a filtered namespace is not a result. Zero denials in kube-system is what a filter produces whether or not the objects there would pass [KYV-E41].
5. A default installation provides four separate Deployments, each with one replica, and high availability is achieved per controller [KYV-E43]. The chart leaves `admissionController.replicas` unset and the admission PodDisruptionBudget disabled (minAvailable 1 when enabled) [KYV-E42], while the HA guide states a minimum of three admission replicas [KYV-E11].
6. Inference, not a cited fact: with one admission replica, no PodDisruptionBudget and `failurePolicy: Fail`, a node drain that evicts the only admission pod leaves matching writes without a webhook backend until the pod is rescheduled, and the API server then rejects them as in the Fail row of webhook failure semantics [KYV-E04] [KYV-E08]. Confirm with an `admission-trace` and the installed replica count; do not state it as observed.
7. A policy-level `failurePolicy` selects the `-ignore` or `-fail` webhook entry [KYV-E07], so read the effect per entry. A cluster with three admission replicas and a PodDisruptionBudget can still show the Fail behavior during a full outage; redundancy changes how likely it is, not what it means.
8. Inference, not a cited fact: reports describe results Kyverno produced, so a filtered object has no pass or fail result to report [KYV-E10]; confirm with a `population-report` before relying on it.

## Worked example (synthetic)

**Requester supplied**: the invoice service Pods in `payments` are the intended scope. A second policy is written against ReplicaSets in `payments` and the team reports it "never fires". The installed `kyverno` ConfigMap and webhook configurations are supplied, plus the Deployment replica counts.

**Agent derived**: the invoice service Pods in `payments` are not in a filtered namespace, so Pod CREATE reaches the webhook [KYV-E41]. The ReplicaSet policy never runs because ReplicaSet is a filtered kind in the chart defaults [KYV-E41]; this is filtering, not a policy bug and not an error. The install shows one admission replica and no PodDisruptionBudget, matching the chart defaults [KYV-E42] [KYV-E43] and below the HA minimum [KYV-E11].

**Still needed**: confirmation the ConfigMap in the supplied installed-config is the live one, and the failurePolicy of the Pod policies.

**Decision today**: `conditional`, verdict `needs-review`; the ReplicaSet policy is moved to the Pod or Deployment kinds it was meant to protect, and the single replica is raised before any Fail-policy release.

**Cheapest discriminating check**: check whether kyverno_admission_requests_total has any series with resource_kind ReplicaSet [KYV-E12]; none confirms the filter.

**Developer-facing denial message**: not applicable; a filtered write produces no message. For the Pod policies the developer sees the policy's own denial text.

**Escalation owner**: platform engineer owning the Kyverno installation.

**Keep out of evidence**: ConfigMap values beyond the filter and selector lists, cluster endpoints, webhook CA bundles, customer data.

## Counterexample

A dashboard shows no denials in kube-system for a month and the report is read as compliance. kube-system is skipped by the chart's resource filters and the webhook selector [KYV-E41], so Kyverno evaluated none of it. Zero denials there measures the filter, not the workloads.

## Escalation threshold

Hand to the accountable platform owner when installed-config differs from the chart defaults in ways the requester cannot explain. Hand to `kyverno-policy-review-agent` when the question is whether a policy's match is correct, and to `kubernetes-live-admission-policy-guard-agent` for any live change to filters, replicas or webhook configuration; this skill never dispatches it.

## What would change the verdict

- Installed-config showing resource filters or selectors that differ from the chart defaults.
- An `admission-trace` showing a webhook call for a kind this reference treats as filtered.
- Installed replica count of three or more with a PodDisruptionBudget, which narrows the single-instance concern but not the Fail semantics.
- A `controlled-test` of a node drain in a disposable cluster showing the actual write behavior.
