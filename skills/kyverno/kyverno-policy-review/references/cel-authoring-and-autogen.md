# Which CEL authoring choices change what a ValidatingPolicy evaluates, and how do you prove them offline?

## Question

Which fields and expressions in a ValidatingPolicy decide what is evaluated (messages, audit annotations, variables, autogen, native policy generation, CEL libraries and live lookups), and which of those can an offline `kyverno test` actually show?

## Release scope

- ValidatingPolicy and ImageValidatingPolicy `policies.kyverno.io/v1` at v1.20.0; the CRDs also serve v1beta1 (storage) and v1alpha1 (deprecated) [KYV-E03]. Legacy ClusterPolicy is not installable or testable from v1.20.0 [KYV-E01]; before v1.20 it has its own settings [KYV-E28] and is not mapped here.
- Offline observations below were made with the pinned kyverno CLI v1.20.0 in this repository's starter kits. They are not controller behavior.

## Assumptions

- The requester supplies the policy as authored, not a rendered or installed copy, unless stated.
- Defaults quoted here are release-specific; confirm them against the installed release.
- An offline result applies only to the objects in the test file; objects not listed have no evidence.
- Live lookups are tested offline only through the Test manifest's mock inputs [KYV-E52]; a compiled lookup without a mock proves syntax, not behavior.

## Evidence to request

- `manifest`: `spec.validations[]` (expression, message, messageExpression, reason), `spec.auditAnnotations`, `spec.variables`, `spec.autogen`.
- `manifest`: every CEL expression, noting any call to `resource.Get`, `resource.List`, `http.Get`, `http.Post` or `globalContext.Get`.
- `offline-test`: `kyverno test` output with the CLI version, including a result for each pod controller the control must cover.
- `installed-config`: whether the policy was installed with autogen and native generation settings as authored.
- `controlled-test`: a disposable-cluster run for any rule that depends on live data.

## Interpretation

1. A validation carries expression, message, messageExpression and reason; messageExpression takes precedence over message and falls back to it when it fails or yields an empty, whitespace-only or multi-line string [KYV-E48]. Always keep a static `message` as the safe fallback, and judge the denial text a developer actually sees from whichever of the two the evaluation produces.
2. auditAnnotations attach key-value data to PolicyReport results [KYV-E48]. Use them to carry the owner or remediation hint into reports; they do not change the allow or deny decision.
3. Variables are named CEL expressions reused by validations; an expression that reads a variable is only as correct as the variable. Review each variable for the field it reads and for optional-field handling, because an unset field read without a guard is an evaluation error, which `failurePolicy` governs [KYV-E04].
4. Autogen: `autogen.podControllers.enabled` defaults to true and `controllers` defaults to all supported pod controllers [KYV-E37]. A Pod rule therefore also covers controllers that embed a pod template unless the author narrows it.
5. Autogen resolves controller names through a built-in table keyed by lowercase plural resource names (daemonsets, deployments, replicasets, statefulsets, jobs, cronjobs); other bare names resolve only through a cluster RESTMapper, which an offline CLI run does not have [KYV-E38]. The CRD field description lists Kind names, which conflicts with those keys [KYV-E39] [KYV-E38]; use the plural resource names, as the resolver does. Observed with the pinned kyverno CLI v1.20.0 in this repository's starter kits: with plural names a Deployment is evaluated offline; with Kind names the CLI produced no result for the Deployment at all.
6. Native ValidatingAdmissionPolicy generation is a per-policy opt-in: `autogen.validatingAdmissionPolicy.enabled` defaults to false [KYV-E37]. The generator creates a native policy only when that is enabled, skips generation when pod-controller autogen produces configs, and is blocked by an exception with compensating validations [KYV-E40] [KYV-E26]. A native policy needs a binding to take effect [KYV-E27]. Do not read "the controller toggle defaults to true" as "a native policy exists" [KYV-E26].
7. CEL libraries include `resource.Get`, `resource.List`, `http.Get`, `http.Post`, `globalContext.Get`, `parseServiceAccount`, `time.now` and an `image()` type read with methods such as `tag()`, `digest()` and `identifier()` [KYV-E49]. Observed with the pinned CLI: the CLI compiles every CEL expression when it loads the policy even if no object matches; undeclared functions fail, and `image(x).registry` (field style) fails while `image(x).registry()` compiles. `resource.Get` and `resource.get` both compile at v1.20.0.
8. A rule that calls `resource.*`, `http.*` or `globalContext.*` depends on live data [KYV-E49]. Offline tests must supply that data as mock input: the Test manifest accepts `context`, `clusterResources`, `apiCallResponses` and `globalContextEntries`, plus `variables` and `userinfo` for request data [KYV-E52]. An unmocked lookup is not a test of the rule. A compile success proves syntax, not the data a cluster would return.
9. Offline `pass`, `fail` and `skip` show the rule logic for the supplied objects only. The admission effect of Audit versus Deny is not visible offline: both gave the same `fail` in this repository's fixtures.

## Worked example (synthetic)

**Requester supplied**: the validating-policy-starter kit for the `payments` invoice service: policy `example-require-run-as-non-root`, `autogen.podControllers.controllers: [deployments, statefulsets, daemonsets, jobs, cronjobs]`, `autogen.validatingAdmissionPolicy.enabled: false`, and four test results.

**Agent derived**: Observed with the pinned kyverno CLI v1.20.0 in this repository's starter kits: `legitimate-workload` passes, `violating-workload` fails, the `catalog` Pod is skipped (outside the namespace condition), and the Deployment `violating-deployment` fails through the autogenerated rule. The Deployment result exists only because the controller names are lowercase plural resource names [KYV-E38]. Native generation is off, so no native policy is expected [KYV-E37] [KYV-E40].

**Still needed**: `installed-config` showing the installed autogen block matches; `controlled-test` for any live-lookup rule (none in this kit).

**Decision today**: `conditional` / `needs-review`: rule logic is `supported` for the four tested objects; admission effect is not shown offline.

**Cheapest discriminating check**: switch the controller names to Kind names and confirm the Deployment result disappears (no result offline); an absent result is not a pass, and it shows the plural names are what pins the coverage.

**Developer-facing denial message**: "example-require-run-as-non-root: set securityContext.runAsNonRoot=true on every container and init container. If the image cannot run as non-root, ask the payments platform owner for a time-bound PolicyException." It names the field, the scope and the owner.

**Escalation owner**: payments platform owner.

**Keep out of evidence**: real image digests, registry credentials and any customer identifiers in pod templates.

## Counterexample

A policy sets `controllers: [Deployment, StatefulSet]` because the CRD description lists Kind names [KYV-E39]. The Pod test passes and the author concludes controllers are covered. Observed with the pinned CLI in this repository's starter kits, Kind-named controllers produced no result for the Deployment at all, so the Deployment is untested offline; the resolver keys are plural resource names [KYV-E38]. An absent result is not a pass.

## Escalation threshold

Hand to `kubernetes-live-admission-policy-guard-agent` for any installed-policy change. Hand to `kyverno-admission-reliability-agent` when the question is evaluation timeouts or error behavior. Stop and ask for a `controlled-test` when a rule depends on resource, HTTP or global-context data.

## What would change the verdict

- `installed-config` showing different autogen controllers or native generation settings than the manifest [KYV-E37].
- An offline result for each required pod controller using plural resource names [KYV-E38].
- A `controlled-test` showing the live lookup returns the data the rule assumes [KYV-E49].
- Mock input field names confirmed against the pinned CLI, with a test that uses them.
