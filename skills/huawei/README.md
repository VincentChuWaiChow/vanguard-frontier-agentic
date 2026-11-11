# 🔴 Huawei Cloud Skills

<p align="center">
  <img src="../../assets/logos/cloud/huawei/huawei-cloud.svg" alt="Huawei Cloud logo" width="140" />
</p>

This folder contains Huawei Cloud-focused skills curated for this marketplace.

## Local marketplace portfolio

This folder contains these local Huawei Cloud skills (see `catalog/skills.json` for the authoritative inventory):

- `huawei-cce-container-platform-operator`
- `huawei-certificate-manager-issuer-review`
- `huawei-change-impact-advisor`
- `huawei-codearts-devops-operator`
- `huawei-compliance-sovereignty`
- `huawei-cost-anomaly-watch-coordinator`
- `huawei-cost-finops-analyst`
- `huawei-daily-operations-briefing-coordinator`
- `huawei-dew-kms-lifecycle-steward`
- `huawei-drs-data-replication-operator`
- `huawei-dws-dli-data-analyst`
- `huawei-ecs-compute-operator`
- `huawei-event-driven-architecture-review`
- `huawei-functiongraph-serverless-operator`
- `huawei-gaussdb-rds-dba`
- `huawei-geminidb-architect`
- `huawei-iac-change-safety-review`
- `huawei-iam-least-privilege-review`
- `huawei-ief-edge-computing-operator`
- `huawei-landing-zone-architect`
- `huawei-live-cce-rollout-guard`
- `huawei-live-cost-budget-action-guard`
- `huawei-live-gaussdb-mutation-guard`
- `huawei-live-iam-policy-change-guard`
- `huawei-live-kms-key-destruction-guard`
- `huawei-live-obs-bucket-policy-guard`
- `huawei-load-balancer-traffic-engineer`
- `huawei-maestro`
- `huawei-migration-architect`
- `huawei-modelarts-mlops-engineer`
- `huawei-network-architect`
- `huawei-obs-data-perimeter-governor`
- `huawei-obs-storage-steward`
- `huawei-observability-incident-responder`
- `huawei-registry-artifact-governor`
- `huawei-resilience-bcdr-review`
- `huawei-secmaster-security-operations`
- `huawei-serverless-production-readiness`
- `huawei-solution-architect`
- `huawei-support-incident-coordinator`
- `huawei-ticket-triage-escalation-coordinator`
- `huawei-waf-cost-optimization-review`
- `huawei-waf-reliability-review`
- `huawei-waf-security-review`

## Portfolio posture

Role-based Huawei Cloud skills for evidence-backed architecture, operations, security, networking, FinOps, and guarded live-environment operations.

These skills are intentionally conservative:

- prefer Huawei Cloud Console and `hcloud` CLI evidence for live state grounding
- prefer read-only discovery before mutation
- require explicit account ID, region, project, principal, approval, rollback posture, and verification for guarded live actions
- challenge overly broad IAM policies, missing SCP controls, DEW key sprawl, public OBS bucket exposure, and MLPS 2.0 Level 3 gaps
- note that **Enterprise Projects** are cost attribution constructs, not security boundaries — always verify IAM and SCP scope independently
- `huawei-compliance-sovereignty` is a prerequisite check before any workload is considered compliant for China government or regulated enterprise deployments
- note that ModelArts uses **Ascend NPU** flavor families in addition to Nvidia GPUs — confirm the flavor type before estimating cost or writing training job configs
- use official Huawei Cloud documentation at https://support.huaweicloud.com/intl/en-us/ when service behavior matters

Run `npm run validate` after changing cataloged Huawei Cloud skills.
