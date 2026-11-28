# Version and kernel compatibility

## Problem

Whether a target version is supported, and whether the nodes can run it, is decided by three facts that are often recorded in three different places: the supported branch list, the per-node kernel, and the Kubernetes version.

Scenario: an enterprise runs three node pools (a RHEL 8.10 pool on kernel 4.18, a 5.14 pool, and a recent 6.x pool for latency-sensitive workloads) and wants to enable a kernel-gated feature cluster-wide while upgrading.

## Supported branches (snapshot 2026-11-25)

- Current stable is 1.20.5 (2026-11-17). Maintained branches: 1.20 (1.20.5), 1.19 (1.19.11), 1.18 (1.18.17) [CIL-LC-E09].
- The community maintains the last three minor releases; older are EOL. Feature releases come about every six months; patches around the middle of each month [CIL-LC-E10].
- Security support: main, 1.20.x, 1.19.x and 1.18.x supported; versions below 1.18.0 are not [CIL-LC-E11].
- The tag and date come from the repository README on main; the releases page was not reachable on the review date, so this is a dated snapshot to re-verify [CIL-LC-E09].

A current version below 1.18.0 is outside security support: a lifecycle plan from it has no supported advisory-fix route inside its own minor and still must go one minor at a time.

## Kernel

- Minimum: Linux kernel 5.10 or equivalent, with 4.18 on RHEL 8.10 given as the equivalent example [CIL-LC-E12].
- Feature-gated minimums from the requirements table: Multicast (Beta) on AMD64 5.10; IPv6 BIG TCP 5.19; Multicast (Beta) on AArch64 6.0; IPv4 BIG TCP 6.3; netkit device mode 6.8 [CIL-LC-E13].
- From the tuning page: bandwidth-manager BBR for pods needs 5.18 [CIL-LC-E29]. A single kernel minimum for kube-proxy replacement and socket load balancing is not stated on the pages reviewed: unknown.

### Heterogeneous kernels (derived)

Cilium agents run per node, so each node's kernel decides what its agent can enable, and a cluster-wide setting that needs a newer kernel meets the oldest node pool. The lowest kernel in the fleet governs any cluster-wide feature enablement. Derived from [CIL-LC-E12] and [CIL-LC-E13]; the pages do not state a per-node degrade behavior for every feature. Where a feature fails closed (for example, an agent exiting when required support is missing, documented for kube-proxy replacement [CIL-LC-E21]), one old pool can stop agents from starting there.

In the scenario, enabling IPv4 BIG TCP (6.3) is `blocked` for the whole cluster by the 4.18 and 5.14 pools; it may only be scoped to the 6.x pool if a per-node mechanism is evidenced, otherwise `needs-review`.

## Kubernetes

Cilium 1.20.5 e2e-tests Kubernetes 1.33, 1.34, 1.35 and 1.36. Older versions not listed have no Cilium support; newer depend on Kubernetes backward compatibility [CIL-LC-E14]. A cluster on a version outside that list is `needs-review` at best.

## ClusterMesh version skew

Clusters in a mesh must differ by at most one minor Cilium release, and must use the same datapath mode [CIL-LC-E15]. Upgrading cluster A two minors ahead of cluster B breaks that rule; upgrade clusters so the skew never exceeds one minor, which usually means alternating clusters one minor at a time. Derived sequencing.

## Review decision table

| Finding | Verdict direction | Claim class |
| --- | --- | --- |
| Current version below 1.18.0 | `blocked` for assuming a supported fix path; plan stepwise | derived |
| Target skips a minor in either direction (upgrade or rollback more than one minor away) | `blocked` until stepwise [CIL-LC-E01]; a patch change within the same minor and a rollback to the immediately preceding minor are in scope, not blocked | observed |
| Any node kernel below 5.10 and not an RHEL 8.10 4.18 build | `blocked` [CIL-LC-E12] | observed |
| Kubernetes version outside 1.33 to 1.36 for 1.20.5 | `needs-review` [CIL-LC-E14] | observed |
| Feature needs kernel above the lowest pool | `blocked` cluster-wide, `needs-review` if scoped per pool | derived |
| Mesh peers would differ by more than one minor | `blocked` [CIL-LC-E15] | observed |
| Install method is a managed distribution | `needs-review`, vendor statement required | assumption |

## Failure modes

- A feature is enabled in the values diff on the strength of a test pool's kernel, then rolled to the fleet: agents on older pools cannot honor it. Cheapest catch is the per-pool kernel column, collected before the diff is reviewed.
- Kubernetes is upgraded first and lands outside the e2e-tested range of the Cilium version in place: upgrade order of the two is a plan decision that needs both ranges written down side by side.
- Patch lag: the cluster is on an old patch of a maintained minor, and the supported-branch table is read as if the cluster were on the newest patch. Derived: branch support does not retroactively cover an old patch; the upgrade guide asks for the latest patch first [CIL-LC-E02].

## Evidence that discriminates

| Question | Evidence | Class |
| --- | --- | --- |
| Kernel per node | `kubectl get nodes -o wide` kernel column [C1] | observational |
| Kubernetes version | `kubectl version` [C1] | observational |
| Agent version per node | `kubectl get pods -n kube-system -o wide` plus image tags of the Cilium pods [C1] | observational |

## Contradictions

- Node image says "5.14" but vendor kernel is a backport build: the docs give only the RHEL 8.10 equivalent; any other backport claim is an assumption and needs the vendor's statement.
- Managed distribution pins its own Cilium: upstream support windows may not apply; ask the vendor.

## Handoff

Which kernel-gated features to turn on (BIG TCP, netkit) is a design choice: kubernetes-network-architecture-review-agent. Mesh topology: cilium-external-connectivity-review-agent.
