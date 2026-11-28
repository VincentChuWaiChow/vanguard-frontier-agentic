# Parity corpus starter kit

A migration parity corpus for one control, `example-require-run-as-non-root`, as a ValidatingPolicy over Pods in `payments`. It holds a conforming Pod, a violating Pod, an adjacent Pod outside scope and a violating Deployment, and names the operation on every result row. Copy it, replace the `TEMPLATE` markers and rerun it against both the source and the target engine.

## What it demonstrates

- One shared corpus (`corpus.yaml`) for every engine and release under comparison, with a `target-policy.yaml` in the CEL form the cohort moves to.
- Six result rows in `kyverno-test.yaml`, each naming its operation: `payments/legitimate-workload` pass on CREATE and UPDATE; `payments/violating-workload` fail on CREATE and UPDATE; `catalog/adjacent-workload` skip on CREATE (the kit pins the adjacent out-of-scope skip); and `payments/violating-deployment` fail on CREATE through autogen (the kit pins the autogen Deployment). An unset operation defaults to CREATE, and UPDATE uses identical old and new objects [KYV-E24].
- The result type flag `isValidatingPolicy` [KYV-E46] and explicit `validationActions`, `failurePolicy` and timeout fields.
- Autogen controllers named by lowercase plural resource name, which is what the resolver keys on [KYV-E38]; that is why the Deployment row produces a result.
- A developer-facing message that names the field to fix and who to ask.

## Verified with

- Pinned kyverno CLI v1.20.0, offline.
- It does not verify the webhook, the controller, registry access, real old/new UPDATE pairs, DELETE, exceptions, or any existing population.

## Adapt it

1. Replace every `TEMPLATE` marker: control name, owner, namespace, operations and expression.
2. Keep each field explicit, including `validationActions`, `failurePolicy` and `timeoutSeconds`.
3. Keep one legitimate, one violating and one adjacent out-of-scope object per control, each with every operation the control covers.
4. Rerun with the pinned kyverno CLI v1.20.0 and the source engine on its own pinned version, then diff the two result tables per operation.

## Traps this kit avoids

- A green UPDATE row for a control comparing `oldObject`: offline UPDATE uses identical objects [KYV-E24]. This control reads only `object`, so its UPDATE rows are not that trap, and an `oldObject` control added here needs real pairs.
- Kind names in autogen controllers producing no Deployment result (the autogen-name observation) [KYV-E38] [KYV-E39].
- A hyphenated identifier or field-style `image(x).registry` failing at load (the load-time compile observation).
- Reading `pass` as "verified" when the object was outside scope (the image-policy pass observation).
- Running legacy manifests with CLI v1.20.0, which rejects them [KYV-E01].
