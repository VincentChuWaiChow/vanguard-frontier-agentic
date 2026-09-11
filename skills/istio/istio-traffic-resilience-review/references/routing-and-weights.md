# Routing and weights

## TRAFFIC-01: proportional weights

Istio route destinations receive traffic in proportion to their weights relative to the sum. `9:1` and `90:10` express the same proportions. A sum of 100 can be a team convention, not a mandatory semantic rule. Do not claim the unallocated remainder is dropped.

Reject or investigate negative values, unusable totals and unsupported field forms according to the actual API. Do not transfer default rules between Istio route weights and Gateway API backend weights without checking their schemas.

## TRAFFIC-02: route applicability

Resolve host, port, protocol, gateway/mesh scope, route order, matches and rewrites. Determine whether the named subset or Service actually selects intended endpoints. Separate route validity, configured proportions and observed distribution. A handful of requests or long-lived sessions does not prove a stable weighted distribution.

Check whether fallbacks unintentionally capture traffic that a preceding rule misses. Preserve explicit tenant/host boundaries across rewrites and header changes. Escalate authentication implications to the authorization owner rather than treating routing headers as trusted identity.

## TRAFFIC-03: mode-sensitive backends

For waypoint/HTTPRoute migrations, inspect Service-backed version separation and the supported DestinationRule policy fields. Do not mechanically translate every subset name into an HTTPRoute backend without defining and validating the corresponding Service.

Sources: VS, MIGRATE, NETWORK. Tests: TRAFFIC-01A, TRAFFIC-01B, TRAFFIC-02, TRAFFIC-03.
