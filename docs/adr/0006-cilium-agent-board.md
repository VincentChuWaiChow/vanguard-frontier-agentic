---
layout: default
title: "ADR-0006: A Four-Specialist Cilium Board With Classified Commands and Ledger-Backed Claims"
permalink: /docs/adr/0006-cilium-agent-board/
---

# ADR-0006: A Four-Specialist Cilium Board With Classified Commands and Ledger-Backed Claims

## Status

Proposed (2026-11-25). Accepted only when the repository owner accepts it.

The supporting design record (reconnaissance, capability matrix, pain register, opportunity scores, contracts, threat model, value model, red team) is [docs/architecture/cilium-board/](../architecture/cilium-board/README.md). It is a dated record and authorizes nothing.

## Context

A brief asked for an enterprise Cilium board of specialist agents and proposed thirteen role hypotheses plus twelve non-obvious opportunities. The repository already had one Cilium policy agent and skill, a Kubernetes network architecture agent covering CNI choice and topology, Istio and agentic boards with a static-maestro pattern, Kubernetes live guards for policy and some architecture resources, and Atlassian authoring skills behind a live-write guard (ADR-0005).

Four findings decided the shape:

1. **CNI modernization and Gateway API design are already owned** by `kubernetes-network-architecture-review-agent`. A Cilium copy would split one decision between two owners.
2. **BGP Control Plane, LB-IPAM, L2 announcements, Helm upgrades and CNI migration have no reviewer and no live guard.** The gap is real, and it also means the board must not propose executing any of them.
3. **Every Cilium specialist needs the same evidence rules**: what Hubble flows can prove, how a command's risk is judged, what a verdict means. Restating them per skill would create four drifting copies.
4. **Cilium behavior is version-sensitive at the minor level**: 1.19 removed BGP v1 and changed the ClusterMesh policy default; 1.20 removed Kafka-aware policy and raised the Gateway API floor. Claims without a version and a source go stale silently.

## Decision

1. **Board:** `cilium-maestro-agent` (static router; corrected to plan-only by the pre-acceptance review: it returns a dispatch plan and never spawns agents) and four specialists, each owning one decision:
   - `cilium-network-policy-review-agent` (retained, id unchanged, version 0.2.0): does policy and transport encryption enforce the intended connectivity boundary? Its scope is deliberately widened from authorization to authorization plus confidentiality (WireGuard and IPsec coverage), and narrowed by moving egress gateway SNAT and failover to the boundary specialist.
   - `cilium-dataplane-diagnostics-agent`: what explains the observed datapath behavior, including capacity.
   - `cilium-external-connectivity-review-agent`: is traffic crossing the cluster boundary reachable, owned, bounded and failure-isolated?
   - `cilium-lifecycle-readiness-agent`: is an upgrade, migration, disruptive change or advisory remediation ready for an authorized change process?
2. **Shared skill `cilium-evidence-boundary`** is the single owner of claim classes, command classification, flow-telemetry limits, verdicts and the Jira and Confluence handoff. Every Cilium agent declares it in `companion_skills`, so export bundles it.
3. **Commands are classified by behavior** into six classes, from offline analysis to destructive, and the class is written wherever a command appears. `cilium connectivity test` is an active diagnostic because it creates workloads; the upgrade preflight is a mutation because it creates a DaemonSet and Deployment; `cilium sysdump` is privileged and sensitive collection.
4. **Every material upstream claim cites a dated ledger row** in the skill's `references/evidence-ledger.md`, using the repository's existing ledger gate. The allowlist is extended to the Cilium project's own hosts, Kubernetes and Gateway API.
5. **No live operator.** Policy changes are named for `kubernetes-live-network-policy-guard-agent`, ClusterMesh peer Secret and Gateway API changes for `kubernetes-live-network-architecture-mutation-guard-agent`, never auto-dispatched; everything else goes to the human change process.
6. **A new gate, `validate:cilium-board`**, enforces command classification, `text`-only fences, the shared companion, no mutating tier, the canonical `AGENT.md` sections, harness freshness through the existing agentic-board renderer, read-only Claude Code grants, and routing coverage. A routing fixture of 28 scenarios is graded by the existing `evaluate()`.
7. **Reuse over invention** *(corrected by the pre-acceptance review: no Cilium contract adopts the `agentic-board-protocol` envelope; the human is the bridge to Atlassian)*: the `agentic-board-protocol` envelope, the evidence-output verdict vocabulary, the ledger gate, the harness renderer and the routing grader are reused unchanged; no schema is added.

## Rejected alternatives

| Alternative | Why rejected |
| --- | --- |
| One agent per Cilium feature area (the brief's thirteen roles) | Five had no distinct decision once existing owners were counted; distinctness is a hard gate, not a weight |
| A Cilium network-architecture agent | Duplicates `kubernetes-network-architecture-review-agent` |
| A Hubble observability agent | Every specialist must judge its own telemetry; a separate agent adds a mandatory hop. Kept as the shared skill's flow-telemetry rules |
| Separate encryption and performance agents | Same users and evidence as policy and dataplane respectively; merged |
| A Tetragon runtime-correlation agent | Separate project with no reference, fixture or guard here; deferred to a runtime-security board |
| A Cilium live operator for BGP, Helm or upgrades | No guard exists; convenience does not justify an unguarded mutation path |
| A Cilium-specific handoff schema | The existing envelope expresses everything found so far |
| Naming the maestro skill `cilium-maestro` | The fixture generator would discover and overwrite the hand-curated adversarial fixtures |

## Consequences

- The board answers more often with `needs-review` than a generalist would, by design: missing versions, peer-router state and telemetry coverage are named instead of assumed.
- Maintenance is concentrated and periodic: the ledgers must be refreshed at each Cilium minor release, starting with the lifecycle skill.
- The Kubernetes maestro now routes non-policy Cilium work to the board. Its generated routing fixture still has no Cilium domain (residual risk R-03 in the design record).
- On Kiro harnesses the read-only limit is prompt-level only, because no verified tool-restricting field exists there.

## Verdict

The original self-assessment (CONDITIONALLY READY) is superseded by the independent pre-acceptance review in [docs/architecture/cilium-board/07-pre-acceptance-review.md](../architecture/cilium-board/07-pre-acceptance-review.md), which recommends acceptance only after the remediation and decisions listed there. The status stays **Proposed** until the repository owner decides.
