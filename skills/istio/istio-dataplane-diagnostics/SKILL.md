---
name: istio-dataplane-diagnostics
description: Diagnose Istio connectivity, 403/404/503 responses, TLS, DNS, route propagation,
  injection/enrollment, ztunnel or waypoint failures using supplied logs, status,
  manifests and metrics. Use for incidents and ambiguous proxy-versus-application
  failures. Produce a bounded hypothesis ledger and evidence requests, not speculative
  fixes. Do not collect live logs, execute into pods, enable debug logging or run
  network probes without a separate authorized operator.
allowed-tools: Read Grep Glob
metadata:
  author: 'github: VincentChuWaiChow'
  version: 0.1.0
  category: observability
  lifecycle: beta
  execution_tier: static-review
---
# Istio Dataplane Diagnostics
Rank failure hypotheses and identify the next observation that separates them.
## Decision and boundary
Which failure hypothesis is supported, contradicted, or untested?
Operate as static-review. Read supplied files only; never collect live evidence, probe, mutate, or inherit ambient credentials. Host-enforced tool permissions remain mandatory.
## Required inputs
Require intended behavior/access, rendered resources, exact environment/version/mode scope, and an evidence ledger with capture times and visibility limits. Continue bounded analysis when fields are missing, but mark affected conclusions `needs-review`.
## Workflow
1. Establish affected and unaffected traffic, precise time window, last change, versions, topology, and expected behavior; preserve unknowns.
2. Map the request path and classify observations by DNS, transport, TLS, routing, policy, upstream application, capacity and configuration propagation.
3. Build a hypothesis ledger with supporting and opposing evidence. A status code or one response flag does not establish a root cause.
4. Select the least intrusive next observation that discriminates between leading hypotheses. Separate proposed collection from supplied results.
5. When offline analysis is appropriate, specify explicit --use-kube=false and a self-contained manifest set for an authorized offline runner; never execute through hidden kubeconfig access.
6. Return bounded diagnosis and owner handoffs. Keep production policy changes, restarts, debug logging and active probes out of the review workflow.

## Reference loading
Read [evidence-boundary.md](references/evidence-boundary.md), [applicability.md](references/applicability.md), and [output-contract.md](references/output-contract.md) first. Load the following only for the active branch.
| Reference | Load when |
| --- | --- |
| [hypothesis-workflow.md](references/hypothesis-workflow.md) | Triage an incident and separate transport, proxy and application hypotheses. |
| [signal-interpretation.md](references/signal-interpretation.md) | Read supplied proxy logs/configuration without overclaiming causes. |
| [collection-and-offline-analysis.md](references/collection-and-offline-analysis.md) | Propose bounded collection or external offline analyzer use. |
| [negative-tests.md](references/negative-tests.md) | Test resistance to ambiguous or missing evidence. |
| [official-sources.md](references/official-sources.md) | Verify a version-sensitive rule or source disagreement. |
| [review-contract.json](references/review-contract.json) | Produce or validate a portable structured review. |

## Deliverable
Timeline, hop-by-hop observations, ranked hypotheses and one next discriminating observation per unresolved branch.
Separate observed evidence, derived conclusions, assumptions and unknowns. Include one counterexample for every consequential finding. Never describe unperformed tests as passing or a bounded review as production certification.
