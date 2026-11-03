---
name: sap-cloud-alm-sre-incident-review
description: "Review SAP Cloud ALM for SRE and operations governance: health monitoring configuration, alerting rule coverage, integration and exception monitoring, business process monitoring, incident and problem management process, root-cause analysis workflow, and SLA and service continuity controls. Does not access live Cloud ALM tenants or mutate monitoring configuration."
allowed-tools: Read Grep Glob WebSearch WebFetch
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-06-19"
  category: observability
  lifecycle: experimental
---

# SAP Cloud ALM SRE and Incident Review

## Purpose

Assess the SRE and operations governance posture of SAP Cloud ALM deployments. Review health monitoring configuration: service and system health check coverage, monitoring scope per managed system, health status threshold calibration, and gap identification for uncovered SAP cloud and on-premise services. Assess alerting rule coverage: alert threshold design, notification channel configuration, alert routing to the correct operations team, and alert fatigue risk from miscalibrated thresholds. Review integration and exception monitoring: iFlow exception alerting in SAP Integration Suite, API proxy error rate monitoring, interface error log coverage, and exception handling in integration scenarios. Evaluate business process monitoring: key figure configuration for business-critical processes (order-to-cash, procure-to-pay, financial close), step-level process health tracking, SLA breach detection for business process milestones, and business user notification coverage. Assess incident and problem management process: incident creation workflow, severity classification, escalation path configuration, problem record linkage for recurring incidents, and knowledge article coverage. Review root-cause analysis workflow: Real User Monitoring (RUM) and synthetic monitoring integration, alert-to-incident traceability, cross-system dependency mapping for root-cause investigation, and handoff between Cloud ALM and underlying system support. Assess SLA and service continuity controls: SLA definition completeness, breach threshold alerting, service continuity plan coverage for critical SAP services, and reporting configuration for SLA performance. Does not connect to live Cloud ALM tenants, mutate monitoring configuration, or access production system data.

## When to use

Use this skill when the user asks to:

- review SAP Cloud ALM health monitoring coverage: which SAP cloud services and on-premise managed systems are registered, whether service health checks are configured for all business-critical systems, and whether monitoring gaps exist for any production service in scope,
- assess alerting rule design and threshold calibration: whether alert thresholds are set to detect meaningful degradation before SLA breach, whether notification channels are configured for all alert types, whether alert routing correctly targets the responsible operations team, and whether alert fatigue from over-sensitive thresholds is a risk,
- evaluate integration and exception monitoring in Cloud ALM: whether iFlow exceptions from SAP Integration Suite are surfaced in Cloud ALM, whether API proxy error rate alerts are configured, whether integration error logs feed into Cloud ALM incident creation, and whether key integration scenarios have end-to-end monitoring coverage,
- review business process monitoring configuration: whether SAP Cloud ALM business process monitoring key figures are defined for critical business processes, whether step-level health tracking covers order-to-cash, procure-to-pay, and financial close milestones, whether SLA breach detection is configured per process milestone, and whether business users receive proactive notifications for process delays,
- assess incident and problem management process governance: whether SAP Cloud ALM incident management is integrated with the organization's ITSM platform (ServiceNow, JIRA, etc.), whether incident severity classification aligns with business impact, whether escalation paths are defined and tested, whether problem records are created for recurring incidents, and whether root-cause analysis is completed and documented for major incidents,
- review root-cause analysis workflow: whether SAP Cloud ALM Real User Monitoring or synthetic monitoring is configured for critical user journeys, whether alert-to-incident traceability is maintained (alerts link to incident records), whether cross-system dependency maps are available in Cloud ALM for root-cause investigation, and whether the handoff process between Cloud ALM and SAP support (or system-specific support teams) is documented,
- assess SLA and service continuity governance: whether SLAs are formally defined in SAP Cloud ALM, whether breach threshold alerting is configured per SLA, whether SLA performance reporting is scheduled and reviewed by service management, and whether a service continuity plan exists for critical SAP services covered by Cloud ALM.

## When not to use

- When the request requires connecting to a live SAP Cloud ALM tenant, querying real-time health monitoring data, or inspecting production incident records — this skill reviews configuration descriptions, monitoring design documentation, process documentation, and governance artifacts only.
- When the request is about monitoring and observability for custom-built BTP applications using non-SAP Cloud ALM tools (Dynatrace, Prometheus, Grafana) — this skill focuses on SAP Cloud ALM capabilities.
- When the request is about SAP Solution Manager (SolMan) operations in an on-premise context as the primary monitoring tool rather than Cloud ALM.
- When the request is about SAP Transport Management or change management governance rather than health monitoring and incident management.

## Does not touch live systems

This skill operates on user-provided Cloud ALM monitoring configuration descriptions, alert rule lists, business process monitoring key figure definitions, incident management process documentation, SLA definitions, and written descriptions of the operations governance posture. It does not connect to any SAP Cloud ALM tenant, query real-time health data, create or update incident records, modify alert configuration, or access any managed SAP system. All live inspection is out of scope.

## Lean operating rules

- Classify findings by SRE governance domain before recommending. Every finding must be assigned to a domain (Health Monitoring / Alerting / Integration and Exception Monitoring / Business Process Monitoring / Incident and Problem Management / Root-Cause Analysis / SLA and Service Continuity) before a remediation path is proposed.
- Monitoring coverage gaps for production systems are never acceptable. Any production SAP service or system not covered by SAP Cloud ALM health monitoring is a `high` finding. Blind spots in production monitoring mean service degradation can go undetected until business impact occurs.
- Alert thresholds must be calibrated to SLA requirements. An alert threshold that fires only after an SLA breach has already occurred provides no operational value. Alert thresholds must give operations teams sufficient time to respond before breach. Miscalibrated or absent thresholds are `high` findings.
- Incident severity must reflect business impact. An incident classification that does not account for business impact (number of affected users, revenue at risk, regulatory reporting deadline) is a governance gap. Incidents affecting critical business processes must be escalated to the highest appropriate severity.
- Problem management is required for recurring incidents. Three or more incidents with the same root cause within a rolling 30-day period without a linked problem record and active root-cause investigation is a `medium` finding.
- SLA definitions must exist before SLA reporting is meaningful. Generating SLA performance reports without formally defined SLAs (agreed availability targets, response time targets, resolution time targets) is a `medium` finding — the reports lack a verified baseline against which breach is measured.
- Integration and exception monitoring must cover business-critical interfaces. Any integration scenario on the critical business process path (order-to-cash, procure-to-pay, financial close, payroll) without Cloud ALM exception monitoring and alert coverage is a `high` finding.
- Evidence from user-provided Cloud ALM configuration documentation or official SAP Cloud ALM documentation takes precedence over inference.

## Evidence rules

Label all claims with one of:

- `documentation-based` — grounded in official SAP Cloud ALM documentation (help.sap.com)
- `user-provided evidence` — Cloud ALM monitoring configuration descriptions, alert rule lists, business process monitoring key figure definitions, incident management process documentation, SLA definitions, or written operations governance descriptions provided by the user
- `inference` — derived reasoning not directly confirmed by official docs or user evidence; must always be labeled as such

## Live-environment rules

**This skill does not touch live systems.** There is no SAP Cloud ALM API call, real-time health data query, incident record access, alert configuration modification, or managed system connection in this skill's execution path. Users must supply Cloud ALM configuration documentation, monitoring design descriptions, process documentation, or governance artifacts for this skill to review.

## References

Load only when needed:

- [Workflow and output contract](references/workflow-and-output.md) — SRE governance domain taxonomy, risk classification, output format.
- [Safety checklist](references/safety-checklist.md) — non-negotiables, common Cloud ALM operations review mistakes, when to push back.
- [Official sources](references/official-sources.md) — SAP Cloud ALM health monitoring, alerting, integration monitoring, business process monitoring, and incident management documentation.

## Response minimum

Return, at minimum:

- **Problem classification**: SRE governance domain(s) in scope (Health Monitoring / Alerting / Integration and Exception Monitoring / Business Process Monitoring / Incident and Problem Management / Root-Cause Analysis / SLA and Service Continuity) and specific finding(s) per domain.
- **Evidence used**: documentation-based / user-provided evidence / inference.
- **Risk level**: critical (production system with no monitoring coverage, complete SLA reporting absence for regulated service) / high (alert threshold miscalibration, critical integration interface with no exception monitoring, business-critical incident severity misconfiguration, missing SLA definition) / medium (problem record absence for recurring incidents, incomplete root-cause analysis, missing SLA breach alerting) / low (best practice deviation, documentation gap without immediate operational impact).
- **Recommended action**: specific governance control per finding (add managed system to Cloud ALM monitoring scope, recalibrate alert threshold, configure integration exception monitoring, define SLA in Cloud ALM, create problem record for recurring incident pattern, implement root-cause analysis workflow, etc.).
- **Refusal / escalation triggers**: if live Cloud ALM tenant access, real-time health data, or production incident record review is required to complete the assessment, state that clearly and do not proceed. If a production system monitoring blind spot is identified, escalate to the operations team immediately — do not defer.
- **Business impact**: service availability risk, SLA breach risk, business process delay risk (financial close, order-to-cash, payroll), regulatory reporting risk, or mean time to detect (MTTD) and mean time to resolve (MTTR) increase from monitoring gaps.
- **Next verification step**: validate monitoring coverage recommendations against the current SAP Cloud ALM managed system list and alert rule configuration before deploying changes to the production monitoring scope.
