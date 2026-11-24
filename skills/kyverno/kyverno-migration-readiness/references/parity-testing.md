# What can an offline parity run prove, and what needs a running API server or controller?

## Question

For a migration corpus run through the kyverno CLI, which behaviors does a matching result actually establish and which need a running API server or controller?

## Release scope

- `kyverno test` compares results declared ahead of time in a test manifest with actual results [KYV-E25]. Pinned CLI: v1.20.0 in this repository's offline fixtures.
- At v1.20.0 the CLI rejects legacy manifests in `apply` and `test` [KYV-E01]; legacy parity needs an older CLI whose version must be pinned and named; how an older CLI models UPDATE is unverified here.
- In the CLI test file, operation UPDATE sets both object and oldObject to the same resource, DELETE sets object to null, and an unset operation defaults to CREATE [KYV-E24].

## Assumptions

- The corpus contains both violating and conforming resources per control, including one adjacent out-of-scope object.
- The pinned CLI version is recorded with every result.
- Offline results describe inputs, not the cluster.

## Evidence to request

- `offline-test`: declared expectations and actual results for source and target policies on the same corpus, per operation.
- `controlled-test`: real old/new object pairs for each UPDATE-sensitive control.
- `admission-trace`: webhook timeout and failure-policy outcomes for the target.
- `population-report`: background results for existing resources.

## Interpretation

1. A passing `kyverno test` shows the declared expectations agree with the actual results for those inputs [KYV-E25]; it does not show the expectations are right.
2. Observed with the pinned kyverno CLI v1.20.0 in this repository's offline fixtures: Audit and Deny give the same result (`fail`) for the same violation (fixture 02), so an Audit-to-Deny promotion shows no offline difference and the admission effect is invisible offline.
3. Observed with the same CLI: an object excluded by match conditions reports `skip` (fixture 01).
4. Observed with the same CLI: an exception whose `expiresAt` is in the past is still applied (`skip`) (fixture 03), while the controller's lookup skips expired exceptions [KYV-E14]. Offline results cannot establish expiry behavior.
5. For UPDATE, object and oldObject are identical [KYV-E24]; a policy comparing `object` with `oldObject` passes trivially (fixture 04). UPDATE parity on such a control is unproven without real old/new pairs.
6. Behaviors that need a running API server or controller:

| Behavior | Why offline cannot show it | Evidence that can |
| --- | --- | --- |
| Real old/new UPDATE pairs | CLI sets both to the same object [KYV-E24] | `controlled-test` with real pairs, or `admission-trace` |
| Webhook timeouts and failurePolicy outcome | The API server calls webhooks; the CLI makes no webhook call | `admission-trace`, `installed-config` of webhook configuration |
| Controller exception expiry | Offline applies expired exceptions (fixture 03) [KYV-E14] | post-expiry `admission-trace` |
| Background reports for existing resources | Reports describe resources that exist in the cluster [KYV-E09]; the CLI evaluates supplied inputs only | `population-report` |
| Generate, mutate-existing, scheduled deletion | Run by background and cleanup controllers [KYV-E22] [KYV-E23] | `controlled-test` |

7. A parity difference is the finding: name the corpus item, operation, source result, target result and the evidence still missing.

## Worked example (synthetic)

**Requester supplied**: a migration corpus for `payments` with `invoice-batch-root`, `invoice-retry-worker` and `invoice-api` (a Deployment). Source and target results agree for CREATE on every item; for the control `invoice-tier-label-immutable` the target result is `pass` for UPDATE and the source (run with an older CLI) is `fail`.

**Agent derived**: UPDATE parity differs for that control, and the target `pass` is the identical-object artifact [KYV-E24] (fixture 04). CREATE parity holds for the inputs given. The corpus agrees for CREATE but differs for UPDATE, so parity is blocked for the cohort that depends on this control.

**Still needed**: real old/new object pairs for `invoice-api` showing the label change, from a `controlled-test`, run on both sides.

**Decision today**: `blocked` for that cohort, verdict `blocked`. Why: on a required control an unexplained allow/deny difference is itself the demonstrated defect for the parity requirement, so the cohort cannot move even though the target `pass` is probably an artifact. `conditional` for CREATE-only controls on offline parity alone, never `supported`, because the offline run does not show the admission effect or any webhook behavior.

**Cheapest discriminating check**: one real UPDATE changing `billing.example/tier` on `payments/invoice-api` in a disposable cluster on both policy families.

**Developer-facing denial message**: `invoice-tier-label-immutable: billing.example/tier cannot change in place. Create a new Deployment for the new tier; escalate to the billing platform owner.` The developer is told to create a new Deployment instead of editing the label.

**Escalation owner**: billing platform owner for the control; platform release manager for the cohort.

**Keep out of evidence**: customer tenant identifiers and image digests tied to customers.

## Counterexample

A green `kyverno test` on a migrated policy is cited as proof that the upgrade preserves behavior. The suite covered only CREATE and used identical objects for UPDATE [KYV-E24], so the UPDATE-sensitive control was never exercised; the green result is true and says nothing about that control.

## Escalation threshold

Hand to `kyverno-policy-review-agent` when a difference traces to a rule's semantics, to `kyverno-admission-reliability-agent` for webhook behavior, and to the platform release manager for any cohort decision.

## What would change the verdict

- Real old/new pairs from a `controlled-test` showing agreement on UPDATE.
- A pinned CLI that models UPDATE with real old/new objects, with the version recorded; whether any release does is unverified here.
- DELETE coverage in the corpus where the control applies to deletion.
- An unexplained difference on any required control: `blocked`.
