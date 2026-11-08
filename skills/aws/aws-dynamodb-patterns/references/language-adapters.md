# Language adapters: one service contract, many SDKs

## Shared contract

Use the project's installed SDK version and existing client lifecycle. Resolve its
current primary docs through Context7 before generating version-sensitive code;
do not install/upgrade as an implicit side effect. The examples below are small
integration excerpts, not full runnable projects or a promise of compilation in
every environment. No credentials are embedded; use workload identity/default
credential providers and explicit approved Region. Public tenant/cursor validation
happens before the internal SDK call.

[query-input.json](../assets/query-input.json) is the **low-level AttributeValue**
request for the base-table order list. Replace the table explicitly. Each example
returns one service page unless it shows a loop; never mistake that for all items.
The language-independent loop is:

```text
cursor = validated continuation, or absent
repeat within deadline and page budget:
    page = Query(same table/index/key condition/order, ExclusiveStartKey=cursor)
    process page.Items even if empty
    cursor = page.LastEvaluatedKey
    if cursor absent or empty: finish
return next cursor if API budget ended, or report incomplete stream
```

Do not convert an empty page into end-of-list. Do not combine native-value document
clients with low-level `{S: ...}`/`{N: ...}` maps. All adapters must retain conditions,
error classification, precision, cancellation and operation identity—not only keys.

When filling a bounded HTTP match page, request at most the number of matches
still needed, or buffer surplus returned matches. Do not slice a response and
advance to its LastEvaluatedKey: that loses the matches you discarded. The
`matchingPage` export in the JS asset implements this remaining-count rule.

## JavaScript (AWS SDK v3)

Packages: `@aws-sdk/client-dynamodb` for low-level maps; optional
`@aws-sdk/lib-dynamodb` for native-value document commands. This version uses
**low-level** maps, matching the JSON asset and other languages:

```javascript
import { DynamoDBClient, QueryCommand } from "@aws-sdk/client-dynamodb";
import { queryPages } from "./assets/query-pages.mjs";

// Within an async application scope. client lifetime belongs to the application.
const client = new DynamoDBClient({ region: approvedRegion });
const input = {
  TableName: approvedTable,
  KeyConditionExpression: "#pk = :pk AND begins_with(#sk, :prefix)",
  ExpressionAttributeNames: { "#pk": "PK", "#sk": "SK" },
  ExpressionAttributeValues: {
    ":pk": { S: authorizedCustomerPK }, ":prefix": { S: "ORDER#" }
  },
  ConsistentRead: true, ScanIndexForward: false, Limit: 50
};
for await (const page of queryPages(
  request => client.send(new QueryCommand(request)), input, { maxPages: 20 }
)) {
  await consume(page.Items ?? []); // may be empty; queryPages still continues
}
```

This exhausts up to a page budget; a budget failure reports incomplete results.
For an HTTP page endpoint, return a signed/stored next cursor instead of draining
all pages. Pass cancellation/deadline through the application's SDK send adapter.

## TypeScript (AWS SDK v3)

Use the same JS algorithm; add SDK types instead of casting unknown JSON into
domain objects. Here is a typed **one-page** boundary:

```typescript
import { DynamoDBClient, QueryCommand } from "@aws-sdk/client-dynamodb";
import type { QueryCommandInput, QueryCommandOutput } from "@aws-sdk/client-dynamodb";

export async function queryOrderPage(
  client: DynamoDBClient, input: QueryCommandInput
): Promise<QueryCommandOutput> {
  return client.send(new QueryCommand(input));
}
```

Validate decoded items into a domain schema. For `DynamoDBDocumentClient` use
commands/types from `@aws-sdk/lib-dynamodb` and native values (`":pk": pk`, **not**
`{S: pk}`). Choose marshalling rules for undefined/null, sets, binary and big/decimal
numbers. JS `number` cannot represent every DynamoDB Number exactly; configure
lossless wrappers or explicit numeric strings/decimals and validate at boundaries.

## Java (AWS SDK for Java 2.x)

Dependency: `software.amazon.awssdk:dynamodb` managed by the project's pinned BOM.
Do not confuse legacy `com.amazonaws.services.dynamodbv2` with Java SDK 2.x.
The excerpt uses Java 9+ `Map.of`; on Java 8 populate maps explicitly instead.
Caller supplies a reusable `DynamoDbClient`, approved table, authorized PK and
validated low-level cursor. Import `java.util.Map` and
`software.amazon.awssdk.services.dynamodb.model.*`:

```java
static QueryResponse orderPage(
    DynamoDbClient client, String table, String pk,
    Map<String, AttributeValue> cursor) {
  QueryRequest.Builder req = QueryRequest.builder()
      .tableName(table)
      .keyConditionExpression("#pk = :pk AND begins_with(#sk, :prefix)")
      .expressionAttributeNames(Map.of("#pk", "PK", "#sk", "SK"))
      .expressionAttributeValues(Map.of(
          ":pk", AttributeValue.builder().s(pk).build(),
          ":prefix", AttributeValue.builder().s("ORDER#").build()))
      .consistentRead(true).scanIndexForward(false).limit(50);
  if (cursor != null && !cursor.isEmpty()) req.exclusiveStartKey(cursor);
  return client.query(req.build());
}
```

Also import `software.amazon.awssdk.services.dynamodb.DynamoDbClient`.
Continue using `response.lastEvaluatedKey()`; alternatively use `queryPaginator`
with explicit application work/deadline bounds. Enhanced Client is optional; its
bean mapping is not the low-level API. Use `BigDecimal`/exact mappings for numbers.

## Kotlin (AWS SDK for Kotlin)

Dependency: `aws.sdk.kotlin:dynamodb`, version pinned by the project. Use the native
Kotlin suspend API or deliberately reuse Java 2.x; do not mix their model types.

```kotlin
import aws.sdk.kotlin.services.dynamodb.DynamoDbClient
import aws.sdk.kotlin.services.dynamodb.model.AttributeValue
import aws.sdk.kotlin.services.dynamodb.model.QueryRequest
import aws.sdk.kotlin.services.dynamodb.model.QueryResponse

suspend fun orderPage(
    client: DynamoDbClient, table: String, pk: String,
    cursor: Map<String, AttributeValue>? = null
): QueryResponse = client.query(QueryRequest {
    tableName = table
    keyConditionExpression = "#pk = :pk AND begins_with(#sk, :prefix)"
    expressionAttributeNames = mapOf("#pk" to "PK", "#sk" to "SK")
    expressionAttributeValues = mapOf(
        ":pk" to AttributeValue.S(pk), ":prefix" to AttributeValue.S("ORDER#")
    )
    consistentRead = true
    scanIndexForward = false
    limit = 50
    if (!cursor.isNullOrEmpty()) exclusiveStartKey = cursor
})
```

Continue while `!response.lastEvaluatedKey.isNullOrEmpty()`; propagate coroutine
cancellation, close the client at its owning lifecycle boundary, and avoid blocking
Java clients on coroutine event loops. SDK Number representations need exact parsing.

## Rust (AWS SDK for Rust)

Dependency: project-pinned `aws-sdk-dynamodb` and a compatible `aws-config`/runtime.
Accept a configured client and preserve typed `AttributeValue`; this async excerpt
returns one page and propagates an error instead of panicking:

```rust
use aws_sdk_dynamodb::{Client, types::AttributeValue};
use aws_sdk_dynamodb::operation::query::QueryOutput;
use std::collections::HashMap;

async fn order_page(
    client: &Client, table: &str, pk: &str,
    cursor: Option<HashMap<String, AttributeValue>>,
) -> Result<QueryOutput, Box<dyn std::error::Error + Send + Sync>> {
    let page = client.query()
        .table_name(table)
        .key_condition_expression("#pk = :pk AND begins_with(#sk, :prefix)")
        .expression_attribute_names("#pk", "PK")
        .expression_attribute_names("#sk", "SK")
        .expression_attribute_values(":pk", AttributeValue::S(pk.into()))
        .expression_attribute_values(":prefix", AttributeValue::S("ORDER#".into()))
        .consistent_read(true).scan_index_forward(false).limit(50)
        .set_exclusive_start_key(cursor.filter(|key| !key.is_empty()))
        .send().await?;
    Ok(page)
}
```

Continue with nonempty `last_evaluated_key`; do not assume items or keys are always
present. Preserve `AttributeValue::N(String)` exactly and handle binary explicitly.
The boxed error keeps the excerpt small; production code should inspect SDK service
errors and preserve retry/condition/unknown-outcome distinctions.

## PHP (AWS SDK for PHP v3)

Dependency: `aws/aws-sdk-php` with the project's lockfile. Caller supplies
`Aws\DynamoDb\DynamoDbClient`; low-level values are typed arrays:

```php
function orderPage(\Aws\DynamoDb\DynamoDbClient $client, string $table,
                   string $pk, array $cursor = []): \Aws\Result {
    $request = [
        'TableName' => $table,
        'KeyConditionExpression' => '#pk = :pk AND begins_with(#sk, :prefix)',
        'ExpressionAttributeNames' => ['#pk' => 'PK', '#sk' => 'SK'],
        'ExpressionAttributeValues' => [
            ':pk' => ['S' => $pk], ':prefix' => ['S' => 'ORDER#']
        ],
        'ConsistentRead' => true, 'ScanIndexForward' => false, 'Limit' => 50
    ];
    if ($cursor !== []) $request['ExclusiveStartKey'] = $cursor;
    return $client->query($request);
}
```

Continue with `$page['LastEvaluatedKey'] ?? []` until empty; do not stop on an empty
Items array. `Aws\DynamoDb\Marshaler` is optional: do not double-marshal or turn
exact Number strings into floats/overflowing platform integers. Catch typed SDK
exceptions at the application boundary, not by swallowing all failures as empty lists.

## Python, Go, C#, Ruby, C++ and other languages

Language-neutral support means preserving the service contract, not translating
method names by intuition. For any other language:

1. Resolve its supported AWS SDK and installed version through Context7/current
   primary docs. Prefer the native official SDK; if absent, disclose that gap and
   use a documented compatible client/SigV4 implementation, not handwritten auth.
2. Map `Query`, typed values, expression aliases, cursor and service errors using
   that client's actual API. For example, boto3's low-level client uses typed maps;
   its resource layer uses native values and `Decimal`, not Python floats.
3. Reuse the same key fixtures, pagination and concurrent-write tests. Match number,
   binary/set/null/absent encoding. For Go/C# inspect null/empty cursor behavior;
   don't assume it matches JS. Verify paginator laziness, cancellation and retries.
4. Compile/test in the target dependency lockfile/runtime. If unavailable, label
   examples uncompiled rather than advertising universal runtime compatibility.

## Optional DynamoDB Toolbox (JS/TS only)

The supplied inspiration explicitly targets **Toolbox v2**. Toolbox is a JS/TS
query/schema layer over AWS SDK clients, not an AWS service feature or a Java,
Kotlin, Rust or PHP library. Do not prescribe it to other languages.

Inspect `package.json` and lockfile first; resolve that version's primary docs
through Context7. Check import paths, schema `item`/other constructors, `Entity`,
`Table`, `.build()` command API, hidden physical keys, pagination and marshalling
before writing code. Do not mix v1 `attributes`/`schema` examples with v2 (or assume
current unversioned docs equal v2). First demonstrate equivalent low-level requests,
then add a wrapper only when the user/project wants it. Wrapper schema validation
does not create database uniqueness, strong GSI reads or new transaction guarantees.

Primary references: [JS v3](https://docs.aws.amazon.com/sdk-for-javascript/v3/developer-guide/javascript_dynamodb_code_examples.html),
[Java 2.x](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/ProgrammingWithJava.html),
[Kotlin](https://docs.aws.amazon.com/sdk-for-kotlin/latest/developer-guide/kotlin_dynamodb_code_examples.html),
[Rust](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/rust_dynamodb_code_examples.html),
[PHP v3](https://docs.aws.amazon.com/sdk-for-php/v3/developer-guide/php_dynamodb_code_examples.html),
[Toolbox](https://www.dynamodbtoolbox.com/).
