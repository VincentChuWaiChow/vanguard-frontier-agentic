---
name: istio-review-routing
description: Coordinate an Istio review across ambient enrollment, authorization,
  traffic resilience, Gateway API, upgrades and diagnostics. Use for multi-domain
  requests or when review ownership is unclear. Select only necessary specialist companions,
  pass bounded sanitized evidence and reconcile conflicting findings. Return a handoff
  for all live changes; never auto-dispatch a live guard, manufacture approval or
  average away a blocker.
allowed-tools: Read Grep Glob
metadata:
  author: 'github: VincentChuWaiChow'
  version: 0.1.0
  category: architecture
  lifecycle: beta
  execution_tier: static-review
---
# Istio Review Routing
Route bounded Istio questions to decision owners and preserve unresolved disagreements.
## Decision and boundary
Which specialist owns each required decision, and what can the combined evidence establish?
Operate as static-review. Read supplied files only; never collect live evidence, probe, mutate, or inherit ambient credentials. Host-enforced tool permissions remain mandatory.
## Required inputs
Require intended behavior/access, rendered resources, exact environment/version/mode scope, and an evidence ledger with capture times and visibility limits. Continue bounded analysis when fields are missing, but mark affected conclusions `needs-review`.
## Workflow
1. Parse the user goal into decisions rather than resource names. Record whether the request is static review, runtime collection or mutation.
2. Route to the minimum sufficient set of specialists using the routing table. Establish ambient path before relying on waypoint authorization conclusions.
3. Pass each owner only its scope, required intent and relevant sanitized evidence. Do not broaden privileges through delegation.
4. Preserve specialist evidence IDs, untested assumptions and conflicts; reconcile applicability before combining findings.
5. Return blocked when a required confirmed condition violates intent; otherwise return needs-review when a required input or decision is unresolved. Do not use voting or numerical averages.
6. For live changes, stop at an approval-bound handoff to the existing Kubernetes guard. No specialist verdict is execution consent.

## Reference loading
Read [evidence-boundary.md](references/evidence-boundary.md), [applicability.md](references/applicability.md), and [output-contract.md](references/output-contract.md) first. Load the following only for the active branch.
| Reference | Load when |
| --- | --- |
| [routing-table.md](references/routing-table.md) | Select owners and handoff order. |
| [aggregation-and-conflicts.md](references/aggregation-and-conflicts.md) | Reconcile findings without losing evidence boundaries. |
| [live-handoff.md](references/live-handoff.md) | A request proposes cluster reads, probes or changes. |
| [negative-tests.md](references/negative-tests.md) | Evaluate routing and refusal boundaries. |
| [official-sources.md](references/official-sources.md) | Verify a version-sensitive rule or source disagreement. |
| [review-contract.json](references/review-contract.json) | Produce or validate a portable structured review. |

## Deliverable
Minimal specialist plan, evidence ownership, combined verdict and unresolved disagreements.
Separate observed evidence, derived conclusions, assumptions and unknowns. Include one counterexample for every consequential finding. Never describe unperformed tests as passing or a bounded review as production certification.
