# Which release and policy-family changes force a migration, and what is unverified?

## Question

For a move to Kyverno v1.20.0 or between policy families, which kinds must be re-expressed, which API versions are served, and which rollback claims cannot yet be made?

## Release scope

- Kyverno v1.20.0 deletes the legacy kyverno.io policy engine, removes legacy execution from the admission, background and reports controllers, blocks creates and spec updates of legacy policy APIs, and makes the CLI reject legacy manifests in `apply` and `test` [KYV-E01].
- The migration guide still prints "ClusterPolicy is deprecated and will be removed in a future release" and lists the legacy kinds; that wording lags the removal. Trust the release notes [KYV-E02] [KYV-E01].
- Legacy kinds that must be re-expressed: ClusterPolicy, Policy, PolicyException, CleanupPolicy and ClusterCleanupPolicy; the guide lists them [KYV-E02] and the release notes confirm the legacy engine removal [KYV-E01], which governs where the two differ.
- At tag v1.20.0 the `policies.kyverno.io` CRDs serve v1, v1beta1 and v1alpha1; v1beta1 is the storage version and v1alpha1 is marked deprecated [KYV-E03].
- Before v1.20, legacy ClusterPolicy defaults (failurePolicy `Fail`, webhook timeout 10 seconds within 1 to 30, `validationFailureAction` deprecated in favor of per-rule `failureAction`) are from v1.18 documentation only [KYV-E28].
- Native ValidatingAdmissionPolicy is a different family: stable since Kubernetes v1.30, needs a binding, and its binding actions are Deny, Warn and Audit with Deny and Warn not combinable [KYV-E27].

## Assumptions

- The source and target releases are the stated ones; the latest release is never assumed.
- Every legacy-kind object in the source estate is in scope until the inventory shows otherwise.
- Authoring new manifests as `policies.kyverno.io/v1` is possible, but conversion fidelity is a parity question, not a given.

## Evidence to request

- `installed-config`: inventory of installed policy kinds and API versions, and the CRD served and storage versions.
- `manifest` or `rendered-gitops`: every legacy-kind object in Git, with its owner.
- `offline-test`: a pinned CLI version and its result on the corpus.
- The environment's upgrade documentation, specifically any rollback guidance for the legacy removal.

## Interpretation

1. If any source policy is a legacy kind, the v1.20.0 target blocks creates and spec updates of it, so each must be re-expressed first [KYV-E01] [KYV-E02].
2. CLI `apply` and `test` reject legacy manifests at v1.20.0 [KYV-E01]; a corpus that still uses legacy kinds cannot be run through the pinned CLI, so parity needs a re-expressed corpus or an older CLI for the source side with its own version pinned.
3. Use served versus storage versions to decide which API version authored manifests should target and to check stored objects [KYV-E03]; do not choose a version from the docs page alone.
4. Treat the documentation's "future release" language as stale on the v1.20.0 question [KYV-E02] [KYV-E01].
5. Any rollback guidance for the legacy removal is unverified here; obtain the environment's upgrade documentation before promising any rollback path.
6. A move to native ValidatingAdmissionPolicy needs a binding and a choice of actions [KYV-E27]; a Kyverno exception with compensating validations prevents translation of its policy into a native one [KYV-E26].

## Worked example (synthetic)

**Requester supplied**: a `payments` estate on Kyverno v1.19.x with legacy ClusterPolicies for run-as-non-root and image rules, one legacy PolicyException, and a plan to upgrade to v1.20.0 next week.

**Agent derived**: the three supplied objects (two legacy ClusterPolicy objects and one legacy PolicyException, so two legacy kinds) must be re-expressed before the upgrade, because v1.20.0 blocks legacy creates and spec updates [KYV-E01] [KYV-E02]; the rollback path after the upgrade is unverified.

**Still needed**: the inventory of every legacy-kind object, the re-expressed policies, and the upgrade documentation's rollback section.

**Decision today**: `blocked` for the upgrade as planned, verdict `blocked`: required controls are not yet expressed in a family the target runs.

**Cheapest discriminating check**: `installed-config` listing legacy kinds in the source cluster.

**Developer-facing denial message**: `payments-run-as-non-root: set securityContext.runAsNonRoot=true on every container. Escalate to the payments platform owner if the image cannot run as non-root.` The same message is kept across the migration so developers see no change; confirm the migrated policy preserves it.

**Escalation owner**: platform release manager.

**Keep out of evidence**: cluster identifiers, registry credentials, customer data in manifests.

## Counterexample

A team reads "will be removed in a future release" in the migration guide and schedules the legacy rewrite after the upgrade. The release notes say the removal and the blocks on legacy creates and updates are already in v1.20.0 [KYV-E01]; the guide's wording lags, so trust the release notes [KYV-E02] [KYV-E01].

## Escalation threshold

Hand to `kyverno-policy-review-agent` for the semantics of any re-expressed rule, to `kyverno-admission-reliability-agent` for latency and availability effects, and to the platform release manager before any upgrade is authorized.

## What would change the verdict

- An inventory showing no legacy kinds in the source estate.
- Upgrade documentation stating a rollback path and its limits.
- A target release other than v1.20.0 with different release notes.
- Served and storage version evidence that differs from the tagged CRDs.
