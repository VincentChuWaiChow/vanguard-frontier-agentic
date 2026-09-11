# Migration boundaries

## AMB-06: compare three states

Produce a before/during/after matrix for identities, allowed/denied requests, enrollment, attachment, route ownership and extension behavior. Include partial rollouts, existing connections, startup/probe access and rollback traffic. Do not treat migration as a namespace-label edit.

The reviewed migration guidance identifies an L7 enforcement gap during its documented transition. Where uninterrupted L7 enforcement is required, do not approve that transition without a target-release-supported procedure and explicit operating constraints. A maintenance window alone is not proof that traffic is blocked or access remains protected.

## AMB-07: API and extension inventory

Check each used feature against the exact target mode/release. The reviewed guidance recommends HTTPRoute for waypoints, treats waypoint VirtualService support as Alpha, and rejects mixing those APIs for the same workload. It distinguishes supported DestinationRule traffic policies from subset-based routing expressed through separate Service backends in HTTPRoute.

Inventory EnvoyFilter, WasmPlugin, RequestAuthentication, CUSTOM/AUDIT behavior, telemetry customizations and non-HTTP dependencies. Do not assume a sidecar extension will survive a waypoint transition. Require either an evidenced equivalent, approved retirement, or a migration blocker.

## Stop conditions

Block when a confirmed required feature has no supported replacement. Mark unknown target support, partial inventory, or untested rollback as needs-review. Propose an isolated verification environment; never create it from this skill.

Sources: MIGRATE and WAYPOINT in official-sources.md. Tests: AMB-06, AMB-07.
