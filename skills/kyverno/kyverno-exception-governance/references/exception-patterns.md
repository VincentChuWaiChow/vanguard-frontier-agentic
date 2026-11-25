# Which exception shapes keep a deviation bounded, and how do you test them before approval?

## Question

Which PolicyException fields bound a deviation to one workload and one policy, which shapes silently widen it, and what can an offline test show before approval?

## Release scope

- `policies.kyverno.io/v1` PolicyException at v1.20.0, which works with all CEL policy types [KYV-E16]. Legacy kyverno.io PolicyException is removed at v1.20.0 [KYV-E01]; before v1.20 it is a different object and is not covered here [KYV-E28].
- Offline observations were made with the pinned kyverno CLI v1.20.0 in this repository's starter kits; they are not controller behavior.

## Assumptions

- The approved scope is written by the approver as one workload, one namespace, one policy and one rule.
- The requester supplies the exception as authored; installed state is a separate request.
- Governance annotations (approval record, expiry owner) are the requester's records, not controller inputs.

## Evidence to request

- `manifest`: the PolicyException `spec.policyRefs`, `matchConditions`, `images`, `allowedValues`, `validations`, `reportResult` and `expiresAt` [KYV-E13].
- `manifest`: the policy named in `policyRefs`, to see whether its CEL reads `exceptions.allowedImages` or `exceptions.allowedValues`.
- `installed-config`: `enablePolicyException` and `exceptionNamespace` [KYV-E15].
- `offline-test`: the exception test with the approved workload (with and without the compensating control) and an adjacent workload.
- `rendered-gitops`: evidence that removal at expiry is reconciled, if the owner relies on it.

## Interpretation

1. Match conditions must name the exact workload. The kit's condition checks namespace and name together; a namespace-only condition exempts every workload in that namespace, and a condition that is always `true` exempts everything the policy matches. Treat both as scope widening, not as drafting style.
2. An exception matches a policy only by `policyRefs` name and kind [KYV-E14]. Both must be exact and `policyRefs` is required [KYV-E13]. A ref to the wrong kind or an old policy name matches nothing, so the deviation silently never applies.
3. `expiresAt` (RFC3339) is the removal mechanism the controller honors: once expired the exception is no longer applied [KYV-E13], and the exception lookup skips expired exceptions [KYV-E14]. A date in an annotation or description is inert data. The expired object still exists until something deletes it; applying is not the same as existing.
4. Observed with the pinned kyverno CLI v1.20.0 in this repository's starter kits and fixtures: an expired exception is still applied offline (`skip`), so offline tests cannot establish expiry [KYV-E14].
5. Compensating validations are all-or-nothing: they must all pass for the exception to be granted, otherwise the policy is evaluated as if the exception did not exist [KYV-E16]. Observed in the kit: the approved workload with the compensating label reports `skip`; the same workload without it reports `fail`; an adjacent workload with no exception reports `fail`.
6. `images` and `allowedValues` are exception-supplied lists [KYV-E13] that a policy reads in CEL as `exceptions.allowedImages` and `exceptions.allowedValues` [KYV-E50]. Inference: a policy that never reads them gives those fields no effect, so granting an image list through a policy that ignores it approves nothing and misleads reviewers. Confirm by reading the policy expressions.
7. Exceptions are honored only when `enablePolicyException` is on (default false) and the object sits in an allowed `exceptionNamespace` (default empty; `*` allows all) [KYV-E15]. A well-formed exception in the wrong namespace has no effect.
8. A skipped object reports `skip` by default; `reportResult` may be set to `pass` [KYV-E13]. A `pass` made by an exception is not evidence the control held.
9. Compensating validations also stop a ValidatingPolicy being translated into a native ValidatingAdmissionPolicy [KYV-E26], so adding one can change how the policy is enforced.
10. Governance annotations record approval and ownership for humans; the controller does not enforce them. They support the record, never the expiry.

## Worked example (synthetic)

**Requester supplied**: the policy-exception-starter kit for the `payments` invoice service: ValidatingPolicy `payments-run-as-non-root`; PolicyException `legacy-exporter-root` with `policyRefs` (name and kind ValidatingPolicy), an exact name-and-namespace match condition for `legacy-invoice-exporter`, a compensating validation requiring `network-isolation=strict`, and `expiresAt` set.

**Agent derived**: Observed with the pinned kyverno CLI v1.20.0 in this repository's starter kits, three cases: `legacy-invoice-exporter` with the label is `skip`; `invoice-batch-root` in the same namespace, no exception, is `fail`; `legacy-invoice-exporter` without the label is `fail`. The exception is bounded to one workload and fails closed when the control is missing [KYV-E16].

**Still needed**: `installed-config` for the enablement flag and exception namespace [KYV-E15]; reconciliation evidence for deleting the object after expiry; confirmation the authorized creator is the named role.

**Decision today**: `conditional` / `needs-review`: scope and compensating control are supported offline; expiry and enablement are not shown offline.

**Cheapest discriminating check**: re-run the kit with the match condition relaxed to namespace-only and confirm `invoice-batch-root` flips from `fail` to `skip`; that flip is the widening to forbid.

**Developer-facing denial message**: when the compensating control is missing the policy message applies: "payments-run-as-non-root: set securityContext.runAsNonRoot=true on every container. Escalate to the payments platform owner if the image cannot run as non-root." The exception's own message tells the owner to set the `network-isolation=strict` label.

**Escalation owner**: payments platform owner (risk delegate approves; the agent never approves).

**Keep out of evidence**: ticket text with customer names, real digests, approver identities beyond role.

## Counterexample

A request replaces the exact-workload condition with `true`, or with `object.metadata.namespace == 'payments'`, "to make renewals easier". The test for the approved workload still reports `skip`, so a single-case test looks fine. The adjacent `invoice-batch-root` now also reports `skip`: the deviation silently covers every workload the policy matches in that scope. Always test an adjacent workload.

## Escalation threshold

Hand to `kyverno-policy-review-agent` when the better fix is changing the policy so no exception is needed. Hand to `kubernetes-live-admission-policy-guard-agent` for any live create or delete. Stop and name the risk delegate when approval is missing.

## What would change the verdict

- `installed-config` showing exceptions disabled or the namespace not allowed [KYV-E15].
- A policy that reads `exceptions.allowedImages` or `exceptions.allowedValues`, making those fields effective [KYV-E50].
- `rendered-gitops` or `admission-trace` showing the exception no longer applies after `expiresAt` [KYV-E14].
- An adjacent-workload test showing `fail` under the exact condition.
