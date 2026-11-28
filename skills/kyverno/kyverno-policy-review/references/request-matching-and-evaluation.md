# Which requests and stored objects does this rule actually see, and when is it evaluated?

## Question

For the operation the control cares about (create, update, delete, subresource write), does the rule receive the right object, the old object where needed, and the requester identity, and is it evaluated at admission, in background scans, or both?

## Release scope

Field names and defaults here are for ValidatingPolicy `policies.kyverno.io/v1` at v1.20.0 [KYV-E03] [KYV-E05]. Legacy ClusterPolicy is not installable from v1.20.0 [KYV-E01]; on earlier releases it has its own settings [KYV-E28] and this reference does not map them field by field. Confirm the installed release before relying on a default.

## Assumptions

- CREATE, UPDATE and DELETE are the usual admission operations; on UPDATE the request carries the new object and the stored one, and on DELETE the new object is null.
- A subresource (such as status or scale) is a different request target from its parent resource and needs to be matched explicitly in the rule's resource list.
- The requester can name the one operation the control must catch.

## Evidence to request

- `manifest`: `matchConstraints.resourceRules` (apiGroups, apiVersions, operations, resources including any subresource entries) and `matchConditions`.
- `manifest`: `spec.evaluation.admission.enabled` and `spec.evaluation.background.enabled` as authored.
- `manifest`: every CEL expression, noting use of `object`, `oldObject` and request identity variables.
- `offline-test`: `kyverno test` output with the CLI version and the `operation` used.
- `population-report`: PolicyReports with an inventory, if existing resources are in question.
- `admission-trace`: a sample admission response for the operation that matters, if available.

## Interpretation

1. List the operations in `matchConstraints`. A rule listing only CREATE never sees an UPDATE that introduces the violation; a rule listing UPDATE must say what it compares.
2. On UPDATE, a rule that must detect a change needs both `object` and `oldObject`. On DELETE the new object is null, so an expression written for `object` fails or is skipped differently there. Do not assume one expression is valid for every listed operation.
3. Match subresources deliberately. If the prohibited write arrives through a subresource, confirm the resource list includes it.
4. Request identity variables, such as the user name, exist only in the admission request. A rule that uses them requires background evaluation to be false [KYV-E05].
5. Admission and background evaluation both default to true [KYV-E05]. A policy with both enabled is evaluated at admission and against existing resources; the two produce different kinds of evidence.
6. Existing resources that predate a Deny policy appear as background findings in reports and are never rejected retroactively [KYV-E09]. A Deny policy therefore does not clean up the cluster, and a report full of failures for old objects is a backlog, not a rejection record [KYV-E09].
7. Report results include pass, fail, warn and error; an error means an evaluation error prevented a normal result and is not the same as a violation [KYV-E10].
8. Observed with the pinned kyverno CLI v1.20.0 in this repository's offline fixtures: an object excluded by `matchConditions` reports `skip`, not `pass`. Read a skip as "out of scope", and check that the exclusion is intentional.

## Worked example (synthetic)

**Requester supplied**: The `payments-run-as-non-root` ValidatingPolicy (Deny, CREATE and UPDATE on pods, `matchConditions` limiting it to namespace `payments`) and three pods, from fixture 01.

**Agent derived**: Decision table observed with the pinned kyverno CLI v1.20.0 in this repository's offline fixtures:

| Object | Namespace | Why | Offline result |
| --- | --- | --- | --- |
| invoice-retry-worker | payments | Every container sets `runAsNonRoot: true`; the legitimate operation | pass |
| invoice-batch-root | payments | Container has no `securityContext.runAsNonRoot`; the in-scope violation | fail |
| catalog-root | catalog | Excluded by the `payments-namespace-only` condition; adjacent namespace | skip |

**Still needed**: `installed-config` confirming the same action and conditions are installed, and `population-report` if pods already running as root in `payments` matter; those are findings, not rejections [KYV-E09].

**Decision today**: `conditional` / `needs-review` for admission behavior (offline results do not show the webhook decision); the rule logic itself is `supported` for the three tested objects.

**Cheapest discriminating check**: Add a pod in `payments` with one compliant and one non-compliant container, to confirm `all(...)` fails on any single container.

**Developer-facing denial message**: "payments-run-as-non-root: set securityContext.runAsNonRoot=true on every container. Escalate to the payments platform owner if the image cannot run as non-root." It names the field, the scope (every container) and the owner.

**Escalation owner**: payments platform owner, for images that cannot run as non-root.

**Keep out of evidence**: real image digests and any customer identifiers in pod specs.

## Counterexample

A control that matters only on UPDATE (fixture 04: the `billing.example/tier` label on a Deployment must not change) passes offline for the wrong reason. In the kyverno CLI test file, `operation: UPDATE` sets both `object` and `oldObject` to the same resource [KYV-E24], so a comparison of the two is trivially equal. Observed with the pinned kyverno CLI v1.20.0 in this repository's offline fixtures: the test returns `pass` even though a real tier change would not be representable. The pass says nothing about an actual change; admission evidence or a controlled test with distinct old and new objects is needed.

## Escalation threshold

Stop when the control depends on UPDATE or DELETE semantics and only offline results exist; request a `controlled-test` or `admission-trace`. Send webhook availability questions to `kyverno-admission-reliability-agent`. A change to the policy goes to `kubernetes-live-admission-policy-guard-agent`.

## What would change the verdict

- An `admission-trace` showing the UPDATE that changes the label being denied (or admitted).
- A `controlled-test` with distinct `object` and `oldObject`.
- `installed-config` showing different operations, conditions or evaluation toggles than the manifest.
- A `population-report` showing the backlog of existing violations and an owner for it [KYV-E09].
