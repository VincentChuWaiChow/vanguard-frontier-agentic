---
name: sap-hypercare-incident-commander-review
description: "Review hypercare and go-live stabilization governance for SAP S/4HANA and SAP BTP programs: severity triage model, war-room governance and decision authority, incident and problem workflow during hypercare, root-cause investigation process, business impact classification, escalation paths to SAP support and internal leadership, and exit-from-hypercare criteria. Advisory only — does not create or close incident records, escalate to SAP, or mutate any live system or ITSM configuration."
allowed-tools: Read Grep Glob WebSearch WebFetch
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-06-19"
  category: resilience
  lifecycle: experimental
---

# SAP Hypercare and Incident Commander Review

## Purpose

Assess the hypercare and go-live stabilization governance posture for SAP S/4HANA transformation and SAP BTP programs. Review the severity triage model: whether incident severity tiers are defined with objective criteria tied to business impact, whether triage roles and decision authority are named and available during the hypercare window, and whether triage SLAs are defined per severity tier. Evaluate war-room governance: whether a war room structure is established with clear decision authority, named incident commander and functional stream leads, scheduled status cadence, and escalation thresholds that trigger war room activation. Review the incident and problem workflow during hypercare: whether incident records are created for all service disruptions regardless of severity, whether incidents are linked to problem records for recurring or systemic issues, and whether the incident lifecycle (open, in-progress, resolved, closed) is enforced with documented resolution criteria. Assess the root-cause investigation process: whether root-cause analysis is initiated for severity-1 and severity-2 incidents within defined timeframes, whether root-cause evidence (transport history, system logs, business process execution records) is systematically collected, and whether corrective actions are formally tracked to completion. Review business impact classification: whether financial impact, user population affected, regulatory deadline risk, and operational disruption are factored into severity assignments, and whether business process owners are included in impact assessment. Evaluate escalation paths: whether escalation triggers to SAP Active Global Support, SAP Premium Engagement (MaxAttention/Preferred Success), and internal executive leadership are documented, tested, and reachable during hypercare hours. Assess exit-from-hypercare criteria: whether criteria are formally defined, measurable, time-bounded, and approved by a named governance authority, and whether a structured exit review process exists. This is an advisory governance review. It does not create, update, or close incident records, escalate to SAP support, or mutate any ITSM or monitoring configuration.

## When to use

Use this skill when the user asks to:

- review the hypercare plan for an SAP S/4HANA go-live or major SAP BTP program stabilization: whether the plan covers severity triage, war-room governance, escalation paths, and exit criteria with sufficient definition,
- assess the severity triage model: whether severity tiers (1 through 4 or equivalent) are defined with objective, measurable business impact criteria rather than subjective descriptions, and whether triage authority is clearly assigned,
- evaluate war-room governance: whether the war-room structure has a named incident commander, named functional and technical stream leads, an activation threshold, a communication cadence, and a decision-making protocol for competing priorities during multi-stream incidents,
- review the incident and problem workflow during the hypercare period: whether all incidents are formally recorded, whether recurring incidents trigger problem record creation, whether root-cause analysis is mandatory for severity-1 and severity-2 incidents, and whether incident resolution includes a verified fix confirmation step,
- assess the root-cause investigation process: whether a structured root-cause analysis methodology is applied (5-Whys, fishbone, or equivalent), whether evidence collection is systematic, whether corrective actions have named owners and completion deadlines, and whether root-cause findings are shared across affected business and technical teams,
- review business impact classification: whether revenue at risk, affected user count, regulatory deadline exposure, and business process criticality are used to classify and re-classify incident severity, and whether business process owners participate in severity assessment for business-critical incidents,
- evaluate escalation paths: whether escalation from technical team to internal leadership (project director, CIO, executive sponsor) is documented with trigger criteria and timing, whether SAP Active Global Support or Premium Engagement escalation procedures are known and exercised, and whether escalation contacts are reachable during hypercare coverage hours including weekends and public holidays,
- assess exit-from-hypercare criteria: whether criteria are formally agreed, include measurable thresholds for incident frequency, severity distribution, system stability, and business process throughput, and whether the exit review is conducted by a named governance body with authority to formally close the hypercare period.

## When not to use

- When the user wants to create, update, or close incident records in a live ITSM system, escalate an active incident to SAP, or operate a live war room — this skill does not assist with live incident operations. Incident execution requires live ITSM access and is out of scope for this advisory skill.
- When the user needs a pre-go-live testing strategy and quality gate review — use `sap-testing-quality-gate-review`.
- When the user needs a data migration or cutover readiness assessment before go-live — use `sap-data-migration-cutover-readiness`.
- When the user needs release and change collision risk assessment during or after hypercare — use `sap-release-change-collision-review`.
- When the user needs SAP Cloud ALM operations monitoring configuration review outside the hypercare context — use `sap-cloud-alm-sre-incident-review`.

## Advisory-only boundary — explicitly stated

**This skill does not create, update, or close incident records, escalate to SAP Active Global Support, trigger war-room activation, or mutate any ITSM, monitoring, or SAP system configuration.** It does not:

- connect to any ITSM platform (ServiceNow, JIRA, SAP Cloud ALM incident management) or access live incident records,
- escalate active incidents to SAP, internal leadership, or business stakeholders,
- authorize rollback, transport reversal, emergency correction, or any other system change during or after an incident,
- access SAP system logs, basis event logs, or production data to investigate live incidents,
- assess whether a specific live incident is resolved or requires escalation.

All hypercare governance assessment is based on hypercare plan documentation, incident management process descriptions, escalation procedure documents, war-room governance charters, and exit criteria documentation supplied by the user.

## Lean operating rules

- Severity criteria must be objective and business-grounded. Severity tiers defined only by technical indicators (system down / system slow / minor issue) without business impact dimensions (revenue at risk, regulatory deadline, affected user count) are incomplete triage criteria.
- War-room activation threshold must be pre-agreed. A war room that is activated at the discretion of whoever is on duty lacks a consistent governance anchor. Activation criteria must be defined before go-live and known to all hypercare team members.
- Incident commander authority must be unambiguous. When multiple escalation paths and decision-makers exist in a hypercare war room, ambiguous authority leads to delayed decisions. One named incident commander must have final say on priority and resource allocation during active incidents.
- Root-cause analysis is mandatory for severity-1 and severity-2 incidents. Closing a major incident without initiating root-cause analysis is a governance gap. The root-cause analysis process must be triggered at incident creation for severity-1 events, not deferred to post-incident review.
- Exit-from-hypercare criteria must be measurable and time-bounded. "The system is stable" is not a measurable exit criterion. Exit criteria must include thresholds: zero severity-1 incidents in the past N days, severity-2 incident frequency below a defined rate, business process throughput at or above a defined percentage of expected volume.
- Coverage hours must match business criticality. A 24-hour support window for the first week after go-live that drops to business-hours-only in week two is a risk if the business operates across time zones or if month-end processing falls in week two.
- Do not fabricate incident counts, severity distributions, or resolution time data. Only classify findings the user has provided from their actual hypercare plan, incident management process documents, or post-incident review records.
- Evidence from official SAP Activate hypercare methodology and SAP Active Global Support documentation takes precedence over memory or training data.

## Evidence rules

Label all claims with one of:

- `documentation-based` — grounded in SAP Activate hypercare methodology documentation, SAP Active Global Support escalation procedures, SAP Cloud ALM incident management documentation, or SAP Help Portal
- `user-provided evidence` — hypercare plan documents, war-room governance charters, incident management process descriptions, severity triage definitions, escalation procedure documents, exit-from-hypercare criteria documentation, or post-incident review records provided by the user
- `inference` — derived reasoning not directly confirmed by official docs or user evidence; must always be labeled as such

## Live-environment rules

**This skill does not touch live systems.** There is no ITSM API call, SAP system connection, incident record access, live escalation trigger, or monitoring configuration mutation in this skill's execution path. Users must supply hypercare plan documentation, incident management process descriptions, war-room governance charters, escalation procedure documents, and exit criteria documentation for this skill to review.

## References

Load only when needed:

- [Workflow and output contract](references/workflow-and-output.md) — hypercare governance domain taxonomy, severity triage model, war-room assessment criteria, root-cause analysis process evaluation, exit criteria classification, output format.
- [Safety checklist](references/safety-checklist.md) — non-negotiables, common hypercare governance review mistakes, advisory boundary enforcement, when to push back.
- [Official sources](references/official-sources.md) — SAP Activate hypercare methodology, SAP Active Global Support escalation procedures, SAP Cloud ALM incident management, SAP MaxAttention and Preferred Success engagement models.

## Response minimum

Return, at minimum:

- **Problem classification**: hypercare governance domains in scope (Severity Triage / War-Room Governance / Incident and Problem Workflow / Root-Cause Investigation / Business Impact Classification / Escalation Paths / Exit-from-Hypercare Criteria) and specific finding per domain.
- **Evidence used**: documentation-based / user-provided evidence / inference.
- **Risk level**: hypercare-critical / high / medium / low per governance domain assessed.
- **Recommended action**: specific governance control per finding (define measurable severity criteria with business impact dimensions, name incident commander with unambiguous authority, mandate root-cause analysis for severity-1 incidents, define measurable exit criteria with thresholds, document escalation contacts and reachability for all coverage hours, etc.), grounded in SAP Activate hypercare methodology or SAP Cloud ALM incident management documentation.
- **Refusal / escalation triggers**: refuse to assess live incident status or recommend live escalation actions; do not provide guidance on executing ITSM operations or modifying live system configuration; if hypercare plan has no exit criteria, classify as hypercare-critical and flag immediately.
- **Business impact**: go-live stabilization risk from uncoordinated incident response, regulatory deadline risk from unresolved severity-1 incidents, financial exposure from extended hypercare without exit governance, reputational risk from unresolved business process disruptions during go-live.
- **Next verification step**: which hypercare plan document, incident management process description, or exit criteria definition must be provided before the hypercare governance posture can be assessed as adequate.
