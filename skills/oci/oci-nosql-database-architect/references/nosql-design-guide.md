# Oracle NoSQL Database Cloud Service design guide

Load this for table, key, consistency, capacity, TTL, child-table, Global
Active table, or IAM questions. Every bracketed ID resolves in
[evidence-ledger.md](evidence-ledger.md). Anything without an ID here is
labeled **inference** and must be presented that way.

Do not import DynamoDB or Cosmos DB vocabulary. NDCS has *shard keys* (not
partition keys), *read/write units* (not RCUs/WCUs or RUs), and *Global Active
tables* (not global tables or multi-region writes). Their guarantees differ.

## Capability matrix

| Area | Status | What is verified | Evidence |
| --- | --- | --- | --- |
| Data model | SUPPORTED | Fixed-schema tables and schema-less JSON columns | [OCI-NOSQL-E14] |
| Primary / shard key | SUPPORTED | Shard key is a portion of the primary key; both are immutable after creation | [OCI-NOSQL-E02] [OCI-NOSQL-E04] |
| Read consistency | SUPPORTED | EVENTUAL or ABSOLUTE, per handle or per read; ABSOLUTE costs 2x read units | [OCI-NOSQL-E03] |
| Transactions | CONDITIONAL | ACID only across records sharing a shard key (or one parent-child hierarchy) | [OCI-NOSQL-E05] [OCI-NOSQL-E08] |
| Capacity | SUPPORTED | Provisioned or On-Demand, switchable both ways; 1 KB units; index updates cost writes | [OCI-NOSQL-E01] [OCI-NOSQL-E14] [OCI-NOSQL-E15] |
| TTL | SUPPORTED | Table default plus per-row override, HOURS or DAYS | [OCI-NOSQL-E07] |
| Parent-child tables | SUPPORTED | Shared shard key, inherited primary key, no referential integrity | [OCI-NOSQL-E08] |
| Multi-region | CONDITIONAL | Global Active tables: async active-active, latest-timestamp wins, frozen schema | [OCI-NOSQL-E09] [OCI-NOSQL-E10] [OCI-NOSQL-E11] [OCI-NOSQL-E12] |
| IAM | SUPPORTED | Verb-to-permission mapping on `nosql-rows` and `nosql-indexes` | [OCI-NOSQL-E13] |
| SDKs | CONDITIONAL | Java, Python, Node.js verified; Go, .NET, Spring, Rust repos exist but details unverified | [OCI-NOSQL-E16] [OCI-NOSQL-E21] |
| Backup, private networking, CMK, pricing, quotas | UNVERIFIED | Not retrieved this pass; do not assert | [OCI-NOSQL-E22] |

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

If no shard key is declared, the full primary key is used [OCI-NOSQL-E06]:
maximum distribution, minimum transaction scope.

**Parent-child tables** model 1:N relationships with ACID writes across the
hierarchy because all levels share the root's shard key [OCI-NOSQL-E08]. The
service does not enforce referential integrity, so orphan prevention is the
application's job, and a parent cannot be dropped before its children.

## Consistency

| Need | Choose | Cost | Evidence |
| --- | --- | --- | --- |
| Read-your-latest-write for a key in this region | ABSOLUTE | 2x read units | [OCI-NOSQL-E03] |
| Latency/cost-sensitive reads that tolerate staleness | EVENTUAL | 1x read units | [OCI-NOSQL-E03] |
| Latest value across regions on a Global Active table | **Not available** — remote updates are always eventually consistent | — | [OCI-NOSQL-E10] |

Set consistency per read for the few reads that need ABSOLUTE rather than on
the handle for everything; the handle-wide setting doubles every read's cost
[OCI-NOSQL-E03]. *(recommendation)*

## Capacity and cost

- Units are 1 KB granular: a read unit is up to 1 KB/s of eventually
  consistent reads, a write unit up to 1 KB/s of writes [OCI-NOSQL-E15].
  Row size, not row count, drives units. ABSOLUTE reads double the read
  figure [OCI-NOSQL-E03].
- **Index updates consume write units** [OCI-NOSQL-E15]. Every secondary
  index is a standing write tax; the default answer to "add an index?" is
  "show the query that needs it." *(recommendation)*
- Provisioned caps throughput at the limits you set; On-Demand lets Oracle
  manage it [OCI-NOSQL-E01]. Switching is allowed both ways [OCI-NOSQL-E14],
  so start where the traffic shape is known and measure. Whether and how often
  a switch is rate-limited was **not verified** — check before planning
  frequent flips.
- Prices, free-tier terms and quotas are volatile and were not retrieved
  [OCI-NOSQL-E22]. Any cost estimate must cite a pricing page fetched at the
  time of the estimate, or be labeled an assumption.

## TTL

Table TTL defaults to zero (never expire), takes HOURS or DAYS, and a row TTL
overrides the table value [OCI-NOSQL-E07]. Expired rows stop being
retrievable. Do not use TTL as an exact business-deletion deadline (for example
legal erasure) unless purge timing has been verified separately. *(inference:
the docs state the rows become unavailable and are later purged from disk, but
give no purge timing)*

## Global Active tables

Use for regional DR or local-latency reads with multi-region writes
[OCI-NOSQL-E09]. Before approving, confirm all of:

- [ ] The schema is frozen (`with schema frozen`, or `with schema frozen force`
      if there are no JSON columns) [OCI-NOSQL-E12]. Plan schema evolution
      with that constraint in mind.
- [ ] The application tolerates **latest-timestamp-wins** conflict resolution
      [OCI-NOSQL-E09]. Counters, balances and inventory decrements written in
      two regions will lose updates. *(inference)* Route those writes to one
      home region or redesign them.
- [ ] Nothing depends on cross-region transactions or cross-region ABSOLUTE
      reads; both are region-local [OCI-NOSQL-E10].
- [ ] No identity column is used as a global key [OCI-NOSQL-E11]; use
      application-generated unique keys.
- [ ] The receiver region has the same-name compartment, no same-name table,
      and the replica's IAM grants are planned [OCI-NOSQL-E11].
- [ ] Per-region read/write limits and capacity mode are sized for failover
      traffic, not only steady state [OCI-NOSQL-E11]. *(recommendation)*

## IAM least privilege

Grant the verb the workload actually needs on `nosql-rows` [OCI-NOSQL-E13]:

| Workload | Verb | Grants |
| --- | --- | --- |
| Read-only service or report | `read nosql-rows` | NOSQL_ROWS_READ |
| Writer that never deletes | `use nosql-rows` | + NOSQL_ROWS_INSERT |
| Writer that deletes | `manage nosql-rows` | + NOSQL_ROWS_DELETE |

Index create/drop is `manage nosql-indexes` [OCI-NOSQL-E13]. Keep it off
application identities. Scope every statement to a compartment, never the
tenancy. *(recommendation)* Policy statements must use real group and
compartment names supplied by the user; never invent them.

## When to push back

- Shard key proposed without the list of access patterns and the atomicity
  requirement.
- Low-cardinality shard key (tenant, status, country, date bucket).
- "Use ABSOLUTE everywhere to be safe" without a cost line.
- Global Active table for a workload with multi-region counters or
  read-modify-write without a home-region write rule.
- Any claim about backup, private endpoints, CMK, pricing or quotas without a
  freshly retrieved source [OCI-NOSQL-E22].
- Confusing NDCS with Autonomous Database or Oracle Database JSON features.
