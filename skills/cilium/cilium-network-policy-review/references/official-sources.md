# Official sources

Reference for the `cilium-network-policy-review` skill. Load it when you need the upstream page for a policy, ClusterMesh or encryption question. Material claims this skill makes about upstream behavior are recorded, with retrieval dates and status, in [evidence-ledger.md](evidence-ledger.md); this file is the reading list, not the evidence.

Prefer a versioned URL (`/en/v1.20/`, `/en/v1.19/`, `/en/v1.18/`) that matches the installed Cilium version. `/en/stable/` moves with each release and is not immutable evidence.

## Policy

- [Network Policy](https://docs.cilium.io/en/stable/network/kubernetes/policy/) — the three policy formats (`NetworkPolicy`, `CiliumNetworkPolicy`, `CiliumClusterwideNetworkPolicy`) and how Cilium applies them.
- [Policy language](https://docs.cilium.io/en/stable/security/policy/language/) — selectors, `toEndpoints`, `toCIDRSet`, `toFQDNs`, `toServices`, `toEntities`, and L7 rule syntax for the installed version.
- [Policy enforcement modes](https://docs.cilium.io/en/stable/security/policy/intro/) — `default`, `always` and `never`, and the identity-based model.
- [Kubernetes NetworkPolicy](https://kubernetes.io/docs/concepts/services-networking/network-policies/) — the upstream semantics that Cilium implements for standard `NetworkPolicy`.

## ClusterMesh policy

- [ClusterMesh policy](https://docs.cilium.io/en/stable/network/clustermesh/policy/) — cross-cluster selectors and `policy-default-local-cluster`.
- [Inspect policy-default-local-cluster](https://docs.cilium.io/en/stable/cmdref/cilium_clustermesh_inspect-policy-default-local-cluster/) — the command reference for listing policies whose scope would change; its classification is in [mcp-and-evidence.md](mcp-and-evidence.md).

## Transparent encryption

- [Transparent encryption overview](https://docs.cilium.io/en/stable/security/network/encryption/) — scope and strict-mode known issues.
- [WireGuard](https://docs.cilium.io/en/stable/security/network/encryption-wireguard/) — coverage table, node-to-node encryption and limitations.
- [IPsec](https://docs.cilium.io/en/stable/security/network/encryption-ipsec/) — key distribution and rotation.

## Observation

- [Hubble](https://docs.cilium.io/en/stable/observability/hubble/) — flow observation. What flows can and cannot prove is owned by the companion skill `cilium-evidence-boundary`.

## Version and change history

- [Upgrade notes](https://docs.cilium.io/en/stable/operations/upgrade/) — version-specific policy changes for the current minor; use the versioned page for earlier minors.
- [Security advisories](https://github.com/cilium/cilium/security/advisories) — policy-bypass advisories and their affected ranges; applicability is owned by `cilium-lifecycle-readiness-agent`.

## Out of scope for this skill

- Egress gateway SNAT, gateway selection and failure: `cilium-external-connectivity-review-agent`.
- Tetragon runtime policies: a separate project with no owner in this catalog.
