# What do mutate-existing and scheduled deletion change, and what stays changed?

## Question

What will a MutatingPolicy with `mutateExisting` or a DeletingPolicy actually rewrite or remove, which permissions does that need, and what remains after the policy is reversed?

## Release scope

- MutatingPolicy and DeletingPolicy in `policies.kyverno.io` at v1.20.0 per the cited documentation [KYV-E22] [KYV-E23].
- Legacy mutate-existing and cleanup policies (`ClusterPolicy`, `CleanupPolicy`, `ClusterCleanupPolicy`) belong to the legacy engine deleted in v1.20.0 [KYV-E01] [KYV-E02]; the migration guide's wording lags the release notes, so trust the release notes [KYV-E01].

## Assumptions

- The trigger and target selection are as written in the supplied manifest; nothing is inferred from names.
- The reviewer has not seen the cluster; counts of matching objects are requested, not estimated.
- A second run is not assumed to be harmless.

## Evidence to request

- `manifest`: MutatingPolicy `mutateExisting` settings with trigger and target selection; DeletingPolicy schedule, match and conditions.
- `installed-config`: the background or cleanup controller RBAC (ClusterRole and bindings) and the targets' current state.
- `rendered-gitops`: whether a reconciler also writes the mutated fields or recreates the deleted objects.
- `population-report` or inventory: the count and list of objects the selection matches.
- `controlled-test`: a disposable-cluster run, including a second run, if idempotency matters.

## Interpretation

1. `mutateExisting` applies mutations to existing resources through background processing and usually needs extra RBAC for the background controller [KYV-E22]. Do not assume the mutation is applied without RBAC evidence; what a missing permission produces in this release is unverified.
2. When trigger and target share a type, all matching existing resources are mutated on each trigger [KYV-E22]. Enumerate the match set; one trigger can rewrite many objects.
3. A DeletingPolicy runs on a cron schedule with one-minute minimum granularity and performs destructive actions [KYV-E23]. A schedule faster than the review cycle can remove objects before anyone reacts.
4. The cleanup controller needs `get`, `list`, `watch` and `delete` on the targets [KYV-E23]. Broad RBAC widens what a mis-written selector can delete; compare RBAC with the selection.
5. GitOps interaction: a reconciler that renders the deleted kind may recreate it on its next sync, and then the policy deletes it again; a reconciler that renders the mutated field may revert it, and a mutation then rewrites it. Neither outcome is converged. Ask for `rendered-gitops`; route reconciler settings to `argocd-gitops-review-agent` or `fluxcd-kustomization-helmrelease-review-agent`.
6. Second-run result: whether a second mutation or generation run changes anything is an idempotency question to test with a `controlled-test`, not to assume.
7. Never run the policy, a sync or a cleanup to find out; a live run is a separate guarded operation.

## Worked example (synthetic)

**Requester supplied**: a MutatingPolicy with `mutateExisting` that sets an `invoice.example/retention` label on every ConfigMap in `payments` when an Invoice trigger object changes; a DeletingPolicy that removes ConfigMaps labeled `invoice.example/archived=true` older than a threshold on a nightly schedule; RBAC granting the cleanup controller delete on ConfigMaps cluster-wide.

**Agent derived**: if the trigger and the target share a type, all matching existing objects are mutated on each trigger [KYV-E22]; the supplied manifest must show whether they do; the deletion is destructive on a schedule [KYV-E23]; cluster-wide delete RBAC is wider than the stated target namespace.

**Still needed**: the matched-object inventory, the exact schedule, and evidence a reconciler does not own these ConfigMaps.

**Decision today**: `conditional`, verdict `needs-review`; narrowing the RBAC and enumerating the match set are conditions.

**Cheapest discriminating check**: a `population-report` or inventory of the objects the selectors match, plus the ClusterRole.

**Developer-facing denial message**: not applicable; mutation and deletion are not admission denials.

**Escalation owner**: platform operations engineer; data owner for the ConfigMaps.

**Keep out of evidence**: ConfigMap contents, invoice identifiers, retention policy text beyond its ID.

## State after rollback

No cited source shows mutations being reversed when the policy is removed; treat persistence as the safe assumption and verify with downstream state evidence. DeletingPolicy is destructive [KYV-E23], and no cited source describes restoration of the objects it deleted, so they need a separate restore plan; a live restore is a separate guarded operation, for example by `kubernetes-live-velero-restore-guard-agent`, mentioned here only as a handoff. Whether a backup exists for the deleted objects is an unknown to settle before approval.

## Counterexample

A team removes a DeletingPolicy after it deleted the `invoices-archive` ConfigMaps and reports "rolled back". The policy no longer runs, but no cited source describes restoration of the deleted objects, and DeletingPolicy is destructive [KYV-E23]. The rollback claim covers only future runs.

## Escalation threshold

Hand to `kubernetes-live-admission-policy-guard-agent` for any live change, to the data owner when deleted objects held business data, to the GitOps agents for reconciler conflicts, and to `kubernetes-live-velero-restore-guard-agent` only as a handoff for restoration.

## What would change the verdict

- An inventory showing the match set is small and expected.
- RBAC narrowed to the stated targets.
- A restore-tested backup for the deleted kind.
- A `controlled-test` showing a second run changes nothing.
