# Which MutatingPolicy, GeneratingPolicy and DeletingPolicy fields decide blast radius, and how do you test them offline?

## Question

For a lifecycle policy that rewrites, creates or deletes objects, which fields set how many objects it touches and what survives removal, and which of those can the pinned kyverno CLI exercise offline?

## Release scope

- MutatingPolicy, GeneratingPolicy and DeletingPolicy in `policies.kyverno.io` at v1.20.0 [KYV-E44] [KYV-E21] [KYV-E45]. The legacy engine is removed in v1.20.0 [KYV-E01]; the migration guide's wording lags [KYV-E02] [KYV-E01]. Legacy ClusterPolicy before v1.20 is not encoded here [KYV-E28].
- Offline observations are from the pinned kyverno CLI v1.20.0 in this repository's starter kits, never controller behavior.

## Assumptions

- The requester supplies the policy as it will be applied, not a sketch.
- The target object set is unknown until enumerated from a population-report or inventory.
- A reconciler owning the same objects is assumed possible until evidence says otherwise.
- Offline tests show what the policy computes for supplied inputs; they do not show background or cleanup controller behavior.

## Evidence to request

- `manifest` or `rendered-gitops`: the policy with `patchType`, `reinvocationPolicy`, `targetMatchConstraints`, `targetMatchConditions`, the evaluation switches, `schedule`, `conditions` and `deletionPropagationPolicy`.
- `installed-config`: the ClusterRole for the background and cleanup controllers.
- `population-report` or an inventory: the objects each selector matches today.
- `offline-test`: the kyverno-test file with `patchedResources` or `generatedResource` and the REASON column of the output.
- `controlled-test`: a disposable-cluster run for anything the CLI cannot exercise.

## Interpretation

1. MutatingPolicy mutations use patchType ApplyConfiguration or JSONPatch, and reinvocationPolicy is Never or IfNeeded [KYV-E44]. Read both. Inference, not a cited fact: a patch that is not idempotent combined with IfNeeded may apply more than once. The mutating starter guards with an idempotent matchCondition that skips objects already labelled.
2. `targetMatchConstraints` and `targetMatchConditions` select mutate-existing targets [KYV-E44]. mutateExisting applies mutations to existing resources through background processing, usually needs extra background-controller RBAC, and when trigger and target share a type mutates all matching existing resources on each trigger [KYV-E22]. The blast radius is the target selector's match set, not the trigger.
3. GeneratingPolicy `generator.Apply` with a data source is what the generating starter uses. `synchronize`, `generateExisting` and `orphanDownstreamOnPolicyDelete` all default to false at v1.20.0, the orphan option applies only to data-type generation, and `skipBackgroundRequests` defaults to true [KYV-E21].
4. With a data source and synchronization, deleting the policy deletes downstream unless orphaning is enabled; deleting the trigger deletes downstream; editing downstream is reverted; generateExisting performs a one-time scan of existing triggers [KYV-E20] [KYV-E34]. Without synchronization, downstream is retained on policy delete [KYV-E34].
5. DeletingPolicy has `schedule`, `conditions`, `matchConstraints`, `variables` and `deletionPropagationPolicy` (Foreground, Background or Orphan) [KYV-E45]. It runs on a cron schedule with one-minute minimum granularity, is destructive, and the cleanup controller needs get, list, watch and delete on the targets [KYV-E23]. Wider RBAC than the stated selector widens the real blast radius.
6. CLI test results declare the policy type with `isMutatingPolicy` or `isGeneratingPolicy` and compare `patchedResources` or `generatedResource` [KYV-E46]; `kyverno test` compares declared results with actual results [KYV-E25].
7. Observed with the pinned kyverno CLI v1.20.0 in this repository's starter kits (the mutating-patch observation): a MutatingPolicy test with a wrong expected patch fails with "Resource diff"; the correct one passes.
8. Observed with the same CLI (the generating-test observation): a GeneratingPolicy test with a wrong expected object fails with "Resource diff". When the trigger is out of scope, the test passes with reason "Excluded" whether the expected result is `pass` or `skip`. Read the REASON column as a test-reading rule, and keep an in-scope trigger as the discriminating case; an excluded trigger cannot be pinned offline.
9. Neither offline test shows background application to existing objects, synchronization against a reconciler, or scheduled deletion; those need a `controlled-test` [KYV-E22] [KYV-E23].

## Worked example (synthetic)

**Requester supplied**: the mutating starter adds `cost-center: payments` to ConfigMap CREATE in `payments`, with `mutateExisting` disabled; the generating starter creates an `invoice-limits` ConfigMap for each Namespace labelled `tenant-type: invoice`, with synchronization on and `orphanDownstreamOnPolicyDelete` true. Both kits' kyverno-test files are supplied with passing output.

**Agent derived**: the mutating kit's matchCondition skips objects that already carry the label, so a second pass would add nothing and an owner's value is not overwritten. That is shown offline by the owner-labelled case `payments/ledger-settings` only: it reports `skip` and its patchedResources comparison against `unchanged.yaml` passes; the unlabelled `payments/invoice-settings` comparison passes too. Admission reinvocation itself is not shown. The generating kit's in-scope Namespace `tenant-a` produces the expected ConfigMap. The orphan switch is true on purpose: with a data source and synchronization, policy deletion would otherwise delete the generated ConfigMaps [KYV-E34] [KYV-E21].

**Still needed**: whether a GitOps reconciler also owns `invoice-limits` (two writers under synchronization), and the inventory of Namespaces generateExisting would touch if it were enabled.

**Decision today**: `conditional`, verdict `needs-review`; both kits' offline computed results are consistent; admission is not shown. Enabling mutateExisting or generateExisting requires the match-set inventory first.

**Cheapest discriminating check**: settle the open question first, whether a GitOps reconciler also owns `invoice-limits` (reconciler ownership) and which Namespaces carry the trigger label (trigger inventory); both need `installed-config` or `population-report` evidence. As a test-reading rule, an out-of-scope Namespace test reads "Excluded" in the REASON column whichever result is expected, so keep the in-scope case as the pass that counts.

**Developer-facing denial message**: not applicable; mutation and generation are not admission denials.

**Escalation owner**: platform engineer owning the policy; the data owner for generated or deleted objects.

**Keep out of evidence**: ConfigMap contents, tenant identifiers, ClusterRole subjects beyond role names.

## State after rollback

Revert in Git and then fill the state table for every object class. Data-source downstream objects follow the synchronization and orphan switches [KYV-E34]; clone-source downstream objects are retained [KYV-E35]; no cited source shows mutated fields being reversed, so treat persistence as the safe assumption and verify; DeletingPolicy is destructive [KYV-E23] and no cited source describes restoring what it deleted, so that needs a separate restore plan. A live restore is a guarded operation, for example `kubernetes-live-velero-restore-guard-agent`, named only as a handoff.

## Counterexample

A generation test "passes", so the team signs off. The trigger Namespace in the test lacked the matching label, so the case was excluded and the test passed with reason "Excluded" whether the expected result was `pass` or `skip`. Nothing was generated or compared. Only an in-scope trigger with a `generatedResource` comparison pins the behavior.

## Escalation threshold

Hand to the data owner and `kubernetes-live-velero-restore-guard-agent` (as a handoff, never auto-dispatched) when objects were deleted or overwritten. Hand to `kyverno-admission-reliability-agent` when the mutation sits on a Fail webhook path, and to a human owner for any enablement of mutateExisting, generateExisting or a DeletingPolicy schedule.

## What would change the verdict

- A `population-report` or inventory showing the selector match set is smaller or larger than assumed.
- Evidence a reconciler owns the downstream or mutated objects.
- A `controlled-test` showing downstream state after policy deletion matches the state table.
- A backup record for the objects a DeletingPolicy would remove.
