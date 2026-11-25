# What can the pinned kyverno CLI prove for a migration corpus, and where does it mislead?

## Question

For each offline observation the pinned kyverno CLI v1.20.0 produces on a migration corpus, what does it prove, what can it not prove, and what evidence replaces it?

## Release scope

- The CLI compares results declared ahead of time with actual results [KYV-E25]. Pin the CLI version to the target release and record it with every result.
- CLI v1.20.0 rejects legacy manifests in `apply` and `test` [KYV-E01]; legacy parity needs an older CLI whose version is named, and how it models UPDATE is unverified here. The migration guide's wording lags the removal [KYV-E02] [KYV-E01].
- Result type flags per kind are `isValidatingPolicy`, `isMutatingPolicy`, `isImageValidatingPolicy`, `isGeneratingPolicy`, `isDeletingPolicy` and `isValidatingAdmissionPolicy`; mutation and generation compare `patchedResources`, `generatedResource(s)` or `cloneSourceResource` [KYV-E46].
- Observations below were made with the pinned kyverno CLI v1.20.0 in this repository's offline fixtures and starter kits; none is controller behavior.

## Assumptions

- The corpus holds a conforming, a violating and an adjacent out-of-scope object per control.
- Every corpus row names its operation; an unset operation defaults to CREATE [KYV-E24].
- The CLI version, the policy revision and the corpus revision are recorded together.

## Evidence to request

- `offline-test`: the kyverno-test file, the corpus, and the full output including the REASON column, for source and target.
- `controlled-test`: real old/new object pairs for UPDATE-sensitive controls.
- `installed-config`: the exception objects and the webhook configuration for the target.
- `admission-trace` and `population-report` for behavior the CLI cannot show.

## Interpretation

1. Read each row of the table before accepting a green corpus. "Replaces" names the evidence that settles what the offline run cannot.

| Behavior | What it proves | What it cannot prove | What evidence replaces it |
| --- | --- | --- | --- |
| Audit and Deny give the same result (`fail`) for one violation (fixture 02) | The expression evaluates the violation | The admission effect of Deny, or that Audit does not block | `admission-trace` per stage |
| Object excluded by matchConditions reports `skip` (fixture 01) | The scope expression excludes that object | That a different out-of-scope object is also excluded | An adjacent case per cohort |
| Expired exception still applied, `skip` (fixture 03) | The exception matches the policy by name | Expiry; the v1.20.0 controller lookup skips expired exceptions [KYV-E14] | Post-expiry `admission-trace` |
| UPDATE uses identical old and new objects (fixture 04) | The expression tolerates UPDATE [KYV-E24] | That a real change is caught by an `oldObject` comparison | `controlled-test` with real pairs |
| DELETE sets object to null [KYV-E24] | A DELETE row evaluates | Behavior on the real deleted object | `controlled-test` |
| The autogen-name observation: autogen controllers named by lowercase plural resource name evaluate a Deployment; Kind names produced no result for it | A Deployment path exists when plural names are used [KYV-E38] | Behavior on a cluster, where a RESTMapper resolves other names; the CRD description gives Kind names, which conflicts with the resolver [KYV-E39] | Use the plural names [KYV-E38]; `controlled-test` |
| The load-time compile observation: every CEL expression compiles at load, even with no matching object; field-style `image(x).registry` fails, `image(x).registry()` compiles; hyphenated attestor names fail | The policy is syntactically and type valid for this CLI | That the expression is correct for real inputs | `offline-test` with in-scope objects |
| The image-policy pass observation: ImageValidatingPolicy reports `pass` for an image outside `matchImageReferences` | Nothing failed | That an image was verified, signed or pulled | `admission-trace`; registry-side evidence |
| The mutating-patch observation: MutatingPolicy compares `patchedResources`; a wrong patch fails with "Resource diff" | The computed patch for that input | Background or existing-object mutation | `controlled-test` |
| The generating-test observation: GeneratingPolicy compares `generatedResource`; an out-of-scope trigger passes with reason "Excluded" whatever result is expected | The generated object for an in-scope trigger | Synchronization, generateExisting, deletion behavior; anything for an excluded trigger | In-scope case; `controlled-test` |
| The exception-with-compensating-validations observation: labelled approved workload `skip`, unlabelled `fail`, adjacent workload `fail`, adjacent labelled workload `fail` (the widening detector) | The exception and its compensating check evaluate as designed [KYV-E16] | Controller exception lookup, namespace flags [KYV-E15], expiry | `installed-config`; `admission-trace` |
| The resource-library spelling observation: `resource.Get` and `resource.get` both compile | Both spellings load at v1.20.0 | That either returns data; inference: the library needs a cluster (KYV-E49 lists the functions only, so confirm with a `controlled-test`) [KYV-E49] | `controlled-test` |

2. `kyverno test` agrees declared and actual results [KYV-E25]; a wrong declaration agreed with a wrong result is still green. Review the declarations against the control's intent.
3. A parity difference on a required control is the finding. Name the item, operation, source result, target result and the evidence still missing.
4. Do not extend an offline observation to a version other than the pinned one.

## Worked example (synthetic)

**Requester supplied**: the parity-corpus-starter for `payments`: six rows for `example-require-run-as-non-root`: `legitimate-workload` and `violating-workload` Pods, each with a CREATE and an UPDATE row; the adjacent `catalog/adjacent-workload` (expected `skip`); and the `payments/violating-deployment` Deployment (expected `fail` through autogen with lowercase plural controller names). All six rows match their declared results in the pinned CLI.

**Agent derived**: every row names its operation, which is the minimum; the kit also pins the adjacent skip and the autogen Deployment. The control reads only `object`, so the UPDATE rows prove the expression tolerates UPDATE and reproduce the CREATE results [KYV-E24]. For a control that compared `oldObject`, the same four green rows would prove less.

**Still needed**: the source engine's results on its own pinned release for the same corpus, and a real old/new pair for any `oldObject`-comparing control.

**Decision today**: `conditional`, verdict `needs-review`; CREATE parity holds for the corpus inputs, and nothing about the admission effect, webhook or population is shown.

**Cheapest discriminating check**: a `controlled-test` changing a field on `payments/legitimate-workload` in a disposable cluster, on both policy families.

**Developer-facing denial message**: `example-require-run-as-non-root: set securityContext.runAsNonRoot=true on every container and init container. If the image cannot run as non-root, ask the payments platform owner for a time-bound PolicyException.` It names the field to fix and who to ask.

**Escalation owner**: payments platform owner for the control; platform release manager for the cohort.

**Keep out of evidence**: customer image digests, tenant identifiers, exception justifications naming people.

## Counterexample

A corpus is green and the cohort is approved. Its UPDATE rows are all for controls comparing `oldObject`, and offline UPDATE sets old and new to the same object [KYV-E24], so every such row passes trivially. Green described the harness, not the migration.

## Escalation threshold

Hand to `kyverno-policy-review-agent` when a difference traces to rule semantics, to `kyverno-admission-reliability-agent` for webhook behavior, and to the platform release manager for any cohort decision.

## What would change the verdict

- Real old/new pairs showing agreement on UPDATE-sensitive controls.
- Source and target results on named, pinned CLI versions with a diff per operation.
- DELETE coverage where the control applies to deletion.
- An unexplained difference on a required control: `blocked`.
