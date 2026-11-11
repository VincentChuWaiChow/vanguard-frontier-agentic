# Runtime correctness contract

| Surface | Contract / regression |
|---|---|
| Reads | `EVENTUAL` may be stale; `ABSOLUTE` returns current designated-key value at higher RU cost. Do not infer a multi-page transaction snapshot |
| Writes | `put` upserts; prefer absent/present/version conditions when needed. Preserve opaque SDK row version; a false success/conflict is not a retryable timeout |
| Transactions | Java `WriteMultiple` / Node `writeMany` provides same-shard atomic operations; unrelated tables/shards are not cross-shard ACID. Use abort-on-fail for dependent conditions |
| Query | Prepared SQL with declared/bound variables, authenticated tenant predicate; allowlist table/column identifiers, never interpolate user SQL |
| Pagination | SDK continuation is opaque; continue empty pages with a non-null continuation. Do not replace with OFFSET or assume LIMIT is total completeness |
| Public cursor | Store/sign/encrypt server-side cursor envelope bound to tenant, query/parameters, region/table/schema and expiry. No raw cross-tenant cursor replay |
| Retry | Budget elapsed time and total attempts including SDK internal retries; jitter only transient throttling/network/service failures. Do not retry auth/schema failures or blindly repeat ambiguous non-idempotent writes |
| Capacity | RU/WU cost depends on row/index work; absolute reads cost more. Rate-limit callers and hot shard load; on-demand is not unlimited |
| Evolution | Preserve old-reader compatibility and key/type contracts; new primary/shard key requires reviewed new-table migration. Cover deletes, expiry and rollback cutoff |

[Injected pagination helper](../assets/query-pages.mjs) takes `client.query.bind(client)`
and a trusted prepared statement; it executes no cloud call on import. It is not
an authorization layer, exporter or public-cursor codec. SDK objects/version/
continuation types vary by language; round-trip through that SDK's documented
serializer, never stringify a binary token opportunistically.

[SDK query contract](https://oracle.github.io/nosql-node-sdk/interfaces/QueryResult.html),
[write APIs](https://docs.oracle.com/en/cloud/paas/nosql-cloud/pdhxw/index.html),
[consistency](https://docs.oracle.com/en/cloud/paas/nosql-cloud/csnjv/oracle/nosql/driver/Consistency.html), checked 2026-11-08.
