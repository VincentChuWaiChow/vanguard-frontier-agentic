# Routing ownership

| Decision | Skill ID | Evidence boundary |
| --- | --- | --- |
| Enrollment, proxy path, enforcement location, migration | istio-ambient-mesh-review | Workload/path map |
| Combined access semantics and identities | istio-authorization-policy-review | Intended access matrix and applicable policy set |
| Route behavior, retries, capacity, mirroring, experiments | istio-traffic-resilience-review | Service objective and traffic policy |
| Gateway/Route attachment, cross-namespace references, status | istio-gateway-api-review | Qualified resource graph and controller status |
| Source-to-target transition and rollback readiness | istio-upgrade-readiness | Component/version inventory and release-specific evidence |
| Failure origin and next discriminating observation | istio-dataplane-diagnostics | Timeline and supplied signals |

Run only the owners needed for the decision. Load their companions through the host's resolver; do not assume filesystem sibling paths are present in standalone exports. When an owner is unavailable, report that handoff gap and do not pretend it ran.

For ambient authorization start with path/attachment, then access semantics. For upgrade plus migration establish version/mode prerequisites before feature conclusions. For incidents use diagnostics first, then a specialist only when evidence identifies its decision.

Architecture justification, multicluster trust, egress containment and extension safety are deferred specialist areas. Route to a qualified existing owner or mark the review boundary. Do not present the backlog as implemented coverage.

Tests: ROUTE-01, ROUTE-02, ROUTE-03.
