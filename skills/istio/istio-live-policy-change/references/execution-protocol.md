# Execution protocol for the existing guard

## Preconditions

Verify scope, access-matrix review, approval authenticity, target identity, immutable artifacts, current baseline, change ownership and rollback feasibility. Record audit correlation. If the live adapter is unavailable or any requirement is missing, return a blocked/needs-review handoff without executing.

## GUARD-04: bounded execution

Use only the existing approved adapter and exact permitted operation. Build structured arguments from validated identifiers; do not concatenate an arbitrary supplied shell command. Pin context and namespace on each operation. Recheck preconditions immediately before each mutation. Restrict server-side dry runs to separately authorized target access; they contact the server and can invoke admission logic.

Apply one approved phase at a time and record actual tool result, modified resource identity and postcondition. Do not expand an unsuccessful apply into deletion, restart or broad access relaxation. On timeout, determine whether the operation took effect before retrying; avoid duplicate or conflicting action.

## Review semantics to preserve

PERMISSIVE admits plaintext alongside mTLS; it is not plaintext-only. Empty DENY rules differ from a DENY containing an empty rule. Ambient L7-at-ztunnel denial differs from a waypoint bypass case. Resolve these through approved review evidence rather than embedding stale blanket rules in operator metadata.

No executable cluster mutation script ships with this companion. Its protocol constrains an existing authorized guard; it does not create one.

Sources: PEER, AUTHZ, VFA. Test: GUARD-04.
