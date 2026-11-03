---
name: sap-signavio-process-mining-value
description: "Review SAP Signavio process intelligence and process mining configurations for value discovery: process discovery coverage, conformance checking against reference models, bottleneck and rework analysis, value realization tracking, and linkage of process mining findings to S/4HANA process improvement opportunities. Flags analytical blind spots, conformance gaps, and unrealized value levers. Does not modify process models, mining configurations, or live SAP systems."
allowed-tools: Read Grep Glob WebSearch WebFetch
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-06-19"
  category: data
  lifecycle: experimental
---

# SAP Signavio Process Mining Value Review

## Purpose

Assess SAP Signavio Process Intelligence (process mining) configurations and analytical outputs for value discovery quality and realization effectiveness. Evaluate process discovery coverage — which end-to-end processes are mined (Procure-to-Pay, Order-to-Cash, Lead-to-Cash, Record-to-Report, Hire-to-Retire, Plan-to-Produce), what event log coverage exists, and whether the extracted process variants are representative of real execution. Review conformance checking setup against SAP reference process models and customer-defined target processes — assessing how deviations are classified, whether root causes are traced, and whether conformance scores drive actionable remediation. Analyze bottleneck and rework identification methodology — confirming that waiting times, rework loops, and touchpoint counts are measured at the right granularity and linked to business impact. Assess value realization tracking — whether improvement initiatives identified from process mining findings are tracked to completion and whether business outcomes (cycle time reduction, cost savings, error rate reduction) are measured. Evaluate the linkage between Signavio process mining findings and S/4HANA process improvement opportunities including clean core alignment and process standardization levers. Does not modify process models in Signavio Process Manager, mining configuration in Signavio Process Intelligence, or any live SAP system.

## When to use

Use this skill when the user asks to:

- review SAP Signavio Process Intelligence event log configuration: which SAP source systems are connected (S/4HANA, SAP ECC, Ariba, SuccessFactors, etc.), what event log extraction covers (case ID, activity, timestamp, attributes), and whether the event log is representative of the full end-to-end process,
- assess process discovery quality: number of process variants discovered, variant coverage threshold (percentage of cases covered by reviewed variants), whether low-frequency outlier variants are investigated or dismissed, and whether the discovered process map reflects real execution rather than an idealized flow,
- evaluate conformance checking setup against SAP reference process models (SAP Signavio Business Process Management Suite reference content) or customer-defined target process models: how conformance deviations are classified, whether severity levels are assigned to deviation types, and whether root cause investigation is systematically performed for high-frequency deviations,
- identify bottleneck analysis quality: whether waiting times between process steps are measured and attributed (system wait, resource wait, external dependency), whether the bottleneck root cause links to a specific system, role, or organizational unit, and whether Signavio simulation is used to model the impact of bottleneck removal,
- assess rework and exception loop identification: whether rework loops (repeated activities, reversal patterns, re-approval cycles) are defined and measured, whether rework rates are tracked over time, and whether high-rework activities are prioritized for S/4HANA process improvement,
- review value realization tracking: whether process mining findings are linked to improvement initiatives with owners, timelines, and measurable KPIs, and whether post-implementation measurement confirms that the improvement initiative delivered the expected cycle time, cost, quality, or compliance outcome,
- evaluate the linkage between Signavio process mining insights and SAP S/4HANA process improvement opportunities: whether mining findings inform clean core decisions, SAP Best Practice process adoption, process standardization scope, or automation candidates (SAP Build Process Automation, iRPA),
- assess SAP Signavio Process Intelligence dashboard and investigation design: KPI selection, benchmark comparison (SAP Business Process Intelligence benchmarking data), filter and drill-down coverage, and whether the dashboard enables root cause analysis without requiring custom SQL or data export.

## When not to use

- When the user needs live access to a Signavio Process Intelligence tenant, event log data, or mining output — this skill accepts only user-provided mining configuration descriptions, conformance check summaries, bottleneck analysis reports, dashboard screenshots, or written descriptions of the Signavio setup.
- When the request is about Signavio Process Manager (BPMN process modeling, process collaboration, process repository governance) rather than Process Intelligence (process mining, conformance checking, bottleneck analysis) — process modeling governance is a distinct scope.
- When the request is about SAP LeanIX or SAP Signavio Journey Modeler rather than Process Intelligence mining capabilities.
- When the request is about building a custom process mining implementation from raw SAP event data outside of Signavio — this skill focuses specifically on the SAP Signavio Process Intelligence platform.
- When the request is about SAP Datasphere or SAP Analytics Cloud KPI dashboards that are not derived from Signavio process mining output.

## Does not touch live systems

This skill operates on user-provided Signavio Process Intelligence configuration descriptions, event log specification summaries, conformance check reports, bottleneck analysis outputs, value tracking dashboards, dashboard screenshots, or written descriptions of the Signavio process mining setup. It does not connect to any Signavio Process Intelligence tenant, SAP S/4HANA system, Ariba, SuccessFactors, or any other source system. It does not modify process models, event log configurations, mining run parameters, conformance rules, or dashboard definitions. All live inspection is out of scope.

## Lean operating rules

- Classify process mining findings before recommending. Every finding must be assigned to an analytical domain (event log coverage / process discovery / conformance / bottleneck analysis / rework analysis / value realization / S/4HANA linkage) before a recommendation is proposed.
- Event log coverage determines analytical validity. If the event log does not capture the full end-to-end process (missing handoff events, truncated case journeys, incomplete attribute capture), all downstream mining findings are potentially misleading. Coverage gaps must be classified first.
- Conformance checking without a target model is not conformance checking. Conformance analysis requires an explicit reference process model. Mining that produces a process map without comparing it to a defined target state measures actual behavior, not deviation from intent.
- Bottleneck attribution must reach a root cause. Identifying that step X has a long average waiting time is a first-order observation, not an actionable finding. The review must assess whether the root cause (system response time, resource shortage, approval queue depth, external dependency) has been identified and whether it is addressable.
- Value realization requires measurement, not projection. A process mining engagement that produces a prioritized list of improvement opportunities but does not track post-implementation outcomes has delivered analysis, not value realization. The distinction matters for ROI justification.
- S/4HANA process improvement linkage must be specific. General statements that process mining "informs the S/4HANA roadmap" are insufficient. The link should identify which specific SAP Best Practice process variant, automation capability, or clean core decision is supported by which specific mining finding.
- Evidence from user-provided artifacts or official SAP Signavio documentation takes precedence over inference.
- Load only the reference needed for the analytical domain under review.

## Evidence rules

Label all claims with one of:

- `documentation-based` — grounded in SAP Signavio Process Intelligence documentation, SAP Help Portal Signavio content, or SAP process mining best practice guidance
- `user-provided evidence` — event log configuration descriptions, conformance check reports, bottleneck analysis outputs, value tracking summaries, dashboard screenshots, or written descriptions provided by the user
- `inference` — derived reasoning not directly confirmed by official docs or user evidence

## Live-environment rules

**This skill does not touch live systems.** There is no Signavio Process Intelligence API call, tenant connection, event log query, mining run trigger, or process model modification in this skill's execution path. Users must supply mining configuration descriptions, conformance check summaries, bottleneck analysis reports, or written descriptions of their Signavio process mining setup for this skill to review.

## References

Load only when needed:

- [Workflow and output contract](references/workflow-and-output.md) — process mining finding taxonomy, severity assignment, output format.
- [Safety checklist](references/safety-checklist.md) — non-negotiables, common process mining review mistakes, when to push back.
- [Official sources](references/official-sources.md) — SAP Signavio Process Intelligence, conformance checking, value tracking, and S/4HANA process improvement documentation.

## Response minimum

Return, at minimum:

- **Problem classification**: analytical domain(s) affected (event log coverage / process discovery / conformance / bottleneck analysis / rework analysis / value realization / S/4HANA linkage) and specific finding(s) per domain.
- **Evidence used**: documentation-based / user-provided evidence / inference.
- **Risk level**: critical (event log coverage gap that invalidates all downstream findings; unmeasured high-value process entirely outside mining scope) / high (conformance checking without a target model; bottleneck attribution stops at symptom; rework loops unmeasured in a high-cost process) / medium (value realization not tracked post-implementation; S/4HANA linkage too generic; dashboard missing key drill-down) / low (best practice deviation in mining configuration or dashboard design).
- **Recommended action**: specific analytical or configuration improvement per finding (event log extension, target model definition, conformance severity classification, bottleneck root cause drill-down, rework loop definition, value KPI addition, specific S/4HANA improvement linkage, etc.).
- **Refusal / escalation triggers**: if a finding requires live Signavio tenant access, active mining run output, or real event log data to evaluate, state that clearly and ask the user to supply the relevant export or summary.
- **Business impact**: unrealized process efficiency gains, compliance risk from undetected conformance deviations, continued cost of rework and exception handling, or ROI justification gap for the process mining program.
- **Next verification step**: confirm recommended analytical changes (event log extension, target model definition, KPI addition) in the Signavio Process Intelligence tenant before treating mining outputs as a reliable basis for S/4HANA improvement decisions.
