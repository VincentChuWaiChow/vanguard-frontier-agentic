# BGP Control Plane review

Purpose: decide whether a Cilium BGP design delivers traffic to a service VIP and keeps doing so through node and agent failure. Scope is Cilium BGP Control Plane as documented for 1.20.5; version differences are called out where the fact sheets give them.

## The problem this addresses

A BGP session in `Established` state and an advertised prefix are statements about the routing control plane. They say nothing about whether a packet sent to the VIP reaches a backend. The documentation is explicit that BGP Control Plane does not program the datapath and must not be used to establish reachability within the cluster [CIL-EX-E19]. Reviews that accept "peers are up" as proof of connectivity are the most common false approval in this area.

## Mechanism from first principles

1. The BGP speaker is embedded in the Cilium agent on each node selected by the cluster config [CIL-EX-E17].
2. Configuration is split into four v2 resources: `CiliumBGPClusterConfig` (instances, peers, `nodeSelector`), `CiliumBGPPeerConfig` (shared peering settings), `CiliumBGPAdvertisement` (what is injected into the routing table) and `CiliumBGPNodeConfigOverride` (per-node overrides). The operator creates a per-node `CiliumBGPNodeConfig` whose status holds live state [CIL-EX-E12].
3. Advertisement types are `PodCIDR`, `CiliumPodIPPool` and `Service`; for `Service`, the addresses are `LoadBalancerIP`, `ClusterIP` or `ExternalIP` chosen by a label selector [CIL-EX-E14].
4. The router learns the prefix and forwards to a node. Whether that node then delivers to a backend is the datapath, governed by traffic policy and kube-proxy replacement, not by BGP.

## Version applicability

- `CiliumBGPPeeringPolicy` (BGPv1) was deprecated by 1.18 and its control plane was removed in 1.19; migration to `cilium.io/v2` resources is required before upgrading [CIL-EX-E13]. A cluster still using it on 1.19 or later is a lifecycle problem: hand off to `cilium-lifecycle-readiness-agent`.
- The release in which the v2 CRDs first became stable is not confirmed on the pages reviewed [CIL-EX-E39]. Do not state it.
- Overlapping advertisement matches: before 1.18 the last match won; now overlap is supported and handled per attribute, with communities unioned [CIL-EX-E14].

## Configuration that changes behavior

| Setting | Effect | Source |
| --- | --- | --- |
| `externalTrafficPolicy: Cluster` | VIP advertised unconditionally from all selected nodes | [CIL-EX-E15] |
| `externalTrafficPolicy: Local` | Advertisement stops on nodes with no local endpoint | [CIL-EX-E15] |
| `internalTrafficPolicy` | Governs `ClusterIP` advertisement; `ClusterIP` ignores eTP, `ExternalIP` and `LoadBalancerIP` ignore iTP | [CIL-EX-E15] |
| `--enable-no-service-endpoints-routable` (default true) | With eTP Cluster and all backends gone the VIP stays advertised; false withdraws it. With eTP Local advertisement stops regardless | [CIL-EX-E16] |
| `gracefulRestart.enabled`, `restartTimeSeconds` | Peer keeps routes while the agent restarts; without it an agent restart removes advertised routes | [CIL-EX-E17] |
| `holdTimeSeconds`, `keepAliveTimeSeconds` | Hold default 90s; minimums 3 and 1 | [CIL-EX-E17] |
| `authSecretRef` | TCP MD5 (RFC 2385); secret key `password`, namespace `kube-system` by default | [CIL-EX-E20] |

## Failure modes

- Blackhole window: with eTP Cluster and graceful restart, a node that goes down can blackhole traffic until the restart timer expires; the operation guide gives 120s by default [CIL-EX-E17]. Compare that with any stated recovery objective.
- BFD is not supported, so detection of a dead path rests on hold time (default 90s) and not on sub-second probes [CIL-EX-E17].
- Retained advertisement on empty services: with eTP Cluster and the default flag, a service with zero backends still attracts traffic to every selected node [CIL-EX-E16].
- Silent non-configuration: status conditions `cilium.io/NoMatchingNode`, `MissingPeerConfigs`, `ConflictingClusterConfig` on the cluster config and `cilium.io/MissingAuthSecret` on the peer config mean part of the design is not applied [CIL-EX-E18].
- Reporting disabled: `bgpControlPlane.statusReport.enabled=false` removes the status the review depends on [CIL-EX-E18].
- Only address families Cilium is configured for can be advertised [CIL-EX-E19].

## Evidence that discriminates

| Observation | Class | What it separates |
| --- | --- | --- |
| `cilium bgp peers` | [C2] conservative: reaches the agents and shows peer addressing | Session state versus no session |
| `cilium bgp routes advertised ipv4 unicast` | [C2] same reason | Whether the VIP is actually advertised |
| `cilium bgp routes available ipv4 unicast` | [C2] same reason | Routes Cilium holds versus routes it sends |
| `kubectl describe ciliumbgpnodeconfigs <node>` | [C1] | Per-node status and conditions |
| `cilium-dbg bgp route-policies` | [C2] agent pod, debugging | Policy applied to advertisements |
| Peer router route table and forwarding state | outside the user's cluster; request from the network owner | Whether the peer installed and uses the route |

Logs are tagged `subsys=bgp-control-plane` (agent) and `subsys=bgp-cp-operator` (operator) [CIL-EX-E18]. Command classes follow the companion skill `cilium-evidence-boundary`.

## Contradictory results

- Session Established, route advertised, VIP unreachable: control plane is healthy; the fault is in peer forwarding, an intermediate firewall, traffic-policy delivery (eTP Local with no local endpoint on the chosen node) or the datapath. Hand the datapath branch to `cilium-dataplane-diagnostics-agent`.
- Route advertised in Cilium, absent on the peer: peer-side import policy, MD5 mismatch or max-prefix limits are candidates; none are visible from the cluster. Mark `needs-review`.
- Status shows a condition but peers look fine: the condition describes a different node set or config; check `nodeSelector` coverage before trusting the peer list.

## Handoff boundaries

- BGP CRD, Helm and cilium-config changes have no live guard; every one is a human change-process handoff and this review never proposes executing it.
- Egress authorization, encryption and policy semantics belong to `cilium-network-policy-review-agent`. Upgrades and BGPv1 migration belong to `cilium-lifecycle-readiness-agent`.

## Scenario

A bank advertises payment-gateway `LoadBalancerIP` addresses to two top-of-rack routers. The platform team reports all sessions Established and the VIP advertised from three nodes, and asks for approval. The Service uses `externalTrafficPolicy: Local`. The review finds that advertisement is conditional on a local endpoint, and only one of the three nodes runs a backend. Peer evidence shows ECMP across all three advertising nodes before the change window, which is inconsistent with the Local rule and indicates the status snapshot is stale. Verdict: `needs-review`, `evidence_level: partial`. The next observation is the peer router forwarding table plus `cilium bgp routes advertised ipv4 unicast` [C2] collected by an authorized operator at the same timestamp. The graceful restart and 120s blackhole figure are recorded as the recovery bound until the owner shows a different configured value.
