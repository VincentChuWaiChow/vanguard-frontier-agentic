---
name: istio-ambient-mesh-review
description: Review Istio ambient enrollment, ztunnel versus waypoint enforcement,
  mixed sidecar/ambient paths and migration boundaries from rendered manifests and
  supplied sanitized evidence. Use for missing waypoints, L7 policies on ztunnel,
  identity changes across waypoints, ingress bypass questions, namespace enrollment
  claims or sidecar-to-ambient transitions. Do not collect live evidence or mutate
  a cluster.
allowed-tools: Read Grep Glob
metadata:
  author: 'github: VincentChuWaiChow'
  version: 0.2.0
  category: security
  lifecycle: beta
  execution_tier: static-review
---
# Istio Ambient Mesh Review
Establish workload enrollment and the actual ambient enforcement location.
## Decision and boundary
Is the intended workload enrolled, and where does each policy execute?
Operate as static-review. Read supplied files only; never collect live evidence, probe, mutate, or inherit ambient credentials. Host-enforced tool permissions remain mandatory.
## Required inputs
Require intended behavior/access, rendered resources, exact environment/version/mode scope, and an evidence ledger with capture times and visibility limits. Continue bounded analysis when fields are missing, but mark affected conclusions `needs-review`.
## Workflow
1. Inventory namespaces, pods, Services, identities, injection/ambient labels, exclusions, CNI evidence and revisions; do not infer pod participation from a namespace label alone.
2. Build source -> proxy/waypoint -> receiving enforcement point -> workload paths separately for Service, direct workload, ingress, sidecar, ambient and non-mesh traffic.
3. Classify every relevant policy by actual target and mode before interpreting L4/L7 behavior. Separate fail-safe denial from a waypoint policy whose path is unestablished.
4. Trace identities at each hop. Resolve waypoint enrollment and applicable workload-level anti-bypass controls without promising a universal bypass-prevention manifest.
5. For migrations, compare access and availability before, during and after each stage; surface documented feature gaps and rollback limitations.
6. Return evidence-backed path findings and paired test proposals. Handoff combined authorization semantics, routing, or upgrade decisions to their owners.

## Reference loading
Read [evidence-boundary.md](references/evidence-boundary.md), [applicability.md](references/applicability.md), and [output-contract.md](references/output-contract.md) first. Load the following only for the active branch.
| Reference | Load when |
| --- | --- |
| [enrollment-and-paths.md](references/enrollment-and-paths.md) | Workload enrollment or a missing/bypassed waypoint is in question. |
| [enforcement-rules.md](references/enforcement-rules.md) | Classify L4/L7 attachment, identity, and peer authentication. |
| [migration-boundaries.md](references/migration-boundaries.md) | Review a sidecar-to-ambient or mixed-mode transition. |
| [negative-tests.md](references/negative-tests.md) | Define counterexamples and required observations. |
| [official-sources.md](references/official-sources.md) | Verify a version-sensitive rule or source disagreement. |
| [review-contract.json](references/review-contract.json) | Produce or validate a portable structured review. |

## Deliverable
Workload enrollment matrix, path/enforcement map, and before/during/after migration findings.
Separate observed evidence, derived conclusions, assumptions and unknowns. Include one counterexample for every consequential finding. Never describe unperformed tests as passing or a bounded review as production certification.
