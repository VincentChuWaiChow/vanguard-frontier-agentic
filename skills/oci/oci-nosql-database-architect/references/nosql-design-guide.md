# Oracle NoSQL Database Cloud Service design guide

Load this for table, key, consistency, capacity, limits, TTL, child-table,
Global Active table, backup or IAM questions. Every bracketed ID resolves in
[evidence-ledger.md](evidence-ledger.md). Anything without an ID is labeled
**inference** or **recommendation** and must be presented that way.

Do not import DynamoDB or Cosmos DB vocabulary. NDCS has *shard keys* (not
partition keys); *read units and write units*, which Oracle abbreviates RU and
WU but which are not DynamoDB RCUs/WCUs or Cosmos DB Request Units; and
*Global Active tables* (not DynamoDB global tables). Their guarantees differ.

## Capability matrix

| Area | Status | What is verified | Evidence |
| --- | --- | --- | --- |
| Data model | SUPPORTED | Fixed-schema tables and schema-less JSON columns | [OCI-NOSQL-E14] |
| Primary / shard key | SUPPORTED | Shard key is a portion of the primary key; both are immutable after creation | [OCI-NOSQL-E02] [OCI-NOSQL-E04] |
| Read consistency | SUPPORTED | EVENTUAL or ABSOLUTE, per handle or per read; ABSOLUTE costs 2x read units | [OCI-NOSQL-E03] |
| Transactions | CONDITIONAL | ACID only across records sharing a shard key (or one parent-child hierarchy) | [OCI-NOSQL-E05] [OCI-NOSQL-E08] |
| Capacity | CONDITIONAL | Provisioned or On-Demand; 1 KB units; hard per-table caps; mode switch once per day | [OCI-NOSQL-E01] [OCI-NOSQL-E15] [OCI-NOSQL-E23] |
| Table limits | SUPPORTED | 5 indexes/table, 512 KB rows, 30 tables/region | [OCI-NOSQL-E24] |
| TTL | SUPPORTED | Table default plus per-row override, HOURS or DAYS | [OCI-NOSQL-E07] |
| Parent-child tables | SUPPORTED | Shared shard key, inherited primary key, no referential integrity | [OCI-NOSQL-E08] |
| Multi-region | CONDITIONAL | Global Active tables: async active-active, latest-timestamp wins, frozen schema; not in dedicated hosted environments | [OCI-NOSQL-E09] [OCI-NOSQL-E10] [OCI-NOSQL-E12] [OCI-NOSQL-E30] |
| Backup / restore | CONDITIONAL | Customer-run, via Oracle's migrator tool to Object Storage | [OCI-NOSQL-E30] |
| IAM | SUPPORTED | Verb-to-permission mapping on `nosql-rows`, `nosql-tables`, `nosql-indexes`; table-name scoping | [OCI-NOSQL-E13] [OCI-NOSQL-E29] |
| SDKs | CONDITIONAL | Java, Python, Node.js, Go, .NET verified; Spring Data and Rust repos exist but details unverified | [OCI-NOSQL-E16] [OCI-NOSQL-E21] [OCI-NOSQL-E32] [OCI-NOSQL-E33] |
| Private networking | CONDITIONAL | Service gateway path from a VCN (solution page); no NDCS-specific private endpoint found | [OCI-NOSQL-E34] [OCI-NOSQL-E22] |
| Customer-managed keys | CONDITIONAL | Dedicated environment only; AES-256 in OCI KMS; single-version keys only | [OCI-NOSQL-E35] |
| Pricing, quota increases | UNVERIFIED | Not found this pass; do not assert | [OCI-NOSQL-E22] |

## Key design

The primary key and shard key are permanent [OCI-NOSQL-E04]. Treat key design
as a one-way door: a wrong shard key means a new table plus a data migration,
not an ALTER. That alone justifies refusing to approve a schema without an
access-pattern list. *(inference from E04)*

Shard key decision checklist [OCI-NOSQL-E05]:

1. **Cardinality** — high-cardinality identifiers (`customerID`, `userID`)
   spread data; low-cardinality fields (country, status) create hot shards.
2. **Atomicity** — if a business operation must commit several records
   atomically, those records must share a shard key. This pulls toward
   *coarser* shard keys and directly conflicts with cardinality. Name the
   trade-off explicitly; do not paper over it.
3. **Uniform distribution** — no single shard should cap system capacity.
4. **Query isolation** — a query that does not target one shard runs on all
   shards. Make the frequent queries shard-targeted.

If no shard key is declared, the full primary key is used [OCI-NOSQL-E06].
That gives the widest distribution and the narrowest transaction scope.
*(inference)*

**Parent-child tables** model 1:N relationships with ACID writes across the
hierarchy [OCI-NOSQL-E08]. Referential integrity is not enforced and a parent
cannot be dropped before its children [OCI-NOSQL-E08], so orphan prevention is
the application's job. *(inference)*

## Consistency

| Need | Choose | Cost | Evidence |
| --- | --- | --- | --- |
| Read-your-latest-write for a key in this region | ABSOLUTE | 2x read units | [OCI-NOSQL-E03] |
| Latency/cost-sensitive reads that tolerate staleness | EVENTUAL | 1x read units | [OCI-NOSQL-E03] |
| Latest value across regions on a Global Active table | **Not available** — remote updates are always eventually consistent | — | [OCI-NOSQL-E10] |

Set consistency per read for the few reads that need ABSOLUTE rather than on
the handle for everything; the handle-wide setting doubles every read's cost
[OCI-NOSQL-E03]. *(recommendation)*

## Capacity, limits and cost

- Units are 1 KB granular: a read unit is up to 1 KB/s of eventually
  consistent reads, a write unit up to 1 KB/s of writes [OCI-NOSQL-E15].
  Row size, not row count, drives units. ABSOLUTE reads double the read
  figure [OCI-NOSQL-E03].
- **Writes can cost reads.** Conditional puts (IfAbsent/IfPresent), UPDATE
  statements and deletes consume read units at the absolute (x2) rate in
  addition to write units [OCI-NOSQL-E25]. A write-heavy workload with
  conditional writes needs read capacity too.
- **Index updates consume write units** [OCI-NOSQL-E15], and a table may have
  at most 5 indexes [OCI-NOSQL-E24]. Each index is a standing write cost and a
  scarce slot; the default answer to "add an index?" is "show the query that
  needs it." *(recommendation)*
- Rows are capped at 512 KB [OCI-NOSQL-E24]; a growing JSON document needs a
  child table or splitting before it hits that ceiling. *(recommendation)*
- **Hard per-table ceilings** [OCI-NOSQL-E23]: Provisioned up to 40,000 read
  and 20,000 write units; On-Demand only up to 10,000 read and 5,000 write
  units, and only 3 On-Demand tables per region. A table that may exceed the
  On-Demand ceiling must be Provisioned.
- **Mode switches are rationed** [OCI-NOSQL-E23]: capacity mode can change
  only once per day, and limits can decrease at most four times per 24 hours
  (increases are unlimited). A failover or cost plan that flips mode or shrinks
  capacity repeatedly will fail. Size for peak, then scale down deliberately.
- Prices and quota increases are volatile and were not retrieved
  [OCI-NOSQL-E22]. Any cost estimate must cite a pricing page fetched at the
  time of the estimate, or be labeled an assumption. Do not use
  hosted-environment totals: the source contradicts itself [OCI-NOSQL-E31].

## TTL

Table TTL defaults to zero (never expire), takes HOURS or DAYS, and a row TTL
overrides the table value [OCI-NOSQL-E07]. Expired rows stop being retrievable
and are later purged from disk [OCI-NOSQL-E07], but no purge timing is given.
Do not use TTL as an exact business-deletion deadline (for example legal
erasure) unless purge timing has been verified separately. *(inference)*

## Backup and restore

Backup and restore are the **customer's** responsibility, using Oracle's
migrator tool and Object Storage [OCI-NOSQL-E30]. Treat "the cloud backs it up"
as a defect. A design is not READY without a scheduled export, a tested
restore and a stated RPO/RTO. *(recommendation)* A restore writes to a table,
so it is a mutation under the approval rules in `SKILL.md`.

## Global Active tables

Use for regional DR or local-latency reads with active-active writes
[OCI-NOSQL-E09]. They are not available in a dedicated hosted environment
[OCI-NOSQL-E30]. Before approving, confirm all of:

- [ ] The schema is frozen (`with schema frozen`, or `with schema frozen force`
      if there are no JSON columns) [OCI-NOSQL-E12]. Any later schema change
      means **dropping all replicas, altering, and re-adding them**
      [OCI-NOSQL-E26]. That is a multi-region production change. Include at
      least one JSON column, as Oracle recommends, to avoid it [OCI-NOSQL-E26].
- [ ] The application tolerates **latest-timestamp-wins** conflict resolution
      [OCI-NOSQL-E09]. Counters, balances and inventory decrements written in
      two regions will lose updates. *(inference)* Route those writes to one
      home region or redesign them.
- [ ] Nothing depends on cross-region transactions or cross-region ABSOLUTE
      reads; both are region-local [OCI-NOSQL-E10].
- [ ] Global keys do not use identity columns, which Oracle advises against in
      most cases because they are not unique across replicas [OCI-NOSQL-E11].
- [ ] Child tables are planned: a child of a Global Active table is a
      singleton unless its schema is frozen and a replica is added to it too
      [OCI-NOSQL-E27].
- [ ] The receiver region has the same-name compartment, no same-name table,
      and the replica's IAM grants are planned [OCI-NOSQL-E11].
- [ ] Each region's write capacity covers replicated incoming writes, which are
      billed [OCI-NOSQL-E25], and failover peak load. Remember mode switches
      are once per day [OCI-NOSQL-E23]. *(recommendation)*

## IAM least privilege

| Workload | Statement | Grants | Evidence |
| --- | --- | --- | --- |
| Read-only service or report | `read nosql-rows` | NOSQL_ROWS_READ | [OCI-NOSQL-E13] |
| Writer that never deletes | `use nosql-rows` | + NOSQL_ROWS_INSERT | [OCI-NOSQL-E13] |
| Writer that deletes | `manage nosql-rows` | + NOSQL_ROWS_DELETE | [OCI-NOSQL-E13] |
| Capacity / schema operator | `use nosql-tables` | + NOSQL_TABLE_ALTER (limits, ALTER TABLE) | [OCI-NOSQL-E29] |
| Table lifecycle | `manage nosql-tables` | + NOSQL_TABLE_CREATE, NOSQL_TABLE_DROP | [OCI-NOSQL-E29] |
| Index lifecycle | `manage nosql-indexes` | index create/drop | [OCI-NOSQL-E13] |

Rules:

- Application identities get `nosql-rows` verbs only: `read`, `use`, or
  `manage nosql-rows` when the application must delete rows [OCI-NOSQL-E13].
  Keep every `nosql-tables` and `nosql-indexes` verb off application
  identities, because even `use nosql-tables` allows ALTER TABLE and limit
  changes [OCI-NOSQL-E29]. *(recommendation)*
- Scope to a compartment, never the tenancy, and narrow to one table with
  `target.nosql-table.name` where possible [OCI-NOSQL-E29].
- On Global Active tables, row permissions are **per region** and do not stop
  replication: a write allowed in one region reaches every replica regardless
  of the receiving region's permissions [OCI-NOSQL-E28]. The effective write
  policy is the most permissive region's. *(inference)*
- Replica add/drop needs NOSQL_TABLE_ALTER in the sender and every receiver
  region, plus NOSQL_TABLE_CREATE or NOSQL_TABLE_DROP in the target receiver
  [OCI-NOSQL-E28].
- Policy statements must use real group and compartment names supplied by the
  user; never invent them.

## When to push back

- Shard key proposed without the access patterns and the atomicity
  requirement.
- Low-cardinality shard key (tenant, status, country, date bucket).
- "Use ABSOLUTE everywhere to be safe" without a cost line.
- On-Demand for a table whose peak may exceed 10,000 read or 5,000 write units
  [OCI-NOSQL-E23].
- A plan that relies on switching capacity mode more than once a day
  [OCI-NOSQL-E23].
- Global Active table for multi-region counters or read-modify-write without a
  home-region write rule, or for a dedicated hosted environment
  [OCI-NOSQL-E30].
- No backup schedule or untested restore [OCI-NOSQL-E30].
- Any claim about an NDCS private endpoint, pricing or quota increases without a
  freshly retrieved source [OCI-NOSQL-E22].
- CMEK planned on a shared (non-dedicated) environment, or a key rotation plan
  that relies on multi-version keys [OCI-NOSQL-E35].
- Confusing NDCS with Autonomous Database or Oracle Database JSON features.
