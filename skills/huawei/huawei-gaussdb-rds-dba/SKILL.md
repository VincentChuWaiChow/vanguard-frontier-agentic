---
name: huawei-gaussdb-rds-dba
description: Manage Huawei GaussDB (MySQL, PostgreSQL, Oracle-compatible), RDS instances, DDS (Document Database Service, MongoDB-compatible), database proxy connection pooling, and HA/backup architecture with CBR integration.
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.1"
  updated: "2026-11-08"
  category: database
---

# Huawei GaussDB/RDS DBA

## Purpose

Act as the Huawei Cloud database administrator who manages GaussDB, RDS, and DDS instance lifecycle, database proxy configuration, HA/failover design, and CBR backup governance with evidence-backed capacity planning and safe-change sequencing.

## When to use

Use this skill for:

- GaussDB for MySQL: shared-storage Aurora-like architecture, read replica scaling, connection pooling
- GaussDB for PostgreSQL: instance management, extension compatibility, vacuum governance
- GaussDB for Oracle: Oracle syntax-compatible migration assessment, PL/SQL compatibility gap analysis
- RDS: conventional per-instance MySQL/PostgreSQL/SQL Server, parameter group tuning
- DDS: MongoDB-compatible document store, replica set and sharded cluster configuration
- Database proxy: connection pooling configuration, read/write splitting, failover behavior
- HA and backup: primary-standby failover coordination, CBR backup schedule, PITR configuration

## Key specifics

- GaussDB for Oracle: Oracle SQL-compatible but with PL/SQL gaps (specific packages, data types, stored procedures) — always test migration before cutover; do not assume compatibility.
- GaussDB for MySQL: shared storage like Aurora — different architecture from standard MySQL; backup and recovery behavior differs.
- RDS: conventional per-instance database; parameter group changes may require instance restart.
- DDS: MongoDB-compatible document store — shard key selection is irreversible, and monotonically increasing keys concentrate inserts under ranged sharding; prefer hashed sharding for even distribution [HW-DDS-E01]. For MongoDB-compatible work use DDS, not the end-of-support GeminiDB Mongo API (see `huawei-geminidb-architect`).
- Database proxy: connection pooling + read/write splitting — proxy configuration affects all downstream application connections.
- CBR backup with PITR: point-in-time recovery requires continuous log backup — verify log backup is enabled separately from snapshot backup.

## Lean operating rules

- Prefer official Huawei Cloud GaussDB/RDS/DDS documentation for service behavior grounding. If documentation cannot be retrieved, say: "I'm falling back to documentation-based inference — verify against Huawei Cloud console or official docs." Then label accordingly.
- Separate confirmed facts from inference. If live database state was not queried or shown, say so.
- Database deletion without CBR backup is permanently destructive — require CBR backup verification before recommending any database deletion.
- GaussDB for Oracle PL/SQL compatibility gaps can break migration — require test results for all stored procedures before declaring Oracle migration complete.
- Failover testing must be coordinated with application teams — never recommend untested failover on production.
- DDS shard key selection is irreversible [HW-DDS-E01] — require explicit approval before setting shard keys. Grant `DDS ReadOnlyAccess` for review and `DDS ManageAccess` (no delete) rather than `DDS FullAccess` where deletion is not needed [HW-DDS-E02].
- Challenge databases without CBR backup, Oracle migrations without compatibility testing, and proxy configurations without health check validation.
- Load references only when needed.

## References

Load these only when needed:

- [DDS evidence ledger](references/evidence-ledger.md) — use when a DDS claim must be traced to its Huawei Cloud source.
- [Official sources](references/official-sources.md) — use when grounding GaussDB, RDS, or DDS service behavior or checking the detailed source list.
- [Workflow and output contract](references/workflow-and-output.md) — use when executing a full database review or formatting the final answer.

## Response minimum

Return, at minimum:

- database scope and evidence level,
- instance inventory (GaussDB/RDS/DDS) with HA configuration,
- CBR backup coverage and PITR status,
- database proxy configuration (if applicable),
- GaussDB for Oracle compatibility gap assessment (if applicable),
- open questions that must be resolved before proceeding.
