---
name: istio-authorization-policy-review
description: Review Istio AuthorizationPolicy, PeerAuthentication and RequestAuthentication
  together against intended access. Use for default-deny design, overlapping CUSTOM/DENY/ALLOW
  policies, empty rules, JWT requirements, wildcard principals, L7-on-TCP risks, policy
  targetRefs, trust identity or authorization regressions. Review supplied evidence
  only; separate policy semantics from observed enforcement and do not apply changes.
allowed-tools: Read Grep Glob
metadata:
  author: 'github: VincentChuWaiChow'
  version: 0.1.0
  category: security
  lifecycle: beta
  execution_tier: static-review
---
# Istio Authorization Policy Review
Evaluate the combined access decision against an explicit intended access matrix.
## Decision and boundary
Does the applicable policy set implement the intended access matrix?
Operate as static-review. Read supplied files only; never collect live evidence, probe, mutate, or inherit ambient credentials. Host-enforced tool permissions remain mandatory.
## Required inputs
Require intended behavior/access, rendered resources, exact environment/version/mode scope, and an evidence ledger with capture times and visibility limits. Continue bounded analysis when fields are missing, but mark affected conclusions `needs-review`.
## Workflow
1. Obtain the intended source/destination/protocol/identity allow-deny matrix and enumerate relevant namespace/root policies; record incomplete visibility.
2. Resolve targets, selectors, targetRefs, revision support and enforcement points; send unresolved ambient attachment to the ambient reviewer.
3. Evaluate CUSTOM, DENY and ALLOW together; keep missing/empty rules distinct from an empty matching rule. Include other applicable ALLOW policies.
4. Separate transport authentication, JWT validation, identity presence and authorization. Do not infer access requirements from validation resources alone.
5. Check protocol-specific missing attributes, path normalization, trusted forwarded addresses and narrow exceptions against actual traffic shape.
6. Produce access decisions with assumptions and negative tests; propose bounded remediations but never mutate or self-authorize a live guard.

## Reference loading
Read [evidence-boundary.md](references/evidence-boundary.md), [applicability.md](references/applicability.md), and [output-contract.md](references/output-contract.md) first. Load the following only for the active branch.
| Reference | Load when |
| --- | --- |
| [policy-set-semantics.md](references/policy-set-semantics.md) | Evaluate actions, rule shapes, attachment and overlap. |
| [authentication-and-identity.md](references/authentication-and-identity.md) | Evaluate mTLS identities and JWT requirements. |
| [protocol-and-normalization.md](references/protocol-and-normalization.md) | Review HTTP/TCP, host, path, header and forwarded-identity boundaries. |
| [negative-tests.md](references/negative-tests.md) | Define access regression tests. |
| [official-sources.md](references/official-sources.md) | Verify a version-sensitive rule or source disagreement. |
| [review-contract.json](references/review-contract.json) | Produce or validate a portable structured review. |

## Deliverable
Expected-versus-derived access matrix, policy-set reasoning, and paired authentication/authorization tests.
Separate observed evidence, derived conclusions, assumptions and unknowns. Include one counterexample for every consequential finding. Never describe unperformed tests as passing or a bounded review as production certification.
