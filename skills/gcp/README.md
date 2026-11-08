# 🟩 GCP Skills

<p align="center">
  <img src="../../assets/logos/cloud/gcp/google-cloud.svg" alt="Google Cloud logo" width="140" />
</p>

This folder contains GCP-focused skills curated for this marketplace.

## Local marketplace portfolio

This folder contains these local GCP skills (see `catalog/skills.json` for the authoritative inventory):

- `gcp-agent-runtime-review`
- `gcp-alloydb-ai-developer`
- `gcp-alloydb-cloudsql-dba`
- `gcp-anthos-multicloud-architect`
- `gcp-apigee-api-platform-operator`
- `gcp-bigquery-cost-performance-analyst`
- `gcp-bigtable-architect`
- `gcp-certificate-manager-issuer-review`
- `gcp-change-impact-advisor`
- `gcp-cloud-auth-advisor`
- `gcp-cloud-run-functions-operator`
- `gcp-cloudbuild-deploy-cicd-operator`
- `gcp-compliance-assured-workloads`
- `gcp-compute-engine-operator`
- `gcp-cost-anomaly-watch-coordinator`
- `gcp-cost-finops-analyst`
- `gcp-daily-operations-briefing-coordinator`
- `gcp-data-pipeline-engineer`
- `gcp-event-driven-architecture-review`
- `gcp-firebase-developer`
- `gcp-gcs-data-perimeter-governor`
- `gcp-gemini-api-developer`
- `gcp-gke-platform-operator`
- `gcp-iac-change-safety-review`
- `gcp-iam-least-privilege-review`
- `gcp-landing-zone-architect`
- `gcp-live-bigquery-dataset-deletion-guard`
- `gcp-live-cloud-run-traffic-migration-guard`
- `gcp-live-cost-budget-action-guard`
- `gcp-live-gke-rollout-guard`
- `gcp-live-iam-policy-change-guard`
- `gcp-live-kms-key-destruction-guard`
- `gcp-load-balancer-traffic-engineer`
- `gcp-maestro`
- `gcp-migration-cutover-architect`
- `gcp-network-architect`
- `gcp-networking-observability`
- `gcp-observability-incident-responder`
- `gcp-registry-artifact-governor`
- `gcp-resilience-bcdr-review`
- `gcp-resource-inventory-analyst`
- `gcp-secret-kms-lifecycle-steward`
- `gcp-security-posture-hardening`
- `gcp-serverless-production-readiness`
- `gcp-solution-architect`
- `gcp-spanner-architect`
- `gcp-support-incident-coordinator`
- `gcp-ticket-triage-escalation-coordinator`
- `gcp-vertex-ai-mlops-engineer`
- `gcp-vpc-service-controls-architect`
- `gcp-waf-cost-optimization-review`
- `gcp-waf-reliability-review`
- `gcp-waf-security-review`

## Official upstream reference

When adding or reviewing GCP skills, check the official Google skills repository first:

- https://github.com/google/skills

Use it as the primary upstream reference for GCP-specific workflow ideas, patterns, and alignment with Google-maintained guidance. This local portfolio was bootstrapped from that upstream repository's GKE, BigQuery, AlloyDB, Cloud Run, Firebase, Gemini API, and WAF-pillar skills.

## Portfolio posture

Role-based GCP skills for evidence-backed architecture, operations, security, networking, FinOps, and guarded live-environment operations.

These skills are intentionally conservative:

- prefer `gcloud` CLI and Cloud Console evidence for live GCP state grounding
- prefer read-only discovery before mutation
- require explicit project ID, region (or global scope for VPC), IAM principal, approval, rollback posture, and verification for guarded live actions
- challenge overly broad IAM bindings, SA key sprawl, public GCS/BigQuery exposure, missing VPC Service Controls perimeters, and unclear resource ownership
- note that GCP VPC is **global** — networking mutations can affect all regions; always confirm scope
- use official GCP documentation and live `gcloud` CLI evidence when service behavior matters

Run `npm run validate` after changing cataloged GCP skills.
