# Data types and native vector search

Verified against current AWS primary documentation on 2026-11-08. DynamoDB now
supports **native vector indexes and SearchVectors**, but this does **not** introduce
a new AttributeValue primitive named VECTOR. Keep storage types, index types and
search operations separate. Reverify through the [maintenance cadence](maintenance.md).
For what coordinates mean, numeric range/precision and concrete Bedrock/S3 Vectors
architectures, read [embedding integrations](embedding-integrations.md).

## AttributeValue families

| Family | Low-level tag | Example / boundary |
|---|---|---|
| String | S | `{"S":"document"}`; encode dates explicitly, not a native date type |
| Number | N | `{"N":"12.50"}`; decimal text on the wire, not a JS float by default |
| Binary | B | `{"B":"SGVsbG8="}` in JSON/CLI base64 representation |
| Boolean | BOOL | `{"BOOL":true}`; distinct from string `"true"` |
| Null | NULL | `{"NULL":true}`; explicit null differs from an absent attribute |
| List | L | Ordered, may mix types; embeddings specifically require numeric elements |
| Map | M | Nested name-to-AttributeValue map |
| String/number/binary set | SS / NS / BS | Homogeneous, unordered, unique members; empty sets are invalid |

Primary/index scalar keys use S/N/B, not L/M/sets. Empty ordinary lists/maps are
different from empty sets; a vector cannot be an empty list. Preserve decimal
precision, binary encoding and null/absence distinctions in each SDK's marshaller.
Do not double-marshal native document clients with low-level tags.

[typed-vector-item.json](../assets/typed-vector-item.json) is a synthetic complete
item illustrating all ten tags. Its three-coordinate embedding is:

```json
{"Embedding":{"L":[{"N":"0.1"},{"N":"0.2"},{"N":"0.3"}]}}
```

Do not substitute NS: sets lose coordinate order and duplicates. Do not replace
numeric elements with S, nested vectors or an invented V/VECTOR tag. DynamoDB
does not generate embeddings. Use the same embedding model/version, dimensions
and preprocessing for stored/query vectors. Vector indexes use float32 values;
higher precision in the original Number attributes does not make the index exact.

## Native vector index vs ordinary access vs external search

| Need | Correct path |
|---|---|
| Exact key lookup / deterministic key-range query | GetItem / Query on the base table or ordinary GSI/LSI |
| Approximate nearest-neighbor embedding similarity | Native vector index + SearchVectors |
| Full-text, analytics or a different hybrid-search architecture | Evaluate a separate engine/integration, including DynamoDB-to-OpenSearch, with sync/security/freshness costs |

Native vector search does **not** require OpenSearch. It is approximate and
eventually consistent; it does not inherit strong search consistency from an MRSC
base table. Do not reuse Query/Scan/PartiQL, a Query paginator, or ConsistentRead
as a vector-search implementation. SearchVectors has no pagination and TopK is
not an arbitrary page size; results can be fewer than requested.

## Optional three-dimensional example (not the default Orders table)

Use [vector-create-table.json](../assets/vector-create-table.json) or the matched
[vector-table.cfn.yaml](../assets/vector-table.cfn.yaml), **one ownership path only**.
They define a new on-demand table, a Tenant HASH search partition and Status
INLINE_FILTER. Three dimensions are a readable fixture, not a production embedding
recommendation. Replace the schema/model deliberately for the actual workload.

For vector indexes, **every SearchSchema attribute**, including INLINE_FILTER
attributes, must be declared as a scalar in AttributeDefinitions. Embedding itself
is stored as L and is **not** declared there. This is an important exception to
the simplified “only primary/secondary key fields” rule for ordinary table/GSI/LSI
examples. The index identifies Embedding through VectorAttribute instead.

### Commands: inspect, optionally create/write, then search

These are examples only, not an executable deployment script. No live operation
is authorized by reading this guide. Run from the skill directory; select approved
account/role/Region, table, costs and data scope. Keep the original table unchanged.

```bash
set -euo pipefail
: "${AWS_PROFILE:?}" "${AWS_REGION:?}" "${VECTOR_TABLE_NAME:?Approved NEW demonstration table}"
aws_target=(aws --profile "$AWS_PROFILE" --region "$AWS_REGION" --no-cli-pager --output json)
"${aws_target[@]}" sts get-caller-identity
# Separately approved billable NEW-table creation, not an update to Orders.
"${aws_target[@]}" dynamodb create-table --table-name "$VECTOR_TABLE_NAME" \
  --cli-input-json file://assets/vector-create-table.json
"${aws_target[@]}" dynamodb wait table-exists --table-name "$VECTOR_TABLE_NAME"
"${aws_target[@]}" dynamodb describe-table --table-name "$VECTOR_TABLE_NAME" \
  --query 'Table.{State:TableStatus,Indexes:VectorIndexes}'
```

The complete CFN alternative retains resources on deletion/replacement and enables
deletion protection. Both examples leave AWS-owned encryption as the default and
are not production-complete IAM/tag/PITR/alarm configurations; use the other guides.

```bash
# Separately approved synthetic data mutation; prevent overwriting an existing key.
"${aws_target[@]}" dynamodb put-item --table-name "$VECTOR_TABLE_NAME" \
  --item file://assets/typed-vector-item.json --cli-binary-format base64 \
  --condition-expression 'attribute_not_exists(PK)'
# Authorized bounded search; three coordinates match the example index.
"${aws_target[@]}" dynamodb search-vectors --table-name "$VECTOR_TABLE_NAME" \
  --cli-input-json file://assets/vector-search.json
```

[vector-search.json](../assets/vector-search.json) uses **SearchVector as a bare
array of Number AttributeValues**, not the stored item's L wrapper. Its
SearchConditionExpression supplies the configured Tenant equality and optional
Status equality; this is not FilterExpression. Only equality on declared top-level
search-schema attributes is currently supported. SearchResults contains Item and
Score; lower COSINE/EUCLIDEAN scores are better, higher DOT_PRODUCT scores are better.
Embedding is not returned by default; request it explicitly only when needed.

Wait for the intended vector index's ACTIVE state and verify dimensions/schema.
Search endpoint readiness can lag that status; use bounded retries only for the
documented transient not-yet-ready/missing-index case, not for every validation
failure, invalid dimension or unauthorized call. A successful search is not a
recall, latency, cost or tenant-security benchmark.

## Constraints and design tests

- Native vector indexes require PAY_PER_REQUEST [DP-E214]. Provisioned table/index
  capacity is not a supported alternative for this feature.
- Dimensions: 1–4,096; TopK: 1–100; validate finite numeric float32-compatible
  values and matching model dimensions. Define zero-vector handling for cosine.
- One optional HASH and up to 18 inline filter attributes. When HASH is configured,
  every search must supply its equality value. Missing embedding/HASH attributes
  can exclude an item from the index; inspect write failures/validation rules too.
- Index configuration is immutable: dimensions, distance, projection and schema
  changes require a new index, backfill validation and staged cutover.
- Default five vector indexes (requestable increase to 20), **also consuming the
  table's GSI quota**. Online GSI/vector creation/deletion is serialized. Refresh
  actual quotas, intended Region availability and SDK/CLI/CFN support at use time.
- Search has a 16 MB response boundary and no service cursor. Do not fabricate
  LastEvaluatedKey or fetch “page two” by changing TopK. Test recall/filtered result
  count and whether a separate exact retrieval step is necessary.
- Existing ALL-projection GSIs/LSIs copy embeddings and can increase write/storage/
  read costs. Measure index amplification and current vector-search pricing; do
  not blindly apply ordinary Query capacity/projection rules to SearchVectors.

## IAM and tenant isolation

The documented [vector policy](../assets/vector-search-policy.json) grants
`dynamodb:SearchVectors` on one named **vector index ARN**. It does not imply Query,
item writes, CreateTable or UpdateTable rights. Creation/index lifecycle uses those
existing administration actions. Follow the specific vector IAM documentation,
not a blanket copy of ordinary Query's resource list.

SearchVectors does **not** support FGAC keys LeadingKeys, Attributes or Select.
Tenant HASH routing is locality/filtering, **not IAM-enforced tenant isolation**:
an identity with index permission can choose a different Tenant. Derive filters
from authenticated context and restrict the exposed application interface. For
strict data-layer tenant boundaries, use separate IAM-scoped tables/indexes and
test both positive and cross-tenant negative access. Never claim an index filter
or this demonstration policy alone provides a complete multi-tenant boundary.

## Installed-tool evidence and lint limitation

The inspected CLI 2.37.4 exposes SearchVectors and its typed-array input skeleton;
that is local syntax support, not a successful service request or all-language SDK
support. Verify the target SDK version; a missing model/operation is not proof the
service lacks native support. Do not silently install or upgrade packages.

Strict cfn-lint 1.57.2 **and the user-upgraded 1.60.0** emit E3039 for the required
Tenant/Status declarations. Inspection of AttributeMismatch.py confirms that rule
collects base/GSI/LSI KeySchema but not vector SearchSchema. The template retains
AWS-documented declarations; no ignore is embedded in it or the default template.
Tests independently compare the full declared set with base keys plus vector HASH
and INLINE_FILTER attributes. All other local CFN rules pass with E3039 excluded
for this vector fixture only; strict vector lint remains a documented compatibility
MISS, not a claimed pass. Service deployment acceptance remains unverified.

Sources: [data types](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/HowItWorks.NamingRulesDataTypes.html),
[working with native vectors](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/VectorSearchWorkingWith.html),
[SearchVectors API](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_SearchVectors.html),
[requirements](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/VectorSearch.Requirements.html),
[vector IAM](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/iam-policy-example-search-vectors.html),
[quotas](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/ServiceQuotas.html),
[CFN vector index](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-properties-dynamodb-table-vectorindex.html).
