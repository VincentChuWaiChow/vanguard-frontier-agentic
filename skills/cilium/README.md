# 🐝 Cilium Skills

<p align="center">
  <!-- 🖼️ Add a Cilium logo to assets/logos/cnative/cilium/ and update this path -->
  <span style="font-size:3.5em">🐝</span>
</p>

Skills behind the Cilium board of specialist agents. Each specialist skill owns one decision; one shared skill owns the evidence and safety rules they all apply. The board's design record is [ADR-0006](../../docs/adr/0006-cilium-agent-board.md).

## Skills

| Skill | Owns | Companion of |
| --- | --- | --- |
| `cilium-evidence-boundary` | Claim classes, command classification by real behavior, Hubble flow-telemetry limits, verdicts, and the handoff to the Jira and Confluence authoring skills | every Cilium agent |
| `cilium-network-policy-review` | Whether policy and transport encryption enforce the intended connectivity boundary | `cilium-network-policy-review-agent` |
| `cilium-dataplane-diagnostics` | The most defensible explanation of observed datapath behavior, including capacity | `cilium-dataplane-diagnostics-agent` |
| `cilium-external-connectivity-review` | Reachability, address ownership and failure isolation across the cluster boundary (BGP, LB-IPAM, L2 announcements, egress gateway, ClusterMesh) | `cilium-external-connectivity-review-agent` |
| `cilium-lifecycle-readiness` | Whether an upgrade, rollback, CNI migration, disruptive change or advisory remediation is ready for an authorized change process | `cilium-lifecycle-readiness-agent` |
| `cilium-review-routing` | Which specialist owns each decision in a multi-domain request | `cilium-maestro-agent` |

## Posture

- **Static review by default.** The skills read supplied artifacts. The policy skill may read live state only through an authorized read-only tool against a user-confirmed cluster.
- **Every command is classified.** A command is named only with its class, from offline analysis to destructive, decided by what it does rather than by its verb. `tests/validate-cilium-board.py` fails the build on an unclassified command.
- **Every upstream claim is cited.** Each skill's `references/evidence-ledger.md` records the source, section, retrieval date and status of every material claim; `tests/validate-skill-references.py` fails the build on an uncited row or a citation without a row.
- **An unobserved flow proves nothing.** Hubble keeps a bounded per-node buffer; absence of a flow is never evidence that traffic did not occur.
- **No live operator.** Policy changes go to `kubernetes-live-network-policy-guard-agent`, ClusterMesh peer Secret and Gateway API changes to `kubernetes-live-network-architecture-mutation-guard-agent`. BGP, Helm, upgrade and migration changes have no guard and go to the human change process.

## Maintenance

The ledgers are dated snapshots of Cilium 1.20.5 documentation, retrieved 2026-11-25. At each Cilium minor release, re-fetch the rows of the lifecycle skill's breaking-changes and advisory references first; they are the most version-sensitive. Run `npm run validate` after changing any Cilium skill.
