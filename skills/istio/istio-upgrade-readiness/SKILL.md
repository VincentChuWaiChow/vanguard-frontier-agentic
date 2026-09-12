---
name: istio-upgrade-readiness
description: Assess an Istio source-to-target upgrade or downgrade using supplied
  inventory, installation configuration, release documentation and test evidence.
  Use for control/data-plane skew, canary revisions, Helm or istioctl transitions,
  CRD storage changes, extension compatibility, ambient components or rollback readiness.
  Produce a dependency plan and blockers; do not install, upgrade, restart, delete
  or assume the latest release.
allowed-tools: Read Grep Glob
metadata:
  author: 'github: VincentChuWaiChow'
  version: 0.1.0
  category: delivery
  lifecycle: beta
  execution_tier: static-review
---
# Istio Upgrade Readiness
Evaluate supported transitions and operational preparation before an upgrade is authorized.
## Decision and boundary
Is the specified source-to-target transition supported and operationally prepared?
Operate as static-review. Read supplied files only; never collect live evidence, probe, mutate, or inherit ambient credentials. Host-enforced tool permissions remain mandatory.
## Required inputs
Require intended behavior/access, rendered resources, exact environment/version/mode scope, and an evidence ledger with capture times and visibility limits. Continue bounded analysis when fields are missing, but mark affected conclusions `needs-review`.
## Workflow
1. Inventory exact versions and connected revisions for control planes, sidecars, gateways, waypoints, ztunnel and CNI, plus Kubernetes and CRD/API bundles.
2. Require explicit target version, installation/upgrade ownership and source-specific release documentation; leave unverified support status unresolved.
3. Evaluate directional control/data-plane relationships, not absolute minor distance alone. A skew comparison is not a whole-environment support verdict.
4. Derive sequence from the actual installation method and dependencies; distinguish sidecar, ambient and shared cluster-wide resources.
5. Evaluate CRD persistence/conversion, extensions, feature removal, webhooks, capacity, disruption budgets, probes and rollback artifacts before readiness.
6. Return blockers and a phase-by-phase plan with required measurements; route any execution request to an approved operator without performing it.

## Reference loading
Read [evidence-boundary.md](references/evidence-boundary.md), [applicability.md](references/applicability.md), and [output-contract.md](references/output-contract.md) first. Load the following only for the active branch.
| Reference | Load when |
| --- | --- |
| [inventory-and-support.md](references/inventory-and-support.md) | Establish versions, ownership and support eligibility. |
| [skew-and-order.md](references/skew-and-order.md) | Evaluate directional skew and installation-specific sequencing. |
| [crds-extensions-and-rollback.md](references/crds-extensions-and-rollback.md) | Review persistence, extensions and irreversible boundaries. |
| [negative-tests.md](references/negative-tests.md) | Define upgrade counterexamples and transition tests. |
| [official-sources.md](references/official-sources.md) | Verify a version-sensitive rule or source disagreement. |
| [review-contract.json](references/review-contract.json) | Produce or validate a portable structured review. |

## Deliverable
Version/component matrix, dependency sequence, evidence blockers, staged checks and rollback limits.
Separate observed evidence, derived conclusions, assumptions and unknowns. Include one counterexample for every consequential finding. Never describe unperformed tests as passing or a bounded review as production certification.
