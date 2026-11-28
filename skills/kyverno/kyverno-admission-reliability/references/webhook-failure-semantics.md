# Which writes are rejected, delayed or admitted unevaluated when the Kyverno webhook or a dependency fails?

## Question

When the Kyverno admission controller is unreachable, slow, erroring or blocked on a dependency, which API writes fail, which wait, and which pass without evaluation, and what does on-call see in each case?

## Release scope

- Applies to the policies.kyverno.io API group at v1.20.0, where the legacy kyverno.io policy engine is removed [KYV-E01]. Legacy ClusterPolicy is not evaluated on v1.20.0 or later; on earlier releases its failurePolicy defaults to Fail and its timeout field (webhookTimeoutSeconds) defaults to 10 seconds within 1 to 30 [KYV-E28].
- The migration guide's deprecation wording lags the removal; trust the release notes [KYV-E02] [KYV-E01].
- Kubernetes webhook defaults for the v1 admission API are failurePolicy Fail and a 10 second timeout within 1 to 30 seconds [KYV-E08].

## Assumptions

- The failure under review is a call or evaluation error, not a rule that evaluates to false.
- The requester can supply the installed webhook configurations; none is inferred from memory.
- Which namespaces or objects a webhook excludes is unknown until the selectors are supplied.

## Evidence to request

- `installed-config`: ValidatingWebhookConfiguration entries, including `failurePolicy`, `timeoutSeconds`, `namespaceSelector`, `objectSelector` and `rules`.
- `installed-config`: the ValidatingPolicy and ImageValidatingPolicy objects with `spec.failurePolicy` and `spec.webhookConfiguration.timeoutSeconds` [KYV-E36]. The ImageValidatingPolicy CRD states no failurePolicy default, so its installed value must be read [KYV-E36].
- `runtime-metric`: admission request and latency series for the window [KYV-E12].
- `admission-trace`: the exact API response text and timestamp for a failed write.
- `installed-config`: placement and replica evidence for recovery workloads, and the webhook selectors that decide whether they are matched.

## Interpretation

1. For ValidatingPolicy, failurePolicy is Ignore or Fail and defaults to Fail; ImageValidatingPolicy has the same two values but its CRD states no default, so read the installed value [KYV-E36]. It governs evaluation and call errors, not validations that evaluate to false [KYV-E04].
2. Kyverno registers separate `-ignore` and `-fail` validating webhook entries, so a policy's failurePolicy selects which entry carries its requests [KYV-E07]. Read effects per entry, not per cluster.
3. timeoutSeconds defaults to 10 and must be between 1 and 30; on expiry the request fails or ignores the policy result per the failure policy [KYV-E06]. ImageValidatingPolicy carries the same timeout field and range [KYV-E36].
4. A violation is a denial from a reachable webhook and is unaffected by failurePolicy [KYV-E04].
5. Recovery workloads (the admission controller pods, their dependencies) are themselves writes. If a Fail entry matches them and the controller is down, recreating them can be blocked. Whether they are excluded is shown by webhook selectors, which must be supplied; do not assume a default exclusion.
6. Image verification contacts the registry during admission — inference; confirm with an admission-trace. Image policies need registry secrets listed in their own credentials field [KYV-E19]. Exactly how a registry dependency error surfaces is unverified and needs an admission-trace.

| Failure | failurePolicy Fail | failurePolicy Ignore | What on-call sees |
| --- | --- | --- | --- |
| Controller unreachable | Matching writes rejected by the API server [KYV-E04] [KYV-E08] | Matching writes admitted unevaluated [KYV-E04] | Fail: API error naming the webhook, no policy message. Ignore: writes succeed; only webhook availability metrics and absent evaluations hint at it |
| Timeout | Request fails after timeoutSeconds [KYV-E06] | Request proceeds without the policy result [KYV-E06] | Fail: delayed then rejected write with a timeout error. Ignore: delayed then admitted |
| Evaluation error | Request fails [KYV-E04] | Request admitted unevaluated [KYV-E04] | Fail: error without a violation message. Ignore: write admitted; an `error` report result may appear later [KYV-E10] |
| Registry dependency error (image policies) | Unverified: needs admission-trace | Unverified: needs admission-trace | Unverified: needs admission-trace |

## Worked example (synthetic)

**Requester supplied**: Deployment create for the invoice service in namespace `payments` rejected twice. Attempt A response: "failed calling webhook ... context deadline exceeded", no policy text. Attempt B response: a message from policy `require-signed-images` stating the image is not verified. Both inside one 20 minute window.

**Agent derived**: A is a webhook call error (observed: no policy message, webhook named). B is a denial from a reachable webhook (observed: policy message). They share a window but not a cause. Impacted operations differ: A affects every write matched by the Fail entry, including unrelated deployments; B affects only images failing verification.

**Still needed**: webhook `timeoutSeconds` and `failurePolicy` for the entry in A, controller replica status at capture time, and whether A hit the `-fail` entry.

**Decision today**: `indeterminate`, verdict `needs-review`, for the cause of A. B is a policy denial and is routed to policy review if its correctness is questioned.

**Cheapest discriminating check**: an admission-trace for A showing the webhook name and error class, plus controller pod availability for the same minute.

**Developer-facing denial message**: for B, "Image registry.example.invalid/payments/invoice is not signed by a trusted identity; sign the image or use a signed build." For A: not applicable, because no policy message exists; the developer should retry only after the platform owner confirms webhook health.

**Escalation owner**: SRE or on-call engineer accountable for API write availability.

**Keep out of evidence**: service account tokens, kubeconfig content, registry credentials, customer identifiers.

## Counterexample

A team sees writes succeeding during a controller outage and concludes the policy is not enforced. With Ignore, those writes were admitted unevaluated [KYV-E04]; the policy was bypassed, not satisfied. Success is not evidence of compliance.

## Escalation threshold

Stop and hand to the on-call owner when matching writes are being rejected now; hand any live webhook or policy change to `kubernetes-live-admission-policy-guard-agent`. Questions about whether a denial was correct go to `kyverno-policy-review-agent`.

## What would change the verdict

- Webhook configurations showing selectors that exclude the recovery workloads.
- An admission-trace identifying the registry dependency error surface.
- A controller version other than v1.20.0 or a legacy ClusterPolicy still evaluated.
- Evidence that the failing request matched an Ignore entry rather than a Fail entry.
