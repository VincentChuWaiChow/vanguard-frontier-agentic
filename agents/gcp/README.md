# 🟩 GCP Agents

<p align="center">
  <img src="../../assets/logos/cloud/gcp/google-cloud.svg" alt="Google Cloud logo" width="140" />
</p>

Google Cloud Platform agent catalog for this marketplace. 😄

## 🧱 Agent tiers

| Tier | Purpose | Default access | Live GCP mutation |
|---|---|---|---|
| Role / advisory agents | Review, design, diagnose, coordinate | read-only | not allowed by default |
| Guarded live operators | Work in repos or shells that may target real GCP environments | workspace-write | approval-gated and target-confirmed only |

## 🚦 Guarded live-GCP operators

| Agent | Primary use | Default live posture | Must refuse when |
|---|---|---|---|
| `gcp-live-gke-rollout-guard-agent` | live GKE deployment and node pool mutations | PDB audit + health evidence + rollback required | rollout safety signals are weak or cluster control-plane change is unreviewed |
| `gcp-live-iam-policy-change-guard-agent` | live IAM binding mutations and org policy changes | current bindings snapshot + blast-radius review + approval required | SA key creation or org-level policy change lacks dual approval |
| `gcp-live-kms-key-destruction-guard-agent` | Cloud KMS key version destruction and key ring deletion | rotation-vs-destruction separation + CMEK usage audit + waiting period | key is in active CMEK use and destruction has no scheduled waiting period |
| `gcp-live-cost-budget-action-guard-agent` | live budget threshold mutations and CUD commitments | budget baseline + alert thresholds + quota read-only | budget action would remove cost alerts or CUD commitment lacks approval |
| `gcp-live-bigquery-dataset-deletion-guard-agent` | BigQuery dataset deletion and table truncation | full table inventory + authorized view audit + retention check | dataset contains billable partitions or authorized views with active downstream consumers |
| `gcp-live-cloud-run-traffic-migration-guard-agent` | Cloud Run revision traffic percentage changes | revision health + min-instances review + rollback percentage plan | traffic shift requested without health checks or rollback revision confirmed |

## 👀 Read-only advisory agents

| Agent | Focus |
|---|---|
| `gcp-maestro-agent` | classify and route GCP tasks to the narrowest specialist |
| `gcp-iam-least-privilege-review-agent` | IAM bindings, SA key sprawl, Workload Identity Federation |
| `gcp-security-posture-hardening-agent` | Security Command Center posture, Assured Workloads, org policies |
| `gcp-cost-finops-analyst-agent` | Billing exports, CUDs, SUDs, label governance, budget alerts |
| `gcp-gke-platform-operator-agent` | GKE Standard/Autopilot, node pools, Workload Identity, upgrades |
| `gcp-vertex-ai-mlops-engineer-agent` | Vertex AI training cost governance, Pipelines, Feature Store |
| `gcp-agent-runtime-review-agent` | static review of Agent Runtime (formerly Agent Engine) and ADK agents |
| `gcp-network-architect-agent` | global VPC design, Shared VPC, Cloud Interconnect, Cloud NAT |
| `gcp-resilience-bcdr-review-agent` | multi-region HA, Cloud SQL HA, Spanner global, RTO/RPO, runbook gaps |
| `gcp-iac-change-safety-review-agent` | Terraform/Deployment Manager blast radius, deletion protection, plan safety |
| `gcp-event-driven-architecture-review-agent` | Pub/Sub dead-letter, Eventarc, Cloud Tasks ordering, retry storm prevention |
| `gcp-load-balancer-traffic-engineer-agent` | Global/Regional HTTPS LB, Cloud Armor, health checks, connection draining |
| `gcp-serverless-production-readiness-agent` | Cloud Run min-instances, cold-start, VPC connector, secrets injection |
| `gcp-certificate-manager-issuer-review-agent` | Certificate Manager issuance, Google-managed vs self-managed, LB binding |
| `gcp-cost-anomaly-watch-coordinator-agent` | BigQuery $5/TB scan cost alerts, committed-use gaps, idle resource waste |
| `gcp-change-impact-advisor-agent` | pre-change blast radius: org hierarchy, VPC scope, BigQuery dataset consumers |
| `gcp-registry-artifact-governor-agent` | Artifact Registry vulnerability scanning, tag immutability, retention policies |
| `gcp-gcs-data-perimeter-governor-agent` | GCS bucket public exposure, VPC Service Controls perimeter, IAM conditions |
| `gcp-ticket-triage-escalation-coordinator-agent` | P0/P1/P2/P3 triage, Google Cloud support SLA enforcement, evidence collection |
| `gcp-support-incident-coordinator-agent` | severity mapping, TAM escalation, evidence scrubbing, PIR rights |
| `gcp-daily-operations-briefing-coordinator-agent` | billing delta, Cloud Monitoring alert owners, GKE health, quota warnings |

## 🛡️ Operating note

- 😄 advisory agents stay read-only by default
- 🚦 guarded live operators must confirm project ID, region, IAM principal, approval, rollback, and verification before mutation
- ⚠️ VPC is global in GCP — network mutations can affect all regions; the guard always confirms scope before any routing or firewall change
- 🔑 Service Accounts are resources, not just identities — key creation has org-wide blast radius; `gcp-live-iam-policy-change-guard-agent` treats SA key creation as a live-guard action
- 🧾 all live-guard agents produce a structured verdict response — see [`docs/evidence-output-spec.md`](../../docs/evidence-output-spec.md)
