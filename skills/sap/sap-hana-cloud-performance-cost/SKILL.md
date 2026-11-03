---
name: sap-hana-cloud-performance-cost
description: "Review SAP HANA Cloud performance and cost posture: instance sizing and elasticity, performance modeling (column store, indexes, partitioning, query execution plans), cost management (compute and storage scaling, auto-stop, tier selection), and monitoring (expensive statements, thread sampling, workload classes). Flags sizing risks, modeling anti-patterns, cost governance gaps, and monitoring blind spots. Does not touch live systems."
allowed-tools: Read Grep Glob WebSearch WebFetch
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-06-19"
  category: data
  lifecycle: experimental
---

# SAP HANA Cloud Performance and Cost Review

## Purpose

Assess the performance architecture and cost governance posture of an SAP HANA Cloud implementation. Review instance sizing and elasticity configuration, data modeling for performance (column store vs. row store usage, partitioning strategy, index design, materialized views), query execution plan analysis (EXPLAIN PLAN, SQL Plan Cache, expensive statement trace), cost management controls (compute scaling, storage tier selection, auto-stop configuration), and monitoring coverage (workload classes, thread sampling, key performance views). Surface sizing risks, data modeling anti-patterns, query performance bottlenecks, cost governance gaps, and monitoring blind spots. Does not connect to or mutate any live HANA Cloud instance.

## When to use

Use this skill when the user asks to:

- review HANA Cloud instance sizing decisions (vCPU, memory, storage) against workload characteristics and expected concurrency,
- assess elasticity configuration: scale-out node design, dynamic tiering, compute unit auto-scaling, and scheduled scaling for predictable peak loads,
- evaluate column store data modeling decisions: table partitioning (range, hash, round-robin), sort-column selection for delta compression, index type selection (inverted index, composite inverted index, full text index), and row store vs. column store table type choices,
- analyze SQL query execution plans using EXPLAIN PLAN output, SQL Plan Cache entries, or expensive statement trace results provided by the user,
- review SQLScript stored procedure design for performance: cursor vs. set-based logic, tabular function use, plan stability, and SQLSCRIPT PLAN PROFILER findings,
- assess cost management configuration: HANA Cloud instance compute tier selection, storage auto-scaling thresholds, auto-stop schedule for non-production instances, and data tiering (hot/warm/cold via SAP HANA Native Storage Extension or Data Lake),
- review monitoring posture: expensive statement trace configuration, workload class definition, thread sampling setup, and key performance views (M_EXPENSIVE_STATEMENTS, M_SQL_PLAN_CACHE, M_LOAD_HISTORY_SERVICE) coverage,
- flag HANA Cloud performance anti-patterns: missing table partitioning on large tables, cursor-based SQLScript that can be rewritten as set-based, missing delta merge schedule, or non-production instances without auto-stop.

## When not to use

- When the user needs live inspection of a running HANA Cloud instance, live system views, or query plan output from an active connection — this skill accepts only user-provided EXPLAIN PLAN output, trace exports, monitoring screenshots, or architecture descriptions.
- When the request is about Datasphere semantic modeling or data product architecture built on top of HANA Cloud — use `sap-datasphere-data-product-architecture`.
- When the request is about SAP Analytics Cloud models connected to HANA Cloud via live connection — use `sap-analytics-cloud-planning-governance` for the SAC side.
- When the request concerns BTP-level entitlement or capacity governance for the HANA Cloud service instance — use `sap-btp-governance-review`.

## Does not touch live systems

This skill operates on user-provided descriptions, EXPLAIN PLAN output, SQL Plan Cache exports, expensive statement trace results, architecture documents, monitoring screenshots, or written descriptions of the HANA Cloud configuration. It does not connect to any HANA Cloud instance, execute SQL, invoke HANA Cloud APIs, access the SAP HANA Cloud Central console, or access any connected source or consumer system. All live inspection is out of scope.

## Lean operating rules

- Classify findings before recommending. Every finding must be assigned to a review domain (sizing and elasticity, data modeling, query performance, SQLScript, cost management, monitoring) before remediation is proposed.
- Column store is the default for analytic workloads; row store is for high-throughput transactional lookups on small tables. Every table type choice must be justified by workload type. Column store tables on high-frequency single-row OLTP paths, and row store tables on large analytic scan workloads, are both architectural risks.
- Partitioning is required for large tables. HANA Cloud tables exceeding 2 billion rows require partitioning. Tables between 100 million and 2 billion rows benefit significantly from partitioning for parallel scan performance. The absence of partitioning on large column store tables is a `high` performance finding.
- EXPLAIN PLAN analysis must account for column store hints. HANA Cloud query optimizer decisions can be influenced by hints (CS_ALL_COLUMNS, NO_CS_JOIN, USE_HEX_PLAN). EXPLAIN PLAN output that shows unexpectedly high row-store operator usage warrants a hint review before indexing changes.
- SQLScript cursor loops over column store tables are a performance anti-pattern. Row-at-a-time cursor processing in SQLScript forces row engine execution on column store data, bypassing the column engine parallel scan. Set-based rewrite is always preferred.
- Auto-stop is mandatory for non-production HANA Cloud instances. HANA Cloud compute is billed per running time. Non-production instances without auto-stop configuration accrue avoidable compute cost during idle periods.
- Workload classes are required for multi-tenant cost and resource governance. Without workload class definitions, all queries compete for HANA resources without priority or resource limits, making production workload isolation impossible.
- Evidence from user-provided EXPLAIN PLAN output, trace files, or architecture descriptions takes precedence over inference. Context7 HANA Cloud documentation supplements official SAP Help Portal docs for SQL and SQLScript specifics.
- Load only the reference needed for the component under review.

## Evidence rules

Label all claims with one of:

- `documentation-based` — grounded in SAP HANA Cloud Help Portal documentation or official SAP HANA Cloud administration and SQL reference guides
- `user-provided evidence` — EXPLAIN PLAN output, SQL Plan Cache exports, expensive statement trace results, monitoring screenshots, or architecture descriptions provided by the user
- `context7-supplementary` — grounded in SAP HANA Cloud SQL reference or SQLScript documentation fetched from Context7 (supplementary; applies when SQL statement-level or SQLScript procedure-level guidance is in scope)
- `inference` — derived reasoning not directly confirmed by official docs or user evidence

## Live-environment rules

**This skill does not touch live systems.** There is no HANA Cloud SQL connection, instance API call, HANA Cloud Central console access, system view query, or SQLScript execution in this skill's execution path. Users must supply EXPLAIN PLAN output, SQL Plan Cache exports, expensive statement trace results, monitoring view query results, or written descriptions of their HANA Cloud implementation for this skill to review.

## References

Load only when needed:

- [Workflow and output contract](references/workflow-and-output.md) — review domain taxonomy, finding severity, output format.
- [Safety checklist](references/safety-checklist.md) — non-negotiables, common mistakes, when to push back.
- [Official sources](references/official-sources.md) — SAP HANA Cloud sizing, modeling, partitioning, indexing, cost management, SQLScript, and monitoring docs.
- [Context7 framework docs](references/context7-framework-docs.md) — SAP HANA Cloud SQL and SQLScript reference for EXPLAIN PLAN, plan profiler, and column store hint guidance (supplementary; use when SQL statement or SQLScript procedure analysis is in scope).

## Response minimum

Return, at minimum:

- **Problem classification**: review domain(s) affected (sizing and elasticity / data modeling / query performance / SQLScript / cost management / monitoring) and specific finding(s).
- **Evidence used**: documentation-based / user-provided evidence / context7-supplementary / inference.
- **Risk level**: critical (data loss risk or production availability risk from sizing failure) / high (performance bottleneck or uncontrolled cost accumulation) / medium (governance gap or monitoring blind spot) / low (best practice deviation).
- **Recommended action**: specific remediation per finding (partitioning strategy, index addition or removal, EXPLAIN PLAN hint, SQLScript rewrite guidance, auto-stop configuration, workload class definition, or monitoring view addition).
- **Refusal / escalation triggers**: if live HANA Cloud instance access, SQL execution, or system view query is required to complete the review, state that clearly and do not proceed.
- **Business impact**: production availability risk, query timeout risk, uncontrolled cost overrun, data access performance degradation, or monitoring gap leading to silent incidents.
- **Next verification step**: validate recommended changes against the current HANA Cloud instance configuration and execute EXPLAIN PLAN on the modified query or table before applying changes to production.
