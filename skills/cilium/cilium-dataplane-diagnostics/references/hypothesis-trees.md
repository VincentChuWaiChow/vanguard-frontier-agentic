# Hypothesis trees

## Rule

One status code or one drop line is not a root cause. For each symptom, hold at least two competing hypotheses, record supporting and contradicting evidence, and pick the least-invasive check that separates them. Generic DNS, EndpointSlice and traffic-policy triage is owned by `kubernetes-network-architecture-review-agent` (hand off; that skill is not bundled with this one); this reference goes deeper into the Cilium datapath only. Command classes follow the companion skill `cilium-evidence-boundary`. Every command below is evidence an authorized operator may collect, never a step this skill takes.

## 1. Intermittent connection timeouts

| Hypothesis | Supports | Contradicts | Next least-invasive check |
| --- | --- | --- | --- |
| CT or NAT map full | `CT: Map insertion failed` drops; `cilium_bpf_map_pressure` at or above 1.0 for the CT or NAT map | Pressure well below 1.0 at failure time; no such drops | `cilium-dbg monitor --type drop` [C2]; metric series for the map [C1] |
| Backend churn or missing backend | `Service backend not found` drops; failures track rollouts | Stable backends and no such drops | `cilium-dbg service list` [C2] |
| Endpoint regeneration failing | `endpoint_regenerations_total` with `outcome=fail`; endpoints not in ready state | All endpoints ready; no failures | `cilium-dbg endpoint list` [C2] |
| MTU or PMTU black hole | Only larger transfers stall; see hypothesis tree 4 | Failures on small requests | See the routing and MTU reference |

Refs: [CIL-DP-E16] [CIL-DP-E18] [CIL-DP-E34] [CIL-DP-E35].

## 2. Service VIP or NodePort unreachable while pods are healthy

| Hypothesis | Supports | Contradicts | Next check |
| --- | --- | --- | --- |
| KPR not active on the affected node | `status` lacks `KubeProxyReplacement: True` there | True on that node | `cilium-dbg status` [C2] |
| Socket-LB bypassed in pod namespaces | `hostNamespaceOnly` set; sidecar present; failure only from meshed pods | Unmeshed pods also fail | Helm values review [C0] |
| Backend not programmed in LB map | VIP present in service list, no backends in LB list | Backends listed | `cilium-dbg bpf lb list` [C2] |
| DSR dispatch unsupported for routing mode | DSR on with opt dispatch under tunnel; or routers drop IP options | Native routing with supported mode | Values review [C0]; see the service load balancing reference |
| KPR toggled live | Failures began when kube-proxy was removed or KPR switched | No toggle in the window | Change timeline; co-route to `cilium-lifecycle-readiness-agent` |

Refs: [CIL-DP-E21] [CIL-DP-E23] [CIL-DP-E26] [CIL-DP-E27] [CIL-DP-E07].

## 3. Cross-node-only failure

| Hypothesis | Supports | Contradicts | Next check |
| --- | --- | --- | --- |
| Encapsulation or underlay path fault | Same-node works; tunnel mode; underlay UDP filtered | Native routing with working underlay | `hubble observe` flows by node via Hubble Relay [C2] (or [C2] when run inside the agent pod); per-node evidence |
| Encryption peer or key problem | WireGuard peer count differs from nodes minus one; port 51871 blocked | Peer count matches | `cilium-dbg status` [C2] encryption line |
| Asymmetric routing | Request seen at destination, reply not seen at source | Both directions captured | Paired captures; see the routing and MTU reference |
| Stale or unroutable IP | `Stale or unroutable IP` drops | Absent | `cilium-dbg monitor --type drop` [C2] |

Refs: [CIL-DP-E38] [CIL-DP-E09].

## 4. Large payloads fail, small succeed

| Hypothesis | Supports | Contradicts | Next check |
| --- | --- | --- | --- |
| Effective MTU too high for encap, encryption or DSR overhead | Handshake succeeds; failure above a size threshold; overhead layers enabled | Failure at all sizes | Compute expected MTU from supplied settings [C0] |
| ICMP fragmentation-needed filtered (PMTU black hole) | Underlay or cloud filters ICMP; no PMTU adaptation | PMTU adaptation observed | Controlled DF probe, [C3], authorized operator only |
| Policy L7 or proxy path | Failure only on L7-redirected flows | Failure on plain L4 flows too | Redirect verdicts in Hubble [C1] |

Refs: [CIL-DP-E29] [CIL-DP-E32].

## 5. Latency or throughput regression after a change

| Hypothesis | Supports | Contradicts | Next check |
| --- | --- | --- | --- |
| Host routing fell back to legacy | `status` no longer shows `Host Routing: BPF` | Still BPF | `cilium-dbg status` [C2] |
| Hubble or monitor overhead | Documented 1-15 percent range; recent enablement | Hubble unchanged | Values diff [C0] |
| Encryption enabled or changed | Overhead and CPU tied to enablement | No encryption change | Values diff [C0] |
| Map or regeneration pressure | Rising regeneration time stats; map pressure rising | Flat | Metric series [C1] |

Refs: [CIL-DP-E33] [CIL-DP-E34] [CIL-DP-E39]. Regression after an upgrade: co-route with `cilium-lifecycle-readiness-agent`.

## 6. Drops with non-policy reasons

Do not treat the reason alone as the cause. Use the table in the drop reasons reference to pick the owner, then:

- `CT: Map insertion failed` -> capacity reference.
- `No mapping for NAT masquerade` -> NAT map and masquerade configuration.
- `FIB lookup failed` or `Stale or unroutable IP` -> routing hypotheses (tree 3).
- `Policy denied` -> hand off to `cilium-network-policy-review-agent`.

Refs: [CIL-DP-E09] [CIL-DP-E10].

## Contradictory results

- Drops absent in monitor but symptom persists: the drop may occur outside the datapath, or the monitor missed events. Interpretation of lost events belongs to companion skill `cilium-evidence-boundary`.
- Pressure metric below 1.0 but `CT: Map insertion failed` seen: the metric may be a different map or a different time window; do not discard either.

## Scenario

A payments platform reports sporadic 30-second stalls to a database Service every few hours. Tree 1 gives two live hypotheses: CT full versus backend churn. The supplied metric shows the CT pressure ratio peaking at 0.97 and no `Service backend not found` drops. The next observation is the drop stream at the next stall, not a map resize.
