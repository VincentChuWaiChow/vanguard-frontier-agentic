# Which parts of a mesh question can Kyverno decide, and which need Istio evidence?

## Question

When a Kyverno policy is asked to enforce or evidence a mesh property (for example that workloads use a waypoint), which part is a manifest-level decision here and which must go to Istio review?

## Release scope

The Kyverno side applies to any release where the policy kind is installed (see the policy semantics reference). The Istio statements here are from Istio ambient documentation cited per claim and are not tied to a Kyverno release [KYV-E29] [KYV-E30]. Mesh version and mode (ambient or not) are an input the requester must supply.

## Assumptions

- The mesh question is about ambient mode with ztunnel and, where used, a waypoint proxy.
- Kyverno sees admitted objects, not live mesh traffic.

## Evidence to request

- `manifest`: the Kyverno policy and the workload or namespace manifest carrying the `istio.io/use-waypoint` label.
- `installed-config`: the Istio resources for the waypoint and the `istio.io/WaypointBound` condition, supplied by the Istio owner.
- `manifest`: the Istio authorization policy and its attachment, if the question is about traffic access.
- `admission-trace`: a sample showing the label admitted or denied.

## Interpretation

1. Apply the two-claims rule. Claim 1: Kyverno can evaluate a declared mesh configuration, such as presence of the `istio.io/use-waypoint` label, within its policy scope. Claim 2: effective mesh traffic authorization needs Istio evidence for the destination, attachment, traffic type and actual enforcement point.
2. The label records intent only. If the named waypoint is missing or the traffic type does not match, ztunnel routes directly and the waypoint's L7 policy never takes effect [KYV-E30].
3. Binding status appears in the `istio.io/WaypointBound` condition [KYV-E30]. A label with no such condition is not evidence of enforcement.
4. In ambient mode, ztunnel enforces L4 policy and the waypoint is the L7 proxy; a client outside the mesh sends directly to the destination without traversing its waypoint [KYV-E29]. A Kyverno rule cannot change that path.
5. Never infer observed traffic from admitted manifests. Say what the manifest supports and what it does not.
6. If only manifests exist, both verdicts are limited: the Kyverno decision to label presence, and the mesh decision to "intent declared, enforcement unknown".

## Worked example (synthetic)

**Requester supplied**: A ValidatingPolicy requiring the `istio.io/use-waypoint` label on Services in namespace `payments`, and an invoice-api Service that carries the label.

**Agent derived**: The label passes admission, so the Kyverno claim (label presence within the policy scope) is supported by the manifest. The supplied evidence has no waypoint resource and no `istio.io/WaypointBound` condition, so effective L7 authorization is unestablished [KYV-E30].

**Still needed**: Istio-side evidence for the waypoint, its binding status, the traffic type and which clients are in the mesh [KYV-E29].

**Decision today**: Kyverno part `supported` / `approved` for label presence; mesh part `indeterminate` / `needs-review`, referred to `istio-maestro-agent` with the decision ID and one return question: "Is the waypoint named by the label present and bound, and does the traffic type match?"

**Cheapest discriminating check**: Supplied `installed-config` showing the `istio.io/WaypointBound` condition on the invoice-api Service.

**Developer-facing denial message**: "payments-waypoint-label: add the istio.io/use-waypoint label to this Service. Escalate to the mesh owner if no waypoint exists for the namespace." It tells the developer what to add and whom to ask; it does not claim the traffic is authorized.

**Escalation owner**: mesh platform owner.

**Keep out of evidence**: service account identities and tenant identifiers.

## Counterexample

The label is present, admission passes, and a report says "payments Services use the waypoint". A client outside the mesh calls the Service directly and no waypoint policy applies [KYV-E29]; or the named waypoint was deleted and ztunnel now routes direct [KYV-E30]. The Kyverno result was true and the mesh conclusion was false.

## Escalation threshold

Hand to `istio-maestro-agent` for any statement about effective traffic authorization, with the decision ID and one return question. Do not wait for the answer to publish the Kyverno part; label it as limited.

## What would change the verdict

- The `istio.io/WaypointBound` condition and the waypoint resource from the Istio owner [KYV-E30].
- Evidence of which clients are in or out of the mesh [KYV-E29].
- An Istio-side review of the authorization policy attachment and traffic type.
