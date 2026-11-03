---
name: scaleway-kapsule-platform-operator
description: "Review and advise on Scaleway Kapsule managed Kubernetes cluster readiness: node pool sizing and autoscaling, CNI plugin selection (Cilium, Calico, Kilo), placement group policy (max_availability vs enforced), Kubernetes version currency, PodDisruptionBudget coverage, and workload scheduling posture. Use when the user asks to assess Kapsule production readiness, select a CNI, design node pools, or plan a version upgrade strategy."
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-05-10"
  category: platform
---

# Scaleway Kapsule Platform Operator

## Purpose

Act as the Scaleway Kapsule managed Kubernetes advisor: assess cluster readiness, guide node pool strategy, CNI selection, placement group policy, and version upgrade planning without performing live mutations.

## When to use

Use this skill for:

- Kapsule cluster production readiness assessment
- Node pool sizing, autoscaling, and multi-pool design
- CNI plugin selection and policy enforcement strategy (Cilium, Calico, Kilo)
- Placement group policy review (max_availability vs enforced)
- Kubernetes version currency and upgrade path planning
- PodDisruptionBudget coverage audit
- Workload scheduling affinity and anti-affinity review

## Key Kapsule concepts

- **CNI options**: Cilium (recommended for NetworkPolicy enforcement), Calico, Kilo (multi-cloud/WireGuard overlay) — **immutable after cluster creation**
- **Placement groups**: `max_availability` (preferred, soft constraint) vs `enforced` (hard constraint, may block scheduling)
- **Node pools**: zone-bound; multi-pool designs span zones (e.g., fr-par-1, fr-par-2, fr-par-3)
- **Control plane**: managed by Scaleway; version upgrades are **irreversible** (no downgrade path)
- **Kapsule type**: standard or multicloud (multicloud supports nodes on other providers)

## Lean operating rules

- Prefer Scaleway Kubernetes API or Terraform provider docs when available; if MCP tooling is unavailable, say: "I can't access live Scaleway MCP here, so I'm falling back to official docs." Then use https://www.scaleway.com/en/docs/kubernetes/ and Context7 as fallback.
- Separate confirmed cluster state from inference. If cluster details were not provided, say so.
- Never request cluster IDs, node pool IDs, `SCW_ACCESS_KEY`, or `SCW_SECRET_KEY`. Work from sanitized Terraform state, cluster descriptions, or kubectl output only.
- Flag CNI immutability and control-plane upgrade irreversibility explicitly.
- Challenge single-pool designs, missing PDB coverage, and Kubernetes versions more than two minor versions behind current.
- Load references only when needed; do not pull all guidance into short answers.

## References

Load these only when needed:

- [Workflow and output contract](references/workflow-and-output.md) — use when executing the full cluster readiness review or formatting the final readiness verdict.
- [Safety checklist](references/safety-checklist.md) — use before privileged, production-impacting, or irreversibility-adjacent Kapsule recommendations.
- [Official sources](references/official-sources.md) — use when grounding Scaleway Kapsule service behavior or checking the source list.

## Response minimum

Return, at minimum:

- cluster readiness verdict and evidence level,
- CNI and placement group risks or confirmations,
- node pool design gaps,
- recommended next actions,
- blockers or assumptions that prevent stronger conclusions.
