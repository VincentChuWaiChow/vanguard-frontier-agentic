# Policy-set semantics

## AUTH-01: evaluate the set, not one resource

Resolve scope and actual attachment first. Consider applicable CUSTOM evaluation, then matching DENY, then applicable ALLOW policies. With no ALLOW policy for the workload, the native ALLOW stage does not itself impose an allowlist. With ALLOW policies present, one applicable matching ALLOW can permit a request that another ALLOW does not match. A prior denial still matters. AUDIT is not an admission decision and requires actual audit support to produce records.

Inputs must cover root/namespace/workload scope as applicable and the effective enforcement point. Unknown external authorization results remain unknown; never invent a CUSTOM provider answer.

## AUTH-02: rule-list shape

Missing `rules` or `rules: []` supplies no matching rules. `rules: [{}]` contains a rule that matches all requests. For DENY, these are not equivalent. For ALLOW, an empty list contributes no permission but does not override another applicable ALLOW. Keep absent, empty and nonempty rule forms visible in the review.

## AUTH-03: attachment and revision

Check the exact supported targetRefs kinds, namespaces and controller revisions before changing selectors to references. Verify mixed-revision handling against target-release documentation. A syntactically valid target reference does not prove policy enforcement. Do not confuse an Istio AuthorizationPolicy target reference with Gateway API Route parent attachment.

## Decision worksheet

For each access-matrix row record target, transport/request identity, applicable policy IDs, matches, external decision evidence, final derived outcome, and unknown inputs. Preserve independent DENY and ALLOW policy behavior; do not collapse the union/intersection structure into one broad selector.

Sources: AUTHZ and L4 in official-sources.md. Tests: AUTH-01, AUTH-02A, AUTH-02B, AUTH-03.
