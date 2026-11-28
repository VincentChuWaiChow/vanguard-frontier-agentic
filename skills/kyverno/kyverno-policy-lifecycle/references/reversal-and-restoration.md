# What does reverting the policy in Git actually restore?

## Question

After a Kyverno lifecycle policy has run and the policy is reverted in Git, which objects return to their prior state, which do not, and what separate plan restores the rest?

## Release scope

- GeneratingPolicy, MutatingPolicy and DeletingPolicy in `policies.kyverno.io` at v1.20.0 [KYV-E20] [KYV-E22] [KYV-E23].
- Legacy policy kinds are removed with the legacy engine in v1.20.0 [KYV-E01]; a revert to a legacy kind on v1.20.0 is blocked by the release, and the migration guide's wording lags [KYV-E02] [KYV-E01]. Behavior of legacy kinds before v1.20 is not encoded here.

## Assumptions

- "Revert" means the desired state in Git returns to the earlier commit and the reconciler applies it.
- Data restoration is a different operation from policy reversal and has its own owner.
- Per-object state is only known from supplied evidence, not from the policy's intent.

## Evidence to request

- `installed-config`: downstream, target and deleted-kind object state before and after the change (labels, data keys, existence).
- `rendered-gitops`: the reverted desired state and whether the reconciler prunes.
- `manifest`: the policy's source type, synchronization, orphan setting, mutation targets and deletion selector.
- `controlled-test`: a disposable-cluster dry run of the reversal, if the claim is consequential.
- A backup or restore record for deleted objects.

## Interpretation

1. Build the state table before judging any rollback claim:

| Object | Before change | After change | After Git revert | After restoration plan | Evidence |
| --- | --- | --- | --- | --- | --- |
| (object name and kind) | (state) | (state) | (state or unverified) | (state or none) | (evidence ID) |

2. Fill "After Git revert" only from cited behavior: data-source downstream objects are deleted on policy deletion with synchronization enabled unless orphaning was enabled, and retained without synchronization [KYV-E34] [KYV-E21]; clone-source downstream objects are retained [KYV-E35]; no cited source shows mutated fields being reversed, so treat persistence as the safe assumption and verify; no cited source describes restoration of deleted objects, and DeletingPolicy is destructive [KYV-E23]. Anything else is `unverified`.
3. Keep two plans: a reversal plan (policy and reconciler state) and a data-restoration plan (objects and fields). A single "rollback" line that merges them hides the gap.
4. Conflict disposition: if the reconciler also owns an object, state who wins on the next sync and mark convergence unproven without `rendered-gitops` evidence.
5. A rollback claim based only on reverting Git is rejected: it needs downstream state evidence for each row.
6. Restoration of deleted data is a live operation: hand it off, for example to `kubernetes-live-velero-restore-guard-agent`; this skill makes no claims about how that agent works.

## Worked example (synthetic)

**Requester supplied**: in `payments`, a GeneratingPolicy with a data source and synchronization on created `invoice-limits` ConfigMaps; another GeneratingPolicy with a clone source created `invoice-signing-config` copies; a DeletingPolicy removed `invoices-archive` ConfigMaps. The team reverted all three in Git and states "payments is back to the prior state".

**Agent derived**: none of the three policies existed at the earlier commit, so the revert removes them (a removal, not a content-only rollback, which would instead sync downstream to the earlier content [KYV-E34]). With synchronization on, the data-source downstream `invoice-limits` objects are deleted by that removal unless orphaning was enabled (retained without synchronization) [KYV-E34] [KYV-E21]; the clone downstream copies stay [KYV-E35]; for the `invoices-archive` ConfigMaps no cited source describes restoration, and DeletingPolicy is destructive [KYV-E23], so they are treated as still deleted until state evidence shows otherwise.

| Object | Before change | After change | After Git revert | After restoration plan | Evidence |
| --- | --- | --- | --- | --- | --- |
| `invoice-limits` ConfigMaps | absent | present (generated) | deleted (synchronization on) unless orphaning enabled | not needed if absent was intended | manifest, installed-config |
| `invoice-signing-config` clones | absent | present (cloned) | still present | delete by separate plan if unwanted | manifest, installed-config |
| `invoices-archive` ConfigMaps | present | deleted | still deleted | restore by separate guarded plan | installed-config, backup record |

**Still needed**: the orphan setting, current object state and a backup record for the archive.

**Decision today**: `blocked` for the claim "reverted to the prior state", verdict `blocked`: the stated requirement cannot hold from a Git revert alone; the reversal plan can be `conditional` once state evidence arrives.

**Cheapest discriminating check**: an `installed-config` capture of the three object kinds after the revert.

**Developer-facing denial message**: not applicable; none of these policies deny a request.

**Escalation owner**: platform operations engineer for the reversal plan; data owner for restoration.

**Keep out of evidence**: ConfigMap data, signing material, tenant and customer identifiers.

## State after rollback

This whole reference concerns state after rollback; the table is the deliverable. Rows with unverified behavior stay `unverified` until downstream state evidence is supplied; never fill them from the policy's intent.

## Counterexample

A GitOps diff shows no remaining difference after the revert, and the team concludes state is restored. The diff compares desired state to what the reconciler manages; objects it never managed (the clones, the mutated fields, the deleted archive) are invisible to it. An empty diff is not evidence that the earlier state is back.

## Escalation threshold

Hand to `kubernetes-live-admission-policy-guard-agent` for any live policy change, to the data owner and a guarded restore handoff for deleted data, and to `argocd-gitops-review-agent` or `fluxcd-kustomization-helmrelease-review-agent` for reconciler conflicts.

## What would change the verdict

- `installed-config` showing every row of the table matches the intended post-revert state.
- Evidence that orphaning was enabled or synchronization was off, changing the data-source row [KYV-E34].
- A restore record for the deleted objects and the owner's confirmation.
- A different release with documented behavior that differs.
