# Which exception mechanism applies to this policy family and release, and what does each field do?

## Question

For a given policy family and Kyverno release, which PolicyException kind is honored, which fields bound it, and what do offline results leave unproven?

## Release scope

- Kyverno v1.20.0 deletes the legacy kyverno.io policy engine, so the legacy kyverno.io PolicyException has no engine behind it on that release [KYV-E01]. The migration guide still lists PolicyException as a legacy kind and its wording lags the removal; trust the release notes [KYV-E02] [KYV-E01].
- On releases before v1.20 the legacy engine still executed legacy policies [KYV-E01]; legacy exception semantics are not encoded here and stay unverified for those releases.
- The `policies.kyverno.io/v1` PolicyException works with all CEL policy types [KYV-E16].
- Pinned CLI: v1.20.0 offline fixtures in this repository.

## Assumptions

- The reviewed object is `policies.kyverno.io/v1` PolicyException, not a legacy kind.
- The controller flags are not supplied. Assume the enable flag and `exceptionNamespace` govern policies.kyverno.io exceptions; confirm with installed-config before relying on that.
- The deployed controller is v1.20.0 or the stated release; no other release is assumed.

## Evidence to request

- `manifest`: the PolicyException `spec.policyRefs`, `spec.matchConditions`, `spec.validations`, `spec.reportResult`, `spec.expiresAt`.
- `installed-config`: the controller arguments showing the exception enable flag and `exceptionNamespace`.
- `rendered-gitops`: the rendered exception and its owner, to see who creates and removes it.
- `offline-test`: `kyverno test` output with the exception file and the CLI version.

## Interpretation

1. The v1 PolicyException spec requires `policyRefs`; optional fields are `matchConditions`, `images`, `allowedValues`, `validations` (compensating controls), `reportResult` (skip or pass, default skip) and `expiresAt` in RFC3339 [KYV-E13].
2. `expiresAt`, once passed, means the exception is no longer applied to incoming requests [KYV-E13].
3. The controller's exception lookup skips expired exceptions and matches an exception to a policy only by `policyRefs` name and kind; match conditions then narrow it [KYV-E14]. A `policyRefs` entry naming the wrong kind matches nothing.
4. Exceptions are disabled by default: the enable flag defaults to false and `exceptionNamespace` defaults to empty [KYV-E15]. The `exceptionNamespace` flag controls where exceptions are allowed, and `*` allows all namespaces [KYV-E15]. Whether an exception outside the allowed namespace is inert is unverified. Assume the flag governs policies.kyverno.io exceptions; confirm with installed-config.
5. Compensating validations must all pass for the exception to be granted; otherwise the policy is evaluated as if the exception did not exist [KYV-E16].
6. An exception with compensating validations prevents translating its ValidatingPolicy into a native ValidatingAdmissionPolicy [KYV-E26]; record the effect on any native-policy plan.
7. `reportResult` controls what a covered resource reports (skip by default, or pass) [KYV-E13]; a `pass` hides the deviation from reports, so ask why.
8. Observed with the pinned kyverno CLI v1.20.0 in this repository's offline fixtures: an exception with `expiresAt` in the past is still applied (`skip`) (fixture 03). The controller's lookup skips expired exceptions [KYV-E14], so an offline result cannot establish expiry behavior.
9. Whether the expired PolicyException object is deleted is not stated by any source here: unverified. Assume the object remains until something removes it.

## Worked example (synthetic)

**Requester supplied**: a PolicyException `ledger-sync-root-expired` in `payments` with `policyRefs` naming `payments-run-as-non-root` (ValidatingPolicy), a match condition on object name `legacy-ledger-sync`, and `expiresAt` in the past; offline test result `skip`.

**Agent derived**: the field set is valid for v1 [KYV-E13]; the offline `skip` is an artifact of the CLI applying an expired exception (fixture 03), so it says nothing about the controller [KYV-E14].

**Still needed**: controller flags (enable and `exceptionNamespace`) and whether the object has been removed.

**Decision today**: `conditional`, verdict `needs-review`, the same outcome as in the disposition reference. Reason: the controller skips expired exceptions [KYV-E14], so the exception is judged not applied, but that is judgment because the controller flags and the live post-expiry decision are unseen. Conditions: the flags and an `admission-trace` after expiry.

**Cheapest discriminating check**: `installed-config` of the controller arguments plus an `admission-trace` of a `legacy-ledger-sync` create after expiry.

**Developer-facing denial message**: `payments-run-as-non-root: set securityContext.runAsNonRoot=true on every container. Escalate to the payments platform owner if the image cannot run as non-root.` It tells the developer to fix the securityContext or escalate; it does not mention the exception.

**Escalation owner**: payments platform owner for the workload; risk delegate for any renewal.

**Keep out of evidence**: image digests tied to customers, change-record contents beyond its ID, user identities.

## Counterexample

An offline test returns `skip` for a workload under an exception whose `expiresAt` is long past, and a reviewer concludes the exception still protects the workload. The controller skips the expired exception [KYV-E14], so the real admission result is the policy's own result, judged to be a deny if the policy's actions include Deny (judgment; the controller flags and actions were not supplied); the offline `skip` is misleading (fixture 03).

## Escalation threshold

Stop and hand to `kubernetes-live-admission-policy-guard-agent` when anything is to be created, renewed or deleted in a live cluster. Hand to `kyverno-policy-review-agent` when the fix is to change the policy.

## What would change the verdict

- Controller arguments showing exceptions disabled or restricted to specific namespaces.
- An admission trace showing the post-expiry decision for the covered workload.
- Evidence the exception object was removed, or evidence it remains.
- A release other than v1.20.0 with different exception behavior.
