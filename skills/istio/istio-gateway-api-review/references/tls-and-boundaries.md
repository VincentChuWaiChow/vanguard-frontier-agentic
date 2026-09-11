# TLS and exposure boundaries

## GATEWAY-05: keep TLS stages distinct

Record client-to-gateway TLS, termination versus passthrough, gateway-to-backend TLS and mesh transport separately. Confirm the hostname, SNI, listener, backend protocol and certificate identity relevant to each stage. Do not equate a public certificate with authenticated workload identity or complete backend TLS validation.

Review certificate references through sanitized names, SAN/expiry metadata and applicable cross-namespace permission; do not request private keys or full Secret contents. Include rotation overlap, selected credential, and validity at an explicit review time.

## Exposure checks

Check whether wildcard host/listener scope exposes more than the user intended. Keep health checks and infrastructure traffic explicitly scoped. Investigate hostname intersections, route collisions and fallback paths without inventing generic precedence across API families.

## Handoffs

Send application access semantics to authorization review, outbound trust to the appropriate security owner and certificate lifecycle to the existing certificate-management workflow. This skill does not create, renew or expose credentials.

Source: GRANT plus target-release Gateway/implementation documentation requested by applicability rules. Test: GATEWAY-05.
