# PolicyException starter kit

A PolicyException for one legacy workload of the synthetic invoice service in namespace `payments`, with a compensating control, tested against the policy it names. Files: `policy.yaml`, `exception.yaml`, `resources.yaml`, `kyverno-test.yaml`, and `missing-compensating-control/` (resources and a second test).

## What it demonstrates

- Exactly one approved workload (`legacy-invoice-exporter`) matched by name and namespace, never a whole namespace; `policyRefs` carries the exact policy name and kind [KYV-E13] [KYV-E14].
- A compensating validation (label `network-isolation=strict`): with it the approved workload reports `skip`; without it the same workload reports `fail` (the `missing-compensating-control/` sub-test); an adjacent workload (`invoice-batch-root`) with no exception reports `fail` [KYV-E16].
- A widening detector, the fourth case: `invoice-batch-labelled` is in `payments`, carries `network-isolation=strict` and is not the approved workload, so it reports `fail`. Observed with the pinned CLI: widening the exception's match condition to namespace-only flips it to `skip`, so the kit detects widening. An unlabelled adjacent pod stays `fail` either way and cannot detect it.
- `expiresAt` set as the removal mechanism the controller honors [KYV-E13] [KYV-E14]; annotations for the approval record and expiry owner are inert governance data.
- The enablement caveat written in the file header: exceptions apply only when `enablePolicyException` is on and the namespace is allowed [KYV-E15].

## Verified with

- The pinned kyverno CLI v1.20.0, offline: the four results above: three in the main test (skip, fail, fail) and one in the sub-test (fail).
- Not verified: expiry behavior (an expired exception is still applied offline), the enablement flag, exception namespace, the webhook, the controller, authorized creators and populations.

## Adapt it

1. Replace every `TEMPLATE` marker and the policy under exception.
2. Keep each field explicit: exact `policyRefs` name and kind, an exact-workload match condition, and an approved `expiresAt`.
3. Keep one approved case with the control, one approved workload without it, one adjacent workload without an exception, and one adjacent workload that carries the control (the widening detector).
4. Rerun the kit with the pinned CLI, then confirm enablement and expiry in a controlled test or installed-config.

## Traps this kit avoids

- A `true` or namespace-only match condition exempts adjacent workloads; the labelled adjacent case (`invoice-batch-labelled`) is in the kit to catch it.
- A date in an annotation is not removal; only `expiresAt` is honored [KYV-E14].
- Offline tests cannot show expiry: observed in this repository's fixtures, an expired exception is still applied offline, while the controller's exception lookup skips expired ones [KYV-E14].
- Compensating validations that are not all met mean the exception is not granted [KYV-E16], and they block native policy generation [KYV-E26].
- `images` and `allowedValues` do nothing unless the policy reads them [KYV-E50]; this kit does not use them.
