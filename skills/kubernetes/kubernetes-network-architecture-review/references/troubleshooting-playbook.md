# Connectivity Troubleshooting Playbook (read-only)

This playbook is for diagnosis. It does not mutate cluster state. Every command listed here either reads from the API server, reads from a node's kernel, or emits a controlled probe (a single ping, a single curl).

## Symptom 1 — Pod-to-pod connectivity fails

Hypothesis ladder, in order of cost:

1. **DNS** — does the source pod resolve the destination's Service or pod-IP DNS name? `kubectl exec <src> -- nslookup <name>`. If this fails, jump to the DNS playbook below.
2. **Service VIP exists and has endpoints** — `kubectl get svc <svc>` plus `kubectl get endpointslice -l kubernetes.io/service-name=<svc>`. Empty EndpointSlice → label selector mismatch.
3. **NetworkPolicy** — `kubectl get networkpolicies,ciliumnetworkpolicies,ciliumclusterwidenetworkpolicies -A -o yaml`. If any policy selects either pod, default-deny may be active. **At this point, hand off to `cilium-network-policy-review`** — that skill owns policy correctness.
4. **Underlying L3 reachability** — from the source node, `ip route get <dst-pod-ip>`; on Cilium, `cilium-dbg bpf endpoint list` and `cilium-dbg policy trace --src-pod ns/pod --dst-pod ns/pod`. On a Cilium cluster, datapath-level diagnosis beyond this step (non-policy drop reasons, conntrack or BPF map pressure, socket load balancing) belongs to `cilium-dataplane-diagnostics-agent`.
5. **MTU** — `kubectl exec <src> -- ping -M do -s 1450 <dst-pod-ip>` then increment payload until packets are dropped. The largest size that succeeds is the path MTU.
6. **Conntrack table full** — on the source node, `conntrack -L | wc -l` vs `sysctl net.netfilter.nf_conntrack_max`. Near 90% means new connections will be dropped.

## Symptom 2 — Pod can reach Service VIP but only some endpoints

Likely causes:

- **`externalTrafficPolicy: Local` or `internalTrafficPolicy: Local`** — `kubectl get svc <svc> -o yaml | grep -i trafficpolicy`. With `Local`, pods on nodes without a local endpoint silently fail.
- **EndpointSlice marked `Ready: false` or `Terminating: true`** — `kubectl get endpointslice -l kubernetes.io/service-name=<svc> -o yaml`. ProxyTerminatingEndpoints behavior depends on the kube-proxy / Cilium version.
- **kube-proxy stale rules** — `kubectl -n kube-system logs <kube-proxy-pod>` for sync errors. On iptables mode, full resync runs every `syncPeriod`; a stuck sync leaves stale rules pointing at deleted pods.
- **Topology hints with insufficient endpoints** — Auto mode does not populate hints when one zone has too few; behavior changes silently with deployment scale.

## Symptom 3 — Pod-to-external service intermittent failure

Hypothesis ladder:

1. **DNS** — 5-second timeouts indicate UDP packet drops; check NodeLocal DNSCache health and conntrack pressure on the node. CoreDNS request latency: `coredns_dns_request_duration_seconds`.
2. **Cloud metadata service** — `169.254.169.254` (AWS / Azure IMDS) or `metadata.google.internal` (GCP). **Flag as HIGH severity if any pod can reach it without an explicit NetworkPolicy deny.** Unblocked access lets any pod obtain the node's instance IAM credentials — credential-theft CVE class. Recommend IRSA (AWS), Workload Identity (GCP/Azure), or Pod Identity as the *first* remediation; the policy deny rule is delegated to `cilium-network-policy-review`, but the architectural posture finding must surface here, not be silently delegated.
3. **NAT Gateway port exhaustion** — AWS NAT GW supports ~55k simultaneous connections per destination IP/port. Pods making many short connections to the same external endpoint can exhaust ports; symptom is intermittent SYN drops. Solution: Network Load Balancer instead of NAT GW for that destination, or VPC endpoint.
4. **Egress firewall / WAF** — if a per-namespace egress proxy exists, check its logs.
5. **Path MTU** — see Symptom 1, step 5; path-MTU issues to external endpoints often blame TLS but the failure is L3.

## Symptom 4 — DNS-specific high latency

Hypothesis ladder:

1. **CoreDNS replicas saturated** — CPU throttling: `kube_pod_container_status_throttled_seconds_total{pod=~"coredns.*"}`. Insufficient replicas → autoscaler or manual bump.
2. **`ndots:5` amplification** — `kubectl exec <pod> -- cat /etc/resolv.conf` to confirm `ndots`. External hostnames generate 4× lookups before the absolute name. Mitigations are in the DNS-and-discovery reference.
3. **NodeLocal DNSCache OOM or absent** — `kubectl -n kube-system get pods -l k8s-app=node-local-dns -o wide` (or whatever the label is). If absent, every UDP packet adds conntrack pressure.
4. **Upstream DNS slow** — the cluster's `forward .` target. CoreDNS `coredns_forward_request_duration_seconds` segments by upstream.
5. **conntrack table full** — UDP DNS queries fill it.

## Symptom 5 — Intermittent stalls on large payloads only

Almost always MTU. The TCP handshake (small packets) succeeds, then the first response above the path MTU is dropped because Path MTU Discovery ICMP is filtered.

Diagnose:

- `kubectl exec <pod> -- ping -M do -s <bytes> <peer>` — find the MTU ceiling.
- Check overlay MTU vs node MTU vs underlay. Encapsulation overhead must be subtracted from node MTU when configuring the overlay device (`cilium_vxlan`, `flannel.1`, `cni0`).
- Cross-AZ on AWS — jumbo frames silently capped at 1500.

Fix: configure the CNI overlay MTU correctly and restart the agent DaemonSet (rolling). On Cilium, `cilium config view | grep -i mtu` and Helm value `MTU` (the Cilium agent reads MTU from the host but explicit config is safer).

## Symptom 6 — NodePort or LoadBalancer connection refused intermittently

- `externalTrafficPolicy: Local` — health-check on the LB must be on the right port, otherwise the LB sends to nodes with no local endpoint.
- IPVS mode and `sessionAffinity: ClientIP` — sessions persist past pod deletion until timeout.
- Cilium kube-proxy replacement transition — verify `cilium status` reports KPR `Strict` (or expected mode) on every node.

## Diagnostic commands (read-only, by component)

```shell
# API view
kubectl get nodes -o wide
kubectl -n kube-system get pods -o wide
kubectl get svc,endpointslices,ingress,gateway,gatewayclass,httproute -A

# CoreDNS
kubectl -n kube-system get cm coredns -o yaml
kubectl -n kube-system logs -l k8s-app=kube-dns --tail=200
kubectl exec <pod> -- cat /etc/resolv.conf

# kube-proxy
kubectl -n kube-system get cm kube-proxy -o yaml
kubectl -n kube-system logs -l k8s-app=kube-proxy --tail=200

# Cilium
kubectl -n kube-system exec ds/cilium -- cilium status --verbose
kubectl -n kube-system exec ds/cilium -- cilium-dbg bpf endpoint list
kubectl -n kube-system exec ds/cilium -- cilium-dbg policy trace --src-pod <ns>/<pod> --dst-pod <ns>/<pod>
hubble observe --from-pod <ns>/<pod> --to-pod <ns>/<pod> --last 200

# Node-level (use kubectl debug ephemeral container with --profile=netadmin — do NOT use --privileged)
# Example: kubectl debug node/<node-name> -it --image=registry.k8s.io/e2e-test-images/agnhost:2.45 --profile=netadmin
ip route
ip link show
conntrack -L | wc -l
sysctl net.netfilter.nf_conntrack_max
```

## When the playbook hits the scope boundary

- Policy correctness — hand off to `cilium-network-policy-review`.
- Mesh L7 problems — hand off to `istio-ambient-mesh-review`.
- Live mutation of policy — hand off to `kubernetes-live-network-policy-guard` or `kubernetes-live-mesh-policy-guard`.
- Pod-spec hostNetwork / capabilities issues — hand off to `kubernetes-pod-spec-review`.
- Cloud-side network problems (NAT GW, VPC peering, hosted DNS) — hand off to the cloud-provider network architect agent.
