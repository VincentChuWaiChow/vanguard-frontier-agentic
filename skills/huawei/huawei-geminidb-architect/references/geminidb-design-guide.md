# Huawei Cloud GeminiDB design guide

Load this for GeminiDB API selection, GeminiDB Cassandra API data modeling,
consistency, deletion and capacity rules, IAM or billing questions. Every
bracketed ID resolves in [evidence-ledger.md](evidence-ledger.md). Text without
an ID is labeled inference or recommendation.

GeminiDB is **one service with several wire-compatible APIs** on a
decoupled storage/compute architecture [GDB-E01]. "Compatible" means the API
and drivers. It does not mean that open-source Cassandra, MongoDB or AWS
DynamoDB operational guarantees carry over. Verify each one.

## API selection

| Need | API | Status | Evidence |
| --- | --- | --- | --- |
| Key-value, Redis protocol | GeminiDB Redis API | available | [GDB-E03] |
| Key-value, DynamoDB protocol (migration) | GeminiDB DynamoDB-Compatible API | available; "completely compatible" is a provider claim to test, not proven behavior | [GDB-E03] |
| Wide-column, CQL | GeminiDB Cassandra API | available | [GDB-E03] |
| Wide-column, HBase ecosystem | GeminiDB HBase API | available | [GDB-E03] |
| Time series | GeminiDB Influx API | available | [GDB-E03] |
| Document, MongoDB protocol | **GeminiDB Mongo API: End of Support** | no new instances, no support, no SLA | [GDB-E02] |

**Reject new designs on the GeminiDB Mongo API.** It has reached End of
Support [GDB-E02]. For MongoDB-compatible workloads use DDS, which is covered
by the `huawei-gaussdb-rds-dba` skill. Treat existing Mongo API instances as a
migration risk to schedule. *(recommendation)*

For the DynamoDB-Compatible API, require a compatibility test of every
operation, condition expression, index and consistency behavior the workload
uses before migration. Huawei's "completely compatible" statement [GDB-E03] is
not a substitute. *(recommendation)*

## GeminiDB Cassandra API: data modeling rules

- **Row size:** key plus value at most 64 KB per row, average at most 10 KB.
  Store BLOBs in OBS, not the database [GDB-E04].
- **Partition keys:** distribute load evenly. Avoid BigKey (too much data in
  one partition, which can cause OOM) and HotKey (bursts on one key) [GDB-E05].
  Hard caps per partition key: **100,000 rows and 100 MB** [GDB-E05]. Any
  partition key that grows without bound (one per tenant, one per device
  forever) breaks these. Put a time bucket or other bounding component in the
  key. *(recommendation)*
- **Retention:** every table needs a data-deletion policy [GDB-E04]. Use TTL
  instead of mass deletes. Delete by primary key, at most 1,000 rows at a time
  within one partition key, and never repeatedly delete large ranges in one
  partition, which causes tombstones [GDB-E07].
- **Scale:** keep an instance at or below 50 nodes, then shard vertically (by
  module) or horizontally (by table) [GDB-E07]. Guidance limits are about 50
  billion records per table, 100 tables per database and 20-50 fields per table
  [GDB-E08].

## Consistency and transactions

- Writes are **strongly consistent**, including concurrent writes and across
  tables, but **no transactions** are supported [GDB-E06]. Multi-row or
  multi-table invariants need application-level design (idempotency,
  compensation). *(inference)*
- Migrations through DRS are eventually consistent [GDB-E06]. Cut-over plans
  need a verification window.
- Per-request CQL consistency levels were not verified [GDB-E11]. Do not
  assume open-source Cassandra's `ONE`/`QUORUM` semantics apply.

## Capacity

Size for peak QPS and data growth, add **N/2 redundant nodes** for failure
scenarios, and keep per-node CPU at or below 50% under typical load [GDB-E08].
A design that runs at 80% CPU in steady state has no failover headroom.
*(inference)*

## IAM

System policies are `GeminiDB FullAccess` and `GeminiDB ReadOnlyAccess`
[GDB-E09]. Give operators and reviewers ReadOnlyAccess by default, and grant
FullAccess only to the identities that manage instances, scoped to the
project. *(recommendation)* Custom fine-grained permissions were not verified
[GDB-E11].

## Billing

Cost is instance specifications plus storage. On top of that come backup
storage beyond a free allowance equal to the purchased storage (then OBS
rates), and public-network traffic. Billing is pay-per-use (hourly) or
yearly/monthly, and the two can be switched [GDB-E10]. Prices are not in this
ledger. Cost numbers need a freshly fetched price page.

## Not verified

CQL consistency levels, cross-region DR, international-site backup retention
and PITR, region availability, and driver versions [GDB-E11]. Fetch the
official page or label the point an assumption.

## When to push back

- Any new workload on the GeminiDB Mongo API [GDB-E02].
- DynamoDB-Compatible migration without a per-operation compatibility test.
- Unbounded partition keys, rows over 64 KB, or BLOBs in the database.
- Mass deletes instead of TTL, or no deletion policy at all.
- Instances planned beyond 50 nodes without sharding.
- Steady-state CPU above 50%, or no N/2 failure headroom.
- Transactions assumed on the Cassandra API.
