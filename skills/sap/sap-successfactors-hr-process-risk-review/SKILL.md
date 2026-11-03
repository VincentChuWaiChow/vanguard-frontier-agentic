---
name: sap-successfactors-hr-process-risk-review
description: "Review SAP SuccessFactors HR process risk: Employee Central data and role-based permissions (RBP), org and position management integrity, hire-to-retire process controls, payroll integration data flow, data privacy for PII and GDPR, and joiner-mover-leaver lifecycle governance. Escalates HR-sensitive and PII findings. Does not accept raw PII or touch live systems."
allowed-tools: Read Grep Glob WebSearch WebFetch
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-06-19"
  category: compliance
  lifecycle: experimental
---

# SAP SuccessFactors HR Process Risk Review

## Purpose

Assess HR process risk in SAP SuccessFactors deployments spanning Employee Central, role-based permissions (RBP), org and position management, hire-to-retire process controls, payroll integration, and joiner-mover-leaver (JML) lifecycle governance. Review Employee Central data model configuration for permission group design, permission role assignment, and access control boundary correctness. Assess org and position management integrity: position-to-headcount linkage, approval workflow configuration, and reporting line accuracy. Evaluate hire-to-retire process controls: onboarding workflow completeness, rehire duplicate prevention, termination action steps, and data archiving controls. Review payroll integration data flow governance: replication mapping correctness, payroll-relevant field access controls, and reconciliation controls between Employee Central and payroll processors. Assess data privacy posture: PII field visibility in Employee Central, GDPR-aligned consent management, data subject access and erasure readiness, and cross-border transfer controls. Review JML lifecycle governance: timely provisioning and deprovisioning, role change process completeness, and access certification coverage for HR data. Does not connect to live SuccessFactors tenants, accept raw PII records, or mutate any system configuration.

## When to use

Use this skill when the user asks to:

- review Employee Central role-based permissions (RBP) configuration: permission groups, permission roles, target populations, and whether the principle of least privilege is applied to sensitive HR data (compensation, home address, national ID, bank details),
- assess org and position management risk: whether positions are linked to approved headcount plans, whether the approval workflow for position creation or reclassification is enforced, and whether reporting line changes are governed and auditable,
- evaluate hire-to-retire process controls: onboarding workflow step completeness, rehire duplicate-record detection, termination event action steps (access revocation, equipment return, final pay triggers), and whether data archiving policy is configured,
- review payroll integration governance: field-level mapping between Employee Central and payroll processors (SAP Payroll, Ascent Payroll, third-party), data replication audit controls, and reconciliation process coverage for payroll-relevant changes,
- assess SuccessFactors data privacy posture for GDPR or local privacy regulation compliance: PII field classification in Employee Central, consent management configuration, data subject access request process readiness, erasure workflow availability, and cross-border personal data transfer documentation,
- review joiner-mover-leaver lifecycle governance: whether new hires receive appropriate access on day one, whether role changes trigger timely RBP updates, whether terminated employees are deprovisioned within the required window, and whether access certification campaigns cover HR data access,
- assess SuccessFactors integration risk where Employee Central is the system of record: integration center configuration, data replication scope, error handling for failed replication events, and downstream system propagation of HR status changes.

## When not to use

- When the request requires connecting to a live SuccessFactors tenant, running Employee Central reports, or extracting actual employee records — this skill reviews configuration exports, process descriptions, RBP role lists, integration mapping documentation, and governance artifacts only.
- When the request is about SAP HCM on-premise authorization objects (structural authorizations, HR master data authorizations in R/3 or ECC) rather than SuccessFactors cloud RBP.
- When the request is about SAP Payroll on-premise configuration, payroll schema, or payroll wage type design rather than the integration governance between Employee Central and a payroll processor.
- When the request is about SAP SuccessFactors Learning, Performance, or Recruiting modules as standalone governance topics unrelated to Employee Central data, RBP, or the hire-to-retire process.
- When the request contains actual employee records, national IDs, bank account numbers, salary figures, or other raw PII — this skill does not accept, store, or process raw personal data. Request anonymized or pseudonymized descriptions instead.

## Does not touch live systems

This skill operates on user-provided RBP configuration exports, permission role and group lists, org chart descriptions, process flow documentation, integration center mapping exports, data privacy impact assessment summaries, and written descriptions of the HR governance posture. It does not connect to any SuccessFactors tenant, Employee Central API, Integration Center endpoint, or downstream payroll system. All live inspection is out of scope.

## Lean operating rules

- Classify findings by HR governance domain before recommending. Every finding must be assigned to a domain (RBP / Org and Position / Hire to Retire / Payroll Integration / Data Privacy / JML Lifecycle) before a remediation path is proposed.
- RBP least privilege is the baseline. Any permission role that grants visibility to sensitive HR fields (compensation, bank details, national ID, home address, health data) beyond what the business function requires is a finding. Broadly scoped target populations that expose sensitive data to non-HR users are always flagged.
- Sensitive HR field access must be role-justified. Access to compensation, bank details, national ID, home address, or health-related data fields in Employee Central must have a documented business justification in the permission role design. Absence of justification is a `high` finding.
- JML delays are compliance failures. A terminated employee retaining active SuccessFactors access beyond the defined deprovisioning SLA is a `critical` finding. A new joiner without appropriate access on day one is a `high` finding.
- Payroll integration errors require escalation. Failed or unresolved replication events between Employee Central and a payroll processor that affect payroll-relevant fields (pay grade, work schedule, bank details) are `high` findings requiring immediate payroll team review.
- PII findings must not include raw data. When reviewing data privacy posture, assess field configuration and governance controls without requesting or accepting actual employee records, national IDs, bank account numbers, or health data. If the user supplies raw PII, redirect immediately — do not analyze the PII content.
- GDPR erasure readiness must be assessed, not assumed. If the organization operates SuccessFactors in the EU or processes EU employee data, the existence of a tested erasure workflow in Employee Central is a compliance requirement. Absence of a tested erasure procedure is a `high` data privacy finding.
- Escalate HR-sensitive findings to HR and legal. Any finding that involves unauthorized access to compensation, health data, or national IDs, or any confirmed PII exposure, must be escalated to HR leadership and legal or data protection team before further analysis or change is authorized.
- Evidence from user-provided governance artifacts or official SAP SuccessFactors documentation takes precedence over inference.

## Evidence rules

Label all claims with one of:

- `documentation-based` — grounded in official SAP SuccessFactors Employee Central, RBP, or integration documentation (help.sap.com)
- `user-provided evidence` — RBP configuration exports, permission role lists, org chart descriptions, integration mapping documentation, data privacy impact assessment summaries, or written process descriptions provided by the user
- `inference` — derived reasoning not directly confirmed by official docs or user evidence; must always be labeled as such

## Live-environment rules and PII rules

**This skill does not touch live systems.** There is no SuccessFactors API call, Employee Central OData query, Integration Center invocation, or payroll system connection in this skill's execution path.

**PII rules for this review skill:**

- Never accept, request, or process actual employee records, national ID numbers, bank account numbers, salary figures, home addresses, health data, or any other personal data identifying real individuals.
- If the user provides raw PII in their message, do not analyze the PII content. Immediately redirect the user to provide an anonymized or pseudonymized description of the data structure or governance concern instead.
- When assessing GDPR or data privacy posture, work from field configuration descriptions, data classification documentation, and process descriptions — not from actual employee data.
- Do not request SuccessFactors admin credentials, OData API keys, Integration Center basic auth credentials, or SFTP passwords for payroll file transfers.

## References

Load only when needed:

- [Workflow and output contract](references/workflow-and-output.md) — HR governance domain taxonomy, risk classification, output format.
- [Safety checklist](references/safety-checklist.md) — non-negotiables, common HR process risk review mistakes, when to push back.
- [Official sources](references/official-sources.md) — SAP SuccessFactors Employee Central, RBP, integration, and data privacy documentation.

## Response minimum

Return, at minimum:

- **Problem classification**: HR governance domain(s) affected (RBP / Org and Position / Hire to Retire / Payroll Integration / Data Privacy / JML Lifecycle) and specific finding(s) per domain.
- **Evidence used**: documentation-based / user-provided evidence / inference.
- **Risk level**: critical (terminated employee retaining access, confirmed PII exposure, unresolved payroll replication error affecting pay) / high (sensitive field over-permission, JML SLA breach, missing GDPR erasure workflow, unmitigated pay data access) / medium (governance gap, stale access certification, missing audit trail for org change) / low (best practice deviation, RBP design improvement).
- **Recommended action**: specific remediation per finding (RBP role redesign, target population narrowing, JML SLA enforcement, payroll error escalation, GDPR erasure workflow implementation, access certification campaign, etc.).
- **Refusal / escalation triggers**: if raw PII is supplied, redirect immediately and do not analyze the PII content. If confirmed unauthorized access to compensation or health data is found, escalate to HR leadership and legal before continuing. If live SuccessFactors access is required to complete the review, state that clearly and do not proceed.
- **Business impact**: regulatory exposure (GDPR, local labor law, SOX for payroll controls), HR data breach risk, payroll error risk, or audit finding risk.
- **Next verification step**: validate RBP remediation recommendations against current permission role assignments and Employee Central field configuration before implementing changes. Confirm payroll integration error resolution with the payroll processing team.
