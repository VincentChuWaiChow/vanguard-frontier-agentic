# DynamoDB consistency, indexing and replication decisions

Load this for GSI vs LSI, read consistency, global tables (MREC vs MRSC),
transactions across Regions, or DAX questions. Every bracketed ID resolves in
[evidence-ledger.md](evidence-ledger.md). Text without an ID is labeled
inference or recommendation.

## Read consistency

| Read target | Eventual | Strong | Evidence |
| --- | --- | --- | --- |
| Base table | yes (default) | yes, `ConsistentRead=true`, 2x read cost | [DDB-E01] [DDB-E02] |
| LSI | yes | yes | [DDB-E01] |
| GSI | yes | **no** | [DDB-E01] |
| DynamoDB Streams | yes | **no** | [DDB-E01] |
| DAX | yes | passed through to DynamoDB, not cached | [DDB-E17] |

A design that needs read-after-write on an alternate access pattern cannot get
it from a GSI [DDB-E01]. Either read the base table strongly, use an LSI
(with its limits below), or accept and design for staleness.

## Index decision matrix

Start with the cheapest option that serves the access pattern. Every index is
a standing write and storage cost [DDB-E08].

| Question | No new index | LSI | GSI |
| --- | --- | --- | --- |
| Pattern served by table key + key condition? | **choose this** | — | — |
| Same partition key, different sort order, needs strong reads? | — | candidate [DDB-E01] [DDB-E03] | no strong reads [DDB-E01] |
| Different partition key? | — | not possible [DDB-E03] | candidate [DDB-E06] |
| Table already exists? | — | not possible: creation time only [DDB-E03] | re-verify before relying on it [DDB-E18] |
| Any partition key value may exceed 10 GB (base + index items)? | — | **reject**: writes can be blocked [DDB-E04] | not subject to that limit [DDB-E04] |
| Needs non-projected attributes? | — | fetches allowed, extra cost [DDB-E05] | cannot fetch: project them or read the table [DDB-E05] |
| Only some items should be indexed? | — | — | sparse GSI [DDB-E06] |
| Multi-region strong consistency (MRSC) table? | — | **not supported** [DDB-E14] | supported, eventual [DDB-E14] |
| Composite query on several natural attributes? | — | — | multi-attribute keys before synthetic concatenated keys [DDB-E09] |

Operational traps:

- In provisioned mode an under-provisioned GSI **throttles base-table writes**
  [DDB-E07]. Size GSI write capacity at least equal to the table's, as AWS
  advises [DDB-E07].
- An LSI is a one-way, creation-time decision [DDB-E03] that caps every item
  collection at 10 GB [DDB-E04]. Reject it for unbounded per-key growth such as
  per-tenant event logs. *(recommendation)*

## Global tables: MREC vs MRSC

The mode is fixed at creation and shared by all replicas [DDB-E10]. Choose
deliberately; there is no migration path in place.

| Concern | MREC (default) | MRSC |
| --- | --- | --- |
| Replication | async, typically within a second [DDB-E11] | sync to at least one other Region before ack [DDB-E12] |
| Concurrent writes to one item | last writer wins, silently [DDB-E11] | `ReplicatedWriteConflictException`, retryable [DDB-E13] |
| Strong read | latest only if last written locally [DDB-E11] | latest on any replica [DDB-E12] |
| Conditional writes | evaluated against the local version [DDB-E11] | evaluated against the latest version [DDB-E12] |
| Transactions | atomic only in the invoking Region [DDB-E15] | **not supported** [DDB-E14] |
| TTL / LSIs | supported | **not supported** [DDB-E14] |
| Topology | add or remove replicas | exactly 3 Regions, fixed; empty table to convert [DDB-E12] [DDB-E13] |
| Streams | always on, per-item order only [DDB-E16] | optional [DDB-E16] |

Consequences to state explicitly in a review:

- **MREC + conditional writes is not a distributed lock.** Two Regions can
  both pass a condition on their local version, and last writer wins
  [DDB-E11]. Idempotency keys, inventory and balances need a single write
  Region per item or MRSC. *(inference)*
- **MRSC trades features for consistency.** No transactions, TTL or LSIs
  [DDB-E14]; write and strong-read latency depend on inter-Region distance
  [DDB-E12]. Applications must handle `ReplicatedWriteConflictException`
  with bounded retries [DDB-E13].
- **MREC transactions are not global.** Readers in another Region may see half
  a `TransactWriteItems` [DDB-E15].
- The MRSC Region list is volatile. Re-fetch the source before naming Regions
  [DDB-E12].

## DAX: when caching adds risk

DAX helps repeated eventually consistent point reads. It adds risk when:

- the workload needs strong reads: they bypass the cache, so you pay for DAX
  and still hit DynamoDB [DDB-E17];
- other writers bypass DAX: the item cache stays stale until eviction
  [DDB-E17];
- correctness depends on Query/Scan results reflecting item updates: the query
  cache is not invalidated by item writes [DDB-E17].

DAX pricing and sizing were not verified [DDB-E18]. A DAX recommendation must
cite a freshly retrieved pricing page and a measured read-repeat ratio, or be
labeled an assumption. *(recommendation)*
