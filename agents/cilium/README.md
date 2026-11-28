# 🐝 Cilium Agents

<p align="center">
  <span style="font-size:3.5em">🐝</span>
</p>

The Cilium board: a router and four specialists, each owning one consequential decision. It is deliberately small. Candidates that duplicated an existing agent, needed an unguarded live mutation, or owned no distinct decision were merged, delegated or rejected; the scoring and the rejected alternatives are in [ADR-0006](../../docs/adr/0006-cilium-agent-board.md).

## Board

| Agent | Decision owned | Execution tier |
| --- | --- | --- |
| `cilium-maestro-agent` | Which specialist owns each decision in a multi-domain request, and what the combined evidence establishes | static review |
| `cilium-network-policy-review-agent` | Do policy and transport encryption enforce the intended connectivity boundary without unacceptable rollout risk? | static review |
| `cilium-dataplane-diagnostics-agent` | What is the most defensible explanation of the observed datapath behavior, and is capacity adequate? | static review |
| `cilium-external-connectivity-review-agent` | Is traffic crossing the cluster boundary reachable, owned, bounded and failure-isolated as claimed? | static review |
| `cilium-lifecycle-readiness-agent` | Is the lifecycle change ready to enter an authorized change process? | static review |

Every agent loads the shared `cilium-evidence-boundary` skill first. No agent applies, upgrades, restarts, captures packets or publishes anything.

## Owned elsewhere

| Request | Owner |
| --- | --- |
| CNI choice, overlay versus native routing as a design, CIDR planning, generic Gateway API, DNS | `kubernetes-network-architecture-review-agent` |
| Istio | `istio-maestro-agent` |
| Runtime detection rules | `falco-runtime-threat-rules-review-agent` |
| Cost and chargeback | `finops-maestro-agent` |
| Control evidence lineage, incident recurrence | `atlassian-control-evidence-lineage-agent`, `atlassian-incident-prevention-agent` |

## Live changes

Live-guard agents for Cilium resources live in `agents/kubernetes/` because they operate at the Kubernetes API layer. The board names them; it never dispatches them.

| Change | Path |
| --- | --- |
| CiliumNetworkPolicy, CiliumClusterwideNetworkPolicy, NetworkPolicy, CiliumEgressGatewayPolicy | `kubernetes-live-network-policy-guard-agent` |
| ClusterMesh peer Secret, Gateway API resources | `kubernetes-live-network-architecture-mutation-guard-agent` |
| BGP, LB-IPAM, L2 announcements, Helm values, upgrades, CNI migration | no guard exists: human change process |

All verdicts use the structured response in [`docs/evidence-output-spec.md`](../../docs/evidence-output-spec.md).

## Install

```bash
# The whole board
npx vfa-export-agents --platform claude-code --provider cilium --repo .

# All Kubernetes network agents, including the live guards
npx vfa-export-agents --platform claude-code --role kubernetes-network-engineer --repo .
```
