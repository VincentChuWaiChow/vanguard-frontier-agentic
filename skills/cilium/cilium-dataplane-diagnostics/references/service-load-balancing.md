# Service load balancing

## Problem

Whether a Service VIP, NodePort or external client reaches a backend depends on where and by whom the translation happens. Kube-proxy replacement (KPR), socket-LB scope, Maglev and DSR each change that. This reference verifies the active mode and the limits of each option. Generic Service, EndpointSlice and traffic-policy triage is owned by `kubernetes-network-architecture-review-agent` (hand off; that skill is not bundled with this one).

## Verify kube-proxy replacement

- The Helm default is `kubeProxyReplacement=false`, which only does per-packet ClusterIP load balancing in-cluster; with `true` the agent exits if kernel support is missing, and KPR depends on socket-LB [CIL-DP-E22].
- Documented verification: `cilium-dbg status` [C2] filtered for `KubeProxyReplacement` returns a line such as `True` with devices; `cilium-dbg status --verbose` [C2] shows `Status: True, Socket LB: Enabled` [CIL-DP-E21] [CIL-DP-E03].
- Check every node; one node's output does not describe the others.
- A single kernel minimum for KPR and socket-LB was not found on the pages reviewed; do not state one.

## Socket-LB and hostNamespaceOnly

Socket-LB translates Services at the socket level. With `socketLB.hostNamespaceOnly=true` the rewrite is skipped in pod namespaces and translation falls back to tc at the veth. Documented cases: Istio sidecar, KubeVirt, Kata, gVisor; the agent flag `--bpf-lb-sock-hostns-only` is described as required by service mesh [CIL-DP-E23].

Known issues: reverse-SK maps can fill (unconnected UDP, aborted UDP), making socket termination of connections to deleted backends unreliable; the page suggests `hostNamespaceOnly=true`. NFS or SMB mounts to a ClusterIP may break [CIL-DP-E24]. Reverse-SK map sizing is part of dynamic sizing; see the capacity reference [CIL-DP-E15].

Mesh interplay (sidecar redirect, ambient) hands off to `istio-maestro-agent`.

## Maglev

`loadBalancer.algorithm=maglev` applies only to external north-south traffic; east-west socket-LB traffic is not subject to it. `maglev.tableSize` and `maglev.hashSeed` apply; all agents must share the same seed; memory per node rises [CIL-DP-E25]. Consequence: intra-cluster backend choice cannot be explained by Maglev.

## DSR

`loadBalancer.mode=dsr` encodes Service info in the packet, lowering advertised MTU [CIL-DP-E26]. Dispatch support by routing mode:

| Dispatch | Native routing | Geneve tunnel | VXLAN tunnel |
| --- | --- | --- | --- |
| opt (IPv4 option or IPv6 extension header) | Works | Not supported | Not supported |
| Geneve | Works | Works | Not supported |

Source: [CIL-DP-E26]. Further documented limits: IP options can be dropped by some routers or clouds; cloud source and destination checks must be disabled; hybrid mode uses DSR for TCP and SNAT for UDP to avoid the MTU reduction [CIL-DP-E26]. MTU consequences: see the routing and MTU reference.

## Toggling kube-proxy replacement on a live cluster

The docs state that removing kube-proxy breaks existing service connections and stops service traffic until the replacement is installed, and that adding or removing eBPF KPR on a running cluster must be expected to break existing connections because the two NAT tables are not aware of each other [CIL-DP-E27]. Treat any KPR toggle as disruptive, not as a zero-downtime path. The change belongs to the human change process; co-route with `cilium-lifecycle-readiness-agent`.

## Discriminating evidence

| Question | Check | Class |
| --- | --- | --- |
| Is KPR active on node N? | `cilium-dbg status` | [C2] |
| Does the VIP have backends programmed? | `cilium-dbg service list` [C2] and `cilium-dbg bpf lb list` [C2] | [C2] |
| Are reverse-SK or LB maps under pressure? | `cilium_bpf_map_pressure` series by `map_name` | [C1] |
| Are Service drops present? | `cilium-dbg monitor --type drop` for `Service backend not found` | [C2] |

Cmdref pages for the listing commands exist in 1.20.5 [CIL-DP-E07].

## Traffic policy interaction

`externalTrafficPolicy` and `internalTrafficPolicy` change which backends are eligible. Their semantics, EndpointSlice readiness and topology behavior are owned by `kubernetes-network-architecture-review-agent` (hand off; that skill is not bundled with this one); supply their values as inputs and do not restate them here.

## Contradictory results

- VIP times out from meshed pods only: socket-LB scope hypothesis; a failure from unmeshed pods too weakens it.
- Backend listed in `bpf lb list` but unreachable: translation is likely fine; move to transport or policy hops.
- External client sees uneven distribution while in-cluster is even: consistent with Maglev scope, not with a fault.

## Scenario

A bank migrates a namespace to Istio sidecars on a KPR cluster with default socket-LB. The docs name Istio sidecars as a case needing the socket-LB bypass; the documented lever is `socketLB.hostNamespaceOnly=true` [CIL-DP-E23]; the reviewer lists it as a change-process item, notes that eBPF host routing is documented as incompatible with Istio, and hands mesh behavior to `istio-maestro-agent` [CIL-DP-E33].
