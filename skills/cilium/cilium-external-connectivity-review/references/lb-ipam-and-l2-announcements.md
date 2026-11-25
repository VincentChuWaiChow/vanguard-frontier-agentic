# LB IPAM and L2 announcements review

Purpose: decide who owns a Service IP handed out by Cilium, how that IP is made reachable on a flat L2 network, and what happens when the announcing node fails. Applies to Cilium 1.20.5 documentation.

## The problem this addresses

LB IPAM decides which IP a Service gets. It does not make the IP reachable: it works together with BGP Control Plane or L2 announcements to advertise it [CIL-EX-E22]. Two separate questions follow. First, is allocation correct and bounded (pools, selectors, sharing)? Second, is the announcement path correct under failure (lease holder, failover time, client ARP behavior)? Reviews that treat "the Service has an EXTERNAL-IP" as reachability skip the second question.

## LB IPAM mechanism

- LB IPAM is always enabled but dormant until the first `CiliumLoadBalancerIPPool` exists [CIL-EX-E23].
- Pool fields include `allowFirstLastIPs`, `serviceSelector` and `disabled`. A pool with no `serviceSelector` allocates to any service [CIL-EX-E23].
- The `cilium.io/v2` pool API is current; `v2alpha1` is deprecated in favor of `v2` per the 1.19 upgrade notes [CIL-EX-E23]. A manifest still on `v2alpha1` is a lifecycle item, not an outage.
- Overlap is a soft error: the last added pool is marked `Conflicting` (condition `cilium.io/PoolConflict`, reason `cidr_overlap`) and allocates nothing further [CIL-EX-E21]. Existing allocations are not described as removed on the pages reviewed; treat their fate as unknown.
- Sharing: services with the same `lbipam.cilium.io/sharing-key` share IPs while their ports do not conflict; conflicting ports receive additional IPs in the set. Cross-namespace sharing is off by default and needs `lbipam.cilium.io/sharing-cross-namespace` (comma-separated namespaces or `*`) on both services [CIL-EX-E22].

## Blast radius questions

| Finding | Why it matters | Source |
| --- | --- | --- |
| Pool without `serviceSelector` | Any Service in any namespace that asks for a LoadBalancer can take an address from it | [CIL-EX-E23] |
| Sharing key with `*` namespaces | One tenant's service can co-locate its ports on another tenant's IP | [CIL-EX-E22] |
| Overlapping pool CIDRs | The newer pool silently stops allocating; new services stay pending | [CIL-EX-E21] |
| Pool and BGP advertisement selectors differ | An allocated IP may never be advertised, or a pool may be advertised that was meant to be internal | [CIL-EX-E14] |

The last row is derived from the two selector mechanisms; confirm against the rendered manifests.

## L2 announcements mechanism

- The page is titled "L2 Announcements / L2 Aware LB (Beta)" and states that it is a beta feature [CIL-EX-E24]. Beta means a stability claim needs the owner to accept that status explicitly.
- Resource: `CiliumL2AnnouncementPolicy` with `interfaces` (regex list), `externalIPs` and `loadBalancerIPs`. It answers ARP and NDP queries [CIL-EX-E24].
- Requires `kubeProxyReplacement=true`; announcing devices must be listed in the `devices` option [CIL-EX-E24].
- Leader election is per service using Kubernetes Leases named `cilium-l2announce-<ns>-<svc>` in the Cilium namespace; selection is first come, first serve, so load can be asymmetric [CIL-EX-E25].
- Defaults: `l2announcements.leaseDuration` 15s and `leaseRenewDeadline` 5s; with defaults failover occurs between 10s and 20s [CIL-EX-E25].
- A new leader sends a gratuitous ARP; clients that do not accept it only re-query when their ARP table TTL expires, so their downtime can exceed the lease-based figure [CIL-EX-E26].

## Limitations that decide approval

- One node receives all ARP/NDP requests for a specific IP, so no load balancing happens before traffic reaches the cluster [CIL-EX-E27].
- The feature is incompatible with `externalTrafficPolicy: Local`: IPs may be announced from nodes without pods, causing drops [CIL-EX-E27]. A design pairing L2 announcements with Local to preserve client IPs fails its own premise.
- API-server load is approximately the number of services times 1/`leaseRenewDeadline` [CIL-EX-E28]. With the 5s default, 500 announced services is about 100 requests per second of lease renewals; the arithmetic is derived, the formula is documented. The retry period is a separate setting [CIL-EX-E25].

## Failure modes

- Leader node dies: failover in 10 to 20 seconds by default for new ARP answers; clients with stale ARP entries may wait longer [CIL-EX-E25] [CIL-EX-E26].
- API server slow or unavailable: leases cannot renew, which is a boundary-service outage path tied to control plane health; the pages reviewed do not describe the exact behavior, so mark it `unknown`.
- Leader node healthy but backend path broken: the lease proves the node is alive, not that backends are reachable.
- Asymmetric leadership: many service leaders concentrate on one node; a single node loss moves all of them at once.

## Evidence that discriminates

| Observation | Class | Separates |
| --- | --- | --- |
| `kubectl get <CiliumLoadBalancerIPPool kind> -o yaml` and read status conditions | [C1] | Conflicting pools, disabled pools |
| `kubectl get svc -A -o wide` | [C1] | Which services hold which IPs and traffic policies |
| `kubectl get lease -n <cilium namespace>` filtered for `cilium-l2announce-` names | [C1] | Current lease holder per service |
| ARP table on a client or switch | outside the cluster; ask the network owner | Whether the client followed the gratuitous ARP |

Command classes and sensitive-output rules follow the companion skill `cilium-evidence-boundary`.

## Contradictory results

- Lease holder is node A, traffic arrives on node B: leader and datapath node disagree; check devices list and interfaces regex before blaming the network.
- Service shows an IP but is unreachable and eTP is Local: consistent with the documented incompatibility [CIL-EX-E27].
- Pool shows `Conflicting` while services keep working: no further allocation happens from that pool [CIL-EX-E21], so new requests stall; whether existing assignments are kept is not documented on the pages reviewed. Treat it as a latent outage.

## Handoff boundaries

Pool or L2 policy changes have no live guard; hand off to the human change process. KPR enablement and Helm changes go to `cilium-lifecycle-readiness-agent`. Cloud LB, Gateway API and DNS design go to `kubernetes-network-architecture-review-agent`.

## Scenario

A retailer runs on-premises clusters with a flat VLAN and 120 LoadBalancer services across business units. A single catch-all pool has no `serviceSelector`, and the platform team adds a second pool for the finance unit with a CIDR that overlaps the first. The newer pool becomes `Conflicting`, so no further allocation happens from it and new finance services stay pending, and the team routes finance workloads to the catch-all pool. Review verdict: `blocked` for the pool design (selector-less pool plus overlap), `needs-review` for L2 failover, because the owner claims sub-5-second failover while the documented default is 10 to 20 seconds, plus unknown client ARP behavior. The next observation is the pool conditions via `kubectl get <CiliumLoadBalancerIPPool kind> -o yaml` [C1].
