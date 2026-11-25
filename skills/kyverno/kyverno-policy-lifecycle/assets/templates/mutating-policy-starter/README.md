# Mutating policy starter kit

A MutatingPolicy that adds a `cost-center` label to ConfigMaps created in `payments`, with an idempotent guard and `mutateExisting` off. Copy it, replace the `TEMPLATE` markers and rerun it before trusting any adaptation.

## What it demonstrates

- A `patchType: ApplyConfiguration` mutation with `reinvocationPolicy: Never` [KYV-E44].
- An idempotent matchCondition: the patch applies only when the label is absent, so a second pass changes nothing and an owner's value is never overwritten.
- `mutateExisting` explicitly set to false, because enabling it with trigger and target sharing a type rewrites every matching existing object on each trigger [KYV-E22].
- A `kyverno-test.yaml` using `isMutatingPolicy` and `patchedResources` against `patched.yaml` [KYV-E46]; the expected patch passes and a wrong expected patch fails with "Resource diff" (observed with the pinned CLI).
- Two cases: `payments/invoice-settings` (unlabelled) is patched and reports `pass` against `patched.yaml`; `payments/ledger-settings` already carries `cost-center: finance-ops`, reports `skip` and matches `unchanged.yaml`, so the owner's value is preserved. Observed: for a MutatingPolicy an excluded object reports `skip` and a wrong expected patch still fails with "Resource diff" (unlike GeneratingPolicy, where an excluded trigger passes with reason "Excluded"). The second case is what a second admission pass would see on an already-labelled object; it is offline evidence only, and admission reinvocation itself is not shown.

## Verified with

- Pinned kyverno CLI v1.20.0, offline, against `resources.yaml` and `patched.yaml`.
- It does not verify the admission webhook, the `-ignore` or `-fail` entry, the background controller, mutateExisting behavior, reinvocation ordering against other mutators, or any existing-object population.

## Adapt it

1. Replace every `TEMPLATE` marker: the policy name, the namespace and the label key and value.
2. Keep each field explicit, including `failurePolicy`, `reinvocationPolicy` and `mutateExisting`.
3. Keep one legitimate case, one violating or already-labelled case and one adjacent out-of-scope case in `resources.yaml`, each with a `patchedResources` expectation.
4. Rerun the kit with the pinned kyverno CLI v1.20.0 and read the REASON column, not only the result.

## Traps this kit avoids

- A non-idempotent patch: the guard makes a second run a no-op in the owner-labelled case. Inference: the guard would also make reinvocationPolicy irrelevant, but admission reinvocation is not shown here [KYV-E44].
- Enabling mutateExisting without a match-set inventory [KYV-E22].
- A wrong expected patch hidden by a loose comparison: the offline test fails with "Resource diff".
- Reading an offline `pass` as proof of background or reconciler behavior [KYV-E25].
