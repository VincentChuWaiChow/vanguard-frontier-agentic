# Portable application contract and language adapters

Load only the requested language section. All SDK excerpts below are illustrative,
uncompiled and unexecuted. No dependency installation is implied. The pure injected
[JS helper](../assets/query-pages.mjs) alone has executable fake tests.

| Language | Verified first-party path / boundary |
|---|---|
| JavaScript/TypeScript | Oracle `oracle-nosqldb` native NoSQL Node SDK, not the generic OCI SDK |
| Java | `oracle.nosql.driver` native SDK; manage a long-lived configured handle |
| Rust | Oracle `oracle-nosql-rust-sdk`; async API exists; do not label unsupported |
| Kotlin | Java SDK interop, not a separately claimed Oracle Kotlin NoSQL SDK |
| PHP | No native PHP NoSQL SDK established by reviewed Oracle SDK catalog; use a reviewed service bridge or OCI-signed REST adapter, not Oracle SQL/OCI8 |
| Python/Go/.NET | Native NoSQL SDKs listed in official catalog; preserve their own cursor/type/async APIs |
| Other | Verify current official SDK or implement approved signed REST contract; no invented package |

Common port: `get(fullKey, consistency) -> row, opaqueVersion, expiration`; insert-only/
match-version write -> explicit success/conflict; query prepared+bound variables ->
rows, opaque continuation, consumed units; same-shard batch with abort semantics.
Adapters must enforce tenant/table/region, lossless numbers/binary/timestamps, total
request/retry/time budgets and no data logging. Map SDK-native versions/tokens;
REST `opc-next-page` is not interchangeable with SDK continuation state. Generic
OCI REST has fewer SQL features than native NoSQL SDKs; verify query support.

## JS / TypeScript

Existing configured client and authenticated tenant/approved compartment; identifiers
are fixed and values are bound. Do not share mutable prepared bindings across
concurrent tenants. Choose explicit consistency and read/page caps.

```javascript
import { Consistency } from 'oracle-nosqldb';
import { queryPages } from './query-pages.mjs';
const sql = 'DECLARE $tenant STRING; SELECT o.tenantId,o.orderId FROM orders_v1 o WHERE o.tenantId=$tenant';
const prepared = await client.prepare(sql, {compartment: compartmentId});
prepared.set('$tenant', authenticatedTenant);
for await (const page of queryPages(client.query.bind(client), prepared, {
  maxPages: 20, maxReadKB: 16,
  queryOptions: {compartment: compartmentId, consistency: Consistency.ABSOLUTE}
})) {
  consumeAuthorizedRows(page.rows); // no console logging of customer rows
}
```

TypeScript: type the returned projection and check boundary data at runtime; a
`query<OrderProjection>` type parameter cannot authorize tenants or validate schema.
Versioned update: get with full key, then `client.putIfVersion(table,row,result.version,opts)`;
check `success`; never treat condition failure as automatically retryable.

## Java

Prepared/bound statement below is supplied by the trusted repository layer; never
accept arbitrary SQL or a client-provided tenant variable. Own request per iterator.

```java
QueryRequest request = new QueryRequest()
    .setPreparedStatement(boundStatement)
    .setCompartment(compartmentId)
    .setConsistency(Consistency.ABSOLUTE)
    .setMaxReadKB(16);
try {
    int pages = 0;
    do {
        if (++pages > 20) throw new IllegalStateException("incomplete page budget");
        QueryResult page = handle.query(request);
        consumeAuthorizedRows(page.getResults());
    } while (!request.isDone());
} finally { request.close(); }
```

## Kotlin via Java SDK

`boundStatement` and `compartmentId` have the same trusted provenance as Java;
Java request setters return the request. Coordinate blocking calls with the
application's dispatcher/timeouts, not an accidental unbounded request coroutine.

```kotlin
val request = QueryRequest().setPreparedStatement(boundStatement)
    .setCompartment(compartmentId).setConsistency(Consistency.ABSOLUTE)
    .setMaxReadKB(16)
try {
    var pages = 0
    do {
        check(++pages <= 20) { "incomplete page budget" }
        consumeAuthorizedRows(handle.query(request).results)
    } while (!request.isDone)
} finally { request.close() }
```

## Rust native SDK

PreparedStatement comes from `prepare_only` against a trusted fixed statement;
set the authenticated tenant variable on this request, not shared mutable state.
`execute` collects all results; use reviewed bounded batching for large queries.

```rust
use oracle_nosql_rust_sdk::{QueryRequest, Consistency};
let mut request = QueryRequest::new_prepared(&prepared)
    .compartment_id(&compartment_id)
    .consistency(&Consistency::Absolute)
    .max_read_kb(16);
request.set_variable("$tenant", &authenticated_tenant)?;
let mut pages = 0;
while !request.is_done() {
    pages += 1;
    if pages > 20 { return Err("incomplete page budget".into()); }
    let mut rows = Vec::new();
    request.execute_batch(&handle, &mut rows).await?;
    consume_authorized_rows(rows)?;
}
```

Rust SDK 0.1.4 documentation exposes `execute_batch`; its docs say batching may be
deprecated, so re-check pinned release. This is not all-version compilation proof.

## PHP / signed REST bridge

The following is an application interface, NOT a fictional Oracle SDK class.
A separately reviewed implementation uses OCI request signing/native service bridge
with pre-authorized prepared statement and query parameters. Never hand-copy keys
into source. Request signer/credential/network tests remain required.

```php
interface NoSqlQueryPort {
    // Contract: rows + nullable opaque next; no credentials/tenant from public input.
    public function page(array $approvedRequest): array;
}
function collect(NoSqlQueryPort $port, array $request, int $maxPages = 20): array {
    if ($maxPages < 1) throw new InvalidArgumentException('positive budget');
    $out = [];
    for ($i = 0; $i < $maxPages; $i++) {
        $page = $port->page($request);
        foreach ($page['rows'] as $row) $out[] = $row;
        $next = $page['next'] ?? null;
        if ($next === null) return $out;
        $request['page'] = $next;
    }
    throw new RuntimeException('incomplete page budget');
}
```

Signed REST uses NoSQL service `Query`/`PrepareStatement`, reviewed endpoint/API
version, compartment context, prepared Base64 statement, `isPrepared`, bound
`variables`, consistency and max read KB. Preserve request ID/errors and response
`opc-next-page`; enforce timeout/retry limits and tenant cursor binding outside the
port. Do not confuse this JSON REST path with the native SDK binary protocol.

## Python / Go / .NET / other

Use the existing official NoSQL SDK, not a default generic ORM. Python preserves
QueryRequest continuation until done; Go follows its documented Query loop; .NET
QueryAsyncEnumerable manages continuation. In each case test an empty intermediate
page, bounded completion, partial-result failure, version conflict, cancellation,
number/TTL round trips and same-shard rejection before claiming compatibility.

[Oracle SDK catalog](https://docs.oracle.com/en/cloud/paas/nosql-cloud/dtddt/plan-your-service.html),
[Node SDK](https://oracle.github.io/nosql-node-sdk/),
[Java QueryRequest](https://docs.oracle.com/en/cloud/paas/nosql-cloud/csnjv/oracle/nosql/driver/ops/QueryRequest.html),
[Rust QueryRequest](https://docs.rs/oracle-nosql-rust-sdk/latest/oracle_nosql_rust_sdk/struct.QueryRequest.html),
[REST query model](https://docs.oracle.com/en-us/iaas/tools/python/latest/api/nosql/models/oci.nosql.models.QueryDetails.html), checked 2026-11-08.
