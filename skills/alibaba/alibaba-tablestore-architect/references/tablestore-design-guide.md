# Alibaba Cloud Tablestore design guide

Load this for primary-key and partition-key design, transactions, conditional
updates, index choice, TTL and versions, instance type, capacity units, limits,
access control or SDK questions. Every bracketed ID resolves in
[evidence-ledger.md](evidence-ledger.md). Text without an ID is labeled
inference or recommendation.

Scope: the Wide Column model. Tablestore borrows Bigtable/HBase vocabulary, but
its guarantees differ. Its partition key is the **first primary key column** [TS-E01].

## Capability matrix

| Area | Status | Verified behavior | Evidence |
| --- | --- | --- | --- |
| Primary key | SUPPORTED | 1-4 columns fixed at creation; first column is the partition key; rows sorted by PK | [TS-E01] |
| Partitioning | CONDITIONAL | Auto split/merge, but one partition key value cannot split; keep it under 10 GB | [TS-E02] |
| Transactions | CONDITIONAL | Local transactions within one partition key value only; must be enabled at creation | [TS-E04] |
| Optimistic concurrency | SUPPORTED | Row-existence and column-value conditions; billed as reads | [TS-E05] |
| Secondary indexes | CONDITIONAL | Global = async, eventual; local = sync, strong, shares the partition key | [TS-E06] |
| Search index | CONDITIONAL | Async, typically within 3 s; not real-time | [TS-E07] |
| TTL / versions | CONDITIONAL | Expired data unreadable at once; strict rules when indexes exist | [TS-E08] [TS-E09] |
| Instance type | SUPPORTED | High-performance or Capacity, permanent | [TS-E10] |
| Access control | SUPPORTED | RAM + Control Policies + Network ACLs + Instance Policies, deny wins | [TS-E14] |
| Tunnel/Stream, backup, pricing | UNVERIFIED | Not retrieved this pass | [TS-E16] |

## Primary key and partition key

The primary key (one to four columns, with order and types) is set at table
creation [TS-E01]. Treat it as permanent. *(inference: the docs require it at
creation, and this ledger has no row for changing it)*

Partition key rules:

1. **One partition key value can never be split.** Keep each value's rows under
   10 GB [TS-E02]. Tenant or device IDs that grow without bound violate this.
2. **No sequential partition keys.** A sequentially increasing partition key
   sends every write to the last partition, a tail hotspot that splitting cannot
   fix. A hot range of non-sequential values, such as an IP range, is split
   automatically [TS-E03].
3. **Fix sequence by design** [TS-E03]:
   - reorder the key so a naturally distributed column (machine, user, device,
     order ID) comes first;
   - bucket by time modulo N and read all buckets in parallel; or
   - when the sequential column must lead, prepend a hash prefix. That kills
     range reads in the original order, so plan a search index for them.
4. **Transactions want coarse keys.** Local transactions cover one partition
   key value only [TS-E04], so the cardinality-versus-atomicity trade-off has
   to be stated explicitly. *(inference)*

## Transactions and concurrency

- Local transactions [TS-E04]:
  - enabled only at table creation
  - one partition key value, pessimistic write lock on that value
  - 60-second lifetime, and 60 seconds maximum between operations
  - 4 MB of writes, one request at a time
  - not combinable with auto-increment primary key columns; writes to locked
    data without the transaction ID fail
- They serialize all writers on that key value. Keep them short and rare on hot
  keys. *(inference)*
- Conditional updates give optimistic checks on row existence and up to 32
  column conditions. They read the row and are **billed as reads**, and are
  SDK/CLI only [TS-E05]. Count those reads in capacity plans.
  *(recommendation)*

## Index choice

| Need | Use | Consistency | Constraint | Evidence |
| --- | --- | --- | --- | --- |
| Alternate key, read-after-write | Local secondary index | strong (synchronous) | must lead with the table's first PK column | [TS-E06] |
| Alternate key, any leading column | Global secondary index | eventual (milliseconds) | — | [TS-E06] |
| Multi-field, range, full-text, aggregation | Search index | async, typically within 3 s | not real-time | [TS-E07] |

Limits [TS-E13]: 5 secondary indexes and 36 index fields. Index
tables also count against the 64-tables-per-instance quota, so a table with
five indexes uses six table slots *(inference)*. Budget them. *(recommendation)*

## TTL and versions

- TTL is in seconds and defaults to -1 (never) [TS-E08].
- **Expired data is unreadable immediately**, even before deletion. Raising
  the TTL can **resurrect** expired versions that haven't been deleted yet
  [TS-E08]. Treat a TTL increase as a data-exposure change that needs review.
  *(recommendation)*
- With any index on the table [TS-E09]:
  - TTL must be -1, or updates must be disabled
  - Max Versions must be 1, and is locked once an index exists
  - the table's TTL must be at least each index's TTL

  Decide TTL, versioning and indexes together, at design time.
  *(recommendation)*

## Capacity and cost

- High-performance or Capacity is **permanent**. Only High-performance offers
  reserved throughput, and on-demand throughput costs more than reserved
  [TS-E10].
- CUs are row-granular in 4 KB steps, rounded up [TS-E11]. Many small rows cost
  more CUs than fewer larger rows. Model rows-per-second, not bytes per second.
  *(inference)*
- Prices and cross-region traffic billing were not verified [TS-E16]. Cost
  numbers need a freshly fetched pricing page.

## Limits that shape the model

Primary key String/Binary values are capped at 1 KB, and attribute values at
2 MB per version. Row size is unlimited, but keep rows under about 10,000
columns. Write requests are capped at 4 MB, BatchWriteRow at 200 rows and
BatchGetRow at 100 rows [TS-E12].

## Access control

Combine RAM policies with resource-based Instance Policies, Network ACLs and
Control Policies. When an Instance Policy and a RAM policy conflict, deny wins
[TS-E14]. Use the VPC endpoint for in-cloud clients; Internet access has higher
latency [TS-E18]. Use an Instance Policy or
Network ACL to fence an instance even when a RAM policy is broad. Never rely on
an allow alone. *(recommendation)*

## SDKs

| Language | Package | Evidence |
| --- | --- | --- |
| Java | Maven `com.aliyun.openservices:tablestore` (Java 6+) | [TS-E15] |
| Python | `tablestore` (Python 3 only from 6.0.0) | [TS-E17] |
| Go, Node.js, .NET, PHP | not verified | [TS-E16] |

Resolve the current version from the package registry. Look up method names in
the SDK reference before writing code. Use RAM roles or STS credentials, never
embedded AccessKeys. *(rule)*

## When to push back

- Partition key with unbounded growth per value (over 10 GB) or sequential
  values without a hash prefix.
- Local transactions assumed across partition key values, or not enabled at
  creation.
- Read-after-write expected from a global secondary index or a search index.
- TTL increase treated as harmless, or TTL/versions changed on an indexed table
  without checking the index constraints.
- An instance type change planned "later" [TS-E10].
- Claims about Tunnel/Stream, backup or pricing without a fresh official source
  [TS-E16].
