# Encryption Coverage

Decision this reference supports: does the supplied evidence substantiate the claimed
transport-encryption posture for a named traffic class? Command classes, evidence classes and
verdict vocabulary are owned by the companion skill `cilium-evidence-boundary`.

## Core rules

- "Encryption enabled" in configuration or in a Helm values file is intent, not coverage
  evidence. Coverage is a per-traffic-class claim: which source/destination pairs, on which
  path, in which routing mode, with which acceleration features.
- Encryption is not authentication or authorization. Encrypted transport says nothing about who
  may connect; zero trust needs the policy boundary and the encryption boundary, assessed
  separately. Cilium's mutual authentication (Beta) is deprecated in 1.20 and the upgrade notes
  suggest considering ztunnel transparent encryption instead [CIL-NP-E15].
- Scope on the overview page: transparent encryption of Cilium-managed host traffic and traffic
  between Cilium-managed endpoints, using IPsec, WireGuard or ztunnel (ztunnel is labelled
  Beta) [CIL-NP-E01]. Traffic Cilium does not manage is outside that scope.

## What WireGuard encrypts by default (1.20.5)

- Each node creates its own key pair and publishes the public key in the
  `network.cilium.io/wg-pub-key` annotation on its CiliumNode; tunnels are established between
  all known nodes and UDP port 51871 must be open between nodes [CIL-NP-E02].
- Default coverage is pod-to-remote-pod only. Packets destined to the same node they were sent
  from are not encrypted, and the page states this is intended [CIL-NP-E02]. A claim of
  "all pod traffic is encrypted" is therefore false by design for same-node pairs.
- In tunnel routing mode traffic is double-encapsulated (VXLAN or Geneve plus WireGuard)
  [CIL-NP-E02].
- Node-to-node encryption (`encryption.nodeEncryption=true`) is labelled beta; it adds
  node-to-node, pod-to-node and node-to-pod. Control-plane nodes (label
  `node-role.kubernetes.io/control-plane`) opt out automatically to avoid a key-update
  bootstrapping lockout; overriding with `node-encryption-opt-out-labels` is not recommended
  [CIL-NP-E03].

## Documented gaps in WireGuard coverage

| Gap | Condition | Source |
| --- | --- | --- |
| N/S load balancer traffic not encrypted when an intermediate node redirects to another node | LoadBalancer and NodePort XDP acceleration, or DSR in non-Geneve dispatch mode | [CIL-NP-E04] |
| Egress gateway replies not encrypted | XDP acceleration enabled | [CIL-NP-E04] |
| Pairs not in the "Which traffic is encrypted" table | Assumed unencrypted | [CIL-NP-E05] |
| External client to first node | Not encrypted | [CIL-NP-E05] |
| Packet drops while the WireGuard device is reconfigured | Endpoint add/remove, node updates (issue 33159) | [CIL-NP-E06] |

Rule derived from the table: a traffic class that the auditor names but that does not appear in
the page's table is unencrypted until evidence shows otherwise. The table's L7 proxy and egress
gateway rows were not parsed cell by cell for this skill; read the live page before asserting
either path is covered [CIL-NP-E05].

## Initial-packet window and strict mode

- New remote endpoints can be sent unencrypted until learned: Cilium may send the initial
  packets unencrypted [CIL-NP-E07].
- Strict egress (`encryption.strictMode.egress.enabled` with `encryption.strictMode.egress.cidr`)
  enforces that traffic exiting a node to the CIDR is always encrypted. The pod CIDR must be
  IPv4; IPv6 traffic is not protected and can be leaked [CIL-NP-E07].
- Strict ingress (`encryption.strictMode.ingress.enabled`) drops cluster-internal ingress that
  did not arrive through the WireGuard tunnel. It requires WireGuard (not IPsec), is not
  supported when chaining Cilium on other CNI plugins, does not protect WireGuard node-to-node
  encryption, and relies on BPF attached to pod-routable interfaces, so unmanaged interfaces
  could inject unencrypted traffic [CIL-NP-E08].
- The legacy `encryption.strictMode.*` keys are removed in 1.20; use the `.egress.*` form
  [CIL-NP-E15].
- Compensating control documented on the overview page: egress policy that disallows
  `reserved:world` or allows only trusted `toCIDR`, `toCIDRSet` or `toFQDN` [CIL-NP-E07].

## IPsec and key handling

- Keys live in the Kubernetes Secret `cilium-ipsec-keys` in the Cilium namespace, format
  `key-id algorithm PSK size`; a `+` after the key-id is strongly recommended and forces
  per-tunnel keys [CIL-NP-E09].
- Rotation is manual: patch the secret with a monotonically increasing KEYID (1-15, rolling over
  15 to 1). Do not rotate during upgrades, keep node versions equal, and do not change between
  algorithms with different auth key lengths. With ClusterMesh apply the rotation to all
  clusters and consider raising `ipsec-key-rotation-duration` [CIL-NP-E09].
- With the L7 proxy enabled, IPsec needs `--dnsproxy-enable-transparent-mode=true` [CIL-NP-E09].
- WireGuard keys rotate automatically per node; a regenerated key is a problem only if the node
  cannot reach the API server to publish it. No operator-driven rotation procedure is documented
  on the page reviewed [CIL-NP-E02].
- ClusterMesh with WireGuard: clustermesh-apiserver forwards public keys and UDP 51871 must be
  open between nodes of different clusters [CIL-NP-E10].
- Collection hazard: advisory GHSA-gj49-89wh-h4gj reports that cilium-bugtool output contained
  the WireGuard private key before the fixed releases; rotate keys if bugtool or sysdump
  archives were shared [CIL-NP-E11].

## Evidence that discriminates coverage claims

Class: [C2] runs inside the Cilium agent pod; the operator must authorize and collect.

```text
kubectl -n kube-system exec <cilium-pod> -- cilium-dbg status   [C2]
kubectl -n kube-system exec <cilium-pod> -- cilium-dbg encrypt status   [C2]
```

- A WireGuard node reports `Encryption: Wireguard [cilium_wg0 (Pubkey..., Port: 51871, Peers:
  N)]`; peers should equal nodes minus one [CIL-NP-E12]. A peer count lower than that is
  partial coverage.
- IPsec: `cilium-dbg encrypt status` [C2] prints "Encryption: IPsec", decryption interfaces, keys in
  use, max sequence number and errors [CIL-NP-E09].

Class: [C2] packet capture needs node access and captures payload-adjacent data.

```text
tcpdump -n -i cilium_wg0   [C2] shows the plaintext inner packets
tcpdump -n -i <iface> esp   [C2] shows ESP on the underlay for IPsec
```

- `hubble observe --encrypted` filters flows by encryption state [C1] [CIL-NP-E13]. Whether a
  flow is marked encrypted depends on the Hubble version and its telemetry completeness, which
  `cilium-evidence-boundary` owns; treat it as corroboration, not proof.
- Contradictions: config says WireGuard on but peers are fewer than nodes minus one means
  incomplete mesh; capture on the underlay shows cleartext for a pair the table lists as
  encrypted means a gap condition (XDP, DSR, strict mode off) applies; a clean status with
  cleartext captured only for same-node pods is expected behavior.

## Scenario: auditor asks for evidence that PCI cardholder traffic is encrypted in transit

1. Pin the claim: which pods exchange cardholder data, same node or different nodes, via
   Service, NodePort, LoadBalancer or egress gateway, and which Cilium version and routing mode.
2. A values file with WireGuard enabled is intent. Ask for per-node status output showing
   peers equal to nodes minus one, and an underlay capture (or a flow listing) for a
   representative cardholder pair on different nodes.
3. Enumerate unencrypted-by-design paths: same-node pods, external client to first node, and
   any LoadBalancer path with XDP acceleration or DSR, plus egress-gateway replies with XDP.
   Each needs a compensating control (application TLS, node pinning with policy, disabling XDP)
   or an explicit exception.
4. Check the initial-packet window: is strict egress on, is the cardholder network IPv4 only
   given IPv6 leaks, and is strict ingress applicable (WireGuard, not chained)?
5. Separate the authorization claim: encryption does not show that only the payment service may
   reach the database; that is a policy question answered with the default-deny review.
6. Verdict is `needs-review` with `evidence_level: partial` until a capture or status output
   supports each named path; `approved` is never PCI certification.

## Handoffs

- Enabling, rotating or disabling encryption, Helm changes or cilium-config changes: no guard
  exists; hand to the human change process, and to `cilium-lifecycle-readiness-agent` for
  readiness.
- Egress gateway gateway-node and reply path behavior, ClusterMesh peering and BGP: to
  `cilium-external-connectivity-review-agent`.
- Why a packet drops or where it is dropped: to `cilium-dataplane-diagnostics-agent`.
