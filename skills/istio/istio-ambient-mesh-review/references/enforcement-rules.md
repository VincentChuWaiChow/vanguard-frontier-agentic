# Enforcement decision rules

## AMB-03: L7 rule at receiving ztunnel

Apply only after identifying a policy whose enforcement point is receiving ztunnel. Ztunnel does not interpret L7 conditions; the documented behavior for a policy containing those conditions at that enforcement point is fail-safe denial. Do not label this shape an automatic fail-open bypass. The effect of a waypoint-targeted policy on an unestablished path is a separate question.

Input: complete policy, target workload/mode, attachment and relevant target-release source. Output: wrong enforcement-point finding with access/availability consequences and a proposed test. Without attachment evidence, make the conclusion conditional. Test: AMB-03.

## AMB-04: identity after a waypoint

At destination ztunnel, waypoint-mediated connections carry the waypoint identity, not the original caller identity. Place original-caller reasoning at an enforcement point that has that identity. Separately review workload-level restrictions on reaching the destination without the intended waypoint. Do not silently replace a caller allowlist with a broad waypoint allowlist and call the access policy equivalent.

Counterexample: an original-caller principal constraint at destination ztunnel after waypoint insertion. Test both the permitted caller path and a bypass attempt through an authorized test plan. Test: AMB-04.

## AMB-05: peer authentication

PERMISSIVE admits plaintext as well as mTLS; it does not mean all traffic becomes plaintext. Ambient DISABLE is not a supported way to turn off the secure overlay. STRICT concerns accepted transport, not application authorization. Determine actual effective policy, source path and explicit exceptions before asserting a plaintext exposure.

Do not auto-fix unknown non-mesh clients by disabling transport security. Propose a scoped compatibility review instead. Test: AMB-05.

Sources: L4 and PEER in official-sources.md.
