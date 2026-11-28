# Routing table and domain taxonomy

Use this reference when classifying a task or selecting the right specialist(s).

## Routing table

| Signal keywords | Agent ID | Domain | Live-guard? |
|---|---|---|---|
| RBAC, Role, ClusterRole, RoleBinding, ClusterRoleBinding, ServiceAccount, can-i, least privilege, permissions | kubernetes-rbac-review-agent | RBAC review | No |
| apply RBAC, kubectl apply role, grant permission, bind ClusterRole, create RoleBinding, escalate verb, add permissions | kubernetes-live-rbac-mutation-guard-agent | Live RBAC mutation | YES |
| PSA, PodSecurityAdmission, pod-security label, enforce/audit/warn, restricted profile, baseline profile, privileged profile, PSP migration, namespace label | kubernetes-psa-review-agent | Pod security admission review | No |
| Kyverno, ClusterPolicy, kyverno policy, PolicyException, mutate rule, generate rule, image verify, background scan, failureAction | kyverno-policy-review-agent | Kyverno policy review | No |
| apply Kyverno policy, kubectl apply cpol, change failureAction, delete ClusterPolicy, add PolicyException, ValidatingAdmissionPolicy | kubernetes-live-admission-policy-guard-agent | Live admission policy mutation | YES |
| IRSA, workload identity, serviceAccountToken, OIDC trust, pod identity, azure workload identity, GKE WI, annotate serviceaccount, projected token, eks.amazonaws.com | kubernetes-workload-identity-review-agent | Workload identity review | No |
| Istio, ambient mesh, waypoint, ztunnel, AuthorizationPolicy, PeerAuthentication, mTLS, RequestAuthentication, VirtualService, DestinationRule, HBONE | istio-ambient-mesh-review-agent | Istio mesh review | No |
| apply AuthorizationPolicy, apply PeerAuthentication, change mTLS, delete DENY policy, enable PERMISSIVE, istioctl apply | kubernetes-live-mesh-policy-guard-agent | Live mesh policy mutation | YES |
| CNI choice, kube-proxy mode choice, kube-proxy replacement as a design decision, IPAM, MTU sizing, encapsulation, VXLAN, Geneve, dual-stack, IPv6, Pod CIDR, Service CIDR, EndpointSlices, internalTrafficPolicy, externalTrafficPolicy, topology-aware routing, trafficDistribution, Ingress, Gateway API, GRPCRoute, HTTPRoute, GatewayClass, CoreDNS, NodeLocal DNSCache, ndots, Corefile, Submariner, MCS-API, ClusterMesh topology, ClusterMesh kvstore, NodePort path design | kubernetes-network-architecture-review-agent | Network architecture review | No |
| apply Service patch internalTrafficPolicy, apply Service patch externalTrafficPolicy, annotate topology-mode, set trafficDistribution, patch CoreDNS Corefile, install NodeLocal DNSCache, apply Gateway API resource, apply HTTPRoute / GRPCRoute / TLSRoute / ReferenceGrant, create ClusterMesh peer Secret | kubernetes-live-network-architecture-mutation-guard-agent | Live network architecture mutation | YES |
| Cilium policy, CiliumNetworkPolicy, CiliumClusterwideNetworkPolicy, NetworkPolicy content, deny policies, ClusterMesh policy (policy-default-local-cluster), egress authorization, L7 policy, toCIDRSet, WireGuard or IPsec coverage | cilium-network-policy-review-agent | Cilium network policy review | No |
| Cilium datapath drops (non-policy drop reasons), conntrack or BPF map pressure, Cilium kube-proxy replacement or socket-LB behavior, MTU faults on a running Cilium cluster | cilium-dataplane-diagnostics-agent | Cilium datapath diagnosis | No |
| Cilium BGP Control Plane, LB-IPAM, L2 announcements, egress gateway SNAT or gateway failure, ClusterMesh global services and unreachable-cluster behavior | cilium-external-connectivity-review-agent | Cilium boundary connectivity | No |
| Cilium upgrade, rollback, CNI migration to Cilium, Cilium advisory applicability, disruptive Cilium config change | cilium-lifecycle-readiness-agent | Cilium lifecycle readiness | No |
| A Cilium symptom with several plausible owners (for example "service unreachable" or "broke after the upgrade") | cilium-maestro-agent | Cilium board plan (returns a dispatch plan; does not dispatch) | No |
| apply CiliumNetworkPolicy, kubectl apply cnp, delete default-deny, change toCIDRSet, egress gateway policy | kubernetes-live-network-policy-guard-agent | Live network policy mutation | YES |
| Argo CD, ArgoCD, Application, AppProject, ApplicationSet, sync window, argocd sync, gitops, app of apps, ApplicationSet | argocd-gitops-review-agent | Argo CD GitOps review | No |
| argocd app sync, sync production, delete sync-window, expand AppProject, enable auto-sync, ApplicationSet cluster generator | kubernetes-live-argocd-sync-guard-agent | Live Argo CD sync guard | YES |
| OpenTelemetry, OTEL, otelcol, collector, pipeline, receiver, processor, exporter, Instrumentation CR, TargetAllocator, memory_limiter | opentelemetry-collector-config-review-agent | OpenTelemetry collector review | No |
| cert-manager, ClusterIssuer, Issuer, CertificateRequest, CertificateRequestPolicy, approver-policy, trust-manager, Bundle, ConfigMapBundle, certificate renewal, TLS cert K8s, mTLS cert, SPIFFE, cert-manager webhook | cert-manager-issuer-trust-review-agent | PKI K8s review | No |

## Domain taxonomy

| Domain | Keywords and signals |
|---|---|
| `rbac` | Role, ClusterRole, RoleBinding, ClusterRoleBinding, ServiceAccount, can-i, RBAC, least privilege, permission, verb, subject |
| `admission-security` | PSA, PodSecurityAdmission, pod-security label, enforce, audit, warn, restricted, baseline, privileged, PSP migration, Kyverno, ClusterPolicy, PolicyException, mutate, generate, image verify |
| `workload-identity` | IRSA, workload identity, serviceAccountToken, OIDC, pod identity, azure workload identity, GKE WI, projected token, bound service account |
| `mesh` | Istio, ambient mesh, waypoint, ztunnel, AuthorizationPolicy, PeerAuthentication, mTLS, RequestAuthentication, VirtualService, DestinationRule, Envoy |
| `network-architecture` | CNI choice, dataplane, kube-proxy mode, kube-proxy replacement, IPAM, MTU, encapsulation, dual-stack, IPv6, Pod CIDR, Service CIDR, Service routing surface, EndpointSlices, trafficPolicy, topology-aware routing, trafficDistribution, Ingress, Gateway API, GRPCRoute, HTTPRoute, CoreDNS, NodeLocal DNSCache, ndots, Corefile, multi-cluster topology, ClusterMesh topology and kvstore, Submariner, MCS-API, conntrack, NodePort path |
| `network-policy` | Cilium policy semantics, CiliumNetworkPolicy, NetworkPolicy content, Hubble flow filter, egress gateway policy, L7 policy, ClusterMesh policy boundary |
| `gitops` | Argo CD, ArgoCD, Application, AppProject, ApplicationSet, sync window, app of apps, GitOps, deployment sync |
| `observability` | OpenTelemetry, OTEL, otelcol, collector, pipeline, receiver, processor, exporter, Instrumentation CR, TargetAllocator, tracing, metrics, logs |
| `pki` | cert-manager, ClusterIssuer, Issuer, CertificateRequest, CertificateRequestPolicy, approver-policy, trust-manager, Bundle, ConfigMapBundle, certificate renewal, TLS cert, SPIFFE, cert-manager webhook |
| `live-guard` | apply RBAC live, apply admission policy live, change mTLS live, apply network policy live, apply Service patch live, patch CoreDNS Corefile live, install NodeLocal DNSCache live, apply Gateway API resource live, create ClusterMesh peer Secret live, argocd sync production, requires human gate, production mutation |

## Specialist reference

### RBAC

| Agent | Domain | Use when… |
|---|---|---|
| `kubernetes-rbac-review-agent` | RBAC review | Reviewing Roles, ClusterRoles, bindings, ServiceAccount permissions, or running kubectl auth can-i audit for least privilege |
| `kubernetes-live-rbac-mutation-guard-agent` | Live RBAC mutation | Applying new RBAC objects, granting permissions, binding ClusterRoles, or escalating verbs in a live cluster — gate required |

### Admission security

| Agent | Domain | Use when… |
|---|---|---|
| `kubernetes-psa-review-agent` | Pod security admission | Reviewing PSA labels on namespaces, enforcing/auditing/warning against restricted or baseline profiles, or planning PSP migration |
| `kyverno-policy-review-agent` | Kyverno policy review | Reviewing or authoring Kyverno ClusterPolicies, mutate/generate/verify rules, PolicyExceptions, or running background scan analysis |
| `kubernetes-live-admission-policy-guard-agent` | Live admission policy mutation | Applying or deleting Kyverno ClusterPolicies, changing failureAction, or adding PolicyExceptions in a live cluster — gate required |

### Workload identity

| Agent | Domain | Use when… |
|---|---|---|
| `kubernetes-workload-identity-review-agent` | Workload identity review | Reviewing IRSA annotations, OIDC trust relationships, projected serviceAccountToken usage, Azure Workload Identity, or GKE Workload Identity setup |

### Mesh

| Agent | Domain | Use when… |
|---|---|---|
| `istio-ambient-mesh-review-agent` | Istio mesh review | Reviewing Istio ambient mesh waypoint config, AuthorizationPolicy, PeerAuthentication, mTLS mode, VirtualService/DestinationRule, or RequestAuthentication |
| `kubernetes-live-mesh-policy-guard-agent` | Live mesh policy mutation | Applying or deleting AuthorizationPolicy or PeerAuthentication, changing mTLS mode, or enabling PERMISSIVE mode in a live cluster — gate required |

### Network architecture

| Agent | Domain | Use when… |
|---|---|---|
| `kubernetes-network-architecture-review-agent` | Network architecture review | Reviewing CNI choice, kube-proxy mode, kube-proxy replacement, IPAM, MTU and encapsulation, dual-stack, Pod / Service CIDR sizing (one-way doors), Service routing surface (EndpointSlices, internalTrafficPolicy / externalTrafficPolicy, topology-aware routing, `trafficDistribution`), Ingress vs Gateway API migration, CoreDNS Corefile, NodeLocal DNSCache architecture, multi-cluster topology (ClusterMesh topology, Submariner, MCS-API, ClusterMesh kvstore behavior), or troubleshooting connectivity at the dataplane / Service / DNS layer. Read-only; delegates NetworkPolicy content review and live mutations. |
| `kubernetes-live-network-architecture-mutation-guard-agent` | Live network architecture mutation | Applying Service spec patches (`internalTrafficPolicy`, `externalTrafficPolicy`, `topology-mode`, `trafficDistribution`), patching CoreDNS Corefile (resourceName-locked `ConfigMap/coredns`), installing NodeLocal DNSCache, creating Gateway API resources (`Gateway`, `HTTPRoute`, `GRPCRoute`, `TLSRoute`, `ReferenceGrant`), or creating Cilium ClusterMesh peer `Secret` in a live cluster — gate required. **HARD REFUSE** one-way doors: CNI replacement, kube-proxy mode swap, MTU change, Pod / Service CIDR resize, namespace deletion, kube-system DaemonSet/Deployment writes, CRD operations. Cluster-side enforcement via least-privilege ServiceAccount per `docs/least-privilege-rbac.md`; pre-flight `kubectl auth can-i` matrix runs before any mutation. |

**Scope boundary with policy / mesh / pod-spec specialists:** the architecture agent owns *design correctness, sizing, and operational traps* in dataplane, Service routing, DNS, and multi-cluster topology. It does NOT review NetworkPolicy content (→ `cilium-network-policy-review-agent`), mesh L7 (→ `istio-ambient-mesh-review-agent`), pod `securityContext` / hostNetwork (→ `kubernetes-pod-spec-review-agent`), or perform live mutations (→ `kubernetes-live-network-policy-guard-agent` / `kubernetes-live-mesh-policy-guard-agent`). When a task spans architecture + policy + mesh, dispatch the team in parallel; the architecture findings (kube-proxy replacement mode, CNI version, MTU, Envoy DaemonSet status) are independent inputs the policy and mesh specialists need.

### Network policy

| Agent | Domain | Use when… |
|---|---|---|
| `cilium-network-policy-review-agent` | Cilium network policy review | Reviewing CiliumNetworkPolicy, CiliumClusterwideNetworkPolicy, ClusterMesh policy semantics (`policy-default-local-cluster`), Hubble flow filter, or L7 policy rules. Architecture-level ClusterMesh design (topology, kvstore, CIDR overlap) is owned by `kubernetes-network-architecture-review-agent`. |
| `cilium-maestro-agent` | Cilium board plan | Only when a Cilium symptom has several plausible owners. Single-owner Cilium work routes straight to the specialist in the routing table above; the Cilium maestro returns a dispatch plan and never dispatches. |
| `kubernetes-live-network-policy-guard-agent` | Live network policy mutation | Applying or deleting CiliumNetworkPolicy, removing default-deny rules, changing toCIDRSet, or modifying egress gateway config in a live cluster — gate required |

### GitOps

| Agent | Domain | Use when… |
|---|---|---|
| `argocd-gitops-review-agent` | Argo CD GitOps review | Reviewing ArgoCD Application/AppProject/ApplicationSet config, sync windows, app-of-apps patterns, or GitOps reconciliation strategy |
| `kubernetes-live-argocd-sync-guard-agent` | Live Argo CD sync guard | Triggering an argocd app sync to production, deleting sync windows, expanding AppProject scope, or enabling auto-sync on a production app — gate required |

### Observability

| Agent | Domain | Use when… |
|---|---|---|
| `opentelemetry-collector-config-review-agent` | OpenTelemetry review | Reviewing OpenTelemetry Collector pipelines, receiver/processor/exporter configs, Instrumentation CRs, or TargetAllocator setup for Kubernetes workloads |

### PKI

| Agent | Domain | Use when… |
|---|---|---|
| `cert-manager-issuer-trust-review-agent` | PKI K8s review | Reviewing cert-manager ClusterIssuer/Issuer scope, CertificateRequestPolicy coverage, Certificate SAN and duration risks, trust-manager bundle distribution, or SPIFFE trust domain integration |

**Cross-layer note:** cert-manager is a certificate lifecycle controller, not a CA. When the task involves the cloud Private CA configuration (template ARN, IRSA/Managed Identity scope, CRL reachability, CA hierarchy), escalate to the relevant cloud maestro in parallel: `aws-private-ca-issuer-review-agent` (AWS), `azure-keyvault-certificate-issuer-review-agent` (Azure), `oci-certificates-issuer-review-agent` (OCI). See `docs/pki-cert-manager-agent-guide.md` for multi-agent PKI scenarios.

## Multi-domain dispatch examples

### Example 1: Namespace security posture + Kyverno policies

**User request:** "Review our namespace security posture AND check our Kyverno policies."

**Routing:**
```
Route: kubernetes-psa-review-agent, kyverno-policy-review-agent
Reason: Task spans PSA namespace label enforcement and Kyverno policy review — two separate admission security domains.
Mode: parallel (2)
```

`kubernetes-psa-review-agent` reviews PSA enforce/audit/warn labels across namespaces and identifies any missing or permissive labels; `kyverno-policy-review-agent` reviews ClusterPolicies for correctness, failureAction settings, and background scan results.

---

### Example 2: Service mesh and network policies audit

**User request:** "Audit our service mesh and network policies."

**Routing:**
```
Route: istio-ambient-mesh-review-agent, cilium-network-policy-review-agent
Reason: Task spans Istio ambient mesh review and Cilium network policy review — two distinct network security domains.
Mode: parallel (2)
```

`istio-ambient-mesh-review-agent` reviews waypoint configuration, AuthorizationPolicy, PeerAuthentication, and mTLS posture; `cilium-network-policy-review-agent` reviews CiliumNetworkPolicy default-deny posture, toCIDRSet rules, and ClusterMesh semantics.

---

### Example 3: RBAC, workload identity, and PSA for prod namespace

**User request:** "Check RBAC, workload identity, and PSA for our prod namespace."

**Routing:**
```
Route: kubernetes-rbac-review-agent, kubernetes-workload-identity-review-agent, kubernetes-psa-review-agent
Reason: Task spans RBAC least-privilege review, OIDC workload identity trust, and Pod Security Admission labels — three clearly identified domains.
Mode: parallel (3)
```

All three specialists run in parallel: `kubernetes-rbac-review-agent` audits Role/ClusterRole bindings and verbs for the prod namespace; `kubernetes-workload-identity-review-agent` reviews IRSA or workload identity annotations and OIDC trust policy scope; `kubernetes-psa-review-agent` verifies PSA enforce label, profile, and version pinning on the prod namespace.

---

### Example 4: ArgoCD AppProject blast-radius + Kyverno policies before prod deploy

**User request:** "Review ArgoCD AppProject blast-radius and Kyverno policies before prod deploy."

**Routing:**
```
Route: argocd-gitops-review-agent, kyverno-policy-review-agent
Reason: Task spans Argo CD AppProject scope and Kyverno admission policy review — two distinct GitOps and admission security domains.
Mode: parallel (2)
```

`argocd-gitops-review-agent` reviews the AppProject `sourceRepos`, `destinations`, `clusterResourceWhitelist`, and sync impersonation posture; `kyverno-policy-review-agent` reviews active ClusterPolicies for correctness and background scan violations that would block the deploy.

---

### Example 5: cert-manager setup + workload identity review

**User request:** "Review our cert-manager ClusterIssuer config and the IRSA annotation on the cert-manager ServiceAccount."

**Routing:**
```
Route: cert-manager-issuer-trust-review-agent, kubernetes-workload-identity-review-agent
Reason: Task spans cert-manager PKI K8s config (ClusterIssuer scope, CertificateRequestPolicy) and IRSA workload identity trust for the cert-manager ServiceAccount.
Mode: parallel (2)
```

`cert-manager-issuer-trust-review-agent` reviews ClusterIssuer scope, CertificateRequestPolicy coverage, Certificate SAN and duration risks, and trust-manager distribution; `kubernetes-workload-identity-review-agent` reviews the IRSA annotation, OIDC trust policy, and whether the role is scoped to minimum required actions.

---

### Example 6: Holistic Kubernetes networking review (architecture + policy + mesh)

**User request:** "Review our cluster's networking holistically — CNI choice, kube-proxy mode, our CiliumNetworkPolicies, and the Istio ambient mesh."

**Routing:**
```
Route: kubernetes-network-architecture-review-agent, cilium-network-policy-review-agent, istio-ambient-mesh-review-agent
Reason: Task spans network architecture (CNI, kube-proxy mode, dataplane, DNS), Cilium network policy content review, and Istio ambient mesh L7 review — three distinct networking concerns with non-overlapping scopes. Hard-ceiling 4 specialists; this stays under the limit.
Mode: parallel (3)
```

`kubernetes-network-architecture-review-agent` reviews CNI choice, kube-proxy mode, IPAM, MTU, Pod / Service CIDR sizing as one-way doors, EndpointSlice topology, CoreDNS / NodeLocal DNSCache architecture, and ClusterMesh topology / kvstore behavior. `cilium-network-policy-review-agent` reviews CiliumNetworkPolicy default-deny posture, toCIDRSet rules, ClusterMesh policy semantics (`policy-default-local-cluster`), and L7 policy prerequisites — the L7 review depends on the architecture finding of whether Cilium kube-proxy replacement and Envoy DaemonSet are in place. `istio-ambient-mesh-review-agent` reviews waypoint configuration, AuthorizationPolicy, PeerAuthentication, and mTLS posture. The three outputs are scope-separated; the synthesizer surfaces architecture findings first because they may invalidate policy and mesh assumptions.

**Sequencing note:** Architecture findings (kube-proxy replacement mode, Envoy DaemonSet running, MTU correctness) gate the *interpretation* of policy and mesh outputs but not their dispatch. Run in parallel; in the synthesis step, present architecture posture before policy and mesh — if architecture flags a one-way-door blocker, explicitly mark which policy and mesh recommendations may need to be re-scoped.

---

### Live-guard gate example: network architecture mutation

**User request:** "Apply `service.kubernetes.io/topology-mode: Auto` annotation to the `frontend` Service in the `prod` namespace on the prod cluster."

**Routing:**
```
Route: kubernetes-live-network-architecture-mutation-guard-agent
Reason: Patching a Service annotation on a live production cluster is a live network architecture mutation — gate required even though the operation is reversible.
Mode: live-guard-gate
```

**STOP — Live-guard gate. Before this dispatch can proceed, you must provide:**

1. **Pre-flight RBAC self-check applied:** Confirm `skills/kubernetes/kubernetes-live-network-architecture-mutation-guard/references/least-privilege-rbac.yaml` is applied to the prod cluster, and that the agent's bound ServiceAccount has been pre-flight-tested with the `kubectl auth can-i` matrix from `references/rbac-pre-flight.md`. Every must-not row must return `no`; every must-be row must return `yes`.
2. **Operator principal check:** Confirm your kubeconfig is **not** `cluster-admin` and **not** in `system:masters`. The agent will refuse if `kubectl auth can-i '*' '*' --all-namespaces` returns `yes` for your principal.
3. **Blast-radius assessment:** Which workloads currently consume the `frontend` Service? Cross-zone traffic patterns may shift if `topology-mode: Auto` populates hints with insufficient endpoints per zone.
4. **Rollback path:** `kubectl annotate svc frontend -n prod service.kubernetes.io/topology-mode-` (the `-` suffix removes the annotation) — confirmed reversible in under 30 seconds.
5. **Explicit written confirmation:** Type "I confirm I understand the blast radius and rollback path. Proceed."

For irreversible operations (CNI replacement, kube-proxy mode swap, MTU change, Pod / Service CIDR resize, namespace deletion, kube-system DaemonSet writes, CRD operations), the agent **HARD REFUSES** regardless of operator confirmation — these belong to a human-led cutover plan that the architecture review agent (`kubernetes-network-architecture-review-agent`) can produce but no agent in this repo will execute.

---

### Live-guard gate example: RBAC mutation

**User request:** "Apply the new ClusterRoleBinding for the payments service account in the prod cluster."

**Routing:**
```
Route: kubernetes-live-rbac-mutation-guard-agent
Reason: Applying a ClusterRoleBinding to a live production cluster is a live RBAC mutation — gate required.
Mode: live-guard-gate
```

**STOP — Live-guard gate. Before this dispatch can proceed, you must provide:**

1. **Blast-radius assessment:** Which namespaces, workloads, and users are affected by this ClusterRoleBinding? What is the scope of the verbs and resources being granted?
2. **Rollback path:** What is the exact command to revoke this binding if it grants unintended access, and how long will rollback take?
3. **Explicit written confirmation:** Type "I confirm I understand the blast radius and rollback path. Proceed."

If you cannot supply a rollback path, route to `kubernetes-rbac-review-agent` first to develop a scoped binding with a documented revocation procedure.

---

## Live-guard gate protocol

Before routing to any live-guard agent, surface all three and wait for explicit written confirmation:

1. **Blast-radius assessment** — which resources, namespaces, workloads, or users are affected if this goes wrong?
2. **Rollback path** — what is the tested recovery procedure, exact commands, and estimated recovery time?
3. **Explicit confirmation** — "I confirm I understand the blast radius and rollback path. Proceed."

If the user cannot supply a rollback path, recommend the corresponding review agent to develop the rollback path first before dispatching the live-guard agent.

## Safety checklist reference

Load [references/safety-checklist.md](safety-checklist.md) before any live-guard dispatch or when blast-radius assessment is required.
