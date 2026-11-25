# What happens to generated downstream objects, and will a reconciler fight synchronization?

## Question

For a GeneratingPolicy, what happens to the downstream object when the policy, the trigger, the source or the downstream object itself changes, and does a GitOps reconciler converge with that behavior?

## Release scope

- `policies.kyverno.io` GeneratingPolicy at v1.20.0. Behavior below is from the documentation and the v1.20.0 CRD [KYV-E20] [KYV-E21]; the data-source and clone-source tables are supported cell by cell by [KYV-E34] [KYV-E35].
- Legacy `ClusterPolicy` generate rules ran in the legacy engine, which v1.20.0 deletes [KYV-E01]; on releases before v1.20 legacy generate behavior is not encoded here; the v1.18 legacy documentation cited in this repository covers only failurePolicy, timeout and validationFailureAction [KYV-E28].
- Documentation tables describe the documented behavior; confirm the exact release's behavior with a `controlled-test` before relying on an edge row.

## Assumptions

- The downstream object has a single intended owner; if two writers own it, convergence is not established.
- A data source means the policy carries the content; a clone source means the content is copied from an existing object.
- Synchronization (`Sync`) is distinguished from no synchronization (`NoSync`) in each row.

## Evidence to request

- `manifest`: the GeneratingPolicy with source type (data or clone), synchronization setting and `evaluation.generateExisting`, `evaluation.orphanDownstreamOnPolicyDelete`, `skipBackgroundRequests`.
- `rendered-gitops`: whether Argo CD or Flux also renders, owns or prunes the downstream kind in the target namespace.
- `installed-config`: the downstream objects present, their labels and owner references.
- `controlled-test`: a disposable-cluster run for any edge row the sources do not settle.

## Interpretation

1. Defaults at v1.20.0: `evaluation.synchronize.enabled` defaults to false, so a policy that does not set it is NoSync [KYV-E21]; `generateExisting.enabled` and `orphanDownstreamOnPolicyDelete.enabled` default to false; the orphan option applies only to data-type generation; `skipBackgroundRequests` defaults to true [KYV-E21].
2. `generateExisting` performs a one-time scan of existing triggers [KYV-E20]. Enabling it can create downstream objects for triggers that predate the policy; the number of objects created is a consequence to enumerate before the change.
3. Documented behavior with a data source and synchronization [KYV-E20] [KYV-E34]: deleting the policy deletes downstream resources unless `orphanDownstreamOnPolicyDelete` is enabled; deleting the trigger deletes the downstream resource. Without synchronization the policy delete retains downstream resources and the trigger delete has no effect [KYV-E34].
4. Documented behavior with a clone source: deleting the policy retains the downstream resource, with or without synchronization [KYV-E20] [KYV-E35].
5. The documented event tables; every cell is supported by [KYV-E34] for a data source and [KYV-E35] for a clone source:

| Event | Data source, Sync | Data source, NoSync | Clone source, Sync | Clone source, NoSync |
| --- | --- | --- | --- | --- |
| Delete downstream | Recreated | Deleted | Recreated | Deleted |
| Delete policy | Deleted unless `orphanDownstreamOnPolicyDelete` is enabled | Retained | Retained | Retained |
| Delete trigger | Deleted | No effect | Deleted | No effect |
| Delete source | Not applicable | Not applicable | Deleted | Retained |
| Modify downstream | Reverted | Modified | Reverted | Modified |
| Modify policy | Synced | Unmodified | Unmodified | Unmodified |
| Modify trigger | Deleted | No effect | Deleted | No effect |
| Modify source | Not applicable | Not applicable | Synced | Unmodified |

6. Read the table against the reconciler, not in isolation: "Reverted" with Sync means Kyverno writes the object back, so a GitOps tool that also owns that object will see drift on every reconcile.
7. Sync versus NoSync is per policy: confirm `evaluation.synchronize.enabled` in the supplied manifest before choosing a column; the default is false [KYV-E21].
8. GitOps convergence: a reconciler that also owns or prunes the downstream object can fight synchronization, one side recreating or deleting what the other changes. Do not conclude convergence without `rendered-gitops` showing who owns the downstream kind and `installed-config` showing owner labels. Route reconciler configuration (sync policy, prune, ignore differences) to `argocd-gitops-review-agent` or `fluxcd-kustomization-helmrelease-review-agent`.
9. Second run: whether re-applying the policy or re-triggering changes the downstream object again is an idempotency question; require a `controlled-test` result rather than assuming no change.

## Worked example (synthetic)

**Requester supplied**: a GeneratingPolicy in `payments` that creates a default `invoice-limits` ConfigMap per invoice-tenant namespace from inline data with synchronization on, and a second one that clones `invoice-signing-config` into each tenant namespace; Argo CD also renders `invoice-limits` in the same namespaces.

**Agent derived**: with synchronization enabled, deleting the data-source policy deletes the downstream ConfigMaps unless orphaning is enabled; retained without synchronization [KYV-E34]; the orphan option is off by default [KYV-E21]. Deleting the clone-source policy keeps the clones [KYV-E35]. Argo CD owning the same ConfigMap means two writers: convergence is unproven.

**Still needed**: the Argo CD ownership and prune setting for that kind, the policy's orphan setting, and a second-run result.

**Decision today**: `conditional`, verdict `needs-review`.

**Cheapest discriminating check**: `rendered-gitops` plus `installed-config` showing which controller owns `invoice-limits`.

**Developer-facing denial message**: not applicable; generation does not deny requests.

**Escalation owner**: platform operations engineer; GitOps owner for reconciler settings.

**Keep out of evidence**: ConfigMap data values, signing material, tenant identifiers.

## State after rollback

Separate the two kinds of rollback. If the revert removes the policy (it did not exist at the earlier commit, or the reconciler prunes it): data-source downstream objects are deleted with synchronization enabled unless orphaning was enabled, and retained without synchronization [KYV-E34]; clone downstream objects stay [KYV-E35]. If the revert only restores earlier policy content and the policy stays: with a data source and synchronization the downstream is synced to the earlier content and unmodified without it [KYV-E34]; clone downstream objects are unmodified [KYV-E35]. Whether objects created by `generateExisting` behave the same way is unverified here; settle it with downstream state evidence. Objects generated while the policy existed and changed by other writers are not restored by anything here. Treat any more specific claim as unverified until downstream state evidence is supplied.

## Counterexample

A team argues "synchronization keeps downstream objects aligned with the policy, so a revert is safe". If the revert removes the policy, a data source with synchronization deletes the downstream objects unless orphaning was on (they are retained without synchronization) [KYV-E34] [KYV-E21]; with a clone source it leaves them [KYV-E35]. If it only restores earlier content, synchronization rewrites the downstream to that content instead [KYV-E34]; neither case returns objects that other writers changed. Synchronization governs how downstream follows the policy, not whether state returns to its prior form.

## Escalation threshold

Hand to `argocd-gitops-review-agent` or `fluxcd-kustomization-helmrelease-review-agent` for reconciler settings, to `kubernetes-live-admission-policy-guard-agent` for any live sync or deletion, and to the data owner when downstream objects hold data.

## What would change the verdict

- `rendered-gitops` showing the reconciler does not own the downstream kind: convergence risk drops.
- A `controlled-test` showing the second run makes no change.
- Orphaning enabled, or synchronization shown disabled, and evidenced: policy deletion no longer deletes data-source downstream objects [KYV-E34].
- A different release whose table differs.
