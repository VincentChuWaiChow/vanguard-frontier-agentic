# JSON types and native vector search

NoSQL API uses JSON: string, number, boolean, null, array and object; missing
(`undefined` in query semantics) differs from null. No DynamoDB AttributeValue
tags, native sets, binary/date type or exact arbitrary-precision decimal contract.
Represent timestamps with explicit canonical UTC strings and money/large integers
using a reviewed scaled/string representation where IEEE-754 round-trip precision
would fail. `id` must be a string. Validate finite numeric values and size limits
(ordinary item maximum 2 MB); store binary/media in Blob Storage with pinned refs.

## Current native capability (checked 2026-11-08)

NoSQL **does support native vector indexing and `VectorDistance`**; don't route
every vector request to AI Search or copy MongoDB vector syntax. Feature/account
availability must be verified before provision. A vector embedding policy defines
concrete path, dataType, dimensions and distanceFunction; vector index policy uses
the same concrete path. Nested array/wildcard vector paths aren't supported.

| Setting | Documented values / constraints |
|---|---|
| Vector element types | Current vector page: `float32`, `float16`, `int8`, `uint8`; older pinned ARM 2025-04-15 enum lists float32/int8/uint8 only. Use float32 in supplied asset; don't claim float16 works through every SDK/API version. |
| Numeric meaning | Ordered coordinates, not base64/URI/text; finite numbers; integer vectors must fit int8 -128..127 or uint8 0..255, not just cast arbitrary floats. float16/float32 precision is policy-specific. |
| Dimensions | One fixed positive dimension count per path; max 505 for flat, 4096 for quantizedFlat/diskANN. Model output must match exactly. |
| Distance | `cosine`, `dotproduct`, `euclidean`; model normalization/metric must agree |
| Index | `flat` exact brute force (<=505); `quantizedFlat` quantized brute force; `diskANN` approximate. Last two need >=1000 indexed vectors or fall back to full scan (RU risk). |
| Capacity | Current vector docs exclude shared-throughput accounts; select dedicated container throughput, not database shared throughput |
| Lifecycle | Cannot generically disable vector indexing/search once enabled. Mutability docs vary: NoSQL vector/index pages and unified Fabric-flavored policy guidance differ about drop/re-add. Verify exact API/region/SDK; conservative example uses new container/generation, not a universal immutable assertion. |

The [vector policy](../assets/vector-policy.json) and
[vector index](../assets/vector-indexing-policy.json) use cosine float32/1536
with diskANN, exclude `/embedding/*` from ordinary indexing and retain the vector
index at `/embedding`. They're optional selected assets, not part of core creates.
For a deployment, verify account capability `EnableNoSQLVectorSearch` and exact
CLI/API support, then create a distinct container with these policies; don't
silently change an existing account's capability list or mutate a live vector model.

```sql
SELECT TOP @k c.id, c.source,
  VectorDistance(c.embedding, @embedding) AS distance
FROM c WHERE c.tenantId = @tenant AND c.status = 'published'
ORDER BY VectorDistance(c.embedding, @embedding)
```

Parameterize @k/vector/tenant; trusted partition-key routing and authorization are
still required. Bound top-k and page/attempt/RU budgets. A valid query can still
scan or have poor recall: measure build status, indexed population, filters,
recall@k/ground truth/freshness and charge, not one self-match.

Sources: [native vectors](https://learn.microsoft.com/azure/cosmos-db/vector-search),
[index constraints](https://learn.microsoft.com/azure/cosmos-db/how-to-manage-indexing-policy),
[pinned ARM container schema](https://learn.microsoft.com/azure/templates/microsoft.documentdb/2025-04-15/databaseaccounts/sqldatabases/containers).
