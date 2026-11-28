# Which cohorts move, in what order, and what restores prior behavior?

## Question

Which workload cohorts can be promoted from the current policy behavior to the target, in what order, with what abort criterion, and what does reversing the move actually restore?

## Release scope

- Kyverno v1.20.0 removes the legacy engine and blocks legacy creates and spec updates [KYV-E01]; the migration guide's "future release" wording lags [KYV-E02] [KYV-E01].
- ValidatingPolicy `validationActions` is a set drawn from Deny, Audit and Warn, and `failurePolicy` is Ignore or Fail, defaulting to Fail [KYV-E04]. Audit and Deny are distinct admission effects even though the offline result is the same (fixture 02).
- The Istio canary pattern is cited as a transferable idea only: a canary upgrade runs the new revision beside the old one, moves namespaces by revision label or tag with workload restarts, and rolling back by uninstalling the canary does not revert gateways already upgraded in place [KYV-E32].
- Legacy kinds before v1.20 differ; legacy defaults are from v1.18 documentation only [KYV-E28].

## Assumptions

- Cohorts are defined by namespace or label and each has a named owner.
- The required control per cohort is stated before the move.
- No cohort moves unless its parity evidence is supplied and sanitized.

## Evidence to request

- `offline-test` and `controlled-test`: parity per cohort on CREATE, UPDATE and DELETE, with real old/new pairs for UPDATE-sensitive controls.
- `population-report`: Audit-canary results over the canary window for the cohort.
- `manifest`: exceptions per cohort with `expiresAt`.
- `installed-config`: the target policies' `validationActions` and `failurePolicy`.
- Owner and rollback statement per cohort, and the environment's upgrade documentation.

## Interpretation

1. What transfers from an Istio canary upgrade: move cohorts incrementally by label or namespace, keep the previous behavior available while the new one is observed, and name the rollback point per cohort [KYV-E32].
2. What does not transfer: this procedure does not run two Kyverno controller revisions side by side. The Kyverno analogue is a per-cohort Audit canary followed by Deny. Workloads changed in the meantime are not reverted by reverting the policy, just as rolling back an Istio canary does not revert gateways already upgraded in place [KYV-E32].
3. Kyverno-specific procedure, per cohort:
   - Re-express the control in the target family and confirm parity on CREATE, UPDATE and DELETE.
   - Deploy with `validationActions` containing Audit only [KYV-E04] and observe the canary window through `population-report` results and `runtime-metric` samples.
   - Check that legitimate business operations (for example invoice retries) pass.
   - Promote to Deny only after the abort criteria are absent, then keep the Audit evidence to compare.
   - On abort, return the cohort to Audit or remove the policy; neither undoes changes already made to workloads.
4. Background reports describe resources that exist; they do not show writes that were denied [KYV-E09]. A clean Audit window shows what exists, not what Deny would have blocked from future writes, so pair it with parity evidence. An Audit canary does not take the webhook out of the request path: failurePolicy and timeout risk remain while the policy is in Audit [KYV-E04] [KYV-E06], so the canary needs its own reliability check.
5. Exceptions per cohort must be listed and expiry verified before the cohort moves; see `kyverno-exception-governance`.
6. Decision matrix, one row per cohort:

| Cohort | Required controls | Parity (CREATE/UPDATE/DELETE) | Exceptions | Decision | Abort criterion |
| --- | --- | --- | --- | --- | --- |
| (cohort name) | (controls) | (per operation) | (list) | (outcome) | (measurable condition) |

7. Order: lowest business impact first, UPDATE-sensitive and revenue-critical cohorts last. Each cohort has its own owner and its own abort criterion.

## Worked example (synthetic)

**Requester supplied**: two `payments` cohorts moving the run-as-non-root control to the target. Cohort A, `invoice-retry-worker` pods: parity on CREATE and UPDATE from a `controlled-test` with real old/new pairs (the control does not match DELETE); the invoice retry passes; a seven-day Audit canary shows no unexplained `fail` results; owner and rollback named. Cohort B, `invoice-api` Deployments, control `invoice-tier-label-immutable` (UPDATE-sensitive): CREATE parity agrees; for UPDATE the source result is `fail` and the target offline result is `pass`, unexplained (fixture 04 pattern). Cohort A was also independently challenged by `kyverno-policy-review-agent`, which named `invoice-retry-worker` as the legitimate workload that must pass; result: upheld.

| Cohort | Required controls | Parity (CREATE/UPDATE/DELETE) | Exceptions | Decision | Abort criterion |
| --- | --- | --- | --- | --- | --- |
| A: invoice-retry-worker pods | run-as-non-root | CREATE and UPDATE evidenced with real pairs; DELETE not matched | none | `supported`: promote to Deny | any `fail` on a legitimate retry pod, or a result not explained by the corpus |
| B: invoice-api Deployments | invoice-tier-label-immutable | CREATE agrees; UPDATE source `fail`, target `pass`, unexplained and an identical-object artifact [KYV-E24] | none | `blocked` | real UPDATE pair shows a difference from the source |

**Agent derived**: A satisfies every promotion requirement in the supplied evidence, so promotion to Deny is justified for A. B is blocked on the unexplained UPDATE allow/deny difference, which on a required control is the demonstrated parity defect; the target `pass` is likely the identical-object artifact, but that explanation needs real old/new pairs before B can be reassessed. The challenge by `kyverno-policy-review-agent` upheld A: `invoice-retry-worker` passes. Rollback limits for A: returning to Audit restores admission behavior for future writes only; writes denied meanwhile are not replayed, and changes already made to workloads are not undone. Execution of any promotion or return to Audit goes to `kubernetes-live-admission-policy-guard-agent` as a handoff; this review does not authorize it.

**Still needed**: for B, an explanation of the UPDATE difference from real old/new pairs for `billing.example/tier` changes; for both, the upgrade documentation's rollback limits.

**Decision today**: A `supported` (`approved`); B `blocked` (`blocked`).

**Cheapest discriminating check**: for B, one real UPDATE changing the tier label in a disposable cluster.

**Developer-facing denial message**: `payments-run-as-non-root: set securityContext.runAsNonRoot=true on every container. Escalate to the payments platform owner if the image cannot run as non-root.` It tells the developer to set the securityContext on each container.

**Escalation owner**: platform release manager for the matrix; cohort owners per row.

**Keep out of evidence**: customer data, image digests tied to customers, user identities.

## Counterexample

Cohort A's Audit window is clean, so the team promotes every `payments` cohort to Deny. The window described existing objects [KYV-E09], and cohort B's UPDATE control was never exercised with real pairs; promoting B turns an unproven assumption into a denied write path for a revenue-critical service. A clean window supports only the cohort it observed.

## Escalation threshold

Hand to `kyverno-admission-reliability-agent` if latency or availability is the concern, to `kyverno-policy-review-agent` for rule semantics, to `kubernetes-live-admission-policy-guard-agent` for any live promotion, and to the platform release manager for abort decisions.

## What would change the verdict

- Real old/new pairs for cohort B that explain the UPDATE difference and show agreement: B moves to `conditional` or `supported`.
- Any unexplained allow/deny difference on a required control in cohort A: A becomes `blocked`.
- An exception for a cohort with unverified expiry.
- Upgrade documentation showing a rollback limit that the plan did not state.
