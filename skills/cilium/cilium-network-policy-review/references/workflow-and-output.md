# Workflow and Output Contract

## Workflow

### Step 1 — Identify the policy format

Cilium supports three formats with different scopes and capabilities:

1. **`NetworkPolicy`** (`networking.k8s.io/v1`) — Kubernetes-native, namespace-scoped, L3/L4 only.
2. **`CiliumNetworkPolicy`** (`cilium.io/v2`, "CNP") — namespace-scoped, L3-L7 (HTTP, DNS; Kafka only before its removal in 1.20), FQDN matching, ICMP, identity-based selectors via Cilium endpoint identities.
3. **`CiliumClusterwideNetworkPolicy`** (`cilium.io/v2`, "CCNP") — cluster-wide, same capabilities as CNP, applies across all namespaces.

A namespace can have multiple policies of all three formats simultaneously. The effective policy is the **union** of allows: any policy that allows traffic permits it.

Reference: [Network Policy overview](https://docs.cilium.io/en/stable/network/kubernetes/policy/).

### Step 2 — Verify default-deny posture in the affected namespace

Deny-by-default semantics (derived/unverified; not re-verified in this skill's ledger, confirm against the policy language page for the installed version): a pod selected by at least one ingress policy is expected to be deny-by-default for ingress, and likewise for egress; a pod selected by no policy is expected to be allow-all.

Critical findings:

- Removing the only ingress `NetworkPolicy` selecting a workload — the workload becomes reachable from any pod, any namespace, any cluster (if ClusterMesh).
- Adding a workload to a namespace that has no namespace-wide `default-deny` policy — the new workload is allow-by-default.

Recommended baseline: a `default-deny-all` `NetworkPolicy` per namespace plus explicit `CiliumNetworkPolicy` resources that allow specific intra-namespace and cross-namespace flows.

### Step 3 — Audit L7 rules and Envoy proxy requirement

`CiliumNetworkPolicy` and `CiliumClusterwideNetworkPolicy` support L7 rules via Cilium's embedded Envoy:

- `toPorts.rules.http` — method, path, host, header matching.
- `toPorts.rules.kafka` — Kafka-aware policy (removed in Cilium 1.20; remove `kafka` and `l7proto` rules before upgrading). Flag any such rule on a 1.20 target as an upgrade blocker.
- `toPorts.rules.dns` — DNS FQDN allowlist for egress.

L7 rules require the Envoy proxy to be enabled. Without Envoy, policy with L7 fields **either fails admission or is enforced only at L3/L4**, depending on Cilium version (not confirmed against the 1.20 docs; treat as unknown and confirm before relying on L7).

Stress-tests:

- L7 HTTP rule on a namespace where pods talk via gRPC (HTTP/2) — how an HTTP path matcher applies to HTTP/2 traffic is not re-verified in this skill's ledger; confirm against the policy language page for the installed version before relying on it.
- L7 DNS rule with FQDN `*.example.com` — `*.example.com` wildcard support is not re-verified in this skill's ledger (derived/unverified; confirm against the policy language page for the installed version), and the destination port still matters, so verify port 53 UDP/TCP is allowed at L4. From 1.19 the `**` DNS wildcard matches multiple subdomains, so a rule using it is broader than before [CIL-NP-E17].

Reference: [L7 Policy in Cilium](https://docs.cilium.io/en/stable/security/policy/language/#layer-7-examples).

### Step 4 — Audit egress (the exfiltration path)

Egress is the most-overlooked side of network policy. Critical findings:

- `egress` rules with `toCIDRSet: [{cidr: 0.0.0.0/0}]` and no `except` for internal CIDRs (RFC 1918, link-local, cloud metadata service IPs like `169.254.169.254`) — allows pod to reach the cloud metadata service and exfiltrate cloud credentials.
- `egress` with `toEndpoints: []` (empty selector) — the empty selector is expected to match everything and to be broader than `toEndpoints` being absent (derived/unverified; not re-verified in this skill's ledger, confirm against the policy language page for the installed version).
- `egress` allowing `toFQDNs.matchPattern: '*'` — wildcard DNS matching with no narrow allowlist.

Recommended baseline: explicit `toEndpoints` for in-cluster, `toCIDRSet` with `except` for the cloud metadata CIDR, `toFQDNs` for known external services.

### Step 5 — Egress gateway: hand off

`CiliumEgressGatewayPolicy` IP collision, SNAT behavior, gateway node selection and gateway failure are owned by `cilium-external-connectivity-review-agent`. Keep here only the egress **authorization** question: which pods may reach which destinations under policy. Hand off any `egressIP`, `nodeSelector` or gateway-failover finding by agent id with the manifest excerpt.

### Step 6 — Audit ClusterMesh policy semantics

When ClusterMesh is enabled, identity-based policy selectors (`namespaceSelector`, `endpointSelector`) match across cluster boundaries. Two semantics are possible:

1. **`policy-default-local-cluster: false` (behavior in 1.18 and lower)** — selectors match endpoints in any peer cluster. A `namespaceSelector: {kubernetes.io/metadata.name: prod}` matches `prod` in this cluster AND `prod` in every peer cluster.
2. **`policy-default-local-cluster: true` (introduced as an option in 1.18, default from 1.19)** — selectors match only the local cluster unless a cluster is explicitly targeted (cross-cluster selector label `io.cilium.k8s.policy.cluster`). Policies are not distributed across clusters; each cluster needs its own copy.

**A flag flip changes every existing policy's effective scope simultaneously.** Cilium ships an inspect command for this purpose (see the command block below) to preview which policies would be affected.

Stress-tests:

- ClusterMesh deployment with mixed clusters at different `policy-default-local-cluster` settings — confusing semantics; one cluster's policy may match peer endpoints while another's does not.
- Migrating from `false` to `true` — every policy that previously matched cross-cluster identities now silently stops matching them. **This is a documented operational landmine.**

Class: [C1] reads cluster state through the operator's authorized client.

```text
cilium clustermesh inspect-policy-default-local-cluster --all-namespaces   [C1]
```

Reference: [Cilium ClusterMesh](https://docs.cilium.io/en/stable/network/clustermesh/) and the [inspect command reference](https://docs.cilium.io/en/stable/cmdref/cilium_clustermesh_inspect-policy-default-local-cluster/).

### Step 7 — Use Hubble to verify enforcement

Static policy review is not enough. Use Hubble to confirm what the policy actually does:

Class: [C1] Hubble via Relay; flows may contain sensitive metadata. The operator collects; the agent does not run these. Telemetry completeness is judged per the companion skill `cilium-evidence-boundary`.

```text
hubble observe --to-namespace <ns> --to-pod <pod-prefix> --verdict DROPPED --last 1000   [C1]
hubble observe --from-namespace <ns> --from-pod <pod-prefix> --verdict FORWARDED --last 100   [C1]
hubble observe --type l7 --last 100   [C1]
```

`--to-namespace` and `--to-pod` are assumed to mirror the documented `--from-*` filters; confirm with the operator's CLI help. The first line should return nothing if allow rules are correct; the second confirms expected destinations; the third shows L7 flow records. `--type dns` is not a documented flow type (documented types are capture, drop, l7, policy-verdict, trace, trace-sock).

Reference: [Hubble Observability](https://docs.cilium.io/en/stable/observability/hubble/).

### Step 8 — Stress-test operational hygiene

- Prefer `CiliumNetworkPolicy` over `NetworkPolicy` when L7 is needed — converting back later is harder than starting with the richer format.
- Prefer named `endpointSelector` labels over IP CIDRs for in-cluster traffic — IPs change, labels survive pod recreation.
- Prefer `toFQDNs` over `toCIDRSet` for external services with stable hostnames — DNS rotation no longer breaks the policy.
- Avoid `CiliumClusterwideNetworkPolicy` for namespace-scoped concerns — cluster-wide blast radius.
- Test policy changes in a dev or staging cluster first. eBPF program reload may be asynchronous, so an apply could allow traffic before an isolating policy takes effect (derived/unverified; not re-verified in this skill's ledger, confirm against the policy language page for the installed version).

## Output

Return:

- **target**: which policy format and which scope,
- **evidence level**: `sanitized user evidence` / `documentation-based` / `inference`,
- **default-deny posture** in the namespace(s),
- **L7 enforcement assessment**: Envoy proxy enabled / required, whether L7 rules will actually run,
- **egress posture**: cloud-metadata service blocked, CIDR scope, FQDN allowlist hygiene,
- **ClusterMesh assessment** when applicable (cross-cluster semantics, `policy-default-local-cluster` value),
- **risk findings** (with severity: high / medium / low),
- **safest next actions** with sample manifest changes and `hubble observe` [C1] commands for the operator to collect as verification,
- **rollback plan**: how to revert the change without leaving pods unreachable,
- **assumptions and missing facts**.

## Security notes

- Never recommend removing a default-deny policy without a confirmed replacement that explicitly allows required flows.
- Never recommend `toCIDRSet: [{cidr: 0.0.0.0/0}]` without an `except` block covering the cloud metadata service IP and any other sensitive internal CIDRs.
- Never recommend changing `policy-default-local-cluster` without first having the operator collect the inspect output (Step 6) and reviewing every affected policy.
- Do not print Cilium ClusterMesh peer Secrets or agent service account tokens.
