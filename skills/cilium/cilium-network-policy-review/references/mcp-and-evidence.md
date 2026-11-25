# Evidence Path and Tooling

## Evidence path

1. Prefer live cluster evidence only when the host exposes an authorized read-only tool AND the user has confirmed the target cluster/context identity; otherwise review sanitized YAML statically.
2. Fall back to the official Cilium documentation (docs.cilium.io) for policy syntax, CRD schema, and ClusterMesh semantics when live inspection is unavailable.
3. Ask only for sanitized policy YAML, agent policy-get output, Hubble flow snippets, or ClusterMesh status output when current-state proof matters.
4. Label conclusions as `live evidence`, `documentation-based`, `sanitized user evidence`, or `inference`.

## Useful live-evidence commands

Commands are evidence the user or an authorized operator may collect; the agent never runs them.
Command and evidence classes follow the companion skill `cilium-evidence-boundary`.

Class: [C1] observational, reads authorized API state.

```text
kubectl get networkpolicies,ciliumnetworkpolicies,ciliumclusterwidenetworkpolicies -A -o yaml   [C1]
kubectl -n kube-system get pods -l k8s-app=cilium -o name   [C1]
```

Class: [C1] egress gateway policies are read here only to confirm egress authorization; collision, SNAT and failover review is owned by `cilium-external-connectivity-review-agent`.

```text
kubectl get ciliumegressgatewaypolicies -A -o yaml   [C1]
```

Class: [C2] runs inside the Cilium agent pod (`cilium-dbg` is the agent debug binary in 1.20.5) and needs exec access.

```text
kubectl -n kube-system exec <cilium-pod> -- cilium-dbg status   [C2]
kubectl -n kube-system exec <cilium-pod> -- cilium-dbg policy get   [C2]
kubectl -n kube-system exec <cilium-pod> -- cilium-dbg endpoint list   [C2]
```

Class: [C1] Hubble flows via Relay; flows may contain sensitive metadata, sanitize before sharing.

```text
hubble observe --from-namespace <ns> --verdict DROPPED --last 1000   [C1]
hubble observe --fqdn <fqdn> --verdict DROPPED --last 1000   [C1]
cilium clustermesh status   [C1]
cilium clustermesh inspect-policy-default-local-cluster --all-namespaces   [C1]
```

## Cilium state to confirm before review

- Cilium version (agent image tag, confirmed by the operator) — L7 policy support, ClusterMesh policy defaults and CRD versions change across releases; the 1.19 and 1.20 notes in `references/default-deny-rollout.md` apply only above those versions.
- Proxy enabled — required for L7 policy fields (`toPorts.rules.http`, `toPorts.rules.dns`). Kafka-aware policy and `l7proto` rules were removed in 1.20.
- ClusterMesh enabled (`cilium clustermesh status` [C1]) — multi-cluster policies are evaluated differently when ClusterMesh is up.
- `policy-default-local-cluster` setting (per cluster; Helm `clustermesh.policyDefaultLocalCluster`, default true since 1.19) — changes whether policies match cross-cluster identities by default.
- IPAM mode — affects CIDR-based policy assumptions; egress gateway IP planning is owned by `cilium-external-connectivity-review-agent`.
- Hubble enabled — required for flow observability and policy debugging.
- Tetragon installed (separate but Cilium-affiliated) — runtime security; relevant when reviewing combined eBPF posture.

## Sanitization rules

- Never request kubeconfig contents, ClusterMesh peer Secrets, or Cilium agent tokens.
- Replace identifiable cluster IDs, peer cluster URLs, public egress IPs (when sensitive), and namespace names with placeholders unless the user provides them.
- Do not print Cilium agent service account tokens.
