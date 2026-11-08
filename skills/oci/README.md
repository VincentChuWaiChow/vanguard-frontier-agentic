# 🟥 OCI Skills

<p align="center">
  <img src="../../assets/logos/cloud/oci/oracle-cloud-infrastructure.png" alt="Oracle Cloud Infrastructure logo" width="140" />
</p>

This folder contains OCI-focused skills curated for this marketplace.

## Local marketplace portfolio

This folder contains these local OCI skills (see `catalog/skills.json` for the authoritative inventory):

- `oci-autonomous-database-architect`
- `oci-certificates-issuer-review`
- `oci-cloud-guard-responder`
- `oci-compute-instance-agent-operator`
- `oci-compute-platform-operator`
- `oci-cost-finops-analyst`
- `oci-database-platform-dba`
- `oci-dbtools-sql-analyst`
- `oci-devops-container-platform-engineer`
- `oci-exadata-database-architect`
- `oci-exadata-platform-architect`
- `oci-fusion-apps-environment-operator`
- `oci-goldengate-replication-operator`
- `oci-identity-access-governor`
- `oci-iot-digital-twin-engineer`
- `oci-limits-capacity-planner`
- `oci-live-autonomous-db-lifecycle-guard`
- `oci-live-cost-budget-runaway-guard`
- `oci-live-iam-policy-compartment-guard`
- `oci-live-network-security-rule-guard`
- `oci-live-oke-rollout-guard`
- `oci-live-resource-manager-stack-guard`
- `oci-live-vault-key-destruction-guard`
- `oci-load-balancer-traffic-engineer`
- `oci-maestro`
- `oci-migration-cutover-architect`
- `oci-multi-cloud-architect`
- `oci-mysql-heatwave-ai-specialist`
- `oci-network-architect`
- `oci-nosql-database-architect`
- `oci-observability-incident-responder`
- `oci-recovery-service-operator`
- `oci-registry-artifact-governor`
- `oci-resource-search-inventory-analyst`
- `oci-security-compliance-reviewer`
- `oci-solution-architect`
- `oci-storage-backup-steward`
- `oci-support-incident-coordinator`
- `oci-waf-cost-optimization-review`
- `oci-waf-reliability-review`
- `oci-waf-security-review`
- `oracle-oci-mcp-grounded-advisor`

## Portfolio posture

Role-based OCI skills for evidence-backed architecture, database operations, security, networking, FinOps, identity governance, and guarded live-environment operations.

These skills are intentionally conservative:

- prefer `oracle-oci-mcp-grounded-advisor` via OCI MCP server when available for live OCI state grounding
- prefer read-only discovery before mutation
- require explicit OCID, compartment, tenancy confirmation, approval, rollback posture, and verification for guarded live actions
- challenge overly broad IAM policies, missing compartment isolation, public exposure, and unclear resource ownership
- use official OCI documentation and live CLI evidence when service behavior matters

Run `npm run validate` after changing cataloged OCI skills.
