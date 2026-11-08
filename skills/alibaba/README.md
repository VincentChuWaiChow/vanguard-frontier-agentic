# 🟠 Alibaba Cloud Skills

<p align="center">
  <img src="../../assets/logos/cloud/alibaba/alibaba-cloud.svg" alt="Alibaba Cloud logo" width="140" />
</p>

This folder contains Alibaba Cloud-focused skills curated for this marketplace.

## Local marketplace portfolio

This folder contains these local Alibaba Cloud skills (see `catalog/skills.json` for the authoritative inventory):

- `alibaba-ack-container-platform-operator`
- `alibaba-actiontrail-audit-analyst`
- `alibaba-analyticdb-realtime`
- `alibaba-certificate-manager-issuer-review`
- `alibaba-change-impact-advisor`
- `alibaba-china-compliance`
- `alibaba-cost-anomaly-watch-coordinator`
- `alibaba-cost-finops-analyst`
- `alibaba-daily-operations-briefing-coordinator`
- `alibaba-devops-cicd-operator`
- `alibaba-ecs-compute-operator`
- `alibaba-event-driven-architecture-review`
- `alibaba-function-serverless-operator`
- `alibaba-iac-change-safety-review`
- `alibaba-kms-secret-lifecycle-steward`
- `alibaba-landing-zone-architect`
- `alibaba-live-ack-rollout-guard`
- `alibaba-live-cost-budget-action-guard`
- `alibaba-live-kms-key-mutation-guard`
- `alibaba-live-oss-bucket-policy-guard`
- `alibaba-live-ram-policy-change-guard`
- `alibaba-live-rds-polardb-mutation-guard`
- `alibaba-load-balancer-traffic-engineer`
- `alibaba-maestro`
- `alibaba-maxcompute-dataworks-analyst`
- `alibaba-migration-architect`
- `alibaba-mse-microservice-engine`
- `alibaba-network-architect`
- `alibaba-observability-incident-responder`
- `alibaba-oss-data-perimeter-governor`
- `alibaba-oss-storage-steward`
- `alibaba-polardb-rds-dba`
- `alibaba-ram-iam-review`
- `alibaba-registry-artifact-governor`
- `alibaba-resilience-bcdr-review`
- `alibaba-security-center-hardening`
- `alibaba-serverless-production-readiness`
- `alibaba-solution-architect`
- `alibaba-support-incident-coordinator`
- `alibaba-tablestore-architect`
- `alibaba-ticket-triage-escalation-coordinator`
- `alibaba-waf-cost-optimization-review`
- `alibaba-waf-reliability-review`
- `alibaba-waf-security-review`

## Portfolio posture

Role-based Alibaba Cloud skills for evidence-backed architecture, operations, security, networking, FinOps, and guarded live-environment operations.

These skills are intentionally conservative:

- prefer Alibaba Cloud Console and `aliyun` CLI evidence for live state grounding
- prefer read-only discovery before mutation
- require explicit account ID, region (clearly distinguish CN-* mainland China regions from international regions), principal, approval, rollback posture, and verification for guarded live actions
- challenge overly broad RAM policies, public OSS bucket exposure, missing MLPS 2.0 controls, and unclear Resource Directory boundaries
- note that **China mainland (cn-*) and international regions use separate billing accounts** — always confirm which account context applies
- `alibaba-china-compliance` is a prerequisite check before any workload launched in CN-* regions is considered production-ready
- use official Alibaba Cloud documentation at https://www.alibabacloud.com/help/en/ when service behavior matters

Run `npm run validate` after changing cataloged Alibaba Cloud skills.
