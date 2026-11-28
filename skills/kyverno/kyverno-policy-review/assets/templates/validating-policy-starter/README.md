# ValidatingPolicy starter kit

A policy, resources and CLI test for one control on the synthetic invoice service in namespace `payments`: run containers as non-root. Files: `policy.yaml`, `resources.yaml`, `kyverno-test.yaml`.

## What it demonstrates

- One legitimate Pod (`payments/legitimate-workload`) that passes, one in-scope violation (`payments/violating-workload`, an init container without `runAsNonRoot`) that fails, and one adjacent Pod in another namespace (`catalog/adjacent-workload`) that is skipped because the namespace condition excludes it.
- Pod-level inheritance: a Pod that sets `runAsNonRoot: true` only in its Pod `securityContext` passes (`payments/pod-level-non-root`), because each container without its own setting inherits it; a container that sets `runAsNonRoot: false` overrides the Pod and fails (`payments/container-overrides-pod-level`).
- A Deployment (`payments/violating-deployment`) that fails through autogen, which pins pod-controller coverage offline for the Deployment only [KYV-E37]. The kit lists five controllers (deployments, statefulsets, daemonsets, jobs, cronjobs) but tests only the Deployment; add a row per controller you claim to cover.
- Every default that matters is written explicitly: `validationActions`, `failurePolicy`, `webhookConfiguration.timeoutSeconds`, evaluation toggles and the autogen block [KYV-E04] [KYV-E05] [KYV-E06].
- Native ValidatingAdmissionPolicy generation is left at its per-policy opt-in value of false [KYV-E37] [KYV-E40].
- A denial message that names the field to fix and the owner to ask.

## Verified with

- The pinned kyverno CLI v1.20.0, offline: the policy compiles and the four results in `kyverno-test.yaml` match.
- Not verified: the webhook, admission decisions (Audit and Deny give the same offline `fail`), the controller, existing resources, populations and any live lookup.

## Adapt it

1. Replace every `TEMPLATE` marker (control name, owner, operations, namespace scope, controller list, expression).
2. Keep each field explicit; do not rely on defaults.
3. Keep one legitimate, one violating and one adjacent case, and one case per pod controller you claim to cover.
4. Rerun the kit with the pinned CLI and read the result for every object; an absent result is not a pass.

## Traps this kit avoids

- Kind-named autogen controllers: with Kind names the CLI produced no result for the Deployment at all, so the kit uses lowercase plural resource names [KYV-E38] [KYV-E39].
- An undeclared CEL function or a field-style `image(x).registry` fails at load; the CLI compiles every expression even when no object matches.
- Reading a `skip` on the adjacent Pod as proof the control works; it only shows the object is out of scope.
- Assuming a native ValidatingAdmissionPolicy exists because the controller toggle defaults to true [KYV-E26] [KYV-E37].
