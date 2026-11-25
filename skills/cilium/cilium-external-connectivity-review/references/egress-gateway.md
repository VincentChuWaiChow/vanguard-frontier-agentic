# Egress gateway review

Purpose: decide whether pods that must leave the cluster with a predictable source IP actually do, what drops when the gateway cannot serve, and whether a claimed gateway failover is evidenced. Applies to Cilium 1.20.5 documentation.

## Boundary with the policy skill

This reference owns SNAT, IP ownership, gateway selection and failure behavior. Which destinations a workload is allowed to reach (including the cloud metadata endpoint) is egress authorization and belongs to `cilium-network-policy-review-agent`. A review that approves "pods use the gateway" has not approved "pods may reach that destination".

## The problem this addresses

Partners and legacy firewalls allowlist source IPs. Default node SNAT presents every node IP, so a team asks Cilium to SNAT selected pods through designated gateway nodes with stable IPs. The resulting guarantee is only as strong as the selection rule, the requirements below, and the failure behavior, most of which the documentation states as limits rather than guarantees.

## Mechanism

- `CiliumEgressGatewayPolicy` is cluster-scoped. It selects pods (`selectors[].podSelector`, optional `nodeSelector` that cannot be used alone), destinations (`destinationCIDRs`, `excludedCIDRs`) and the gateway (`egressGateway` with `nodeSelector` and one of `interface`, `egressIP` or neither for the default-route interface; or an `egressGateways` list) [CIL-EX-E33].
- Connections from selected pods to the destination CIDRs are routed to a gateway node and SNATed to a predictable IP, IPv4 and IPv6 [CIL-EX-E29]. Internal cluster IPs (pods, nodes, API server) are excluded from SNAT [CIL-EX-E33].
- With a single `egressGateway` and several matching nodes, the first node in lexical order by name is selected [CIL-EX-E31]. Derived: a new matching node with a lexically earlier name changes the active gateway, which is a connection-breaking change (see below).
- With `egressGateways`, each source endpoint uses one gateway assigned by its CiliumEndpoint UID; adding or removing a matching node reassigns endpoints [CIL-EX-E32].

## Requirements and incompatibilities

| Item | Statement | Source |
| --- | --- | --- |
| Datapath | Requires both BPF masquerading (`bpf.masquerade=true`) and kube-proxy replacement (`kubeProxyReplacement=true`) | [CIL-EX-E29] |
| Enable flag | The exact Helm flag that enables the feature was not captured; do not quote one | [CIL-EX-E36] |
| Identity mode | Not compatible with `identityAllocationMode` kvstore; use `crd` | [CIL-EX-E30] |
| ClusterMesh | Not compatible; the gateway must be in the same cluster as the selected pods | [CIL-EX-E30] |
| CiliumEndpointSlice | Not compatible (GitHub issue 24833) | [CIL-EX-E30] |
| Pod start | A delay exists before policies apply to new pods; early traffic can leave with a non-matching source IP and is not redirected | [CIL-EX-E30] |

An architecture that pairs ClusterMesh with cross-cluster fixed egress IPs fails the documented compatibility rule; this is a `blocked` finding, with the design question handed to `kubernetes-network-architecture-review-agent`.

## Failure behavior

- No node matches the gateway selector: Cilium drops traffic matching the destination CIDRs [CIL-EX-E31].
- The egress IP or interface cannot be resolved: the gateway drops the traffic with reason "No Egress IP configured" [CIL-EX-E31].
- Changing the gateway node breaks existing egress connections, for both single- and multi-gateway policies [CIL-EX-E32].
- Cilium does not automatically respond to a change in the gateway node's network configuration; re-applying the policy forces reselection [CIL-EX-E32].
- Automatic failover or health-checked HA when a gateway node dies is not documented on the page reviewed. This is UNVERIFIED: absence of documentation is not proof of absence of behavior, but an HA claim needs evidence beyond the OSS docs, such as a recorded failover test or vendor documentation [CIL-EX-E35].
- Startup leak window: pods can emit traffic with the wrong source IP before policy applies [CIL-EX-E30]. Where the external firewall is default-deny this is a drop; where it permits node ranges it is a silent leak of the wrong identity.

## Legacy firewall implications

If the partner firewall allowlists only the gateway IP, a gateway change (planned drain, lexical reselection, node replacement) is an outage plus an IP change that the partner must also accept. If the firewall also still allows node CIDRs for resilience, the allowlist is wider than the claimed design. Record both as findings, not as preferences.

## Encryption note

WireGuard documentation states egress gateway replies are not encrypted when XDP acceleration is enabled [CIL-EX-E34]. Reply encryption is owned by `cilium-network-policy-review-agent`; only flag the interaction here.

## Evidence that discriminates

| Observation | Class | Separates |
| --- | --- | --- |
| `cilium-dbg bpf egress list` | [C2] agent pod | Whether the node holds the egress entries for the policy [CIL-EX-E33] |
| `kubectl get <CiliumEgressGatewayPolicy kind> -o yaml` | [C1] | Selectors, destination and excluded CIDRs, chosen gateway selector |
| `kubectl get nodes --show-labels` | [C1] | Which nodes match the gateway selector and their lexical order |
| Destination-side access log source IPs | outside the cluster; ask the partner | Which source IP the destination actually saw |

Collecting drop reasons and flows follows the companion skill `cilium-evidence-boundary`.

## Contradictory results

- Policy applied, destination still sees node IPs: likely the startup window, a destination not covered by `destinationCIDRs`, an `excludedCIDRs` hit, or an unmet requirement (BPF masquerade or KPR). Check requirements before suspecting the datapath.
- Policy applied, traffic dropped, "No Egress IP configured": the selected interface or `egressIP` is not present on the chosen node [CIL-EX-E31].
- Gateway moved cleanly in a test but "HA works" is claimed: a single successful switch, with connection resets, is consistent with the documented break-on-change rule and does not show seamless failover.

## Handoff boundaries

Live CiliumEgressGatewayPolicy changes go to `kubernetes-live-network-policy-guard-agent`; Helm and cilium-config changes have no guard and go to the human change process.

## Scenario

A payments processor allowlists one source IP for a merchant API. The team selects nodes labeled as gateways, runs two such nodes, and claims HA. The review confirms BPF masquerade and KPR are on, but finds the cluster is also joined to a ClusterMesh peer and that the policy selects pods in both clusters. Verdict: `blocked` on the incompatibility [CIL-EX-E30]. For the single-cluster remainder: for the single `egressGateway` form, lexical-first selection applies, so with two matching nodes the lexically first is the only active gateway, there is no documented failover, and a drain of that node breaks live connections to the merchant. `evidence_level: partial`, HA claim classed `unknown`. Next observation: a recorded gateway-node drain test [C5] (it interrupts service; run only through the human change process by an authorized operator), with destination-side logs.
