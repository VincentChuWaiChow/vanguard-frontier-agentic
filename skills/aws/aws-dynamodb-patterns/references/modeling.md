# Modeling from access patterns

## Decide what not to build

Start with known application operations, not an entity diagram translated one
table per SQL table and not a compulsory single-table conversion. Compare:

| Option | Good fit | Price paid |
|---|---|---|
| Simple key/value or one entity/table | Direct key access, distinct ownership/lifecycle | More requests for aggregate reads |
| Single-table, multiple item types | Stable aggregate queries and bounded relationships | Key discipline, denormalization, coupled migration/tooling |
| Multiple DynamoDB tables | Different access/lifecycle/security needs | Application composition; transactions still have service boundaries |
| Relational/search/analytics adjunct | Arbitrary joins, text search, broad reporting | Additional store/sync/freshness and operational cost |

No option earns a cost or latency claim without item-size, request and skew data.
Tables do not enforce a schema for non-key attributes. Application schema/version
validation is still necessary in every SDK or wrapper.

## Worked model: customer orders and a tenant's open-order queue

Assumptions: moderate per-customer/per-tenant peaks, immutable creation time,
tenant-local operations, eventual queue visibility acceptable. These are examples,
not measured sizing. Delimiter-containing IDs must be encoded or rejected
consistently; normalize timestamps to fixed-width UTC for lexical ordering.

| Item | PK | SK | GSI1PK | GSI1SK |
|---|---|---|---|---|
| Customer profile | `TENANT#acme#CUSTOMER#c42` | `PROFILE` | absent | absent |
| Open order | `TENANT#acme#CUSTOMER#c42` | `ORDER#2026-01-02T10:00:00.000Z#o100` | `TENANT#acme#STATUS#OPEN` | `2026-01-02T10:00:00.000Z#o100` |
| Closed order | Same customer PK | Same immutable order SK | absent | absent |

Order attributes: `entityType=Order`, `orderId=o100`, `customerId=c42`,
`status=OPEN`, `version=1`, `totalMinor=1250`, `currency=USD`.
Represent money in explicitly bounded integer minor units or exact decimals;
define scale/currency. Optional `expiresAt` is epoch seconds for disposable records,
not a default retention policy for financial orders. Do not expire orders casually.

### Access paths and actual requests

- **Customer profile**: `GetItem(PK, SK=PROFILE)`; strong read optional.
- **List orders**: `Query(PK = :customer AND begins_with(SK, :orderPrefix))`,
  `:orderPrefix=ORDER#`, descending, `Limit=50`, cursor; strong read optional.
- **Known order key**: `GetItem(PK, complete immutable order SK)`.
- **Open queue**: `Query(IndexName=GSI1, GSI1PK=:tenantOpen)`, descending,
  `Limit=50`; no strong reads. The asset projects keys plus `status`, `orderId`,
  `totalMinor`, `currency`; fetch the base item if more fields or current state are
  required. GSI membership can lag even if subsequent base gets are strong.
- **Order ID only**: unsupported by this layout. Supply customer+creation key,
  introduce a conditional locator item keyed by tenant+order ID, or redesign.
  A locator and order requiring atomic creation may use a transaction. Account for
  its extra item/write/read, Region scope and idempotency.
- **Monthly totals/search across all customers**: not a free query. Define a
  bounded materialized aggregate or export/search/analytics pipeline with explicit
  freshness and reconciliation rather than scanning on each web request.

Single-table co-location here permits a customer aggregate query, but the order
list alone does not prove that merging the profile is better than two tables.

### Sparse index lifecycle

Populate **both** GSI key attributes only for qualifying open orders. Closing an
order must update status/version and remove those keys in the same `UpdateItem`:

```json
{
  "UpdateExpression": "SET #st = :closed, #v = #v + :one REMOVE GSI1PK, GSI1SK",
  "ConditionExpression": "#st = :open AND #v = :expected",
  "ExpressionAttributeNames": {"#st": "status", "#v": "version"},
  "ExpressionAttributeValues": {
    ":closed": {"S": "CLOSED"}, ":open": {"S": "OPEN"},
    ":one": {"N": "1"}, ":expected": {"N": "1"}
  }
}
```

This is a **fragment**: add the real `TableName` and full `Key`, authorize the
tenant, and handle the failed condition as a conflict. Reopening must restore both
keys, matching status and new version in one update. An eventual GSI may still show
the old entry briefly; queue consumers recheck authoritative state conditionally.

## Keys, relationships and indexes

For the full selection sequence, comparison and worked alternatives, read
[LSI vs GSI](lsi-vs-gsi.md). Copyable operations are in [Commands](commands/aws-cli.md).

- Composite sort keys encode hierarchy/ranges. Numbers sort numerically only when
  stored as Number keys; string `2` and `10` sort lexically. Use deliberate padding
  or numeric types, consistent case/encoding, tie-break IDs and immutable keys.
- One-to-many: co-locate bounded child items when read together; paginate large
  collections. Many-to-many: consider two adjacency items/inverted access paths;
  define synchronization/deletion and transaction boundaries. Do not claim joins.
- GSI: distinct partition/sort keys and independently provisioned throughput when
  provisioned mode is used; eventual reads, storage and write amplification.
  `KEYS_ONLY`, `INCLUDE`, `ALL` trade index size versus extra base reads. A request
  projection is not the same as a physically smaller index projection.
- LSI: same partition key, alternative sort key, strong reads possible. Create
  only at table creation; cannot add/delete later. With any LSI, each item
  collection (base+LSI data for one PK) has a 10 GB limit. This is **not** a general
  per-key limit on every table. Avoid adding LSIs for hypothetical future queries.
- GSI non-key uniqueness is not enforced; two items can share an index key.
  Model unique claims under authoritative primary keys with conditions/transactions.
- A compound query not served by an existing key needs an explicit new access path,
  a bounded client composition, or a different datastore—not arbitrary key predicates.

## Skew and cost are part of the model

`GSI1PK=OPEN` is a global hot key. Tenant-prefixing limits scope, not heat: one large
tenant can still dominate. A balanced base table can have an unbalanced index;
index back-pressure can throttle base writes. Monotonic sort keys can keep new
writes at one hot edge. On-demand/adaptive capacity do not remove physical limits.

For a measured hotspot, choose a stable shard count/function plus time buckets
where appropriate. Include:

1. How writers select the shard and how retries preserve it.
2. How readers fan out to all required shards/buckets and merge `(time, id)` order.
3. Per-shard continuation state and any buffered items, bound to a cursor version;
   one `LastEvaluatedKey` cannot represent all shards.
4. Fan-out costs, empty queries, page/deadline budgets, key evolution and backfill.

Estimate base and each affected index separately. A nontransactional 1 KB write
uses one write unit; read unit baseline is 4 KB, strong versus eventual differs,
transactions cost more. Use actual operation-level rounding/aggregation rules from
AWS docs, not `ceil(each item / 4KB)` blindly for Query. Include projected bytes,
storage, replication, backups, streams/consumers and traffic spikes. Fetch current
Region/pricing data before quoting money; label estimates rather than promises.

Sources: [index guidelines](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/bp-indexes-general.html),
[sparse indexes](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/bp-indexes-general-sparse-indexes.html),
[capacity accounting](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/read-write-operations.html),
[GSI back-pressure](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/gsi-throttling.html).
