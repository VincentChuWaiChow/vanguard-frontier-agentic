---
name: sap-license-btp-consumption-finops-review
description: "Advisory FinOps review of SAP licensing and BTP consumption under CPEA and other commercial models: entitlement vs actual consumption, overage and underutilization patterns, cost allocation, commitment optimization, FUE and digital access licensing, and BTP service cost drivers. Does not touch or mutate any live system."
allowed-tools: Read Grep Glob WebSearch WebFetch
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-06-19"
  category: finops
  lifecycle: experimental
---

# SAP License and BTP Consumption FinOps Review

## Purpose

Assess the cost and licensing posture of an SAP landscape using FinOps principles. Evaluate SAP software license entitlements versus actual consumption, identify overage charges and underutilized license blocks, review BTP consumption under the Cloud Platform Enterprise Agreement (CPEA) or other commercial models, analyze cost allocation across teams and use cases, surface commitment optimization opportunities, assess Full Use Equivalent (FUE) and digital access licensing exposure, and identify the primary BTP service cost drivers. Surface licensing gaps, consumption inefficiencies, incorrect license type assignments, and commercial model mismatches that drive unnecessary spend. Does not connect to or mutate any live system, SAP for Me portal, or SAP licensing management platform.

## When to use

Use this skill when the user asks to:

- review SAP software license entitlements versus actual system usage to identify overage or underutilization,
- assess BTP consumption under CPEA, Pay-As-You-Go, or subscription commercial models for cost optimization,
- identify which BTP services are the primary cost drivers and evaluate whether consumption is proportionate to business value delivered,
- review FUE licensing and digital access user type assignments for correct classification and cost efficiency,
- assess cost allocation practices for BTP and SAP license spend across business units, cost centers, or projects,
- identify SAP license commitment optimization opportunities (right-sizing, license type conversion, consolidation),
- review the commercial model fit between the customer's consumption pattern and their current SAP commercial agreement,
- prepare for SAP license audit readiness or True-Up negotiations,
- evaluate indirect access or digital access risk from third-party integrations reading or writing SAP data,
- support an annual BTP or SAP license budget review or cost challenge.

## When not to use

- When the request requires live access to SAP for Me, the SAP License Administration Workbench (LAW), BTP cockpit usage data, or any live measurement tool — this skill accepts only user-provided consumption reports, entitlement summaries, or written descriptions.
- When the request concerns RISE with SAP contract SLA and vendor risk — use `sap-rise-sla-vendor-risk-review`.
- When the request concerns BTP account model governance (subaccount structure, entitlement sprawl, role collections) — use `sap-btp-governance-review`.
- When the request concerns the technical transformation program structure — use `sap-transformation-portfolio-triage-review`.
- When the request requires legal advice on SAP contract terms — this skill provides advisory cost classification, not legal counsel.

## Does not touch live systems

This skill operates on user-provided license entitlement reports, SAP for Me exports, BTP consumption reports, commercial model summaries, cost allocation data, or written descriptions of the SAP licensing and BTP consumption posture. It does not connect to SAP for Me, the License Administration Workbench, the BTP cockpit, the SAP Global License Audit and Compliance team portal, or any live environment. All live inspection is out of scope.

## Lean operating rules

- Classify spend before optimizing. Every license or consumption finding must be classified by license or cost category before an optimization recommendation is made.
- Distinguish entitlement from consumption. An entitlement is the right to use a capability. Consumption is the actual measured use. Optimization findings live in the gap between the two — either overage (consumption exceeds entitlement) or underutilization (entitlement exceeds consumption).
- BTP commercial models have different optimization levers. CPEA is a prepaid credit model — unspent credits expire. Pay-As-You-Go has no commitment but higher unit rates. Subscription is fixed capacity. The correct optimization depends on the model in use.
- FUE is the primary SAP on-premise and RISE license metric. Named User licenses in SAP are counted as FUEs. The FUE value of each named user type (Professional, Limited, Self-Service) differs. Incorrect type assignments are the most common source of license audit exposure.
- Digital access licenses cover machine-to-machine document creation. Third-party systems that create SAP documents (orders, invoices, goods receipts) via integration may require digital access licenses. This is a common source of indirect access audit findings.
- Cost allocation requires metered consumption data. Cost allocation recommendations require BTP service consumption data at subaccount or project level. Without this data, cost allocation assessments are advisory only.
- Commitment optimization follows a forecast-to-actuals comparison. CPEA credit optimization requires comparing forecasted consumption against actual burn rate and identifying services where consumption is consistently under or over the forecast.
- Do not recommend license type downgrades without confirming the functional scope the user actually uses. A Limited User license that is performing Professional User functions is a compliance exposure, not a cost optimization opportunity.
- Evidence from user-provided consumption data or official SAP licensing and BTP commercial model documentation takes precedence over inference.
- Load only the reference needed for the license or cost domain under review.

## Evidence rules

Label all claims with one of:

- `documentation-based` — grounded in SAP BTP commercial model documentation, SAP licensing guidelines, SAP product-specific supplement documents, or SAP price list definitions
- `user-provided evidence` — license entitlement reports, BTP consumption exports, SAP for Me data, cost allocation summaries, or commercial model descriptions provided by the user
- `inference` — derived reasoning not directly confirmed by official SAP documentation or user-provided evidence

## Live-environment rules

**This skill does not touch live systems.** There is no API call, SAP for Me portal access, License Administration Workbench connection, BTP cockpit session, or any live measurement tool access in this skill's execution path. Users must supply license entitlement reports, BTP consumption data exports, commercial model summaries, or written descriptions of their SAP licensing and BTP consumption posture for this skill to review.

## References

Load only when needed:

- [Workflow and output contract](references/workflow-and-output.md) — license and consumption category taxonomy, FinOps finding severity, optimization path decision criteria, output format.
- [Safety checklist](references/safety-checklist.md) — non-negotiables, common licensing misclassification mistakes, when to push back.
- [Official sources](references/official-sources.md) — SAP BTP commercial model documentation, CPEA pricing, FUE and digital access licensing guidelines, SAP pricing and licensing resources.

## Response minimum

Return, at minimum:

- **Problem classification**: license or cost domain(s) affected (FUE and named user licensing / digital access / BTP CPEA consumption / BTP service cost drivers / cost allocation / commercial model fit / True-Up and audit readiness) and specific finding(s).
- **Evidence used**: documentation-based / user-provided evidence / inference.
- **Risk level**: critical (active overage creating immediate audit or compliance exposure) / high (material underutilization of paid commitment or incorrectly classified license type with audit risk) / medium (cost allocation gap or commercial model mismatch with optimization opportunity) / low (best practice deviation in license management process).
- **Recommended action**: specific FinOps optimization per finding (right-size license type, convert commercial model, reallocate CPEA credits, enable cost allocation tagging, reclassify digital access users, prepare True-Up evidence).
- **Refusal / escalation triggers**: if live SAP for Me data, LAW measurement output, or BTP cockpit consumption data is required to complete the assessment, state that clearly and request the user provide the relevant exports. If legal interpretation of license terms is required, escalate to SAP licensing counsel.
- **Business impact**: overage cost exposure, audit penalty risk, budget overrun risk, or opportunity cost from unused committed spend.
- **Next verification step**: validate license consumption findings against current SAP for Me entitlement data and BTP cockpit consumption reports before initiating True-Up or renegotiation.
