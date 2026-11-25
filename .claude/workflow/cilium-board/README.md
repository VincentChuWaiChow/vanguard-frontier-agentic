# Cilium Board — Design Record

> Status: **BUILT — the board is in the catalog.** Decision: [ADR-0006](../../../docs/adr/0006-cilium-agent-board.md).
> These documents are the design record, corrections included. Where implementation or review
> proved a plan decision wrong, the affected section says so in place.

## Executive verdict

- **Five agents, not thirteen.** A static router and four specialists, each owning one decision:
  connectivity boundary (policy and encryption), datapath explanation (including capacity),
  cluster-boundary reachability (BGP, LB-IPAM, L2 announcements, egress gateway, ClusterMesh),
  and lifecycle readiness (upgrades, migration, disruptive changes, advisories). The existing
  policy agent was retained and extended, not replaced.
- **Eight brief candidates were eliminated or merged.** CNI modernization and Gateway API are
  already owned by `kubernetes-network-architecture-review-agent`. Hubble, encryption,
  performance and ClusterMesh were merged into stronger owners. Tetragon was deferred.
  The twelve non-obvious opportunities were each traced to an existing owner, a merge, or a
  deterministic tool; none needed its own agent.
- **The most valuable gap was not in the brief's framing.** Cilium BGP, LB-IPAM, L2
  announcements, Helm upgrades and CNI migration have no reviewer and no live guard. The board
  reviews them and hands every change to the human change process.
- **One shared skill owns the rules that make answers trustworthy**: command classes by real
  behavior, what Hubble telemetry can and cannot prove, the verdict vocabulary, and the handoff
  to the existing Jira and Confluence authoring skills.
- **Every upstream claim is cited** to a dated, status-labeled ledger row, and a new gate fails
  the build on an unclassified command, a stale adapter, a mutating tier or a missing route.
- **Verdict: CONDITIONALLY READY (beta).** The structure is gated. Whether the agents behave
  to these contracts on real incidents has not been measured. See
  [06](./06-red-team-and-release-readiness.md).

## Documents

| File | Contents |
| --- | --- |
| [01-recon-capability-and-pain.md](./01-recon-capability-and-pain.md) | Repository archaeology, capability matrix, pain register |
| [02-opportunity-scoring.md](./02-opportunity-scoring.md) | Scoring model and its corrections, all 25 candidates, dispositions |
| [03-board-contracts-and-orchestration.md](./03-board-contracts-and-orchestration.md) | Board, ownership matrix, contract standard, skill architecture, orchestration, Atlassian loop, fixture |
| [04-threat-model-and-governance.md](./04-threat-model-and-governance.md) | Posture, separation of duties, mutation boundary, threat register |
| [05-value-model-and-kpis.md](./05-value-model-and-kpis.md) | Value and cost models, KPI framework, disconfirming signals |
| [06-red-team-and-release-readiness.md](./06-red-team-and-release-readiness.md) | Adversarial review, evaluation results, residual risks, roadmap, final verdict |

## How the work was done

Recon by Haiku subagents with file:line citations; primary-source research by Sonnet subagents
against docs.cilium.io, github.com/cilium and Context7, with load-bearing claims re-verified by
the orchestrator through Context7 against the 1.20.4 source documentation; reference prose by
Sonnet writers bound to those fact sheets; agent contracts, routing, gates and fixtures by the
orchestrator; an independent Sonnet refuter checked every reference claim against the fact
sheets before release.
