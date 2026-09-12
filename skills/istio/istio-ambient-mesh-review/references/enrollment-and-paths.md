# Enrollment and traffic-path review

## AMB-01: prove workload participation

Applicable to ambient or mixed-mode reviews. Inputs: namespace and workload labels/annotations, actual pod composition, revision, CNI/capture evidence, Services and EndpointSlices where supplied. Build one row per source/destination workload. Keep intended enrollment, observed capture and actual request path in separate columns.

A namespace label is insufficient evidence for a specific pod. Inspect exclusions, sidecar presence, host networking, rollout state and partial enrollment. Do not infer all connections traverse the same path. Compare an enrolled source, an unmeshed source, and each gateway entry path within scope.

Counterexample: a labeled namespace with a pod whose traffic is not captured. Expected result: enrollment remains unestablished, not approved. Test: AMB-01.

## AMB-02: resolve waypoint selection

Identify the relevant Service/workload, applicable waypoint selection, the waypoint's supported traffic type and actual route to it. Distinguish Service-addressed traffic from direct workload-IP traffic. Distinguish an ingress gateway path from an in-mesh path. Verify exact-release ingress/waypoint behavior rather than assuming an L7 ingress always traverses a destination waypoint.

A waypoint resource, a Ready pod, and a namespace label are three different observations. None alone proves the tested request passed through the intended waypoint. Missing attachment/path evidence is `needs-review`, not evidence of an actual bypass.

Counterexample: a Service-scoped waypoint and an unexamined direct-IP path. Request evidence for the second path. Test: AMB-02.

## Path worksheet

Record: source principal; source mode/revision; requested host/IP and port; selected Service/backend; waypoint identity and type; receiving ztunnel identity observation; workload listener; relevant policy IDs; supporting evidence IDs. Mark each edge observed, derived or unknown.

## Boundary handoffs

Send policy-set access semantics to authorization review. Send Service/Route parent attachment to Gateway API review. Send cross-cluster trust or outbound escape claims to an explicit architecture/security review, not a guessed answer.

Sources: WAYPOINT, L4, SECURITY in official-sources.md.
