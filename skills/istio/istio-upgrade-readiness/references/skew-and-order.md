# Skew and order

## UPGRADE-02: direction matters

The reviewed Istio support policy allows the control plane one minor version ahead of its data plane and does not allow the data plane ahead of its connected control plane. Check each actual relationship, not an absolute difference. Same-minor numeric comparison is not complete compatibility evidence; patch/build details and specific release limitations still need review.

Do not label end-of-life versions supported just because a numeric skew check passes. Do not extrapolate this rule to unrelated Istio components without their installation guidance.

## UPGRADE-03: sequence follows dependencies

For the reviewed sidecar Helm procedure, base/shared resources precede control-plane and gateway steps. Do not assert a universal east-west-gateways-first order. Derive an explicit dependency graph for ambient components and multi-cluster topology from the applicable versioned procedure.

## UPGRADE-04: in-place scope

The documented istioctl in-place procedure can cover a one-minor transition under its prerequisites; it is not patch-only. Installation with revisions or a different owner requires its corresponding method. Review preserved profile/values so defaults do not replace existing customizations.

For every step specify prior evidence, change owner, scope, postcondition, stop criterion and permissible rollback. This document is a readiness plan, not an execution permission.

Sources: SKEW, HELM, INPLACE. Tests: UPGRADE-02A, UPGRADE-02B, UPGRADE-03, UPGRADE-04.
