# Conditions and freshness

## GATEWAY-03: choose the right condition instance

Use the status for the relevant controller and Route parent/listener. Interpret `Accepted`, `ResolvedRefs` and `Programmed` where defined for that resource and implementation. Preserve False, Unknown and missing conditions; do not turn all three into the same success state.

Compare each condition's `observedGeneration` with the resource's current `metadata.generation`. A true condition for an earlier generation does not establish acceptance of the new specification. Capture timestamp alone is not a substitute for generation matching.

## GATEWAY-04: status is not request-path proof

Even fresh accepted/programmed status does not establish the requested DNS, load-balancer, TLS, host/path, policy and upstream behavior end to end. Require a separately authorized positive and negative request test tied to the intended parent and backend.

Do not require a condition that the resource/controller contract does not define. Record that scope limitation instead of fabricating a controller failure.

Sources: GSTATUS and GDESIGN. Tests: GATEWAY-03, GATEWAY-04.
