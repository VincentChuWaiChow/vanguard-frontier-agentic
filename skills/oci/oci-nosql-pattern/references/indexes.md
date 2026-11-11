# Native secondary indexes, not LSI/GSI

Choose ordered composite keys for equality prefixes then needed range/order fields;
inspect the actual prepared/query plan and consumed capacity. The example uses
`CREATE INDEX tenant_status_created ON orders_v1(tenantId,status,createdAt)`.
Field indexes may cover fixed scalar fields or typed JSON paths. Schema-less JSON
index declarations can reject rows of incompatible type: profile sanitized shapes
before backfill and test future writes. ARRAY/MAP multikey indexing is distinct
from indexing an embedding's coordinates; it does not provide ANN search. Sparse
`WITH NO NULLS` changes which predicates can use the index; prove missing/null cases.

Indexes consume the table's storage/write budget; there is no independent GSI
provisioned capacity, GSI-only consistency rule or arbitrary global uniqueness.
Adding an index is asynchronous; read work-request error and final index state,
then validate representative query correctness/latency. A failure is not permission
to drop/recreate a table. Plan migration and rate-limited backfill explicitly.

[Create-index semantics](https://docs.oracle.com/en/cloud/paas/nosql-cloud/uapto/),
[Terraform index](https://docs.oracle.com/en-us/iaas/tools/terraform-provider-oci/latest/docs/r/nosql_index.html), checked 2026-11-08.
