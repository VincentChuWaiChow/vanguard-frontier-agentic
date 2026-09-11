# Agents

Role definitions for repeatable review, architecture, operations, and bounded execution work. **<!-- count:global:agents -->735<!-- /count --> enterprise-grade, audit-ready agents** organized in a three-layer ecosystem: maestro routers, domain specialists, and cross-functional protocols.

## Three-layer agentic architecture

Every domain in Vanguard Frontier — cloud providers, Kubernetes, compliance, and business functions (Legal, HR, Marketing) — follows the same deliberate three-layer structure:

| Layer | Role | Examples | Principle |
| ----- | ---- | -------- | --------- |
| **1. 🧭 Maestro (router)** | Entry point. Classifies requests, routes to specialists, never executes risk itself. | `legal-maestro-agent`, `hr-maestro-agent`, GCP maestro, AWS maestro, Kubernetes maestro | **Separation of concerns.** Routing is a recommendation; humans confirm. |
| **2. 🤖 Specialists** | Domain experts with hardened permission models and guarded verdict shapes. Each loads companion skills and emits structured risk analysis. | 13 Legal specialists, 15 HR specialists, 47 AWS advisory agents, 15 Kubernetes agents | **Judgment + guardrails.** Specialists analyze; they do not authorize. |
| **3. 🔗 Cross-functional protocol** | Shared contracts for handoff, escalation, and coordination across boundaries without leaking scope or privilege. | `legal-hr-routing-protocol`, `legal-hr-case-capsule`, `legal-hr-risk-taxonomy` | **Auditability.** Every handoff is traceable; privilege is preserved. |

**How it flows:** a request enters at the **maestro**, which routes to a **specialist**. When a matter crosses a domain boundary — an HR investigation that needs legal review, or a data breach that triggers compliance — the **cross-functional protocol** carries a structured, redacted payload (case capsule) between agents while preserving privilege and recording the escalation path.

---

## Business-function agents — the cross-domain ecosystem

Beyond cloud infrastructure, Vanguard Frontier ships agents for Legal, HR, Marketing, QA/CI-CD, and FinOps — proof that agentic coordination works across organizational boundaries, not just inside one cloud account.

| Domain | Agents | Maestro | Key specialisations | README |
| ------ | ------: | ------- | ------------------- | ------ |
| ⚖️ **Legal** | 13 | `legal-maestro-agent` | Contract review, employment law risk, privacy & data protection, regulatory compliance, IP & open source, litigation & discovery, ethics & investigations, vendor/procurement risk, policy governance, public disclosure, knowledge management | [`legal/README.md`](legal/README.md) |
| 👥 **HR** | 15 | `hr-maestro-agent` | Employee relations, workplace investigations, performance management, compensation & equity, benefits & payroll, recruiting & selection, workforce planning & RIF, leave & accommodation, learning policy, culture & DEI, people analytics, HRIS process controls, termination readiness, risk triage | [`hr/README.md`](hr/README.md) |
| 🔗 **Cross-functional protocol** | 3 skills | — | `legal-hr-routing-protocol`, `legal-hr-case-capsule`, `legal-hr-risk-taxonomy` | [`skills/cross-functional/`](/skills/cross-functional/) |
| 📣 **Marketing** | 14 | `marketing-maestro-agent` | Governance review, compliance, brand, messaging, regulatory alignment | [`marketing/README.md`](marketing/README.md) |
| 🧪 **QA / CI-CD** | 10+ agents | — | Test coverage, E2E execution, flakiness triage, CI pipeline review, deployment gates | [`qa/`](qa/) |
| 💰 **FinOps** | 1 | — | Cross-cloud cost optimization, AI economics modeling, Kubernetes rightsizing | [`finops/README.md`](finops/README.md) |

### Core example — the Legal + HR ecosystem

The **Legal + HR ecosystem is the proof of concept** for cross-functional agentic coordination. Here's why it matters:

- **<!-- count:board:legal+hr:agents -->28<!-- /count --> specialist agents** (<!-- count:board:legal:agents -->13<!-- /count --> Legal + <!-- count:board:hr:agents -->15<!-- /count --> HR, each with a maestro router) handle the most legally exposed, audit-heavy, escalation-aware domains in an organization
- **3 cross-functional protocol skills** (`legal-hr-routing-protocol`, `legal-hr-case-capsule`, `legal-hr-risk-taxonomy`) define how agents hand off, escalate, and coordinate across organizational boundaries
- **Real handoff scenarios:** whistleblower reports → both Legal (privilege) and HR (investigation) own it; wrongful-termination exposure → Legal flags risk, HR confirms readiness before execution; data breach → Legal holds, HR freezes personnel actions
- **Audit-ready design:** every handoff is logged; every escalation has a named decision owner; every irreversible action routes to a human
- **No legal or HR advice:** these agents do NOT replace licensed counsel or qualified HR professionals. They triage, analyze, and escalate. All outputs are inputs for human review.

See [`legal/README.md`](legal/README.md), [`hr/README.md`](hr/README.md), and [`docs/architecture/legal-hr-agent-routing.md`](/docs/architecture/legal-hr-agent-routing.md) for details.

---

## Cloud provider agents — organized by execution tier

Vanguard Frontier's cloud agent portfolios are organized by execution tier: advisory (read-only), execution (workspace-write), and live operators (guarded cloud mutations).

### Provider catalog

| Provider | Current status | Agents | Notes |
| --- | --- | ---: | --- |
| 🟧 AWS | active | <!-- count:board:aws:agents -->47<!-- /count --> | advisory, repo-write execution, and guarded live-AWS operator agents |
| 🟥 OCI | active | <!-- count:board:oci:agents -->39<!-- /count --> | advisory and guarded live-OCI operator agents |
| 🟩 GCP | active | <!-- count:board:gcp:agents -->51<!-- /count --> | advisory, live-guard operators, maestro router |
| 🟦 Azure | active | <!-- count:board:azure:agents -->36<!-- /count --> | advisory and guarded live-Azure operator agents |
| 🟠 Alibaba Cloud | active | <!-- count:board:alibaba:agents -->43<!-- /count --> | advisory, live-guard operators, maestro router |
| 🔴 Huawei Cloud | active | <!-- count:board:huawei:agents -->43<!-- /count --> | advisory, live-guard operators, maestro router |
| ☁️ OVHcloud | active | <!-- count:board:ovhcloud:agents -->6<!-- /count --> | maestro router, IAM, FinOps, MCK, network architect, KMS live-guard |
| 🌐 IONOS Cloud | active | <!-- count:board:ionos:agents -->6<!-- /count --> | maestro router, DCD review, GDPR/compliance, K8s, FinOps, DBaaS live-guard |
| 🇫🇷 Scaleway | active | <!-- count:board:scaleway:agents -->6<!-- /count --> | maestro router, IAM, Kapsule, FinOps, network, Kapsule rollout live-guard |
| 🇩🇪 Hetzner Cloud | active | <!-- count:board:hetzner:agents -->6<!-- /count --> | maestro router, FinOps, infra review, capacity, firewall + server live-guards |
| 💰 Contabo | active | <!-- count:board:contabo:agents -->6<!-- /count --> | maestro router, FinOps, capacity, security hardening, instance + storage live-guards |
| ☸️ Kubernetes | active | <!-- count:board:kubernetes:agents -->16<!-- /count --> | RBAC, workload identity, PSA, live-guards, maestro |
| 🟩 Terraform | active | <!-- count:board:terraform:agents -->9<!-- /count --> | generic IaC review |
| 💰 Multi-cloud | limited | <!-- count:board:multi-cloud:agents -->3<!-- /count --> | FinOps cross-cloud price advisor |
| CNCF ecosystem | active | <!-- count:board:kyverno+argocd+istio+cilium+opentelemetry+prometheus+falco+sigstore+cert-manager+fluxcd+backstage:agents -->18<!-- /count --> | Kyverno, Argo CD, Istio, Cilium, OTEL, Prometheus, Falco, Sigstore, cert-manager, FluxCD, Backstage. Velero ships skills only — no agents — so it is not counted here |

## Agent tiers

All provider portfolios follow the same three-tier model:

### Advisory agents (read-only by default)

Use for review, diagnosis, planning, briefing, triage, and non-destructive coordination. These never write to live cloud environments.

### Execution agents (workspace-write)

Workspace-write in Codex but still non-destructive toward live cloud by default. Used for patching repo files — manifests, IaC, CI/CD configs, rollout definitions.

| Agent | Provider | Primary use |
| --- | --- | --- |
| `aws-deployment-hotfix-operator-agent` | AWS | rapid repo-side deployment corrections |
| `aws-iac-patch-executor-agent` | AWS | bounded IaC patching |
| `aws-pipeline-fix-operator-agent` | AWS | CI/CD config fixes |
| `aws-serverless-rollout-corrector-agent` | AWS | serverless rollout file corrections |
| `aws-ecs-service-remediation-operator-agent` | AWS | ECS/Fargate config remediation |

### Guarded live operators

Workspace-write in Codex, but designed for repos or shells connected to real cloud credentials or deployment authority. They must confirm target identity, require explicit approval, prefer preview or dry-run evidence, and define rollback plus post-change verification before mutation.

See each provider's README for the full live-guard catalog:

- [`agents/aws/README.md`](aws/README.md) — AWS live operators (5)
- [`agents/gcp/README.md`](gcp/README.md) — GCP live operators (6)
- [`agents/alibaba/README.md`](alibaba/README.md) — Alibaba Cloud live operators (6)
- [`agents/huawei/README.md`](huawei/README.md) — Huawei Cloud live operators (6)
- [`agents/azure/README.md`](azure/README.md) — Azure live operators (7)
- [`agents/oci/README.md`](oci/README.md) — OCI live operators (7)
- [`agents/ovhcloud/README.md`](ovhcloud/README.md) — OVHcloud live operators (1: KMS key destruction)
- [`agents/ionos/README.md`](ionos/README.md) — IONOS Cloud live operators (1: DBaaS lifecycle)
- [`agents/scaleway/README.md`](scaleway/README.md) — Scaleway live operators (1: Kapsule rollout)
- [`agents/hetzner/README.md`](hetzner/README.md) — Hetzner Cloud live operators (2: firewall, server lifecycle)
- [`agents/contabo/README.md`](contabo/README.md) — Contabo live operators (2: instance, storage)
