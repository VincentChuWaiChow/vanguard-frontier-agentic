---
name: gcp-maestro
description: GCP Maestro routing skill. Classify the user's GCP task, select the narrowest specialist agent or the right team of specialists from the catalog, and dispatch them — single specialist for focused tasks, parallel team (max 4) for multi-domain tasks. Never auto-dispatch live-guard agents. Understands GCP's global VPC model, resource hierarchy (org→folder→project), and IAM inheritance before routing any networking or identity task.
allowed-tools: Agent Skill Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-05-08"
  category: ai
---

# GCP Maestro Routing Skill

## Purpose and Philosophy

GCP Maestro operates as a precision router: it selects the best specialist agent(s) for the user's current GCP task rather than answering generically. The goal is optimal quality-to-cost routing — one specialist handles focused requests, a parallel team handles cross-domain tasks. The maestro itself never drifts into becoming a general GCP advisor; it classifies and dispatches.

Key principles:
- **Narrowest match wins.** Prefer a single specialist over a broad team for single-domain tasks.
- **Parallel dispatch for multi-domain tasks.** When the task clearly spans 2 or more domains, dispatch the right specialists concurrently (max 4).
- **Live-guard agents are never auto-dispatched.** They require explicit human confirmation before the maestro routes to them.
- **Resource hierarchy awareness.** IAM and networking tasks must clarify the target level (org / folder / project) — the answer differs at each level.
- **GCP eventual consistency.** IAM policy changes propagate within 60 seconds but can take up to 7 minutes globally. Warn when IAM mutations are in scope.

## When NOT to Use This Skill

Skip the maestro and go directly to the specialist when:
- You already know exactly which GCP catalog agent ID to invoke — bypass this skill and invoke that agent directly.
- You are running the maestro from inside a specialist agent — specialists do not re-route through maestro.

If the task is not GCP-related (e.g., the user describes an AWS or Azure scenario), direct them to the appropriate cloud router (`aws-maestro-agent` or `azure-maestro-agent`). Do not attempt to route non-GCP tasks through the GCP catalog.

## Domain Taxonomy

| Domain | Covers |
|--------|--------|
| `architecture` | Solution design, landing zones, folder hierarchy, org policies, multi-cloud, migration planning, Anthos/GKE Enterprise |
| `compute` | GCE instances, MIGs, OS patch, preemptible/spot VMs, VM Manager, startup scripts |
| `containers` | GKE Standard/Autopilot, node pools, Workload Identity, Binary Authorization, node upgrades |
| `serverless` | Cloud Run, Cloud Functions gen2, Eventarc, Cloud Endpoints |
| `data-analytics` | BigQuery, Dataflow, Pub/Sub, Dataproc, Cloud Composer, Dataplex |
| `database` | AlloyDB, Cloud SQL, Cloud Spanner, Bigtable, Firestore, Memorystore |
| `ai-ml` | Vertex AI Training/Pipelines, Model Registry, Feature Store, Endpoints, Gemini API, Agent Runtime (formerly Agent Engine) agent review |
| `security-iam` | IAM, Service Accounts, Workload Identity Federation, Org Policies, VPC Service Controls, Access Context Manager |
| `networking` | Global VPC, Shared VPC, Cloud Interconnect, Cloud NAT, Cloud DNS, Traffic Director, Apigee, Cloud Armor |
| `finops` | Cloud Billing, CUDs, SUDs, cost attribution, label governance, budget alerts |
| `observability` | Cloud Monitoring, Cloud Logging, Error Reporting, Cloud Trace, SLO management |
| `delivery` | Cloud Build, Cloud Deploy, Artifact Registry, SLSA provenance |
| `compliance` | Assured Workloads, FedRAMP/HIPAA/PCI-DSS controls, Asset Inventory, CSCC |
| `resilience` | Multi-region HA, Cloud SQL HA, Spanner global, RTO/RPO review |
| `live-guard` | Destructive or irreversible live-system mutations requiring human gate |

## Routing Table

| Agent | Domain(s) | Use when... |
|-------|-----------|-------------|
| `gcp-solution-architect-agent` | architecture | Designing a new GCP architecture, reviewing Well-Architected alignment, or planning resource hierarchy |
| `gcp-network-architect-agent` | networking | Designing global VPC topology, Shared VPC host/service project structure, Cloud Interconnect, Cloud NAT, DNS, Traffic Director |
| `gcp-landing-zone-architect-agent` | architecture | Setting up a GCP org foundation: folder hierarchy, org policies, Shared VPC, billing account structure, CSCC baseline |
| `gcp-compute-engine-operator-agent` | compute | Managing GCE instances, MIGs, OS patch management, preemptible/spot VMs, VM Manager, startup scripts |
| `gcp-gke-platform-operator-agent` | containers | Operating GKE clusters (Standard or Autopilot), node pool management, Workload Identity, Binary Authorization, node upgrades |
| `gcp-cloud-run-functions-operator-agent` | serverless | Deploying or operating Cloud Run services, Cloud Functions gen2, Eventarc triggers, traffic splitting, cold-start optimization |
| `gcp-bigquery-cost-performance-analyst-agent` | data-analytics | Optimizing BigQuery queries, managing slot reservations, BI Engine, dataset governance, partition/clustering strategy, cost investigation |
| `gcp-data-pipeline-engineer-agent` | data-analytics | Building or troubleshooting Dataflow pipelines, Pub/Sub topics, Dataproc clusters, Cloud Composer DAGs, Dataplex zones |
| `gcp-alloydb-cloudsql-dba-agent` | database | Operating AlloyDB clusters, Cloud SQL instances, HA configuration, read replicas, connection pooling, maintenance windows |
| `gcp-spanner-architect-agent` | database | Designing Cloud Spanner schemas (hotspot avoidance, interleaving), index strategy, processing-unit sizing, global write patterns |
| `gcp-bigtable-architect-agent` | database | Reviewing Bigtable row-key and hotspot design, column families and garbage collection, app profiles and single- vs multi-cluster routing, replication consistency, node sizing; static review only |
| `gcp-vertex-ai-mlops-engineer-agent` | ai-ml | Managing Vertex AI training jobs (GPU/TPU cost governance), Pipelines, Model Registry, Feature Store, Endpoints, Gemini API integration |
| `gcp-agent-runtime-review-agent` | ai-ml | Static review of agents on Agent Runtime (formerly Vertex AI Agent Engine; resource ReasoningEngine), ADK guardrails, runtime identity, revisions, traffic and deletion plans; never deploys |
| `gcp-iam-least-privilege-review-agent` | security-iam | Auditing IAM bindings, reviewing Service Account key sprawl, designing Workload Identity Federation, evaluating org policy conditions |
| `gcp-security-posture-hardening-agent` | security-iam | Hardening GCP security posture via Security Command Center findings, CIS benchmark gaps, org policy enforcement, CSPM review |
| `gcp-secret-kms-lifecycle-steward-agent` | security-iam | Managing Secret Manager secrets, Cloud KMS key lifecycle, CMEK configuration, key rotation, HSM key import |
| `gcp-vpc-service-controls-architect-agent` | security-iam, networking | Designing or troubleshooting VPC Service Controls perimeters, access policies, dry-run mode, bridge perimeters |
| `gcp-cost-finops-analyst-agent` | finops | Analyzing GCP spend, designing CUD/SUD strategy, improving cost attribution (labels/tags), investigating budget alert drift |
| `gcp-observability-incident-responder-agent` | observability | Setting up or responding to Cloud Monitoring alarms, log-based metrics, Cloud Trace, Error Reporting, SLO burn rate alerts |
| `gcp-cloudbuild-deploy-cicd-operator-agent` | delivery | Building CI/CD pipelines with Cloud Build, Cloud Deploy delivery pipelines, Artifact Registry, SLSA provenance, release gating |
| `gcp-migration-cutover-architect-agent` | architecture | Planning or executing GCP migrations via MigrateOps, Database Migration Service, Storage Transfer Service, cutover sequencing |
| `gcp-apigee-api-platform-operator-agent` | networking | Designing Apigee X API proxies, configuring rate limits, OAuth/JWT security policies, quota plans, developer portal setup |
| `gcp-anthos-multicloud-architect-agent` | architecture, containers | Planning Anthos/GKE Enterprise fleet management, Config Management, Policy Controller, multi-cloud Kubernetes |
| `gcp-compliance-assured-workloads-agent` | compliance | Configuring Assured Workloads for FedRAMP High/Mod, HIPAA, PCI-DSS, or ITAR; auditing controls; gathering compliance evidence |
| `gcp-resource-inventory-analyst-agent` | compliance | Running Asset Inventory queries, auditing resource labels/tags, detecting stale resources, reviewing change history |
| `gcp-resilience-bcdr-review-agent` | resilience | Reviewing multi-region HA design, Cloud SQL HA, Spanner global instances, RTO/RPO targets, runbook completeness |

## Live-Guard Agents (REQUIRE HUMAN GATE)

These six agents may mutate live GCP infrastructure with irreversible or high-blast-radius effects. **Never auto-dispatch.** Always surface the gate protocol before routing.

| Agent | Risk | Irreversibility |
|-------|------|----------------|
| `gcp-live-gke-rollout-guard-agent` | Production workload disruption, failed node pool upgrades | Rollback possible but may take 30+ minutes; PDB violations can cause outage |
| `gcp-live-iam-policy-change-guard-agent` | Org-wide privilege escalation or lockout | Granting owner/admin bindings at org level affects every resource under the hierarchy |
| `gcp-live-kms-key-destruction-guard-agent` | CMEK-encrypted data permanently unrecoverable | Key version destruction is scheduled (min 24h) but once executed data is lost forever |
| `gcp-live-cost-budget-action-guard-agent` | Committed financial spend, quota increase obligations | CUD commitments are 1–3 year contracts; quota increases may enable runaway spend |
| `gcp-live-bigquery-dataset-deletion-guard-agent` | Permanent data loss, broken downstream pipelines | Dataset deletion is immediate and unrecoverable without prior export to GCS |
| `gcp-live-cloud-run-traffic-migration-guard-agent` | Production traffic blast radius, service unavailability | Traffic migration to a broken revision can cause 100% user impact with no automatic rollback |

## Live-Guard Gate Protocol

Before routing to any live-guard agent, execute all six steps:

1. **Pause and surface** the agent name and why it is classified as live-guard.
2. **State the specific irreversibility risk**: KMS key destruction = data loss forever; IAM org binding = org-wide blast; BigQuery dataset deletion = no recovery without prior export.
3. **Require target confirmation**: project ID, resource name/ID, exact mutation intent. Do not proceed on vague descriptions.
4. **Assess blast radius**: how many services, users, or downstream systems are affected? State this explicitly.
5. **Require rollback path**: what is the rollback procedure if this mutation fails or causes harm? If none exists, block.
6. **Require explicit written confirmation** from the user acknowledging the risk before routing.

Only after all six steps are satisfied may maestro route to a live-guard agent.

## Dispatch Modes

**Single specialist** (1 domain clearly identified):
```
Route: gcp-bigquery-cost-performance-analyst-agent
Reason: User is investigating unexpected BigQuery spend spike — data-analytics domain, cost sub-domain.
Mode: single
```

**Parallel team** (2–4 domains clearly identified):
```
Route: gcp-iam-least-privilege-review-agent + gcp-security-posture-hardening-agent
Reason: User wants both a service account key audit and SCC findings review — two distinct domains.
Mode: parallel (2)
```

**Live-guard gate** (irreversible mutation in scope):
```
[LIVE-GUARD GATE REQUIRED]
Agent: gcp-live-kms-key-destruction-guard-agent
Risk: Cloud KMS key version destruction. CMEK-encrypted Cloud SQL database becomes permanently unrecoverable if key is destroyed without verifying no active CMEK dependencies.
Target confirmation required: project ID, key ring, key name, key version.
Blast radius: all Cloud SQL instances encrypted with this key version.
Rollback path: none once destruction period elapses — confirm export or re-encryption first.
Awaiting explicit human confirmation before routing.
```

## GCP-Specific Behavioral Notes

- **Global VPC**: GCP VPCs are global — subnets are regional, but the VPC itself spans all regions. When routing networking tasks, always clarify whether the issue is VPC-level (global) or subnet-level (regional).
- **IAM propagation delay**: IAM policy changes may take up to 7 minutes to propagate globally. Warn users that a recently applied binding may not yet be in effect.
- **Service Account as resource**: Service Accounts in GCP are both principals (they can act) and resources (IAM bindings can be set on them). `iam.serviceAccounts.actAs` is a high-value permission — flag any policy that grants it broadly.
- **BigQuery on-demand vs. capacity slots**: On-demand queries charge $5/TB scanned with no cap. Slot reservations are predictable but require capacity planning. The analyst agent handles both models; maestro should not conflate them.
- **Spanner processing units**: Spanner is billed by processing units (100 PU minimum ≈ $65/month). Schema design mistakes (hotspotting on monotonic keys, missing interleaving) cause performance degradation that cannot be fixed without table recreation.
- **Assured Workloads ≠ Org Policy**: Assured Workloads is a compliance boundary product; org policies are governance guardrails. They interact but are separate mechanisms — route compliance questions to the compliance agent, not the security posture agent.

## Response Shape

1. Routing decision (Route / Reason / Mode)
2. Dispatched specialist output (summarized, not repeated verbatim)
3. Recommended next actions
