# CNI migration and disruptive changes

## Problem

Two different risks share this file. First, moving a running cluster from another CNI to Cilium. Second, changing a setting on a running Cilium cluster that the documentation treats as install-time. Both can sever existing connections or leave policy unenforced, and neither is a normal upgrade.

Scenario: a retailer runs Calico-routed clusters with NetworkPolicy and BGP to the data center. It wants a node-by-node move to Cilium and, later, to switch one cluster from tunnel to native routing and enable kube-proxy replacement.

## Documented migration approach [CIL-LC-E16]

- Node-by-node live migration with dual overlays. Cilium is installed in secondary mode; a per-node `CiliumNodeConfig` (`cilium.io/v2`, name `cilium-default`, in kube-system, selecting nodes by a migration label) switches the CNI node by node.
- Per node: cordon and drain, label, restart the Cilium pod, reboot the node, verify, uncordon. Finally remove the old CNI (an optional second reboot).

## Requirements and migration values [CIL-LC-E17]

- A distinct new cluster CIDR; cluster-pool IPAM; a distinct overlay protocol or port (the example uses a different VXLAN port); an existing plugin that uses the Linux routing stack (Flannel, Calico, AWS-CNI are named).
- Migration Helm values include `operator.unmanagedPodWatcher.restart=false`, `cni.customConf=true`, `cni.uninstall=false`, `policyEnforcementMode: never` and `bpf.hostLegacyRouting: true`. After migration these are flipped back, optionally enabling eBPF host routing with a brief per-node interruption, and the CiliumNodeConfig is deleted.

## Untested combinations and the security exposure window [CIL-LC-E18]

The guide lists what it was not tested with: BGP-based routing; changing IP families; migrating from Cilium chained mode; an existing NetworkPolicy provider.

During migration, Cilium NetworkPolicy and CNP enforcement is disabled. This is a security exposure window: pods on migrated nodes can reach destinations that policy would deny, for as long as migration values stay in place. The guide suggests considering temporarily deleting existing NetworkPolicies. Any such deletion is a [C5] mutation that needs its own authorization and a restore copy; this skill never recommends it unprompted.

In the scenario, BGP routing and an existing NetworkPolicy provider are both in the untested list, so a production migration is not supported by evidence: verdict `needs-review` at best, `blocked` without a trial.

The guide strongly recommends a trial migration on a test or lab cluster [CIL-LC-E18]. A readiness review should require a trial that mirrors the untested features (BGP and policy provider), with the exposure window measured.

## Disruptive in-place changes

| Change | Documented stance | Verdict direction |
| --- | --- | --- |
| IPAM mode | Do not change except by a documented migration; may cause persistent connectivity disruption; safest is a fresh cluster; the only online path is cluster-pool to multi-pool [CIL-LC-E19] | any other transition `blocked` |
| `clusterPoolIPv4MaskSize` | Changing it is not possible; do not change existing `clusterPoolIPv4PodCIDRList` elements, add new ones [CIL-LC-E20] | change request `blocked` |
| kube-proxy replacement added or removed | Two documented statements: removing kube-proxy stops service traffic until the Cilium replacement is installed; adding or removing the eBPF replacement on a running cluster breaks existing connections because the two NAT tables are unaware of each other [CIL-LC-E21] | `needs-review` with a drain plan |
| `maxConnectedClusters` | Set once at install; changing may cause disruption and incorrect policy enforcement; supported values 255 and 511; all clusters in the mesh must match [CIL-LC-E23] | `blocked` |
| Cluster ID or name | Changing on a cluster with running workloads requires restarting all workloads, since the ID feeds security identity [CIL-LC-E24] | `blocked` without restart plan |
| Tunnel to native routing | No in-place procedure or warning found on the pages reviewed | UNVERIFIED, `needs-review` |

Absence of documentation for tunnel to native is not proof either way. A routing-mode change is in the same family as the documented disruptive changes, but the reviewed pages state no supported procedure, so the reviewer requires a vendor or upstream statement plus a trial cluster result [CIL-LC-E22].

The ClusterMesh ID range under 511 is not stated on the pages reviewed; a plan that assigns IDs above 255 is `unknown` [CIL-LC-E23].

## Evidence that discriminates

| Question | Evidence | Class |
| --- | --- | --- |
| Current CNI and IPAM mode | `kubectl get configmap -n kube-system -o yaml` for the Cilium ConfigMap [C1] | observational |
| Is kube-proxy still present | `kubectl get ds -n kube-system` listing the DaemonSets present [C1] | observational |
| Which nodes are migrated | `kubectl get nodes --show-labels` [C1] | observational |
| Is policy being enforced on a migrated node | traffic test that policy must deny [C3] | active diagnostic, authorized by the cluster owner |

## Sequencing and ownership (derived)

1. Trial on a lab cluster that includes the untested features; record the enforcement-off window per node.
2. Name the window owner and compensating controls (for example restricting who can schedule onto migrated nodes); controls are the owner's decision.
3. Migrate a canary node pool, verify, then continue; keep the old CNI until every node is verified.
4. Post-migration: flip migration values back and delete the CiliumNodeConfig as the guide describes [CIL-LC-E17]. Flipping values is a [C4] change and deleting the CiliumNodeConfig is a [C5] change, each with its own authorizer under the human change process.

## Contradictions

- Migration values present on a node set believed finished: enforcement may still be disabled; `needs-review`.
- Pod CIDRs of the new Cilium range overlap the old: violates the distinct CIDR requirement; `blocked`.

## Handoff

Execution: human change process. CNI choice and target design: kubernetes-network-architecture-review-agent. Policy parity after migration: cilium-network-policy-review-agent. ClusterMesh peering: cilium-external-connectivity-review-agent.
