---
name: istio-traffic-resilience-review
description: Review Istio VirtualService and DestinationRule or supplied Gateway API
  traffic intent for routing, weights, timeouts, retries, outlier detection, circuit-breaking,
  mirroring and fault injection. Use for canaries, retry storms, unexplained routing,
  write-side-effect risks or resilience changes. Evaluate target version and mesh
  mode before applying a rule. Do not inject faults, mirror traffic, probe services
  or change live configuration.
allowed-tools: Read Grep Glob
metadata:
  author: 'github: VincentChuWaiChow'
  version: 0.1.0
  category: resilience
  lifecycle: beta
  execution_tier: static-review
---
# Istio Traffic Resilience Review
Evaluate route behavior and failure amplification against stated service objectives.
## Decision and boundary
Does the proposed traffic behavior avoid unacceptable routing and failure amplification?
Operate as static-review. Read supplied files only; never collect live evidence, probe, mutate, or inherit ambient credentials. Host-enforced tool permissions remain mandatory.
## Required inputs
Require intended behavior/access, rendered resources, exact environment/version/mode scope, and an evidence ledger with capture times and visibility limits. Continue bounded analysis when fields are missing, but mark affected conclusions `needs-review`.
## Workflow
1. Record user-facing deadline, critical operations, idempotency guarantees, load/capacity constraints and intended canary behavior.
2. Resolve API family, mode, attachment, route precedence, hosts, destinations, ports and versioned backends; delegate Gateway attachment to its owner.
3. Interpret relative route weights separately from percentages and evaluate destination health/availability evidence without inventing runtime distribution.
4. Inventory retry owners across caller, mesh and dependencies; compare total/per-try deadlines, error eligibility, connection pools, ejection and remaining capacity.
5. Treat mirroring and fault injection as business-affecting changes. Require isolation, data-handling scope, explicit experiment approval and stop criteria.
6. Return route and resilience findings with an abort/rollback plan and paired tests; do not execute them.

## Reference loading
Read [evidence-boundary.md](references/evidence-boundary.md), [applicability.md](references/applicability.md), and [output-contract.md](references/output-contract.md) first. Load the following only for the active branch.
| Reference | Load when |
| --- | --- |
| [routing-and-weights.md](references/routing-and-weights.md) | Inspect route selection, destinations and relative weights. |
| [timeouts-retries-and-capacity.md](references/timeouts-retries-and-capacity.md) | Review retry budgets, deadlines and overloaded dependencies. |
| [mirroring-and-faults.md](references/mirroring-and-faults.md) | Review any real-traffic duplication or fault experiment. |
| [negative-tests.md](references/negative-tests.md) | Define resilience and routing counterexamples. |
| [official-sources.md](references/official-sources.md) | Verify a version-sensitive rule or source disagreement. |
| [review-contract.json](references/review-contract.json) | Produce or validate a portable structured review. |

## Deliverable
Route table, resilience budget, canary criteria, side-effect boundaries, and rollback proposal.
Separate observed evidence, derived conclusions, assumptions and unknowns. Include one counterexample for every consequential finding. Never describe unperformed tests as passing or a bounded review as production certification.
