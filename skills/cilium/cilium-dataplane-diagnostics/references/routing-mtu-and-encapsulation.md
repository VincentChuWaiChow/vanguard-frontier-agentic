# Routing, MTU and encapsulation

## Problem

Packets that fit on the pod interface may not fit once encapsulation, encryption or DSR metadata is added. The signature is that small requests succeed and large transfers stall. This reference computes the expected MTU budget from supplied settings and separates MTU faults from policy drops. Generic MTU probing steps are in the playbook of owned by `kubernetes-network-architecture-review-agent` (hand off; that skill is not bundled with this one); this reference covers Cilium-specific overhead and discrimination.

## Mechanism

Cilium auto-detects the MTU of the underlying devices and uses jumbo frames automatically if configured; the agent flag `--mtu` overrides auto-detection [CIL-DP-E28]. The pod MTU is the device MTU minus the overhead of each enabled layer. If the pod MTU is higher than the real path allows, oversized packets are dropped on the wire and the sender only adapts if path MTU discovery works.

## Overheads from the fact sheet (1.20.5 source constants)

| Layer | Overhead | Ledger |
| --- | --- | --- |
| Tunnel, IPv4 outer | 50 B (outer IP, UDP, VXLAN, inner Ethernet) | [CIL-DP-E29] |
| Tunnel, IPv6 outer | 70 B | [CIL-DP-E29] |
| IPsec | 77 B, plus auth key size above 16 | [CIL-DP-E29] |
| WireGuard | 95 B constant | [CIL-DP-E29] |
| DSR Geneve option | 12 B | [CIL-DP-E29] |
| IPIP | 20 B / 48 B | [CIL-DP-E29] |

WireGuard with encapsulation subtracts WireGuard plus tunnel overhead [CIL-DP-E29]. The default tunnel protocol is vxlan; the code uses one 50 B constant [CIL-DP-E31].

| Conflict | Side A | Side B | Treatment |
| --- | --- | --- | --- |
| WireGuard overhead | Code comment itemizes 80 B total | Constant is 95 B | Report the constant, 95 B, and cite [CIL-DP-E30] |

Budget pattern (supplied numbers only):

```text
expected_pod_mtu = device_mtu - sum(overhead of each enabled layer)
example form:  device_mtu - 50 (tunnel IPv4) - 95 (WireGuard)   when both are enabled
compare:       expected_pod_mtu  vs  observed pod MTU (supplied)
```

If the observed pod MTU exceeds the budget, the MTU hypothesis is supported; if it equals the budget, the hypothesis is weakened but not refuted (the underlay may still be smaller than the supplied device MTU).

## Path MTU discovery

Cilium enables Linux packetization-layer PMTUD (RFC 4821) by default for pod endpoints; it can cost initial MSS, and it is disabled with `pmtuDiscovery.packetizationLayerPMTUDMode=disabled`. Helm `pmtuDiscovery.enabled` (ICMP fragmentation-needed replies) defaults to false [CIL-DP-E32]. A PMTU black hole appears when the underlay or cloud drops ICMP fragmentation-needed and the sender never learns the limit.

## Signature and discrimination

- MTU fault: TCP handshake and small exchanges work; transfers fail above a size threshold; the failing flows are not selected by identity or port; drops with the reason codes 136 (`Fragmentation needed`) or 15 (`MTU error message`) are signals that need a check; the sheet gives the codes only, so they do not by themselves discriminate an MTU fault from other causes [CIL-DP-E10].
- Policy drop: failures are size-independent; the drop line says `Policy denied` [CIL-DP-E09].

| Check | Discriminates | Class |
| --- | --- | --- |
| Compare expected and observed pod MTU from supplied values | Overhead mis-budget | [C0] |
| `cilium-dbg monitor --type drop` for reason codes 136 or 15 | Whether the listed signal codes appear; a signal to check against pod and device MTU, not proof | [C2] |
| `cilium-dbg status --verbose` for routing, host routing and encryption | Which layers are active | [C2] |
| Controlled DF-bit probes at increasing sizes | Path MTU ceiling | [C3], authorized operator |
| `hubble observe --verdict DROPPED` flows around the failing transfer, via Hubble Relay | Whether drops are visible at all [CIL-DP-E41] | [C2] ([C2] when run inside the agent pod) |

Interpretation of Hubble completeness (ring buffer, lost events) is owned by the companion skill `cilium-evidence-boundary`.

## Asymmetric routing

In native routing the underlay must route pod CIDRs both ways. If the request reaches the destination but replies leave through another path, stateful translation and CT see only one direction. Evidence: paired observations at source and destination nodes; a request visible at the destination with no reply visible at the source. One-sided observation does not establish asymmetry.

## Contradictory results

- Small packets succeed, large fail, and drop reasons are empty: oversize packets may be lost on the wire before Cilium sees them; absence of a drop line does not refute MTU.
- Pod MTU equals budget yet failures persist: the underlay MTU in the supplied data may be wrong, or an intermediate hop (cloud, firewall) is smaller. Hand underlay questions to the cloud network owner.
- XDP on AWS ENA needs a lowered MTU in the documented example [CIL-DP-E42]; this is not generalizable beyond that example.

## Scenario

A manufacturer enables WireGuard on a tunnel-mode cluster and sees file uploads over 1 MB stall while API calls pass. Budget: device_mtu minus tunnel overhead minus WireGuard overhead (double encapsulation per [CIL-DP-E38]). The supplied pod MTU is higher than that result, supporting the MTU hypothesis; the next observation is the drop stream during an upload, noting whether reason codes 136 or 15 appear as signals to check against the MTU budget. Changing the MTU is a human change process item.
