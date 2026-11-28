# Routing table

Reference for the `cilium-review-routing` skill. Load it when selecting owners, ordering them, or routing out of the board.

## Board owners

Each row is one decision. Route by the decision the user needs, not by the resource named in the request.

| Decision | Owner | Typical signals |
| --- | --- | --- |
| Do policy and transport encryption enforce the intended connectivity boundary? | `cilium-network-policy-review-agent` | CiliumNetworkPolicy, CiliumClusterwideNetworkPolicy, NetworkPolicy, default-deny, toFQDNs, toCIDRSet, L7 rules, metadata endpoint, `Policy denied` drops, WireGuard or IPsec coverage claims, strict mode, `policy-default-local-cluster` |
| What explains the observed datapath behavior, and is capacity adequate? | `cilium-dataplane-diagnostics-agent` | timeouts, non-policy drop reasons, conntrack, BPF map pressure, Service VIP or NodePort unreachable with healthy pods, kube-proxy replacement, socket load balancing, MTU, fragmentation, latency or throughput regression, endpoint regeneration |
| Is traffic crossing the cluster boundary reachable, owned, bounded and failure-isolated? | `cilium-external-connectivity-review-agent` | BGP peers or advertisements, LB-IPAM pools, L2 announcements, egress gateway SNAT or gateway node failure, ClusterMesh, global services, affinity, cluster ID |
| Is the lifecycle change ready to enter an authorized change process? | `cilium-lifecycle-readiness-agent` | upgrade, rollback, version skew, kernel requirements, Helm value changes, deprecations, CNI migration to Cilium, IPAM mode change, kube-proxy replacement toggle, security advisory applicability |

## Symptoms with more than one owner

These are why the router exists. Route all plausible owners, ordered so that each one's output narrows the next.

| Symptom | Order | Why this order |
| --- | --- | --- |
| "Service unreachable" | dataplane, then policy or external-connectivity | The drop reason or missing route decides whether policy (`Policy denied`) or the cluster edge (route not advertised) owns the rest. |
| "Broke after the change or upgrade" | lifecycle, then dataplane, then policy | The change's version notes and disruption mode bound which hypotheses are live. |
| "Cross-cluster traffic fails" | external-connectivity, then policy, then dataplane | Mesh prerequisites and endpoint state first; `policy-default-local-cluster` semantics second; encapsulation and MTU last. |
| "Is our zero-trust posture real?" | policy, then external-connectivity | Authorization and encryption coverage first; egress gateway and mesh trust second. |
| "Can we migrate this cluster to Cilium?" | `kubernetes-network-architecture-review-agent` for the target, then lifecycle | The target design is not this board's decision; transition safety is. |

## Out-of-board owners

| Request | Owner |
| --- | --- |
| Which CNI, overlay versus native routing as a design choice, Pod and Service CIDR planning, generic Gateway API, DNS | `kubernetes-network-architecture-review-agent` |
| Istio, including Istio's Gateway API implementation | `istio-maestro-agent` |
| Runtime detection rules | `falco-runtime-threat-rules-review-agent` |
| Tetragon tracing or enforcement policies | No owner in this catalog; say so and do not improvise |
| Cost, chargeback, unit economics | `finops-maestro-agent` |
| Control evidence lineage for an audit | `atlassian-control-evidence-lineage-agent` |
| Incident recurrence prevention | `atlassian-incident-prevention-agent` |
| Drafting Jira tickets or Confluence pages | `jira-ticket-authoring` and `confluence-document-authoring` skills |

## Rules

- At most four specialists per request. If more seem necessary, the request is several requests; split it.
- Never dispatch every specialist "to be safe". An unnecessary specialist adds cost and a verdict that has to be reconciled.
- A specialist's refusal or `needs-review` is a result, not a routing failure.
- Never route a live change to a specialist. Live changes follow [live-and-out-of-board-handoff.md](live-and-out-of-board-handoff.md).
