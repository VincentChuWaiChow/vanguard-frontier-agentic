# 🕸️ Istio Skills

<p align="center">
  <!-- 🖼️ Add an Istio logo to assets/logos/cnative/istio/ and update this path -->
  <span style="font-size:3.5em">🕸️</span>
</p>

This folder contains Istio-focused skills curated for this marketplace.

## Local marketplace portfolio

The local portfolio separates six review decisions, routing, and guarded live
execution:

- `istio-ambient-mesh-review`
- `istio-authorization-policy-review`
- `istio-dataplane-diagnostics`
- `istio-gateway-api-review`
- `istio-review-routing`
- `istio-traffic-resilience-review`
- `istio-upgrade-readiness`
- `istio-live-policy-change`

## Portfolio posture

Istio skills for evidence-backed service mesh review covering both **sidecar mode** and **ambient mode** (ztunnel + optional waypoint proxies). Ambient mode introduces a layered architecture where L4 zero-trust is enforced at ztunnel and L7 features require an explicit waypoint deployment.

These skills are intentionally conservative:

- review supplied, sanitized evidence by default; route separately authorized
  collection or mutation through the appropriate guarded runtime agent
- resolve the actual ambient traffic path and enforcement target before judging
  L7 policy behavior; do not reduce every missing-waypoint case to silent bypass
- treat `PeerAuthentication` `PERMISSIVE` as accepting both plaintext and mTLS,
  then evaluate the effective policy and path rather than claiming it disables all mTLS
- challenge mesh-wide `PeerAuthentication` changes — the blast radius is the whole mesh
- use official Istio documentation (istio.io) for ambient architecture, ztunnel internals, waypoint placement, HBONE protocol, and `AuthorizationPolicy` semantic differences between sidecar and ambient modes

Run `npm run validate` after changing cataloged Istio skills.
