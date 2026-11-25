# Breaking changes 1.19 and 1.20

## Problem

Release notes list changes that silently break a cluster after a correct-looking upgrade: removed CRDs, removed rule types, changed defaults. A reviewer needs, for each item, the evidence that proves the cluster is clear of it before the upgrade, not after.

Scenario: a bank's platform team moves 1.18.x to 1.19.x and then 1.19.x to 1.20.5. Its estate has legacy BGP peering objects, a Kafka-aware policy written years ago, TLSRoute objects behind a Gateway, and a ClusterMesh with mixed-cluster policy selectors.

This file is a dated snapshot of the 1.19 and 1.20 upgrade notes as retrieved 2026-11-25. It must be refreshed every minor release: the next minor adds its own Action Required items, and a 1.21 plan must not reuse this checklist as complete.

## Mechanism

Upgrade notes split into Action Required (do before the upgrade), behavior changes (re-validate after) and informational items. Removal items are discontinuities: the object or option stops working at the first agent or operator of the new minor. Only evidence collected before the change can show the cluster is clear.

## 1.19 checklist (crossing 1.18 to 1.19) [CIL-LC-E08]

| Item | Evidence that proves it handled | Class |
| --- | --- | --- |
| CiliumBGPPeeringPolicy CRD and BGPv1 control plane removed; migrate to `cilium.io/v2` CiliumBGPClusterConfig, CiliumBGPPeerConfig, CiliumBGPAdvertisement, CiliumBGPNodeConfigOverride before upgrading | a `kubectl get` listing of CiliumBGPPeeringPolicy objects returns none [C1], and the v2 objects exist and are reviewed | observed |
| CiliumLoadBalancerIPPool `v2alpha1` deprecated for `v2` | Manifests in Git use `v2`; live objects read at `v2` | observed |
| ClusterMesh policy selectors default to local cluster only (`policy-default-local-cluster`, enabled by default in 1.19) | Mesh-wide policies reviewed for cross-cluster selectors; semantics handed to cilium-network-policy-review-agent | derived |
| `mesh-auth-enabled` disabled by default | Values diff shows explicit setting if mutual auth is relied on | observed |
| `FromRequires` and `ToRequires` rejected | Search of policy source finds none | observed |
| `**` DNS wildcard matches multiple subdomains | Policy review of FQDN patterns | derived |
| Removed options: node-port, host-port and external-ips enable flags (use kube-proxy replacement), IPv4 egress gateway flag, PCAP recorder, custom calls | Values diff and agent arguments contain none | observed |
| IPsec with kube-proxy replacement and BPF masquerade auto-enables BPF host routing and needs a kernel fix | Per-pool kernel and distro patch evidence if that combination is in use | assumption until shown |

## 1.20 checklist (crossing 1.19 to 1.20) [CIL-LC-E07]

| Item | Evidence that proves it handled | Class |
| --- | --- | --- |
| Mutual Authentication (Beta) deprecated; Ztunnel transparent encryption suggested | Decision recorded if mutual auth is used | assumption |
| Envoy Go Extensions (proxylib) and Kafka-aware policy removed; remove `rules` with `kafka`, `l7` or `l7proto` from CNP and CCNP before upgrading | Search of all CNP and CCNP manifests and live objects finds no such rules | observed |
| CiliumNodeConfig `cilium.io/v2alpha1` removed; use `cilium.io/v2` (available since 1.16) | Live and Git objects all `v2` | observed |
| CNI spec version default now `1.0.0` | Confirm runtime and chained CNI plugins accept it: unknown for third-party plugins | unknown |
| Gateway API minimum v1.6.1; TLSRoute users must install the Experimental TLSRoute CRD, since Standard v1.6 makes existing TLSRoute objects effectively disappear | Installed CRD version and channel recorded; TLSRoute objects exported to a backup before any CRD change | observed |
| Egress rules combining `toServices` and `toPorts` deny when the Service is missing or has no backends | List of such rules with Service existence check; semantics to policy agent | derived |
| Removed options: `clustermesh.enableMCSAPISupport`, `encryption.ipsec.interface`, legacy `encryption.strictMode.*` (use `encryption.strictMode.egress.*`), node-port mode and algorithm flags, the API-server flag, the docker plugin, deprecated CiliumNode fields | Values diff contains none of these keys; strict-mode keys migrated | observed |
| With kube-proxy replacement and SocketLB disabled, or `socketLB.hostNamespaceOnly=true`, NodePort access from pods is load-balanced at the client pod | Client egress and backend ingress policy reviewed to allow it | derived |

Informational in 1.20 [CIL-LC-E07]: empty CNP and CCNP rejected at admission by CEL; ClusterMesh auto certificates default to one year, and Helm-generated certificates renew only on chart re-render; certgen enforces CA validity; BGP peer and route REST and agent CLI access deprecated in favour of the shell-based BGP commands. A mesh whose certificates were rendered once will carry a renewal date to track.

## Applying it to the scenario (derived sequencing)

1. Stay on 1.18 until the BGPv1 objects are replaced by v2 objects and the replacement is reviewed by the external-connectivity owner; only then plan 1.19.
2. Land on the latest 1.19 patch, observe, and only then start the 1.20 work: Kafka and l7proto rules removed, TLSRoute backup taken, Gateway API CRDs at the required version and channel.
3. Each hop is its own change with its own rollback statement; do not combine hops [CIL-LC-E01].

## Failure modes

- Objects removed with the CRD: deleting or replacing a CRD deletes its objects. Back up TLSRoutes and BGP objects as manifests first (handled as [C1] reads; the backup content may carry sensitive routing data).
- Gateway API CRD change is not a Cilium Helm change: a mutating change to those CRDs goes to kubernetes-live-network-architecture-mutation-guard-agent when live, else the change process.
- Counting items: the checklist is not exhaustive for your environment; absence from this table is not proof of no impact.

## Contradictory results

- Live objects show zero BGPv1 policies but Git still contains them: the next sync reintroduces them; treat as unhandled until Git is clean.
- Values diff shows no removed keys but the managed chart overlays its own: ask for the rendered values.

## Handoff

Policy default semantics (`policy-default-local-cluster`, toServices denial): cilium-network-policy-review-agent. BGP v2 design correctness: cilium-external-connectivity-review-agent. Gateway API generic issues: kubernetes-network-architecture-review-agent.
