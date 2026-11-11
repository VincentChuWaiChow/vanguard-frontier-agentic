# Runtime correctness contract

| Concern | Required behavior |
|---|---|
| Identity/routing | String `id` and typed partition key; server-derived tenant, parameterized predicates, validate returned tenant |
| Consistency | Account Strong/BoundedStaleness/Session/ConsistentPrefix/Eventual differs from per-request override. Traditional ConsistencyLevel override can weaken, not strengthen. ReadConsistencyStrategy preview (Java >=4.69/.NET >=3.46; Microsoft pages disagree on whether it is direct mode only [CP-E124], so verify per pinned SDK and mode) can offer stronger Latest Committed/Global Strong reads under documented topology constraints; not universal SDK parity. Strong account consistency isn't multi-write-compatible. |
| Session | Reuse singleton SDK client/session cache; transfer explicit session token when causality crosses stateless clients. Tokens are partition-bound opaque state, not a global transaction or authorization token. |
| Optimistic writes | Read `_etag`; conditional replace/patch/delete using If-Match. 412 is stale-state conflict: refetch/reconcile, never retry blindly/convert to unconditional upsert. Create 409 isn't success unless idempotency verifies stored intent. |
| Transactions | Transactional batch: same full logical partition key AND container, <=100 operations, <=2 MB payload, <=5 seconds. Stored procedures also partition-scoped. Bulk operations/cross-partition queries aren't ACID transactions. |
| Pagination | SDK continuation token is opaque; keep same query/parameters/tenant/partition/API/SDK context. Empty page can have continuation; don't stop solely on zero rows. Don't truncate a service page and return its end token (skips items). |
| Query bounds | Positive page size, page/attempt/deadline/RU caps, cancellation. Keep resumption token; no unbounded fetchAll in hot paths. GROUP BY/some DISTINCT and aggregate cross-partition Python queries restrict continuation; newer Python streamable cross-partition queries support it. Verify exact SDK. |
| Retry | SDK already retries 429; honor `x-ms-retry-after-ms` and cumulative deadline/attempt caps. Never stack unbounded app retries. Distinguish 403/auth, 404 routing/missing, 409 conflict, 412 precondition and 429 throttling. |
| Ambiguous writes | Timeout/connection loss can follow commit. Idempotency key and read/conditional reconciliation before retry; retrying arbitrary increment/create may duplicate effects. Don't retry unknown outcome as a read. |
| Diagnostics | Record charge, activity ID, retry/substatus and elapsed time without items/tokens/secrets. Missing charges aren't free requests. |

The injected [runtime example](../assets/runtime.mjs) uses actual JavaScript
`Container.items.query(...).fetchNext()` and `.item(id, pk).replace(...)` signatures,
but creates no client, installs no package and runs nothing on import. Tests use
fakes: behavior-contract evidence, not SDK compilation or Cosmos acceptance.

## TTL / migration

Container TTL absent/null means off; `-1` enables item-level overrides without
default expiry; positive seconds set default. Item `ttl: -1` opts out; positive
override controls expiry relative to server `_ts` (last modification). TTL applies
only when enabled at container. Expired data stops appearing in queries; physical
deletion is asynchronous and consumes available resources, not a deadline SLA.
TTL isn't a legal-hold/backup-retention mechanism. Do not claim changing TTL
recovers deleted items. Restores can replay TTL and immediately delete old data;
review disable-TTL restore option before recovery.

Schema/model changes need versioned readers, backfill checkpoints/idempotency,
deletion/tombstone reconciliation, bounded RU/cost, dual-write divergence tests,
domain invariant comparison, approved cutover and preserved rollback source.
Latest-version change feed alone doesn't capture delete/TTL events; don't claim
it reproduces an exact snapshot without another mechanism.

Sources: [consistency](https://learn.microsoft.com/azure/cosmos-db/consistency-levels),
[session tokens](https://learn.microsoft.com/azure/cosmos-db/how-to-manage-consistency),
[transactions/ETag](https://learn.microsoft.com/azure/cosmos-db/database-transactions-optimistic-concurrency),
[batch limits](https://learn.microsoft.com/azure/cosmos-db/transactional-batch),
[pagination](https://learn.microsoft.com/cosmos-db/query/pagination),
[TTL](https://learn.microsoft.com/azure/cosmos-db/time-to-live).
