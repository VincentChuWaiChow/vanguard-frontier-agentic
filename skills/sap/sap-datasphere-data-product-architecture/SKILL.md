---
name: sap-datasphere-data-product-architecture
description: "Review SAP Datasphere architecture: spaces, data flows and replication, semantic and analytic models, data products and sharing, data access controls, and integration with SAP Analytics Cloud and SAP HANA Cloud. Flags structural design gaps, access control weaknesses, and data product sharing anti-patterns. Does not touch live systems."
allowed-tools: Read Grep Glob WebSearch WebFetch
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-06-19"
  category: data
  lifecycle: experimental
---

# SAP Datasphere Data Product Architecture Review

## Purpose

Assess the architecture and governance posture of an SAP Datasphere implementation. Review space design for isolation and purpose separation, data flow and replication flow design for completeness and error handling, semantic layer modeling (views, analytical datasets, dimensions, facts) for correctness and reuse, data product definition and cross-space or external sharing configuration, data access control (DAC) and row-level security design, and integration with SAP Analytics Cloud (SAC) and SAP HANA Cloud. Surface structural design gaps, over-permissive sharing configurations, missing access controls, and modeling anti-patterns. Does not connect to or mutate any live Datasphere tenant.

## When to use

Use this skill when the user asks to:

- review the Datasphere space structure for isolation, purpose separation, naming conventions, and governance boundary alignment,
- assess data flow and replication flow design for error handling coverage, delta-load correctness, scheduling, and target table configuration,
- evaluate semantic models (graphical views, SQL views, analytical datasets, dimensions, fact views, hierarchies) for correctness, reuse, and star schema completeness,
- review data product definitions, output ports, and cross-space or external sharing configurations for governance and access appropriateness,
- audit data access control (DAC) rules and row-level security configurations for correctness and completeness,
- assess integration patterns between Datasphere and SAP Analytics Cloud: live connections vs. import connections, model type compatibility, and refresh scheduling,
- review Datasphere integration with SAP HANA Cloud for virtual access, remote table replication, and federation correctness,
- flag data architecture anti-patterns such as redundant replicated tables, missing semantic annotations, overly broad space access, or unmanaged data product output ports.

## When not to use

- When the user needs live inspection of a running Datasphere tenant, space, or data flow execution log — this skill accepts only user-provided descriptions, space exports, model exports, or architecture documents.
- When the request is about SAP Analytics Cloud stories, planning models, or SAC-specific governance — use `sap-analytics-cloud-planning-governance`.
- When the request is specifically about HANA Cloud performance tuning, cost management, or sizing — use `sap-hana-cloud-performance-cost`.
- When the request concerns BTP-level governance (entitlements, subaccounts, role collections) — use `sap-btp-governance-review`.

## Does not touch live systems

This skill operates on user-provided descriptions, architecture diagrams, space configuration exports, data flow screenshots, semantic model exports, data product definitions, or written descriptions of the Datasphere implementation. It does not connect to any Datasphere tenant, invoke Datasphere APIs, access space files, read data flow run logs, or interact with any connected source system. All live inspection is out of scope.

## Lean operating rules

- Classify findings before recommending. Every finding must be assigned to a design domain (space design, data flows, semantic models, data products, data access controls, SAC integration, HANA Cloud integration) before remediation is proposed.
- Spaces are the primary isolation boundary. Each Datasphere space has its own connections, storage quota, users, and data sharing scope. Cross-space data access must go through explicitly defined data product output ports — direct cross-space table access is an architecture violation.
- Semantic layer completeness is required for trusted analytics. Views exposed to SAC must have correct semantic usage set (dimension, fact, analytical dataset, or cube). Views without a semantic usage annotation cannot be used reliably in SAC Live Connection models.
- Row-level data access control (DAC) is not optional for multi-tenant or shared data. Any Datasphere space that contains data from multiple business units or customers without DAC enforcement is a data isolation risk.
- Data products require defined output ports. A data product definition without a configured output port (table sharing, data sharing service, or Open SQL schema) is incomplete and cannot be consumed by other spaces or external consumers.
- Replication flows and data flows handle error differently. Replication flows are designed for bulk and delta replication with automatic restart; data flows are designed for transformation pipelines with explicit scheduling. Confusing the two leads to incorrect delta-load behavior and missing transformation error handling.
- SAC live connections require stable semantic models. Changing the key structure or removing measures from a Datasphere view consumed via SAC live connection breaks the SAC story without warning. Live connection model dependencies must be tracked before structural model changes.
- Data access controls must be tested against real user assignments. DAC configurations that look correct in isolation may fail silently if the DAC combination method (AND/OR) is misconfigured for multi-rule scenarios.
- Evidence from user-provided artifacts or official SAP Datasphere documentation takes precedence over inference.
- Load only the reference needed for the component under review.

## Evidence rules

Label all claims with one of:

- `documentation-based` — grounded in SAP Datasphere Help Portal documentation, SAP Help Portal data product guides, or official SAP Datasphere modeling docs
- `user-provided evidence` — space configuration exports, data flow screenshots, semantic model exports, data product definitions, or architecture documents provided by the user
- `inference` — derived reasoning not directly confirmed by official docs or user evidence

## Live-environment rules

**This skill does not touch live systems.** There is no Datasphere API call, tenant login, space file access, data flow execution, or connection to any source or target system in this skill's execution path. Users must supply space configuration exports, model exports, data product definitions, architecture documents, or written descriptions of their Datasphere implementation for this skill to review.

## References

Load only when needed:

- [Workflow and output contract](references/workflow-and-output.md) — design domain taxonomy, finding severity, output format.
- [Safety checklist](references/safety-checklist.md) — non-negotiables, common mistakes, when to push back.
- [Official sources](references/official-sources.md) — SAP Datasphere space design, data flows, semantic modeling, data products, data access controls, SAC and HANA Cloud integration docs.

## Response minimum

Return, at minimum:

- **Problem classification**: design domain(s) affected (space design / data flows / semantic models / data products / data access controls / SAC integration / HANA Cloud integration) and specific finding(s).
- **Evidence used**: documentation-based / user-provided evidence / inference.
- **Risk level**: critical (data isolation breach or access control failure) / high (data architecture failure or pipeline reliability risk) / medium (governance gap or modeling quality gap) / low (best practice deviation).
- **Recommended action**: specific remediation per finding (space restructure, DAC rule addition, output port configuration, semantic usage annotation, model redesign, or connection type change).
- **Refusal / escalation triggers**: if live Datasphere tenant access, space file inspection, or data flow execution log access is required to complete the review, state that clearly and do not proceed.
- **Business impact**: data isolation risk, analytics quality risk, pipeline reliability risk, cost overrun, or compliance gap.
- **Next verification step**: validate recommended changes against the current Datasphere tenant configuration before applying.
