# Attachment graph

## GATEWAY-01: fully qualify kinds

Distinguish `networking.istio.io/... Gateway` from `gateway.networking.k8s.io/... Gateway`. Record installed Gateway API CRD bundle/channel and the actual controller implementation. Similar names do not imply interchangeable fields or status semantics.

Identify ingress Gateways separately from waypoints. For mesh HTTPRoute cases, branch for Service parent references rather than forcing every route through an ingress listener graph. Confirm that the exact implementation supports the chosen attachment.

## GATEWAY-02: permission belongs to the relationship

Resolve Route-to-Gateway attachment with parent references and listener allowed route controls. Cross-namespace backend/certificate references use the applicable ReferenceGrant rules. Do not use a ReferenceGrant as a substitute for a Gateway listener admitting routes from a namespace.

For a grant, inspect the reference's source group/kind/namespace and destination group/kind/name scope, where the grant lives, and whether the target actually exists. Avoid gratuitous wildcard grants. Follow actual API defaults rather than treating omitted and empty fields as interchangeable.

## Evidence worksheet

For every edge record source, target, namespace, API group/kind, port/sectionName, controller, permission mechanism, known status, and supporting evidence. Cross-resource errors may be asynchronous; API admission alone is not full graph validation.

Sources: GRANT, GDESIGN, MIGRATE, WAYPOINT. Tests: GATEWAY-01, GATEWAY-02A, GATEWAY-02B.
