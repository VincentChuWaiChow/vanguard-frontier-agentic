# PolicyException starter kit

A PolicyException for one legacy workload of the synthetic invoice service in namespace `payments`, with a compensating control, tested against the policy it names. Files: `policy.yaml`, `exception.yaml`, `resources.yaml`, `kyverno-test.yaml`, and `missing-compensating-control/` (resources and a second test).

## What it demonstrates

- Exactly one approved workload (`legacy-invoice-exporter`) matched by name and namespace, never a whole namespace; `policyRefs` carries the exact policy name and kind [KYV-E13] [KYV-E14].
- A compensating validation (label `network-isolation=strict`): with it the approved workload reports `skip`; without it the same workload reports `fail`; an adjacent workload (`invoice-batch-root`) with no exception reports `fail` [KYV-E16].
- `expiresAt` set as the removal mechanism the controller honors; annotations for the approval record and expiry owner are inert governance data [KYV-E13] [KYV-E14].
- The enablement caveat written in the file header: exceptions apply only when `enablePolicyException` is on and the namespace is allowed [KYV-E15].

## Verified with

- The pinned kyverno CLI v1.20.0, offline: the three results above (skip, fail, fail).
- Not verified: expiry behavior (an expired exception is still applied offline), the enablement flag, exception namespace, the webhook, the controller, authorized creators and populations.

## Adapt it

1. Replace every `TEMPLATE` marker and the policy under exception.
2. Keep each field explicit: exact `policyRefs` name and kind, an exact-workload match condition, and an approved `expiresAt`.
3. Keep one approved case with the control, one approved workload without it, and one adjacent workload without an exception.
4. Rerun the kit with the pinned CLI, then confirm enablement and expiry in a controlled test or installed-config.

## Traps this kit avoids

- A `true` or namespace-only match condition exempts adjacent workloads; the adjacent case is in the kit to catch it.
- A date in an annotation is not removal; only `expiresAt` is honored [KYV-E14].
- Offline tests cannot show expiry; they still apply an expired exception [KYV-E14].
- Compensating validations that are not all met mean the exception is not granted [KYV-E16], and they block native policy generation [KYV-E26].
- `images` and `allowedValues` do nothing unless the policy reads them [KYV-E50]; this kit does not use them.
