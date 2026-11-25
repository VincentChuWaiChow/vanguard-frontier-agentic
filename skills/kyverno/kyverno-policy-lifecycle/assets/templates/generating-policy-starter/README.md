# Generating policy starter kit

A GeneratingPolicy with a data source that creates an `invoice-limits` ConfigMap for each Namespace labelled `tenant-type: invoice`. Copy it, replace the `TEMPLATE` markers and rerun it before trusting any adaptation.

## What it demonstrates

- `generator.Apply` with a data source built in `variables`, triggered on Namespace CREATE.
- All three lifecycle switches stated explicitly: `generateExisting` false, `synchronize` true and `orphanDownstreamOnPolicyDelete` true. At v1.20.0 all three default to false, and the orphan option applies only to data-type generation [KYV-E21].
- The orphan switch is true deliberately. With a data source and synchronization, deleting the policy deletes the generated downstream objects unless orphaning is enabled [KYV-E34] [KYV-E20], so leaving it false would make a policy revert delete every generated ConfigMap.
- A `kyverno-test.yaml` using `isGeneratingPolicy` and `generatedResource` against `generated.yaml` [KYV-E46]; a wrong expected object fails with "Resource diff" (O5).

## Verified with

- Pinned kyverno CLI v1.20.0, offline, against `resources.yaml` and `generated.yaml`.
- It does not verify the background controller, synchronization against a GitOps reconciler, generateExisting over real Namespaces, trigger deletion, or the downstream state after policy deletion.

## Adapt it

1. Replace every `TEMPLATE` marker: the policy name, the trigger label and the generated data.
2. Keep each switch explicit; decide orphaning from the behavior table, not from a default [KYV-E34].
3. Keep one in-scope trigger, one violating or out-of-scope trigger and one adjacent object. The out-of-scope trigger passes with reason "Excluded" whatever result is expected, so it never pins generation (O5); the in-scope case is the one that counts.
4. Rerun the kit with the pinned kyverno CLI v1.20.0 and read the REASON column.

## Traps this kit avoids

- A test that passes because the trigger was excluded (O5).
- Silent deletion of downstream objects on policy delete: orphaning is explicit [KYV-E34] [KYV-E21].
- Two writers: with synchronization on, Kyverno reverts edits to the downstream object, so a GitOps reconciler owning it will fight [KYV-E34].
- An unbounded generateExisting scan: it stays false until the trigger set is enumerated [KYV-E20].
