# Protocol and normalization boundaries

## AUTH-07: HTTP attributes on TCP

In native DENY evaluation, unavailable HTTP attributes on TCP can match rather than safely narrow a rule. Check port scope and effective protocol before recommending a method/path DENY. Do not misapply this rule to all policy actions or all waypoint attachment shapes.

Counterexample: a DENY method match without a port boundary on a workload serving TCP as well as HTTP. Determine the affected transport before recommending a correction. Test: AUTH-07.

## AUTH-08: request interpretation

Compare the proxy and application interpretations of paths, encoded characters, duplicate slashes, rewrites, case and host/port handling. Derive tests from the actual normalization settings, API version and application router. Do not provide a universal bypass payload list as evidence that a particular deployment is vulnerable.

Resolve whether source IP means the immediate peer or an address derived from trusted proxy headers. Require evidence for the trusted forwarding chain. User-controlled headers are not authenticated workload identity by themselves.

## Exceptions

Document health checks, metrics, probes, webhooks and infrastructure access as specific intended flows. Do not add an unrestricted path/namespace exception merely to make a failing test pass. Each exception needs an owner and both permitted and prohibited cases.

Sources: AUTHZ and SECURITY. Test: AUTH-08.
