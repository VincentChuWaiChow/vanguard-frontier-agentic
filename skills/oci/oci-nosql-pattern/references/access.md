# OCI principals, tenancy/compartment/table scopes

Choose an existing approved workload/resource/instance principal where supported;
do not install auth, read config/key files, login, or print tokens. SDK initialization
and OCI CLI profiles are different surfaces. A reachable tenancy get and supplied
`PRINCIPAL_OCID` are not evidence of the active caller identity.

| Resource scope | Least-privilege intent |
|---|---|
| `nosql-tables` | metadata read distinct from create/update/drop/move |
| `nosql-rows` | read: `NOSQL_ROWS_READ`; use adds `NOSQL_ROWS_INSERT`; manage adds `NOSQL_ROWS_DELETE` |
| `nosql-indexes` | inspect/read distinct from index creation/deletion |
| work requests | exact request status/errors; avoid broad tenancy inventory |
| recovery storage/key | source read, narrowly scoped destination object write/read, Vault decrypt dependency; never all-resources by default |

Illustrative policy statements, not deployed or an entire role:

```text
Allow dynamic-group OrdersReader to read nosql-tables in compartment Orders
Allow dynamic-group OrdersReader to read nosql-rows in compartment Orders
```

Review documented target/table conditions and effective IAM with the security owner;
these compartment policies alone grant multiple tables, not per-tenant rows.
Application binds tenant/query on every path, including secondary queries, bulk
writes, cursor replay and vector retrieval. Do not promise row-level IAM from a
`tenantId` shard key or infer cross-tenancy grants from a familiar SDK.

[NoSQL policy reference](https://docs.oracle.com/en/cloud/paas/nosql-cloud/odsql/), checked 2026-11-08.
