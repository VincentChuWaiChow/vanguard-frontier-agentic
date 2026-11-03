---
name: sap-mdg-master-data-quality-review
description: "Review SAP Master Data Governance (MDG) configuration and data quality posture: data models and entity types, validation and derivation rules, governance workflow design, consolidation and mass processing configuration, data quality KPI dashboards, and key mapping setup. Flags governance gaps, mis-configured validation rules, workflow bottlenecks, and data quality KPI blind spots. Does not create, modify, or approve master data records."
allowed-tools: Read Grep Glob WebSearch WebFetch
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-06-19"
  category: data
  lifecycle: experimental
---

# SAP MDG Master Data Quality Review

## Purpose

Assess the configuration quality and governance posture of SAP Master Data Governance (MDG) deployments. Evaluate the design of MDG data models and entity types for completeness, extensibility soundness, and alignment with business data standards. Review validation rule configuration (BRFplus-based and custom) for coverage of mandatory field checks, referential integrity, duplicate detection sensitivity, and format standardization. Assess derivation rule design for correctness, dependency order, and risk of silent data overwrite. Evaluate governance workflow design — change request types, workflow routing, parallel versus sequential step configuration, and escalation paths — for process efficiency and control adequacy. Analyze consolidation and mass processing configuration for matching algorithm quality and error handling. Review data quality KPI framework setup for completeness, metric relevance, and dashboard coverage. Assess key mapping configuration for source-to-MDG-to-target system alignment and replication monitoring. Does not create, modify, approve, or reject any master data record in any live system.

## When to use

Use this skill when the user asks to:

- review SAP MDG data model design: custom entity types, node structures (single-object, multi-object, reuse object), key fields, and extensibility approach against the standard MDG data model framework,
- assess MDG validation rules configured via BRFplus or custom BAdI implementations: rule completeness for mandatory fields, cross-field referential checks, format validations, and duplicate detection rule sensitivity (matching algorithm, threshold settings, match profile),
- evaluate MDG derivation rule design: derivation sequence, dependency order correctness, risk of a derivation silently overwriting a user-entered value without notification, and coverage of required auto-population fields,
- review MDG governance workflow configuration: change request type definitions, workflow template design (single-step, multi-step, parallel approval), routing logic based on entity attributes, step ownership assignment, deadline and escalation configuration,
- assess MDG consolidation and mass processing setup: consolidation data model alignment, matching rule quality, best record calculation logic, error handling for consolidation failures, and mass change template authorization controls,
- evaluate MDG data quality KPI framework: KPI definition completeness, metric coverage (completeness, conformance, uniqueness, timeliness), rule-to-KPI mapping, data quality score threshold settings, and whether dashboard coverage includes all in-scope MDG domains (business partner, material, supplier, G/L account, cost center, etc.),
- review key mapping and value mapping configuration: source system–to–MDG key mapping completeness, replication model design (MDG replication framework vs. ALE/IDOC vs. SOA), target system mapping correctness, and replication error monitoring,
- assess MDG edition and scope baseline: which MDG domains are active (Finance, Supplier, Customer, Material, Custom), whether the MDG S/4HANA integration edition or standalone MDG-M is in use, and whether the governance scope matches the data domains that carry business risk.

## When not to use

- When the user needs live inspection of an MDG system, active change requests, or data quality scores — this skill accepts only user-provided configuration descriptions, BRFplus rule exports, workflow configuration summaries, KPI dashboard screenshots, or written descriptions of the MDG landscape.
- When the request is about SAP S/4HANA Finance G/L account or cost center data content rather than the MDG governance process that manages it — use `sap-finance-fico-controls-review` for financial accounting controls.
- When the request is about SAP Data Intelligence or SAP Datasphere data integration pipelines rather than MDG governance processes — those are distinct data management platforms.
- When the request is about ERP data migration quality (data migration workbench, LTMC, BODS migration projects) rather than steady-state MDG governance — data migration review is a distinct scope.

## Does not touch live systems

This skill operates on user-provided configuration descriptions, BRFplus rule summaries or exports, workflow template configuration notes, MDG data model design documents, KPI framework screenshots or descriptions, key mapping configuration exports, or written descriptions of the MDG landscape. It does not connect to any SAP MDG system, S/4HANA backend, MDG web UI, Fiori launchpad, or replication target system. It does not create, modify, approve, reject, or activate any master data change request, consolidation object, or governance workflow step. All live inspection is out of scope.

## Lean operating rules

- Classify MDG findings before recommending. Every finding must be assigned to an MDG governance domain (data model / validations / derivations / workflow / consolidation / data quality KPIs / key mapping) before a remediation path is proposed.
- Duplicate detection rule sensitivity must be assessed in context. An overly strict matching threshold produces false positives that bottleneck the consolidation process. An overly loose threshold allows duplicates to persist. Both extremes are findings; the appropriate threshold depends on the data domain and organization's deduplication tolerance.
- Derivation silent overwrites are a control risk. A derivation rule that overwrites a user-entered field without an explicit notification to the requestor is a data governance risk — users cannot detect when system-derived values replace their input.
- Governance workflow must have escalation paths. A workflow with no deadline enforcement or escalation routing for overdue steps is a governance bottleneck risk — change requests can stall indefinitely without resolution.
- Data quality KPIs must cover all in-scope domains. A KPI framework that measures completeness for material master but not for business partner or supplier exposes blind spots in the MDG governance posture.
- Key mapping gaps create replication failures. Missing or incorrect key mapping between MDG and a target system (ERP, CRM, warehouse management) causes replication failures that may persist undetected if error monitoring is not configured.
- Mass processing requires authorization controls. MDG mass change templates allow bulk modifications to master data records. Unrestricted access to mass change template execution is a `high` data governance risk.
- Evidence from user-provided artifacts or official SAP MDG documentation takes precedence over inference.
- Load only the reference needed for the MDG domain under review.

## Evidence rules

Label all claims with one of:

- `documentation-based` — grounded in SAP Master Data Governance Help Portal documentation, BRFplus documentation, MDG data model guides, or SAP MDG best practice guidance
- `user-provided evidence` — BRFplus rule exports, workflow configuration summaries, KPI dashboard descriptions, key mapping configuration notes, data model design documents, or written descriptions provided by the user
- `inference` — derived reasoning not directly confirmed by official docs or user evidence

## Live-environment rules

**This skill does not touch live systems.** There is no MDG API call, Fiori OData request, S/4HANA RFC invocation, BRFplus runtime execution, or replication trigger in this skill's execution path. Users must supply BRFplus rule summaries, workflow configuration descriptions, data model documents, KPI dashboard exports, or written descriptions of their MDG landscape for this skill to review.

## References

Load only when needed:

- [Workflow and output contract](references/workflow-and-output.md) — MDG finding taxonomy, severity assignment, output format.
- [Safety checklist](references/safety-checklist.md) — non-negotiables, common MDG review mistakes, when to push back.
- [Official sources](references/official-sources.md) — SAP MDG data model, validation/derivation, governance workflow, consolidation, data quality KPI, and key mapping documentation.

## Response minimum

Return, at minimum:

- **Problem classification**: MDG governance domain(s) affected (data model / validations / derivations / workflow / consolidation / data quality KPIs / key mapping) and specific finding(s) per domain.
- **Evidence used**: documentation-based / user-provided evidence / inference.
- **Risk level**: critical (governance bypass, undetected duplicate propagation, unrestricted mass change execution) / high (derivation silent overwrite, missing escalation path, key mapping gap causing replication failures, KPI blind spot for a high-risk domain) / medium (workflow bottleneck, duplicate threshold misconfiguration, incomplete KPI rule mapping) / low (best practice deviation in data model design).
- **Recommended action**: specific configuration or process remediation per finding (BRFplus rule addition, derivation notification step, workflow escalation configuration, matching threshold adjustment, KPI rule extension, key mapping completion, mass change authorization tightening, etc.).
- **Refusal / escalation triggers**: if a finding requires live MDG system access to evaluate (active change request status, live data quality scores, replication queue state), state clearly that live inspection is out of scope and ask the user to supply the relevant export or description.
- **Business impact**: data quality degradation risk, regulatory compliance gap (data accuracy for financial reporting, supplier risk management), operational process disruption from replication failures, or audit finding risk.
- **Next verification step**: confirm recommended configuration changes against the current MDG setup in a development or quality system before promoting to production.
