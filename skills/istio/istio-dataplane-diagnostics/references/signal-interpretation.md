# Signal interpretation

## DIAG-03: distinguish response origin

An HTTP 403 may come from a proxy policy, external authorizer or application; a 503 may reflect upstream connectivity, capacity, TLS or configuration. Use supplied response details/flags, trace or request identifiers and relevant data-plane configuration to narrow the origin. Do not conclude root cause from the numeric status alone.

For example, an upstream connection failure flag motivates transport/TLS/endpoint hypotheses; it is not by itself proof that PeerAuthentication caused the failure. Confirm the response flag definition for the running proxy version.

## DIAG-04: configuration versus propagation

Compare desired objects, controller status, accepted/distributed proxy configuration and the proxy that actually handled the request. Include connected revision and observation time. Ready processes and an admitted manifest do not prove the expected route/policy is active for this request.

For ambient, map waypoint/ztunnel roles explicitly and use the ambient reviewer for attachment uncertainty. For Gateway API, inspect the correct parent/controller and generation. For application failures, retain the application owner handoff.

Sources: NETWORK, L4, GSTATUS. Tests: DIAG-03, DIAG-04.
