# AGENTS.md — Navigation Compass

<!-- count:global:agents -->735<!-- /count --> agents across <!-- count:global:providers -->45<!-- /count --> providers. This file is the index; load provider files on demand.

## File structure

```
agents/<provider>/<agent-id>/AGENT.md          ← harness-neutral role contract (load this)
agents/<provider>/<agent-id>/harnesses/        ← 7 adapters: codex, copilot, claude-code,
                                                  cursor, gemini, kiro-ide, kiro-cli
agents/<provider>/<agent-id>/metadata.json     ← catalog mirror
catalog/agents.json                            ← machine-readable index of every agent
```

## Agent tiers

| Tier | sandbox_mode | When to load |
|---|---|---|
| **review** | `read-only` | Audit, analysis, recommendations — never writes to live systems |
| **router / maestro** | `read-only` | Classifies task → dispatches narrowest specialist(s); never auto-dispatches live-guards |
| **live-guard** | `workspace-write` | Approval-gated mutations; requires current-state capture + explicit sign-off before every write |

Live-guard agents refuse to proceed without: target confirmation (cluster/account/region), current-state evidence (`kubectl get … -o yaml` / equivalent), and explicit platform-team or operator sign-off. Missing any one is a hard stop.

---

## 🟧 AWS — <!-- count:dir:aws:agents -->47<!-- /count --> agents → [`agents/aws/AGENTS.md`](aws/AGENTS.md)

**Entry point:** load `agents/aws/aws-maestro-agent/AGENT.md` for any AWS task; it routes to the right specialist and back.

| Category | Agents | Load when |
|---|---|---|
| **Router** | `aws-maestro-agent` | Any AWS task without a known specialist |
| **IAM / identity** | `aws-iam-least-privilege-review-agent`, `aws-kms-secrets-lifecycle-steward-agent`, `aws-s3-data-perimeter-governor-agent`, `aws-compliance-evidence-mapper-agent` | IAM policy, KMS key policy, S3 perimeter, compliance mapping |
| **Security posture** | `aws-security-posture-hardening-agent`, `aws-bedrock-agent-security-governor-agent` | Security Hub, GuardDuty, Bedrock agent trust |
| **Compute / EKS / ECS** | `aws-eks-platform-operator-agent`, `aws-ecs-fargate-platform-operator-agent`, `aws-ec2-compute-operations-steward-agent`, `aws-ecs-service-remediation-operator-agent` | EKS cluster ops, ECS/Fargate service review, EC2 operations |
| **Databases** | `aws-rds-aurora-performance-investigator-agent`, `aws-dynamodb-data-modeling-performance-review-agent` | RDS/Aurora query tuning, DynamoDB data model |
| **Serverless** | `aws-serverless-production-readiness-agent`, `aws-serverless-rollout-corrector-agent` | Lambda readiness, canary/alias rollout |
| **Networking / edge** | `aws-network-architect-agent`, `aws-api-edge-delivery-review-agent` | VPC/TGW/DirectConnect, API Gateway + CloudFront + WAF |
| **IaC** | `aws-iac-change-safety-review-agent`, `aws-iac-patch-executor-agent` | CDK/CFN/SAM/Terraform review, IaC file patching |
| **Cost / FinOps** | `aws-cost-optimization-governor-agent`, `aws-cost-anomaly-watch-coordinator-agent` | Cost Explorer, budget drift |
| **CI/CD / DevOps** | `aws-ci-cd-release-engineer-agent`, `aws-devops-agent-skill-designer-agent`, `aws-pipeline-fix-operator-agent`, `aws-deployment-hotfix-operator-agent` | CodePipeline, release gates, hotfix patching |
| **Architecture** | `aws-solution-architect-agent`, `aws-migration-cutover-architect-agent`, `aws-resilience-bcdr-review-agent`, `aws-event-driven-architecture-review-agent`, `aws-landing-zone-governor-agent`, `aws-network-architect-agent` | Solution design, migrations, BCDR, event-driven, Control Tower |
| **AI / Bedrock** | `aws-generative-ai-developer-agent`, `aws-agentcore-agent` | Bedrock app dev, AgentCore deployment |
| **Ops / observability** | `aws-observability-incident-responder-agent`, `aws-daily-operations-briefing-coordinator-agent`, `aws-ticket-triage-escalation-coordinator-agent`, `aws-change-impact-advisor-agent`, `aws-data-protection-backup-steward-agent`, `aws-limits-capacity-planner-agent`* | CloudWatch, incident triage, ops briefing |
| **Live-guard (5)** | `aws-live-deployment-guarded-operator-agent`, `aws-live-ecs-rollout-guard-agent`, `aws-live-iac-change-guard-agent`, `aws-live-pipeline-approval-operator-agent`, `aws-live-serverless-release-guard-agent` | Approval-gated live mutations; never auto-dispatched |

> For operational rules, credential chain guidance, and MCP tool usage → [`agents/aws/AGENTS.md`](aws/AGENTS.md)

---

## 🟦 Azure — <!-- count:dir:azure:agents -->36<!-- /count --> agents → [`agents/azure/AGENTS.md`](azure/AGENTS.md)

**Entry point:** load `agents/azure/azure-maestro-agent/AGENT.md` for any Azure task.

| Category | Agents | Load when |
|---|---|---|
| **Router** | `azure-maestro-agent` | Any Azure task without a known specialist |
| **Identity / RBAC** | `azure-rbac-review-agent`, `azure-role-selector-agent`, `azure-entra-id-specialist-agent`, `azure-identity-governance-review-agent` | Role assignments, custom roles, Entra ID posture, identity governance |
| **AKS / containers** | `azure-aks-platform-operator-agent` | AKS cluster ops, node pool, networking |
| **App Service** | `azure-app-service-production-readiness-agent` | Web Apps, Function Apps, deployment slots |
| **Databases** | `azure-cosmosdb-platform-operator-agent`, `azure-cosmosdb-performance-investigator-agent`, `azure-cosmosdb-application-developer-agent` | CosmosDB ops, query perf, app dev patterns |
| **Key Vault / secrets** | `azure-key-vault-secret-lifecycle-auditor-agent` | Secret rotation, access policy, purge-protection |
| **Cost / FinOps** | `azure-cost-optimization-governor-agent`, `azure-cost-estimation-review-agent` | Spend governance, estimate review |
| **Networking** | `azure-network-topology-review-agent`, `azure-private-endpoint-adoption-planner-agent` | Hub-spoke, Private Link, DNS |
| **IaC / governance** | `azure-governance-policy-guardrails-agent`, `azure-subscription-resource-organization-agent`, `azure-landing-zone-architect-agent` | Policy, management groups, landing zones |
| **CI/CD / DevOps** | `azure-platform-automation-devops-agent` | Pipelines, automation, platform DevOps |
| **AI / Foundry** | `azure-ai-foundry-ops-governor-agent` | AI Foundry, model deployments, governance |
| **Architecture** | `azure-solution-architect-agent`*, `azure-migrate-landing-zone-cutover-agent`, `azure-resilience-bcdr-review-agent` | Solution design, migrations, BCDR |
| **Observability / ops** | `azure-observability-investigator-agent`, `azure-resource-health-incident-triage-agent`, `azure-security-posture-hardening-agent` | Monitor, Log Analytics, incident triage |
| **Live-guard (7)** | `azure-live-aks-rollout-guard-agent`, `azure-live-app-service-slot-swap-guard-agent`, `azure-live-arm-deployment-stack-guard-agent`, `azure-live-cost-budget-action-guard-agent`, `azure-live-entra-role-assignment-guard-agent`, `azure-live-keyvault-rotation-purge-guard-agent`, `azure-live-pim-jit-activation-guard-agent` | Approval-gated live mutations; never auto-dispatched |

> For permission models, PIM gate details, and MCP guidance → [`agents/azure/AGENTS.md`](azure/AGENTS.md)

---

## 🟥 OCI — <!-- count:dir:oci:agents -->39<!-- /count --> agents → [`agents/oci/AGENTS.md`](oci/AGENTS.md)

**Entry point:** load `agents/oci/oci-maestro-agent/AGENT.md` for any OCI task.

| Category | Agents | Load when |
|---|---|---|
| **Router** | `oci-maestro-agent` | Any OCI task without a known specialist |
| **IAM / identity** | `oci-identity-access-governor-agent`, `oci-cloud-guard-responder-agent` | OCI IAM policies, dynamic groups, Cloud Guard |
| **Compute** | `oci-compute-platform-operator-agent`, `oci-compute-instance-agent-operator-agent` | Instances, autoscaling, instance agents |
| **Databases** | `oci-autonomous-database-architect-agent`, `oci-exadata-platform-architect-agent`, `oci-database-platform-dba-agent`, `oci-mysql-heatwave-ai-specialist-agent`, `oci-goldengate-replication-operator-agent`, `oci-recovery-service-operator-agent`, `oci-dbtools-sql-analyst-agent` | ADB, Exadata, DBA ops, HeatWave, GoldenGate, recovery |
| **OKE / containers** | `oci-devops-container-platform-engineer-agent` | OKE, DevOps pipelines, container registry |
| **Networking** | `oci-network-architect-agent`, `oci-load-balancer-traffic-engineer-agent` | VCN, FastConnect, LBaaS, DRG |
| **Storage / backup** | `oci-storage-backup-steward-agent`, `oci-registry-artifact-governor-agent` | Object Storage, backups, OCIR |
| **Cost / FinOps** | `oci-cost-finops-analyst-agent`, `oci-limits-capacity-planner-agent` | Cost analysis, limits, quotas |
| **Architecture** | `oci-solution-architect-agent`, `oci-migration-cutover-architect-agent`, `oci-multi-cloud-architect-agent`, `oci-resilience-bcdr-architect-agent`* | Solution design, migrations, multi-cloud |
| **Observability / support** | `oci-observability-incident-responder-agent`, `oci-support-incident-coordinator-agent`, `oci-resource-search-inventory-analyst-agent` | Monitoring, support SRs, resource inventory |
| **Specialist** | `oci-security-compliance-reviewer-agent`, `oci-iot-digital-twin-engineer-agent`, `oci-fusion-apps-environment-operator-agent` | Security posture, IoT/OIC, Fusion SaaS |
| **Live-guard (7)** | `oci-live-autonomous-db-lifecycle-guard-agent`, `oci-live-cost-budget-runaway-guard-agent`, `oci-live-iam-policy-compartment-guard-agent`, `oci-live-network-security-rule-guard-agent`, `oci-live-oke-rollout-guard-agent`, `oci-live-resource-manager-stack-guard-agent`, `oci-live-vault-key-destruction-guard-agent` | Approval-gated live mutations; never auto-dispatched |

> For compartment scoping, Resource Manager rules, and OCI MCP guidance → [`agents/oci/AGENTS.md`](oci/AGENTS.md)

---

## ☁️ OVHcloud — <!-- count:dir:ovhcloud:agents -->6<!-- /count --> agents → [`agents/ovhcloud/README.md`](ovhcloud/README.md)

**Entry point:** load `agents/ovhcloud/ovhcloud-maestro-agent/AGENT.md` for any OVHcloud task.

| Category | Agents | Load when |
|---|---|---|
| **Router** | `ovhcloud-maestro-agent` | Any OVHcloud task without a known specialist |
| **IAM** | `ovhcloud-iam-policy-review-agent` | IAM policy conditions, identity groups, OAuth2 access |
| **Cost / FinOps** | `ovhcloud-cost-finops-analyst-agent` | Public Cloud cost, commitments, idle waste |
| **Kubernetes** | `ovhcloud-kubernetes-platform-operator-agent` | MCK lifecycle, node pools, workload placement |
| **Networking** | `ovhcloud-network-architect-agent` | vRack design, network isolation, connectivity |
| **Live-guard (1)** | `ovhcloud-live-kms-key-destruction-guard-agent` | KMS key destruction; approval-gated, never auto-dispatched |

> Conditional IAM policies (IP, tag, expiration) are unique — always audit policy scope before approval.

---

## 🌐 IONOS Cloud — <!-- count:dir:ionos:agents -->6<!-- /count --> agents → [`agents/ionos/README.md`](ionos/README.md)

**Entry point:** load `agents/ionos/ionos-maestro-agent/AGENT.md` for any IONOS Cloud task.

| Category | Agents | Load when |
|---|---|---|
| **Router** | `ionos-maestro-agent` | Any IONOS Cloud task without a known specialist |
| **DCD topology** | `ionos-datacenter-designer-reviewer-agent` | Data Center Designer review, multi-AZ placement, blast radius |
| **Compliance** | `ionos-security-compliance-reviewer-agent` | GDPR posture, data residency, encryption audit |
| **Kubernetes** | `ionos-kubernetes-platform-operator-agent` | Managed K8s, node pools, workload placement |
| **Cost / FinOps** | `ionos-cost-optimization-analyst-agent` | Cost analysis, resource utilization |
| **Live-guard (1)** | `ionos-live-database-lifecycle-guard-agent` | DBaaS failover/scaling/backup; approval-gated |

> DCD changes have multi-AZ topology blast radius; live-guards must snapshot current state before any mutation.

---

## 🇫🇷 Scaleway — <!-- count:dir:scaleway:agents -->6<!-- /count --> agents → [`agents/scaleway/README.md`](scaleway/README.md)

**Entry point:** load `agents/scaleway/scaleway-maestro-agent/AGENT.md` for any Scaleway task.

| Category | Agents | Load when |
|---|---|---|
| **Router** | `scaleway-maestro-agent` | Any Scaleway task without a known specialist |
| **IAM** | `scaleway-iam-policy-review-agent` | IAM bindings, service accounts, API key governance |
| **Kapsule (K8s)** | `scaleway-kapsule-platform-operator-agent` | Managed K8s readiness, node pools, CNI choice, placement |
| **Cost / FinOps** | `scaleway-cost-optimizer-agent` | Instance type review, reserved utilization, rightsizing |
| **Networking** | `scaleway-network-architect-agent` | VPC, security groups, placement groups for HA |
| **Live-guard (1)** | `scaleway-live-kapsule-rollout-guard-agent` | Kapsule cluster + node pool mutations; approval-gated |

> Placement groups orchestrate HA; Kapsule control-plane and CNI choice are immutable post-creation.

---

## 🇩🇪 Hetzner Cloud — <!-- count:dir:hetzner:agents -->6<!-- /count --> agents → [`agents/hetzner/README.md`](hetzner/README.md)

**Entry point:** load `agents/hetzner/hetzner-maestro-agent/AGENT.md` for any Hetzner Cloud task.

| Category | Agents | Load when |
|---|---|---|
| **Router** | `hetzner-maestro-agent` | Any Hetzner Cloud task without a known specialist |
| **Cost / FinOps** | `hetzner-cost-optimization-analyst-agent` | Instance type review, resource utilization, cost savings |
| **Infra review** | `hetzner-infrastructure-reviewer-agent` | Firewall rules, LB config, network design, public-IP exposure |
| **Capacity** | `hetzner-capacity-planner-agent` | Resource limits, quota, growth planning, region distribution |
| **Live-guard (2)** | `hetzner-live-firewall-rule-guard-agent`, `hetzner-live-server-lifecycle-guard-agent` | Firewall rule + server lifecycle mutations; approval-gated |

> No official Terraform provider — agents recommend REST API / hcloud-python over community Terraform.

---

## 💰 Contabo — <!-- count:dir:contabo:agents -->6<!-- /count --> agents → [`agents/contabo/README.md`](contabo/README.md)

**Entry point:** load `agents/contabo/contabo-maestro-agent/AGENT.md` for any Contabo task.

| Category | Agents | Load when |
|---|---|---|
| **Router** | `contabo-maestro-agent` | Any Contabo task without a known specialist |
| **Cost / FinOps** | `contabo-cost-optimization-analyst-agent` | Contract period analysis, VPS sizing, addon utilization |
| **Capacity** | `contabo-capacity-planner-agent` | Resource planning, region coverage, instance sizing |
| **Security** | `contabo-security-hardening-agent` | SSH key management, default user policy, firewall posture |
| **Live-guard (2)** | `contabo-live-instance-lifecycle-guard-agent`, `contabo-live-storage-operations-guard-agent` | VPS/VDS + Object Storage mutations; approval-gated |

> Contractual periods (1/3/6/12 months) drive billing; live-guards demand explicit period acknowledgment before any lifecycle change. No official Terraform/SDK — automation via `cntb` CLI or REST API.

---

## ☸️ Kubernetes — <!-- count:dir:kubernetes:agents -->15<!-- /count --> agents → [`agents/kubernetes/README.md`](kubernetes/README.md)

**Entry point:** load `agents/kubernetes/kubernetes-maestro-agent/AGENT.md` — routes to all K8s specialists (including CNCF domain agents below) and enforces the live-guard gate.

| Agent | Tier | Load when |
|---|---|---|
| [`kubernetes-maestro-agent`](kubernetes/kubernetes-maestro-agent/AGENT.md) | router | Any Kubernetes task; dispatches to the right specialist(s) in parallel |
| [`kubernetes-rbac-review-agent`](kubernetes/kubernetes-rbac-review-agent/AGENT.md) | review | Roles, ClusterRoles, RoleBindings, ClusterRoleBindings, escalation paths |
| [`kubernetes-workload-identity-review-agent`](kubernetes/kubernetes-workload-identity-review-agent/AGENT.md) | review | IRSA, Azure Workload Identity, GKE WI Federation, projected tokens, OIDC trust policy |
| [`kubernetes-psa-review-agent`](kubernetes/kubernetes-psa-review-agent/AGENT.md) | review | Pod Security Admission labels, enforce/audit/warn modes, PSP migration |
| [`kubernetes-pod-spec-review-agent`](kubernetes/kubernetes-pod-spec-review-agent/AGENT.md) | review | Pod securityContext, capabilities, privileged containers, host network/PID/IPC, readOnly filesystem |
| [`external-secrets-operator-review-agent`](kubernetes/external-secrets-operator-review-agent/AGENT.md) | review | ESO SecretStore, ClusterSecretStore, ExternalSecret, PushSecret scope and auth |
| [`kubecost-chargeback-allocation-review-agent`](kubernetes/kubecost-chargeback-allocation-review-agent/AGENT.md) | review | Kubecost label taxonomy, shared cost model, idle allocation, namespace budget alerts |
| [`kubernetes-live-rbac-mutation-guard-agent`](kubernetes/kubernetes-live-rbac-mutation-guard-agent/AGENT.md) | live-guard | kubectl apply/delete on Roles/ClusterRoles/Bindings |
| [`kubernetes-live-admission-policy-guard-agent`](kubernetes/kubernetes-live-admission-policy-guard-agent/AGENT.md) | live-guard | kubectl apply/delete on Kyverno ClusterPolicy/Policy/PolicyException, VAP |
| [`kubernetes-live-argocd-sync-guard-agent`](kubernetes/kubernetes-live-argocd-sync-guard-agent/AGENT.md) | live-guard | argocd sync, AppProject mutations, sync-window changes |
| [`kubernetes-live-mesh-policy-guard-agent`](kubernetes/kubernetes-live-mesh-policy-guard-agent/AGENT.md) | live-guard | kubectl apply/delete on Istio AuthorizationPolicy, PeerAuthentication |
| [`kubernetes-live-network-policy-guard-agent`](kubernetes/kubernetes-live-network-policy-guard-agent/AGENT.md) | live-guard | kubectl apply/delete on CiliumNetworkPolicy, NetworkPolicy |
| [`kubernetes-live-velero-restore-guard-agent`](kubernetes/kubernetes-live-velero-restore-guard-agent/AGENT.md) | live-guard | velero restore create, backup schedule deletion, backup lifecycle operations |

---

## 📊 Prometheus — <!-- count:dir:prometheus:agents -->1<!-- /count --> agents → [`agents/prometheus/README.md`](prometheus/README.md)

| Agent | Tier | Load when |
|---|---|---|
| [`prometheus-alerting-cardinality-review-agent`](prometheus/prometheus-alerting-cardinality-review-agent/AGENT.md) | review | PromQL alerting rules, recording rules, label cardinality, AlertmanagerConfig routing, inhibition rules |

---

## 🦅 Falco — <!-- count:dir:falco:agents -->1<!-- /count --> agents → [`agents/falco/README.md`](falco/README.md)

| Agent | Tier | Load when |
|---|---|---|
| [`falco-runtime-threat-rules-review-agent`](falco/falco-runtime-threat-rules-review-agent/AGENT.md) | review | Falco rules, macros, exception blast radius, K8s audit webhook gaps, SIEM alert routing |

---

## 🔏 Sigstore — <!-- count:dir:sigstore:agents -->1<!-- /count --> agents → [`agents/sigstore/README.md`](sigstore/README.md)

| Agent | Tier | Load when |
|---|---|---|
| [`sigstore-cosign-supply-chain-review-agent`](sigstore/sigstore-cosign-supply-chain-review-agent/AGENT.md) | review | Cosign signing policy, SBOM attestation, Rekor inclusion, keyless trust root, admission enforcement |

---

## 🔐 cert-manager — <!-- count:dir:cert-manager:agents -->1<!-- /count --> agents → [`agents/cert-manager/README.md`](cert-manager/README.md)

| Agent | Tier | Load when |
|---|---|---|
| [`cert-manager-issuer-trust-review-agent`](cert-manager/cert-manager-issuer-trust-review-agent/AGENT.md) | review | ClusterIssuer scope, CertificateRequestPolicy auto-approval gap, SAN wildcards, trust-manager bundle blast radius |
| [`aws-private-ca-issuer-review-agent`](aws/aws-private-ca-issuer-review-agent/AGENT.md) | review | AWS Private CA issuer: IRSA trust chain, PCA hierarchy, certificate template scope |
| [`azure-keyvault-certificate-issuer-review-agent`](azure/azure-keyvault-certificate-issuer-review-agent/AGENT.md) | review | Azure Key Vault cert issuer: Managed Identity auth, soft-delete, rotation trigger |
| [`oci-certificates-issuer-review-agent`](oci/oci-certificates-issuer-review-agent/AGENT.md) | review | OCI Certificates Service issuer: instance principal auth, validity duration, revocation policy |

---

## 🔄 FluxCD — <!-- count:dir:fluxcd:agents -->1<!-- /count --> agents → [`agents/fluxcd/README.md`](fluxcd/README.md)

| Agent | Tier | Load when |
|---|---|---|
| [`fluxcd-kustomization-helmrelease-review-agent`](fluxcd/fluxcd-kustomization-helmrelease-review-agent/AGENT.md) | review | Kustomization SA scoping and prune safety, HelmRelease version pinning, SOPS encryption, multi-tenant isolation |

---

## 🎭 Backstage — <!-- count:dir:backstage:agents -->1<!-- /count --> agents → [`agents/backstage/README.md`](backstage/README.md)

| Agent | Tier | Load when |
|---|---|---|
| [`backstage-scaffolder-template-review-agent`](backstage/backstage-scaffolder-template-review-agent/AGENT.md) | review | Scaffolder template action blast-radius, input injection, RBAC gate, secret scope, catalog entity poisoning |

---

## 💾 Velero — 1 live-guard → [`agents/velero/README.md`](velero/README.md)

| Agent | Tier | Load when |
|---|---|---|
| [`kubernetes-live-velero-restore-guard-agent`](kubernetes/kubernetes-live-velero-restore-guard-agent/AGENT.md) | live-guard | velero restore create, backup schedule deletion, backup lifecycle operations |

*Agent lives in `agents/kubernetes/` — dispatched via kubernetes-maestro*

---

## 🛡️ Kyverno — <!-- count:dir:kyverno:agents -->1<!-- /count --> agents → [`agents/kyverno/README.md`](kyverno/README.md)

| Agent | Tier | Load when |
|---|---|---|
| [`kyverno-policy-review-agent`](kyverno/kyverno-policy-review-agent/AGENT.md) | review | ClusterPolicy/Policy failureAction, PolicyException scope, background scan, Kyverno-vs-VAP decision |

*Live mutation of Kyverno policies → `kubernetes-live-admission-policy-guard-agent` (above)*

---

## 🔄 Argo CD — <!-- count:dir:argocd:agents -->2<!-- /count --> agents → [`agents/argocd/README.md`](argocd/README.md)

| Agent | Tier | Load when |
|---|---|---|
| [`argocd-gitops-review-agent`](argocd/argocd-gitops-review-agent/AGENT.md) | review | AppProject blast-radius, sync impersonation, RollingSync, sync-window scope |
| [`argo-rollouts-progressive-delivery-review-agent`](argocd/argo-rollouts-progressive-delivery-review-agent/AGENT.md) | review | Canary analysis templates, traffic provider wiring, PDB/maxUnavailable deadlock, blue-green autoPromotion |

*Live ArgoCD mutations → `kubernetes-live-argocd-sync-guard-agent` (above)*

---

## 🕸️ Istio — <!-- count:dir:istio:agents -->7<!-- /count --> agents → [`agents/istio/README.md`](istio/README.md)

| Agent | Tier | Load when |
|---|---|---|
| [`istio-ambient-mesh-review-agent`](istio/istio-ambient-mesh-review-agent/AGENT.md) | review | Ambient mesh, ztunnel L4 vs waypoint L7 enforcement, silent-bypass trap, PeerAuthentication, mTLS posture |

*Live Istio policy mutations → `kubernetes-live-mesh-policy-guard-agent` (above)*

---

## 🐝 Cilium — <!-- count:dir:cilium:agents -->1<!-- /count --> agents → [`agents/cilium/README.md`](cilium/README.md)

| Agent | Tier | Load when |
|---|---|---|
| [`cilium-network-policy-review-agent`](cilium/cilium-network-policy-review-agent/AGENT.md) | review | CiliumNetworkPolicy, ClusterMesh trust, 169.254.169.254 egress posture, WireGuard encryption |

*Live Cilium policy mutations → `kubernetes-live-network-policy-guard-agent` (above)*

---

## 📡 OpenTelemetry — <!-- count:dir:opentelemetry:agents -->1<!-- /count --> agents → [`agents/opentelemetry/README.md`](opentelemetry/README.md)

| Agent | Tier | Load when |
|---|---|---|
| [`opentelemetry-collector-config-review-agent`](opentelemetry/opentelemetry-collector-config-review-agent/AGENT.md) | review | Collector pipeline, memory_limiter position, receiver exposure, exporter cardinality, no-exporter silent loss |

---

## 🟩 Terraform / OpenTofu — <!-- count:dir:terraform:agents -->9<!-- /count --> agents → [`agents/terraform/README.md`](terraform/README.md)

Engine-shared board: one provider covers Terraform and OpenTofu, and every specialist
names the engine behind any version-sensitive claim. See
[`docs/terraform-opentofu-boundary.md`](../docs/terraform-opentofu-boundary.md).
Every agent here is `static-review`; execution always leaves for a cloud live-guard
agent after a human gate.

| Agent | Tier | Load when |
|---|---|---|
| [`terraform-maestro-agent`](terraform/terraform-maestro-agent/AGENT.md) | router | Any IaC task; classifies and routes, applying documented thresholds before adding a second specialist |
| [`terraform-plan-blast-radius-agent`](terraform/terraform-plan-blast-radius-agent/AGENT.md) | review | A plan replaces or destroys something; lifecycle ordering, address churn, `-target`, whether the reviewed plan binds the apply |
| [`terraform-state-reliability-agent`](terraform/terraform-state-reliability-agent/AGENT.md) | review | Backend or locking config, recovery posture, a proposed `state mv`/`state rm`/`force-unlock`, secrets in state, OpenTofu state encryption |
| [`terraform-estate-reconciliation-agent`](terraform/terraform-estate-reconciliation-agent/AGENT.md) | review | Drift disposition, brownfield `import`, and `moved`/`removed` refactors — making the record match reality without destroying anything |
| [`terraform-supply-chain-integrity-agent`](terraform/terraform-supply-chain-integrity-agent/AGENT.md) | review | Provider/module source addresses, `.terraform.lock.hcl` platform coverage, mirrors, `dev_overrides`, registry provenance |
| [`terraform-engine-compatibility-agent`](terraform/terraform-engine-compatibility-agent/AGENT.md) | review | Core or provider-major upgrades, deprecation exposure, and the Terraform-versus-OpenTofu decision as a divergence register |
| [`terraform-policy-evidence-agent`](terraform/terraform-policy-evidence-agent/AGENT.md) | review | Control mapping, enforcement level (advisory / soft-mandatory / hard-mandatory), exception scope and expiry, audit evidence artifacts |
| [`terraform-execution-governance-agent`](terraform/terraform-execution-governance-agent/AGENT.md) | review | Runner identity and credential lifetime, plan-to-apply binding, plan-artifact handling, approval integrity, trigger surface |
| [`terraform-reviewer`](terraform/terraform-reviewer/AGENT.md) | review | Module *contract* review only — input validation, output stability, breaking-change classification, golden paths. Plan, state, policy, and upgrade work belong to the specialists above |

---

## 💰 FinOps / Multi-cloud — <!-- count:dir:finops:agents -->4<!-- /count --> agents → [`agents/finops/README.md`](finops/README.md)

| Agent | Tier | Load when |
|---|---|---|
| [`finops-cloud-price-advisor-agent`](finops/finops-cloud-price-advisor-agent/AGENT.md) | review | Live public pricing from AWS + Azure + OCI APIs; cost estimation for live or prototype environments |

---

## Operational rules

- Move agents by updating both `metadata.json` and `catalog/agents.json` in the same commit.
- Run `npm run validate` after any agent metadata change.
- Never auto-dispatch a live-guard agent from a router or orchestration flow — the human must confirm target + current state first.
- Never flatten harness variants into the provider root; canonical identity always lives in `AGENT.md`.
- IDs are always `-agent` suffixed to avoid collision with skill IDs.
- `AGENT.md` and Markdown harness adapters must be flush-left after frontmatter; indented content renders as code blocks.

## Load sequence for multi-domain tasks

1. Start with the domain's maestro (AWS / Azure / OCI / Kubernetes / Terraform).
2. Maestro classifies and dispatches ≤4 specialists in parallel.
3. For Kubernetes tasks spanning mesh + network + admission: load `kubernetes-maestro-agent` — it holds the full K8s routing table and multi-domain dispatch logic.
4. Never load a live-guard agent without explicit operator intent; maestros surface the live-guard name but do not call it directly.
