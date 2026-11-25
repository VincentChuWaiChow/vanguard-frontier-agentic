# ClusterMesh resilience review

Purpose: verify Cilium-specific behavior of an already chosen ClusterMesh design and test whether cross-cluster failover or disaster-recovery claims hold. Choice of multi-cluster technology and CIDR planning stay with `kubernetes-network-architecture-review-agent`; do not redo them here. Applies to Cilium 1.20.5 documentation.

## The problem this addresses

ClusterMesh makes remote services reachable and shares identity. Teams then claim "if cluster A is lost, cluster B serves the traffic". Successful cross-cluster connectivity is not proven application-level disaster recovery: it shows only that packets and service backends can be exchanged. Data replication, state, capacity and client behavior are separate questions this skill does not certify.

## Prerequisites to verify (not to design)

| Prerequisite | Statement | Source |
| --- | --- | --- |
| Name and ID | Unique human-readable name (max 32 chars, lowercase alphanumeric ends, dashes allowed) and numeric cluster ID 1-255 | [CIL-EX-E01] |
| Cluster count | Default max connected clusters 255; `clustermesh.maxConnectedClusters=511` lowers max cluster-local identities from 65535 to 32767; same value in every cluster; set at install and not changed on existing clusters | [CIL-EX-E02] |
| Addressing | PodCIDRs non-conflicting and unique across clusters and nodes; node InternalIP connectivity between clusters; same datapath mode; Cilium versions at most one minor apart | [CIL-EX-E03] |
| Native routing | Additionally a native routing CIDR covering all PodCIDRs and pod-to-pod IP connectivity across clusters on all ports | [CIL-EX-E03] |
| ID change | Changing cluster ID or name with running workloads requires restarting all workloads because the ID feeds the security identity | [CIL-EX-E04] |

Two items are not stated on the pages reviewed and must not be asserted: the cluster ID range when `maxConnectedClusters=511` [CIL-EX-E37], and which IPAM modes ClusterMesh supports [CIL-EX-E38]. A design relying on IDs above 255 or a specific IPAM mode needs owner evidence.

## Mechanism

- Each cluster runs `clustermesh-apiserver` with etcd, apiserver and kvstoremesh containers. etcd holds local state exposed to remote clusters over mTLS; apiserver synchronizes local state into it; kvstoremesh synchronizes remote cluster state into the local cluster [CIL-EX-E06].
- `clustermesh.apiserver.kvstoremesh.enabled` defaults to true and is described as deprecated as an option, with KVStoreMesh always enabled once the option is removed [CIL-EX-E06].
- The apiserver service type can be LoadBalancer (recommended), NodePort (needs stable node IPs) or ClusterIP (must be routable between clusters); a boundary-crossing address owned by a load balancer or node falls back to questions (a) to (c) in the main skill [CIL-EX-E11].
- All clusters in a mesh form a single trust domain. The checks cannot fully prevent a compromised cluster's control plane from propagating malicious information; connect only mutually trusted clusters with equivalent security postures [CIL-EX-E05]. A mesh between production and a weaker development cluster contradicts that statement.

## Global services and affinity

- A global service is the same name and namespace in each cluster with annotation `service.cilium.io/global: "true"`, which implicitly sets `service.cilium.io/shared: "true"`. Setting `shared: "false"` exposes remote endpoints without sharing local ones [CIL-EX-E07].
- Global NodePort and host-network clients load-balance across remote backends only with `kubeProxyReplacement=true`; for host-namespace clients `socketLB.enabled=true` with `socketLB.hostNamespaceOnly=true` also works [CIL-EX-E40].
- `service.cilium.io/affinity` values are `none` (default), `local` and `remote`. `local` uses remote endpoints only if all local backends are unavailable or unhealthy; `remote` prefers remote [CIL-EX-E08]. The failover claim is therefore only as good as backend health reporting: a local backend that is up but broken keeps traffic local.
- `cilium-dbg service list --clustermesh-affinity` [C2] shows the effective affinity on a node [CIL-EX-E08].

## Failure mode that breaks DR claims

By default, if a remote cluster becomes unreachable Cilium retains last-known service information, which can send traffic to stale or unreachable backends. `clustermesh.cacheTTL` defaults to `0s`, meaning the cache is never revoked. Set above zero (for example `15m`) and connectivity not restored in time, Cilium revokes cached data and stops load-balancing to backends in the unreachable cluster; backends resume when connectivity returns [CIL-EX-E09].

Consequences for review:

- With the default, a lost cluster is not removed from load balancing by the mesh; failure detection falls to connection-level behavior and the application.
- With a non-zero TTL, removal is delayed by the TTL and then abrupt; a 15m example means up to 15 minutes of stale backends. Compare with the stated recovery objective.
- Neither setting proves that the surviving cluster has capacity or data to serve.

## Evidence that discriminates

| Observation | Class | Separates |
| --- | --- | --- |
| Owner-supplied mesh connectivity status from the Cilium CLI (exact command not captured in the fact sheets; classify per `cilium-evidence-boundary` before collecting) | [C2] conservative | Remote cluster connectivity and sync state per cluster |
| `cilium-dbg service list --clustermesh-affinity` | [C2] agent pod | Local versus remote backend selection |
| Helm values for `clustermesh.*` | [C2] if from sysdump or Helm values (sensitive); prefer a supplied excerpt | Configured cacheTTL, maxConnectedClusters, KVStoreMesh |
| Annotated Service manifests in every cluster | [C1] | Global, shared, affinity annotations and name/namespace parity |
| A recorded failure drill with timestamps | requires an authorized operator [C4] | Time to stop sending to the lost cluster |

Hubble cross-cluster flow interpretation and sensitive collection follow the companion skill `cilium-evidence-boundary`.

## Contradictory results

- Mesh connectivity status healthy yet requests fail after a cluster loss: stale cache (TTL 0s) is the first candidate [CIL-EX-E09].
- Affinity `local` set but traffic crosses clusters: local backends were unhealthy or absent; check readiness, not the annotation [CIL-EX-E08].
- Global annotation present in one cluster only: the service is not global across the mesh; both sides need the same name and namespace and the annotation [CIL-EX-E07].

## Handoff boundaries

- `policy-default-local-cluster` and policy that selects endpoints across clusters (label `io.cilium.k8s.policy.cluster`) are owned by `cilium-network-policy-review-agent`. Default behavior changed in 1.19 and policies are not distributed across clusters [CIL-EX-E10]. Mention and hand off only.
- ClusterMesh peer Secret and Gateway API live changes go to `kubernetes-live-network-architecture-mutation-guard-agent`.
- Upgrades, `maxConnectedClusters` and cluster ID changes go to `cilium-lifecycle-readiness-agent` as change-process items; no guard covers Helm changes.
- Egress gateway is incompatible with ClusterMesh [CIL-EX-E30]; see the egress reference.

## Scenario

A regulated insurer runs two clusters and states that if region A is lost, region B absorbs claims-portal traffic through a global service with affinity `local`. The review verifies unique names and IDs, non-overlapping PodCIDRs and mTLS through load balancers. It finds `cacheTTL` unset, so the default `0s` applies [CIL-EX-E09]. When region A is lost, region B clients keep a backend list that includes region A backends. Separately, the database in region B is a read replica, so even correct routing does not serve writes. Verdict: `needs-review`, `evidence_level: partial`; cross-cluster connectivity classed `observed`, the DR claim classed `assumption`. Next observation: a timed failure drill run by an authorized operator under the change process, and the owner's decision on `cacheTTL` with its TTL-delay tradeoff.
