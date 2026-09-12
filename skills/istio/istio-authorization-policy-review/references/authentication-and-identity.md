# Authentication and identity

## AUTH-04: validation does not require credentials

RequestAuthentication can reject invalid supplied credentials while accepting a request with no credential and no authenticated request identity. Requiring authenticated requests needs appropriate authorization. Review missing credentials, invalid credentials, wrong issuer/audience/claims and a valid permitted identity separately.

## AUTH-05: wildcard presence and identity kinds

A `principals: ["*"]` condition requires a nonempty authenticated peer identity. It is not an anonymous allow-all condition. Peer principals and request principals are different inputs; do not substitute one for the other. A valid identity can still be over-broad relative to intended tenant/service access.

## AUTH-06: transport is not authorization

Evaluate effective PeerAuthentication inheritance and supported overrides for the actual mode. Port-specific settings refer to workload ports, not automatically the Service port. STRICT transport does not supply an application allowlist. PERMISSIVE admits both transport forms and is not synonymous with no encrypted traffic.

When root-namespace selector behavior or conflicting precedence is unclear in documentation, report the exact conflict and require the version-specific resolution; do not silently choose the most convenient reading.

## Identity threat questions

Identify who can create pods using permitted service accounts, who owns namespace labels, and whether the intended boundary is per namespace, service account, tenant, or JWT claims. Treat a shared identity across tenants as a design decision requiring explicit approval, not automatic isolation.

Sources: JWT, AUTHZ, PEER, SECURITY. Tests: AUTH-04, AUTH-05, AUTH-06.
