# Packet path model

## Problem

Datapath symptoms are diagnosed against the wrong hop when the reviewer assumes a stock path. A generic diagram is not evidence: the same pod-to-Service request crosses different hops depending on routing mode, kube-proxy replacement, socket-LB scope, host routing, encryption, egress gateway and L7 proxy settings. Derive the path from supplied settings first, then place each observation on a hop.

## Method

1. Read the supplied Helm values or agent config and `status --verbose` output. Record each key below as `observed`, or `unknown` if absent.
2. For each key, apply the hop effect in the table.
3. Write the resulting hop list for the specific flow (pod to ClusterIP, pod to remote pod, external client to NodePort, pod to external).
4. Mark each hop with the evidence that could show it failing (next section).

## Configuration keys and the hops they change

| Key | Effect on the path | Ledger |
| --- | --- | --- |
| Routing mode (tunnel or native) | Tunnel adds an encapsulation and decapsulation hop and reduces usable MTU; native relies on the underlay routing pod CIDRs. Default tunnel protocol is vxlan. | [CIL-DP-E29] [CIL-DP-E31] |
| `kubeProxyReplacement` | With `true`, Service translation is done by Cilium eBPF; with the default `false`, only per-packet ClusterIP load balancing in-cluster is done by Cilium. Agent exits if kernel support is missing when `true`. | [CIL-DP-E22] |
| `socketLB.hostNamespaceOnly` | `true` skips the socket-level rewrite in pod namespaces; translation moves to tc at the pod veth. Documented for Istio sidecars. | [CIL-DP-E23] |
| `bpf.hostLegacyRouting` and eBPF host routing | eBPF host routing needs eBPF KPR plus eBPF masquerade; it bypasses the host netfilter stack. `true` on hostLegacyRouting is the escape hatch. | [CIL-DP-E33] |
| `bpf.masquerade` | Required with KPR for eBPF host routing and for egress gateway. | [CIL-DP-E33] [CIL-DP-E37] |
| Encryption (WireGuard or IPsec) | Adds an encryption hop and per-packet overhead; in tunnel mode WireGuard is double-encapsulated. Same-node traffic is not encrypted. | [CIL-DP-E38] [CIL-DP-E29] |
| Egress gateway | Requires BPF masquerade and KPR; selected pod traffic to given CIDRs is redirected via a gateway node and SNATed. Ownership of that feature is `cilium-external-connectivity-review-agent`. | [CIL-DP-E37] |
| L7 proxy redirect | L4 flows covered by L7 rules are redirected to the node proxy; IPsec with L7 proxy needs transparent DNS-proxy mode. | [CIL-DP-E38] |
| DSR (`loadBalancer.mode`) | Service info is carried in the packet, lowering advertised MTU; dispatch mode depends on routing mode. See the service load balancing reference. | [CIL-DP-E26] |

## Conceptual path (illustration only)

```text
Illustration: pod -> ClusterIP, KPR=true, socket-LB cluster-wide, native routing, no encryption.
Not evidence. Replace hops with those your supplied configuration yields.

client pod
  -> (socket-LB rewrites ClusterIP to backend at connect())   [hop A: service translation]
  -> pod veth, endpoint BPF program                            [hop B: policy, CT lookup]
  -> host routing (eBPF host routing or legacy host stack)     [hop C: routing]
  -> underlay (native) or encapsulation device (tunnel)        [hop D: transport]
  -> remote node: decap, endpoint BPF program, delivery        [hop E: ingress policy, CT]
  -> backend pod
```

With `hostNamespaceOnly=true`, hop A moves to tc at the veth. With a tunnel, hop D becomes encapsulation plus decapsulation. With encryption, a hop is inserted before D. Each such change moves where a failure would appear.

## What each hop failing looks like

| Hop | Typical signal | Discriminating evidence (class) |
| --- | --- | --- |
| Service translation | Connection to VIP times out while direct backend IP works; `Service backend not found` drop | `cilium-dbg service list` [C2] versus `cilium-dbg bpf lb list` [C2] |
| Policy and CT at endpoint | `Policy denied` or `CT: Map insertion failed` drops | `cilium-dbg monitor --type drop` [C2] |
| Routing | `FIB lookup failed`, `Stale or unroutable IP` | `cilium-dbg status --verbose` [C2] for host routing and addresses |
| Transport | Small packets pass, large fail | PMTU probe; see the routing and MTU reference |
| Encryption | Cross-node loss while same-node works | `cilium-dbg status` [C2] encryption line and peer count |
| L7 proxy | Redirect verdicts, then proxy-side errors | Hubble flows; interpretation per companion skill `cilium-evidence-boundary` |

Drop-reason interpretation and owners: see the drop reasons reference.

## Contradictory results

- Direct backend IP works but VIP fails: points at hop A, not at the endpoint or transport. A single success does not clear policy, which acts on both paths.
- Same-node works, cross-node fails: points at hops D and E and encryption, not at Service translation.
- `status` shows KPR `True` on one node but not another: the per-node path differs; do not generalize one node's output.

## Scenario

An enterprise platform team runs native routing with KPR enabled and Istio sidecars. After enabling sidecars, the derived path must be checked for where Service translation happens. The docs name Istio sidecars as a case for skipping the socket rewrite in pod namespaces; the documented setting is `socketLB.hostNamespaceOnly=true`, which moves translation to tc at the veth [CIL-DP-E23]. Mesh-side behavior hands off to `istio-maestro-agent`; applying the Helm change is a human change process, as no guard exists for cilium-config changes.

## Version applicability

Values and flags are from the 1.20.5 documentation. Verify against the cluster's version before relying on a key.
