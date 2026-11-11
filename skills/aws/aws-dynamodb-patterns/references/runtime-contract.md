# Runtime contract and failure tests

## Query interface, not a leaked database client

Prefer domain methods such as `listCustomerOrders(authContext, customerId,
pageSize, cursor)` and `closeOrder(authContext, orderKey, expectedVersion,
operationId)`. Do not accept arbitrary table/index names or raw expression strings
from public callers. Derive tenant keys from verified authorization and verify
customer/order ownership. A tenant prefix is a modeling convention, not an access
control system; use appropriate IAM table/index scope and application enforcement.

Return `{items, nextCursor}` with documented maximum evaluated-page size,
ordering, freshness and empty-page behavior. Collecting up to 50 **matching**
items may need multiple DynamoDB requests; bound that work and return a cursor
even for a short/empty page. Do not promise a snapshot across pages under writes.

Prevent a subtler pagination data-loss bug: if only 10 matches remain to fill an
HTTP page, requesting another 50 evaluated items may return 30 matches. Taking
only 10 and using that response's LastEvaluatedKey discards the other 20 forever.
Set `Limit = min(perRequestCap, targetMatches - collectedMatches)` on every call;
returned matches cannot exceed evaluated items, so no surplus is discarded. Or
retain surplus matches in authenticated server-side cursor state. A prose caveat
does not repair code that still slices/discards. The local `matchingPage` example
and its tests cover this boundary explicitly.

Cursor envelope: authenticated tenant + operation + normalized query/index/order
+ schema version + expiry + typed `LastEvaluatedKey` (or per-shard state). Sign an
opaque envelope or keep it server-side. Validate size, types and binding before
passing it to the SDK. Base64 encoding alone is not integrity, confidentiality or
authorization. Encrypt or store server-side when key disclosure is sensitive.

`ExclusiveStartKey` is a typed key map, not a page number or SQL offset. Preserve
binary encoding and exact number strings across JSON boundaries. Service cursors
and AWS CLI `--starting-token`/`NextToken` are not interchangeable.

## Reads and pagination

- `Query`: equality on one partition key plus supported sort-key condition;
  request expressions use attribute-name aliases for reserved/special names.
- Service reads at most 1 MB or `Limit` evaluated items, then filters. An empty
  `Items` plus nonempty `LastEvaluatedKey` is not completion. Continue until absent
  or empty key, a caller cancellation, or an explicit page/deadline budget.
- `FilterExpression` and `ProjectionExpression` do not reduce capacity for items
  already read. Limit response size separately from database work.
- Table/LSI reads may be strong; GSI and Streams reads are eventual only.
  Strong reads do not turn a read-then-write into an atomic operation.
- `BatchGetItem`: at most 100 items, up to 16 MB per call; partial responses can
  occur earlier. Retry only `UnprocessedKeys` with a bound. Results are unordered;
  include keys and reconstruct caller order; handle missing items explicitly.
- `BatchWriteItem`: at most 25 puts/deletes per call, not updates/conditions;
  per-item atomic, not whole-batch atomic. Retry only `UnprocessedItems`, not the
  whole original batch as if it had all failed. Size and per-item limits also apply.

## Writes: one authority for each invariant

Create an item without overwriting an existing full key:

```json
{
  "ConditionExpression": "attribute_not_exists(PK)",
  "ReturnValuesOnConditionCheckFailure": "ALL_OLD"
}
```

This is a `PutItem` fragment, not a complete request. The condition is evaluated
against the item at the complete primary key; it does **not** enforce uniqueness
of a partition or arbitrary attribute across other items. Returning old data can
expose sensitive fields: omit it or sanitize responses/logs when unnecessary.

- Use `version=:expected` conditional updates to prevent lost writes. Distinguish
  `ConditionalCheckFailedException` from retriable throttling/service failures.
  Never retry a stale version indefinitely or discard the condition to succeed.
- Use `TransactWriteItems` for all-or-nothing multi-item changes when supported:
  up to 100 actions/4 MB, one account and Region, distinct target items. Put the
  condition on the same `Update` action; do not add a separate `ConditionCheck`
  against that same item. Transactions can span tables, not indexes/Regions.
- `ClientRequestToken` deduplicates identical transaction requests within its
  10-minute window. It is not permanent business idempotency. Retain an operation
  record/request hash for the required business window; resolve conflicting reuse.
- A timed-out write may have committed. Reconcile authoritative state or use a
  durable operation identity; blind retry of `ADD`/increment/decrement can duplicate
  a side effect. External payment/email is not atomic with DynamoDB: use an outbox
  or equivalent workflow with deduplicated consumers.
- Bound retries and total deadlines; use SDK retry policy where suitable, add
  application retry for unprocessed batch entries, avoid nested retry storms.
  Surface exhausted budgets and unknown outcomes rather than pretending success.

## Expiry, regions and change safety

- TTL is eventual cleanup, usually within days, using a Number of epoch **seconds**.
  Expired items can still be read/updated. Enforce session/lease/business expiration
  in application reads and write conditions. An expiry filter is a justified
  residual check, not an efficient alternate index or capacity optimization.
- Do not generalize all Global Tables as eventual. MREC replicates asynchronously;
  strong reads at a replica do not guarantee another Region's latest write, and a
  transaction is not replicated atomically as a group. MRSC has different topology,
  feature and availability constraints: verify current docs before recommending it.
  As of the cited research date, MRSC requires three Regions (three replicas or
  two replicas plus a witness) and does not support TTL, LSIs or transaction APIs.
  It is not a drop-in destination for this package's TTL/transaction workflow.
  Recheck those constraints/availability before implementation. GSI reads remain
  eventual in either mode.
- PITR recovery creates a new table; test restore, permissions, index/TTL settings
  and application endpoint cutover. Retain/deletion protection is not backup or
  an independently tested rollback.
- Existing table/index changes: inspect current IaC and drift, estimate backfill
  load/quotas/cost, create an additive plan, verify ACTIVE status/index parity,
  shadow reads and migration checkpoints, then stage cutover. Rollback keeps the
  old path and handles writes made during migration. Do not remove it prematurely.
- Table key changes typically require a new table/data migration. Fixed table
  names can block replacement; inspect the CloudFormation change set, retention
  and deletion protection. Never disable preservation settings to force a deploy.

## Operational acceptance

Use least-privilege workload identities/default credential chains, not embedded
keys. Scope both table and required index ARNs; transactions/backup/KMS need their
own reviewed permissions. Choose encryption/key ownership deliberately. Avoid raw
item/key/cursor logging. Measure consumed capacity, throttling reasons (table vs
GSI/key range), latency, errors, retries, item sizes and skew; use CloudWatch and
Contributor Insights where authorized. DAX/cache is optional: establish supported
SDKs, staleness/invalidation and cache-miss behavior before adding it.

| Adversarial stimulus | Required outcome |
|---|---|
| Empty filtered page with cursor, then matching item | Continue; return later item |
| 1 MB response before requested limit | Continue by key; no exact-count assumption |
| Two creates of same full primary key | Exactly one conditional create succeeds |
| Concurrent close with stale version | Conflict surfaced; no unconditional overwrite |
| GSI has not caught up | No immediate-discovery/uniqueness guarantee |
| One tenant dominates a status GSI | Load evidence or explicit sharding/redesign plan |
| Timeout after decrement commit | No duplicate decrement on blind retry |
| Partially processed batch | Retry unprocessed subset only, bounded |
| TTL item expired but still stored | Business access denied/expired regardless of deletion |
| Cursor copied across tenant/query | Reject before SDK call |

Sources: [Query API](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_Query.html),
[transactions](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/transaction-apis.html),
[TTL](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/TTL.html),
[global tables](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/V2globaltables_HowItWorks.html).
