---
name: cilium-lifecycle-readiness
description: Decide whether a proposed Cilium lifecycle change (minor or patch upgrade, rollback, CNI migration to Cilium, disruptive configuration change such as IPAM, kube-proxy replacement or ClusterMesh identity settings, or advisory remediation) is ready to enter an authorized change process. Use with exact current and target versions, install method, Helm values diff, per-node-pool kernels, Kubernetes version and enabled features. Not for target architecture or CNI choice, live policy diagnosis, or executing anything. Static review only; it never runs preflight, edits Helm values or upgrades.
allowed-tools: Read Grep Glob
disallowed-tools: Write Edit NotebookEdit Bash
metadata:
  author: 'github: VincentChuWaiChow'
  version: 0.1.0
  updated: '2026-11-25'
  category: resilience
  lifecycle: beta
  execution_tier: static-review
---
# Cilium Lifecycle Readiness

Judge whether a Cilium transition is safe enough to hand to an authorized change process; never perform it.

## Decision and boundary

Is the proposed Cilium lifecycle change (upgrade, rollback, CNI migration to Cilium, disruptive configuration change, advisory remediation) ready to enter an authorized change process? Verdicts are `approved`, `blocked` or `needs-review`; `approved` means no blocking condition observed in a bounded review, never execution consent. The target architecture is owned by kubernetes-network-architecture-review-agent; this skill owns whether the transition is safe. No guard agent exists for Helm upgrades, cilium-config changes or BGP CRDs, so every execution request goes to the human change process. Policy semantics of changed defaults go to cilium-network-policy-review-agent; ClusterMesh and BGP topology to cilium-external-connectivity-review-agent.

## When to use / when not to use

Use for upgrade or rollback plans, migration plans into Cilium, in-place changes to IPAM, kube-proxy replacement, routing or ClusterMesh identifiers, and "are we exposed to this advisory" questions. Do not use to choose a CNI, to explain a live datapath symptom (cilium-dataplane-diagnostics-agent), or to run, schedule or approve execution.

## Required inputs

- Exact current version (x.y.z) and target version. Unknown version: withhold every version-dependent conclusion and mark it `unknown`.
- Install method: Helm, CLI or managed distribution. Managed distributions may differ from upstream; say so and do not apply upstream steps unverified.
- Helm values diff (current vs proposed), Kubernetes version, enabled features, and kernel version per node pool (the lowest kernel governs).

## Workflow

1. Classify the change: patch, one-minor upgrade, multi-minor, rollback, migration, in-place config change or advisory remediation.
2. Fix the inventory: version, install method, node-pool kernels, Kubernetes version. Missing items become named unknowns, not guesses.
3. Check the path: one minor at a time, latest patch first, and any multi-minor request is `blocked` pending a stepwise plan [CIL-LC-E01].
4. Check compatibility of target against kernels and Kubernetes version, including feature-gated kernel needs.
5. Walk the breaking-changes checklist for each minor crossed and demand the evidence that proves each item handled.
6. Check the plan's preparation: preflight authorization, same-version components, values diff, canary order, rollback limits and L7 disruption window.
7. For advisories, require version in range AND vulnerable feature enabled before calling an exposure real.
8. Return a verdict, blockers and the evidence still needed; route execution to the human change process.

## Reference loading

Load companion skill `cilium-evidence-boundary` first (command classes, evidence classes, verdict vocabulary, sensitive collection).

| Reference | Load when |
| --- | --- |
| [upgrade-path-and-rollback.md](references/upgrade-path-and-rollback.md) | Any upgrade, patch bump or rollback plan. |
| [version-and-kernel-compatibility.md](references/version-and-kernel-compatibility.md) | Checking supported branches, kernels, Kubernetes versions or ClusterMesh skew. |
| [breaking-changes-1-19-1-20.md](references/breaking-changes-1-19-1-20.md) | The path crosses 1.18 to 1.19 or 1.19 to 1.20. |
| [cni-migration-and-disruptive-changes.md](references/cni-migration-and-disruptive-changes.md) | Migrating to Cilium or changing IPAM, kube-proxy replacement, routing or ClusterMesh identifiers in place. |
| [advisory-applicability.md](references/advisory-applicability.md) | Deciding whether a security advisory applies. |
| [evidence-ledger.md](references/evidence-ledger.md) | Verifying a cited upstream claim. |

## Failure handling

- Companion skill `cilium-evidence-boundary` not installed: stop and return `needs-review`, stating that the command-class and evidence rules are missing; do not continue without them.
- Unknown or non-x.y.z version: `needs-review`; no range, skew or advisory conclusion.
- Managed distribution: upstream facts are leads only; ask for the vendor's compatibility statement.
- Conflicting inputs (values diff vs reported features): state both and resolve toward the more disruptive reading.
- Preflight or sysdump output unavailable: treat the dependent check as `assumed`, never passed.
- Anything documented as untested or absent from the reviewed pages stays `unknown`, not supported.

## Deliverable

Verdict with `evidence_level`; a table of checks with claim class (`observed`, `derived`, `assumption`, `unknown`); blockers; evidence still required; the authorizer for any [C4] or [C5] step; and the handoff to the human change process. Never describe an unperformed preflight or rollback as tested. Cite ledger IDs for upstream claims.
