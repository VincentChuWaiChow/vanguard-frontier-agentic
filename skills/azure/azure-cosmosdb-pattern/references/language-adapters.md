# Language adapters — load the matching section only

Keep the existing SDK/version and credential flow. No install is required by the
skill; SDK snippets below are illustrative and **not compiled against installed
SDKs**. The injected JS example is tested with fakes, not Azure. Do not claim every
SDK supports every vector/change-feed/Entra/continuation feature identically.

| Language | First-party path checked 2026-11-08 | Adapter obligations |
|---|---|---|
| JavaScript / TypeScript | `@azure/cosmos`, `@azure/identity` | Singleton CosmosClient, server credential, partitionKey, parameterized query, fetchNext token/session/charge, ETag |
| Java | `com.azure:azure-cosmos` v4 | CosmosAsyncClient/Container, PartitionKey, CosmosQueryRequestOptions, paged Flux/continuation and diagnostics |
| Kotlin/JVM | Existing Java v4 SDK via JVM interop | No separate Kotlin-native NoSQL SDK claim; adapt Reactor/coroutines/cancellation deliberately |
| Rust | Microsoft `azure_data_cosmos` + `azure_identity`, **public preview** | Current official quickstart warns no SLA/not recommended for production; verify pinned API/feature support, don't import community SDK as first-party GA |
| PHP | No first-party NoSQL PHP SDK identified in retrieved official SDK matrix | NoSQL HTTPS REST adapter or explicitly reviewed community package; no fake official PHP class names |
| Python | `azure-cosmos`, `azure-identity` | `query_items` with `partition_key` [CP-E174], response hooks/ETags; newer streamable cross-partition continuation supported, aggregate cross-partition restrictions remain; verify pinned version |
| .NET | `Microsoft.Azure.Cosmos` v3 | CosmosClient/Container, FeedIterator, PartitionKey, IfMatchEtag/session token, TransactionalBatch |
| Go | `github.com/Azure/azure-sdk-for-go/sdk/data/azcosmos` | Explicit partition keys, pager/cancellation/charges and SDK-specific retry behavior |
| Other | Existing supported SDK or NoSQL REST | Port the same correctness contract; no universal feature/compilation claim |

## JS / TS

Use [runtime.mjs](../assets/runtime.mjs) with an already-authorized injected
`Container`; it doesn't construct a client or call on import. Actual app setup
uses `new CosmosClient({endpoint, aadCredentials: credential})` with an existing
approved Entra credential, database/container selected explicitly. No keys/logging.

```javascript
// Illustrative actual SDK transactional batch; same tenant/full partition only.
const result = await container.items.batch([
  {operationType: "Create", resourceBody: order},
  {operationType: "Create", resourceBody: outbox}
], authenticatedTenant);
// Verify batch result and individual errors; bulk != transactional batch.
```

## Java

```java
CosmosQueryRequestOptions options = new CosmosQueryRequestOptions()
    .setPartitionKey(new PartitionKey(authenticatedTenant));
SqlQuerySpec query = new SqlQuerySpec(
    "SELECT * FROM c WHERE c.tenantId=@tenant",
    List.of(new SqlParameter("@tenant", authenticatedTenant)));
container.queryItems(query, options, Order.class)
    .byPage(continuationToken, 50); // subscribe/consume with bounds/cancellation
// replaceItem(..., new CosmosItemRequestOptions().setIfMatchETag(etag)) for OCC.
```

## Kotlin/JVM

```kotlin
val options = CosmosQueryRequestOptions().setPartitionKey(PartitionKey(tenant))
val query = SqlQuerySpec("SELECT * FROM c WHERE c.tenantId=@tenant",
    listOf(SqlParameter("@tenant", tenant)))
val pages = container.queryItems(query, options, Order::class.java).byPage(cursor, 50)
// Explicit bounded Reactor/coroutine consumption; don't block an event loop.
```

## Rust — official preview, not GA

```rust
// ContainerClient injected; snippet follows official quickstart's preview API.
let response = container.read_item(tenant, order_id, None).await?;
// query_items::<Order>(Query::from(...).with_parameter(...)? , tenant, None)?
// Consume pager with cancellation/charge/continuation policy for pinned crate.
```

## PHP / other REST

Use approved Entra token provider; Cosmos data-plane REST authorization value is
URL-encoded `type=aad&ver=1.0&sig=<token>`. Never dump tokens or implement homemade
credential acquisition. Token scope/audience/cloud endpoint must match service.
HTTPS POST `/dbs/ordersdb/colls/orders/docs` with these request values:

```php
$body = json_encode([
  'query' => 'SELECT * FROM c WHERE c.tenantId=@tenant',
  'parameters' => [['name' => '@tenant', 'value' => $authenticatedTenant]]
], JSON_THROW_ON_ERROR);
$headers = [
  'Authorization: ' . $alreadyEncodedCosmosEntraAuthorization,
  'x-ms-version: 2018-12-31',
  'x-ms-date: ' . gmdate('D, d M Y H:i:s') . ' GMT',
  'x-ms-documentdb-isquery: True',
  'x-ms-documentdb-partitionkey: ' . json_encode([$authenticatedTenant], JSON_THROW_ON_ERROR),
  'x-ms-max-item-count: 50',
  'Content-Type: application/query+json'
];
// Inject existing HTTP client; add opaque x-ms-continuation/session-token when
// resuming, then capture response continuation/session/charge/activity-id/status.
```

Do not let client input choose tenant/endpoint/container. Parameterize all values;
URL-encode resource IDs safely, preserve partition JSON types and use If-Match for
conditional writes. Bound time/retries/pages/RUs; an empty page can continue. A
REST example isn't permission to send requests or a tested PHP SDK integration.

## Portable adapter contract

`read(id, typedPartition, session, deadline)`; `query(query+parameters, partition,
opaqueCursor, pageSize, session, budget)` → items + continuation + session + RU +
activityID; `replace(id, partition, body, expectedEtag, deadline)`; `batch(fullKey,
operations)`. Preserve 404/409/412/429/unknown-write distinctions, tenant binding,
finite numeric/vector/schema values, safe cancellation and replay/idempotency.
No missing RU as zero, partial-page cursor skipping or arbitrary retry of writes.

Sources: [SDKs](https://learn.microsoft.com/azure/cosmos-db/nosql/sdk-nodejs),
[Rust preview](https://learn.microsoft.com/azure/cosmos-db/quickstart-rust),
[Go](https://learn.microsoft.com/azure/cosmos-db/quickstart-go),
[Java query API](https://learn.microsoft.com/java/api/com.azure.cosmos.cosmosasynccontainer),
[REST query](https://learn.microsoft.com/rest/api/cosmos-db/query-documents),
[REST authorization](https://learn.microsoft.com/rest/api/cosmos-db/access-control-on-cosmosdb-resources).
