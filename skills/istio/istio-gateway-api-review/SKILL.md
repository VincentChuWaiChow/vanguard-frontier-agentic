---
name: istio-gateway-api-review
description: Review Istio ingress or waypoint routing that uses Kubernetes Gateway
  API GatewayClass, Gateway, HTTPRoute, GRPCRoute and ReferenceGrant resources, and
  distinguish them from Istio networking Gateways. Use for unaccepted routes, stale
  conditions, cross-namespace backends/certificates, missing listeners, TLS/hostname
  mismatches or Service-attached mesh routes. Consume supplied manifests/status only;
  do not change Gateways or fetch Secrets.
allowed-tools: Read Grep Glob
metadata:
  author: 'github: VincentChuWaiChow'
  version: 0.1.0
  category: networking
  lifecycle: beta
  execution_tier: static-review
---
# Istio Gateway API Review
Resolve listeners, route attachment, references and controller status without conflating API families.
## Decision and boundary
Is the intended listener/route/backend relationship valid, attached and adequately evidenced?
Operate as static-review. Read supplied files only; never collect live evidence, probe, mutate, or inherit ambient credentials. Host-enforced tool permissions remain mandatory.
## Required inputs
Require intended behavior/access, rendered resources, exact environment/version/mode scope, and an evidence ledger with capture times and visibility limits. Continue bounded analysis when fields are missing, but mark affected conclusions `needs-review`.
## Workflow
1. Record the full apiVersion/kind, CRD bundle/channel, implementation/controller version and intended role: ingress, waypoint or mesh Service attachment.
2. Build GatewayClass -> Gateway listener -> Route parent -> backend/certificate references; branch explicitly for Service-attached mesh routes.
3. Check parent selection, listener compatibility, hostname intersection and allowed namespace/kind relationships; do not substitute ReferenceGrant for listener attachment controls.
4. Resolve cross-namespace backend/certificate permissions and destination existence separately from Route-to-Gateway authorization.
5. Evaluate Accepted, ResolvedRefs and Programmed where defined, with correct parent/controller and observedGeneration; treat absent or stale conditions as unresolved.
6. Return graph findings and positive/negative request-path tests; do not infer end-to-end behavior from status alone.

## Reference loading
Read [evidence-boundary.md](references/evidence-boundary.md), [applicability.md](references/applicability.md), and [output-contract.md](references/output-contract.md) first. Load the following only for the active branch.
| Reference | Load when |
| --- | --- |
| [attachment-graph.md](references/attachment-graph.md) | Resolve API groups, parent relationships and backend references. |
| [conditions-and-freshness.md](references/conditions-and-freshness.md) | Interpret controller and parent-specific status. |
| [tls-and-boundaries.md](references/tls-and-boundaries.md) | Review TLS termination, certificates, SNI and hostname exposure. |
| [negative-tests.md](references/negative-tests.md) | Define attachment and traffic-path counterexamples. |
| [official-sources.md](references/official-sources.md) | Verify a version-sensitive rule or source disagreement. |
| [review-contract.json](references/review-contract.json) | Produce or validate a portable structured review. |

## Deliverable
Fully qualified attachment graph, cross-namespace decisions, condition freshness and missing path tests.
Separate observed evidence, derived conclusions, assumptions and unknowns. Include one counterexample for every consequential finding. Never describe unperformed tests as passing or a bounded review as production certification.
