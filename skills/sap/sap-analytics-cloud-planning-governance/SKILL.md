---
name: sap-analytics-cloud-planning-governance
description: "Review SAP Analytics Cloud governance: stories, planning models, versions, data actions, allocations, value driver trees, live vs. import connections, data access control, and performance. Flags planning model design gaps, version management risks, data action correctness issues, and access control weaknesses. Does not touch live systems."
allowed-tools: Read Grep Glob WebSearch WebFetch
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-06-19"
  category: data
  lifecycle: experimental
---

# SAP Analytics Cloud Planning Governance Review

## Purpose

Assess the governance, design, and operational posture of an SAP Analytics Cloud (SAC) implementation covering both analytics and planning workloads. Review story and model design for correctness, planning model structure (versions, categories, dimensions, and measures), data action and multi-action design for correctness and sequencing, allocation step configuration, value driver tree design, live connection vs. import connection selection and refresh governance, data access control (role-based and story-level), and performance posture of models and stories. Surface planning model design gaps, version management risks, data action correctness and sequencing issues, connection type misuse, and access control weaknesses. Does not connect to or mutate any live SAC tenant.

## When to use

Use this skill when the user asks to:

- review SAC story design for layout correctness, widget-to-model binding, filter logic, and performance (page count, number of queries, widget density),
- assess planning model design: version category structure (actual, budget, forecast, rolling forecast), dimension design, account dimension configuration, date dimension setup, and measure definition,
- evaluate data action design for step sequencing, script logic correctness (Advanced Formula), trigger and schedule configuration, and error handling,
- review allocation step design: allocation method selection (spread, distribution, breakback), driver dimension choice, and allocation result validation approach,
- assess value driver tree (VDT) design for formula correctness, node connectivity, and planning model binding,
- audit multi-action configuration for step ordering, action dependencies, and failure handling between actions,
- review live connection vs. import connection decisions for each SAC model: connection type appropriateness, import schedule cadence, incremental load vs. full refresh trade-offs,
- evaluate data access control: role assignment (BI Admin, Planner, Viewer, custom roles), team-level data access, dimension member-level access restrictions in planning models,
- flag planning anti-patterns: missing version locking after actuals close, overly complex data action scripts, unbounded allocation hierarchies, or stories consuming too many import models without caching.

## When not to use

- When the user needs live inspection of a running SAC tenant, model refresh status, or data action execution log — this skill accepts only user-provided descriptions, model exports, story screenshots, or architecture documents.
- When the request is about the underlying Datasphere semantic model that feeds a SAC live connection — use `sap-datasphere-data-product-architecture`.
- When the request concerns BTP-level governance (entitlements, subaccounts, SAC service instance provisioning) — use `sap-btp-governance-review`.
- When the request is specifically about HANA Cloud performance for models connected via live HANA connection — use `sap-hana-cloud-performance-cost`.

## Does not touch live systems

This skill operates on user-provided descriptions, story screenshots, model export files, data action scripts, planning model configuration summaries, connection configuration descriptions, or architecture documents. It does not connect to any SAC tenant, invoke SAC APIs, refresh any model, execute any data action, or access any live connection source. All live inspection is out of scope.

## Lean operating rules

- Classify findings before recommending. Every finding must be assigned to a design domain (stories, planning models, data actions, allocations, value driver trees, connections, data access controls, performance) before remediation is proposed.
- Planning version categories are the governance backbone. Actuals must be locked after the close period. Budget and forecast versions must have explicit owner assignment and locking governance. Missing version locking is a planning integrity risk.
- Data actions are sequential and stateful. The order of steps in a data action matters: steps that read data must execute before steps that overwrite it. Incorrect step ordering produces silent wrong results, not errors.
- Advanced Formula correctness requires explicit scope. Data action Advanced Formula steps that do not explicitly scope their MEMBERSET or RESULTLOOKUP expressions operate on the full model scope, which is a performance and correctness risk.
- Live connections require SAC-compatible model types on the source. SAC live connections to Datasphere, BW, or S/4HANA require source objects with specific metadata. Connecting to an incompatible source object type produces a broken story, not an error at connection time.
- Import models require governed refresh schedules. An import model without a defined refresh schedule becomes stale. Missing refresh schedules on planning-adjacent import models (actuals feed) undermine planning model accuracy.
- Allocation hierarchies must have bounded scope. Allocation steps that traverse unfiltered dimension hierarchies on large planning models cause timeout failures. Allocation hierarchy scope must be bounded by filter or version to production-safe levels.
- Role assignments must follow least-privilege for planning. Planner roles must not be assigned to users who should only view data. BI Admin access must not be used as a default role. Custom roles scoped to specific models and stories are the correct pattern for large multi-team SAC deployments.
- Evidence from user-provided artifacts or official SAP Analytics Cloud documentation takes precedence over inference.
- Load only the reference needed for the component under review.

## Evidence rules

Label all claims with one of:

- `documentation-based` — grounded in SAP Analytics Cloud Help Portal documentation, SAP Help Portal planning guides, or official SAC connection and security docs
- `user-provided evidence` — story screenshots, model export files, data action scripts, planning model configuration summaries, or architecture documents provided by the user
- `inference` — derived reasoning not directly confirmed by official docs or user evidence

## Live-environment rules

**This skill does not touch live systems.** There is no SAC API call, tenant login, model refresh trigger, data action execution, allocation run, or connection to any source system in this skill's execution path. Users must supply story screenshots, model configuration exports, data action scripts, or written descriptions of their SAC implementation for this skill to review.

## References

Load only when needed:

- [Workflow and output contract](references/workflow-and-output.md) — design domain taxonomy, finding severity, output format.
- [Safety checklist](references/safety-checklist.md) — non-negotiables, common mistakes, when to push back.
- [Official sources](references/official-sources.md) — SAC story design, planning models, data actions, allocations, live and import connections, data access controls, and performance docs.

## Response minimum

Return, at minimum:

- **Problem classification**: design domain(s) affected (stories / planning models / data actions / allocations / value driver trees / connections / data access controls / performance) and specific finding(s).
- **Evidence used**: documentation-based / user-provided evidence / inference.
- **Risk level**: critical (planning integrity breach or unauthorized data access) / high (planning model failure, data action correctness error, or connection reliability risk) / medium (governance gap or performance risk) / low (best practice deviation).
- **Recommended action**: specific remediation per finding (version locking, data action step reordering, Advanced Formula scope restriction, connection type change, role scope reduction, allocation filter addition, or refresh schedule definition).
- **Refusal / escalation triggers**: if live SAC tenant access, model refresh, data action execution, or allocation run is required to complete the review, state that clearly and do not proceed.
- **Business impact**: planning integrity risk, reporting accuracy risk, performance degradation, unauthorized data access, or compliance gap.
- **Next verification step**: validate recommended changes against the current SAC tenant model configuration before applying.
