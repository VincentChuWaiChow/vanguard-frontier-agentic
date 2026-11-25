# Which engine, policy family, binding, scope and action implement this business requirement on the installed release?

## Question

Given a business requirement, which policy family on which API group implements it, what binds it to requests, and does the configured action (Deny, Audit, Warn) match the control's intent and rollout stage?

## Release scope

Release-sensitive facts for this review:

| API group | Kind | Release | Status |
| --- | --- | --- | --- |
| kyverno.io | ClusterPolicy, Policy | Supported through v1.19, with deprecation warnings in v1.19 | Present and evaluated before v1.20.0 [KYV-E01] [KYV-E02] |
| kyverno.io | ClusterPolicy, Policy | v1.20.0 | Legacy engine deleted, creates and spec updates blocked, CLI rejects legacy manifests in apply and test [KYV-E01] |
| kyverno.io | Migration guide wording | Any | Still says "deprecated and will be removed in a future release"; this lags the removal. Trust the release notes (KYV-E01) over the guide (KYV-E02). |
| policies.kyverno.io | ValidatingPolicy, ImageValidatingPolicy and the other CEL kinds | v1.20.0 | v1, v1beta1 and v1alpha1 served; v1beta1 is the storage version; v1alpha1 is deprecated [KYV-E03] |
| admissionregistration.k8s.io | ValidatingAdmissionPolicy | Kubernetes 1.30 and later | Stable; needs a ValidatingAdmissionPolicyBinding to take effect [KYV-E27] |

For legacy ClusterPolicy documented at v1.18, `spec.failurePolicy` defaults to Fail and `validationFailureAction` is deprecated in favour of per-rule `failureAction` [KYV-E28]. On v1.20.0 and later none of this is installable, so a review that finds legacy kinds must first establish the installed release.

## Assumptions

- The requester states the Kyverno controller release and the Kubernetes version; if either is unknown, the conclusion is limited to the documented release.
- The business requirement is expressible as one allowed and one prohibited operation.
- Rollout stage claims (canary, owner, deadline) are assertions until a ticket or schedule is supplied as data.

## Evidence to request

- `manifest`: the policy with apiVersion, kind, `spec.validationActions` (or the legacy action field), `matchConstraints`, `matchConditions`.
- `installed-config`: the served versions of the policies.kyverno.io CRDs and the installed controller image tag.
- `manifest`: for a native policy, the ValidatingAdmissionPolicyBinding and its `validationActions`.
- `manifest`: the rollout record stating intent (preventive, detective, advisory), owner, deadline and promotion criteria.
- `offline-test`: `kyverno test` output with the CLI version.

## Interpretation

1. Establish the family and release before reading any field. On v1.20.0 or later a legacy kyverno.io kind is not evaluated: the legacy engine is deleted and creates and spec updates of legacy policy APIs are blocked [KYV-E01]. A legacy manifest in Git is then intent with no enforcement point.
2. Read `validationActions` as a set drawn from Deny, Audit and Warn; `failurePolicy` is Ignore or Fail, defaults to Fail, and governs evaluation errors, not validations that evaluate to false [KYV-E04]. A policy in Audit that fails open on error and a policy in Deny that fails closed on error are different questions.
3. For native ValidatingAdmissionPolicy, a binding is required and Deny and Warn cannot be combined in one binding [KYV-E27]. A policy without a binding does nothing.
4. Classify intent: mandatory preventive (the requirement is that bad writes never land), detective (the requirement is to find and report), or advisory.
5. Apply the action-intent table below. Severity comes from intent, stage, owner and deadline, never from the action value alone.
6. Select the engine (below), then state the denial message the developer would see.

### Action-intent truth table

| Configured action | Intended control | Rollout stage | Observed effect | Assessment and severity | Reasoning |
| --- | --- | --- | --- | --- | --- |
| Audit | Mandatory preventive | Approved canary with owner, deadline and promotion criteria, before the deadline | Violations are recorded, not blocked | Low severity, conditional: requires a stated time bound, a named owner and promotion criteria | A bounded, owned canary is a deliberate step toward prevention; the risk is the missing promotion, not the field value. |
| Audit | Mandatory preventive | Same canary, deadline passed | Violations still admitted | High, blocked when the requirement is prevention | The approved window ended; the control is not doing the job it was approved to do, and nobody owns the gap. |
| Audit | Detective | Steady state | Violations reported | Expected, no finding | Audit matches the intent. |
| Warn | Mandatory preventive | Approved canary (inside or past its bound) | Violations admitted with a warning to the caller | Same reasoning as Audit: low if inside the approved bound, high once past it | A warning informs the requester; it does not stop the write. |
| Deny | Mandatory preventive | Steady state, with no legitimate-operation test supplied | Violations blocked, but legitimate writes unproven | Conditional | Prevention without a passing legitimate case can block valid work; supply the pass case. |
| Deny | Mandatory preventive | Steady state, legitimate and violating cases both tested | Violations blocked, legitimate admitted | Supported for the tested scope | Both sides of the decision are evidenced. |

The two Audit rows get different severities because intent and deadline differ, not because of the field value: the same `validationActions: [Audit]` is acceptable inside an approved bound and a defect after it.

### Engine selection

- Use native ValidatingAdmissionPolicy when the rule is a pure admission check on the request and a binding can scope it [KYV-E27].
- Use a Kyverno CEL policy when the rule needs image verification, generation, mutate-existing or PolicyReports; these are Kyverno capabilities [KYV-E18] [KYV-E20] [KYV-E22], and a native ValidatingAdmissionPolicy is a validation-only mechanism (inference from its definition; confirm against the Kubernetes release in use).
- Kyverno generates a native ValidatingAdmissionPolicy from a ValidatingPolicy only when all of these hold: the controller toggle is on (default true at v1.20.0) [KYV-E26]; the policy sets `autogen.validatingAdmissionPolicy.enabled: true` (default false) [KYV-E37]; pod-controller autogen produces no rules, which for a Pod policy means `autogen.podControllers` is disabled or empty (it is enabled by default) [KYV-E37] [KYV-E40]; and no exception defines compensating validations [KYV-E26]. A default Pod policy therefore stays on the Kyverno webhook path; confirm with `installed-config` showing the generated ValidatingAdmissionPolicy and its binding.

## Worked example (synthetic)

**Requester supplied**: Two policies for namespace `payments`. `payments-run-as-non-root-canary` is `validationActions: [Audit]`, approved for payments cohort A until a stated date, owner "payments platform lead", with a promotion criterion of zero new violations for a stated period. A separate mandatory control `payments-run-as-non-root` is also still `[Audit]`; its approved deadline passed 40 days ago.

**Agent derived**: The cohort A canary is a bounded, owned step: low severity, conditional on the time bound, owner and criteria being recorded. The mandatory control is past its deadline in Audit while the requirement is prevention: high, blocked for the prevention requirement. Same field value, different verdicts. Observed with the pinned kyverno CLI v1.20.0 in this repository's offline fixtures, Audit and Deny give the same result (`fail`) for the same violation, so no offline test separates them.

**Still needed**: `installed-config` showing the action actually installed for the second policy, and a ticket or schedule showing the deadline and whether an extension was approved.

**Decision today**: Canary: `conditional` / `needs-review`. Mandatory control: `blocked` / `blocked` for prevention; the fix is promotion to `[Deny]` after a passing legitimate-operation test.

**Cheapest discriminating check**: `offline-test` of invoice-retry-worker (must pass) and invoice-batch-root (must fail) against the Deny form, then compare `installed-config` action to the manifest.

**Developer-facing denial message**: "payments-run-as-non-root: set securityContext.runAsNonRoot=true on every container. Escalate to the payments platform owner if the image cannot run as non-root." It names the field to fix and the owner to ask.

**Escalation owner**: payments platform lead, for the deadline and promotion decision.

**Keep out of evidence**: customer identifiers, registry credentials, real image digests tied to customers.

## Counterexample

A policy file shows `validationActions: [Deny]` and the reviewer reports prevention. The installed copy was edited to `[Audit]` during an incident and the manifest was never reverted. The manifest proves intent only; the `installed-config` item is what establishes the effective action. Also, `failurePolicy: Fail` does not make a violation blocking if the action is Audit, because it governs errors, not violations [KYV-E04].

## Escalation threshold

Stop and hand to the payments platform lead when an Audit policy is past its deadline for a mandatory control. Send webhook timeout or outage questions to `kyverno-admission-reliability-agent` and exception questions to `kyverno-exception-governance-agent`. A change to the live policy goes to `kubernetes-live-admission-policy-guard-agent`.

## What would change the verdict

- `installed-config` showing a different action or release than the manifest.
- A signed extension of the canary deadline, with a named owner.
- A passing legitimate-operation test for the Deny form.
- A controller release before v1.20.0 for legacy kinds, which changes whether they are evaluated at all [KYV-E01].
