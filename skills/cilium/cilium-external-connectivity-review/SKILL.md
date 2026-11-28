---
name: cilium-external-connectivity-review
description: Review whether traffic that crosses the cluster boundary through Cilium BGP Control Plane, LB IPAM, L2 announcements, egress gateway or ClusterMesh is reachable, correctly owned, bounded and failure-isolated as claimed. Use for load balancer VIP reachability, fixed egress source IPs, gateway node failure, multi-cluster failover and resilience claims on an already chosen design. Static review of supplied manifests, status output and peer-side evidence only. Do not use to choose the multi-cluster technology or CIDR plan, to decide which egress destinations are allowed, to review lifecycle changes, or to run any command.
allowed-tools: Read Grep Glob
disallowed-tools: Write Edit NotebookEdit Bash
metadata:
  author: 'github: VincentChuWaiChow'
  version: 0.1.0
  updated: '2026-11-25'
  category: networking
  lifecycle: beta
  execution_tier: static-review
---
# Cilium External Connectivity Review
Verify that a claimed Cilium boundary-crossing design reaches its destination, has one owner per address, stays inside its blast radius, and survives the failure of the node or cluster that owns the address.
## Decision and boundary
Is traffic that crosses the cluster boundary reachable, correctly owned, bounded and failure-isolated as claimed?
For every boundary flow answer three questions: (a) how does the packet leave or enter, (b) who owns the address it uses, (c) what happens when the node or cluster that owns that address fails.
Core rule: BGP route advertisement is control plane, not datapath reachability. A healthy session never proves end-to-end connectivity [CIL-EX-E19].
Egress authorization (which destinations are allowed, the metadata endpoint) stays with `cilium-network-policy-review-agent`; this skill owns egress gateway SNAT, IP ownership, gateway selection and failure behavior. Encryption of these paths and `policy-default-local-cluster` semantics also go there. Choice of multi-cluster technology and CIDR planning stay with `kubernetes-network-architecture-review-agent`; this skill verifies Cilium-specific behavior of the chosen design. Upgrade, rollback and BGPv1 migration go to `cilium-lifecycle-readiness-agent`.
No live guard covers BGP CRDs, Helm values or cilium-config. Every BGP change is a human change-process handoff; this skill never proposes executing one. Peer-side router state is usually outside the evidence, so expect `needs-review` more often than in other Cilium reviews.
## When to use / when not to use
Use for BGP advertisements, LB IPAM pools, L2 announcement policies, egress gateway policies and ClusterMesh resilience claims. Do not use for pod-to-pod policy intent, datapath drop diagnosis (`cilium-dataplane-diagnostics-agent`) or generic Gateway API and DNS design.
## Required inputs
Cilium version and Helm values for kube-proxy replacement, masquerade, routing mode and identity mode; rendered BGP, pool, L2 and egress CRDs; Service manifests with traffic policies; node labels; peer router config or state; the failure claim to test. Missing fields lower `evidence_level` and mark affected conclusions `needs-review`.
## Workflow
1. Inventory boundary flows. For each, record direction, mechanism (BGP, L2, egress gateway, ClusterMesh) and the answers to (a), (b) and (c) above; leave unknowns as unknowns.
2. Gate on version and feature state. A BGPv1 `CiliumBGPPeeringPolicy` is removed in 1.19 [CIL-EX-E13]; LB pool `v2alpha1` is deprecated [CIL-EX-E23]; L2 announcements are beta [CIL-EX-E24]. Hand lifecycle questions off.
3. Resolve address ownership. Map each VIP, egress IP or pod CIDR to exactly one owner: pool overlap, sharing keys and selector-less pools [CIL-EX-E21] [CIL-EX-E22], L2 lease holder [CIL-EX-E25], gateway node selection [CIL-EX-E31].
4. Separate control plane from datapath. Session state, advertised routes and leases are claims about intent; require evidence of peer routing, firewall paths and traffic-policy behavior before accepting reachability [CIL-EX-E15].
5. Walk failure cases per owner: node down, agent restart, gateway loss, remote cluster unreachable. Compare timers (graceful restart, hold time, lease duration, cacheTTL) with the stated recovery objective [CIL-EX-E17] [CIL-EX-E25] [CIL-EX-E09].
6. Bound the blast radius: which services a pool can reach, which pods a gateway policy selects, which traffic leaves with a non-matching source, what a stale cache keeps serving.
7. Check incompatible combinations (egress gateway with ClusterMesh, L2 with eTP Local, KPR requirements) [CIL-EX-E30] [CIL-EX-E27].
8. Assign handoffs and the verdict. `approved` means no blocking condition seen in a bounded review, never execution consent. Unproven failover claims (egress gateway HA) stay `needs-review` [CIL-EX-E35].
## Reference loading
Load companion skill `cilium-evidence-boundary` first for command classes, evidence classes, verdict vocabulary, flow telemetry and sensitive-collection rules. Then load only the active branch.

| Reference | Load when |
| --- | --- |
| [bgp-control-plane.md](references/bgp-control-plane.md) | Sessions are Established but a VIP is unreachable, or BGP resilience is claimed. |
| [lb-ipam-and-l2-announcements.md](references/lb-ipam-and-l2-announcements.md) | Service IPs come from pools or are announced by ARP/NDP. |
| [egress-gateway.md](references/egress-gateway.md) | A fixed egress source IP or gateway failover is claimed. |
| [clustermesh-resilience.md](references/clustermesh-resilience.md) | Cross-cluster failover or disaster-recovery is claimed on ClusterMesh. |
| [evidence-ledger.md](references/evidence-ledger.md) | Verify a version-sensitive claim or an UNVERIFIED item. |

## Failure handling
Missing peer-side evidence: report the claim as `assumption`, not `observed`. Unknown Cilium version: state which version-gated facts could not be applied. Conflicting data (session Established but traffic fails): keep both and name the discriminating observation. Unavailable tools: all commands are evidence the user or an authorized operator collects; none are run by this review.
## Deliverable
Per boundary flow: answers to (a), (b), (c); verdict; `evidence_level`; claims classed `observed`, `derived`, `assumption` or `unknown`; one counterexample per consequential finding; the next discriminating observation with its command class; handoffs by agent id. State plainly that nothing was tested and that this is not certification.
