# Model access before choosing layout

| Access | Model/query | Tradeoff |
|---|---|---|
| One order | full `(tenantId,orderId)` key get | bounded read, tenant from authenticated context |
| Tenant orders | shard predicate `tenantId = $tenant`; key range if needed | one tenant may become a hot shard |
| Tenant open orders by time | index `(tenantId,status,createdAt)` and equality leading fields | write/storage amplification; verify chosen query plan |
| Global analytics | explicit bounded scatter query/export to analytics | not an invisible request-path scan fallback |

Shard key is a leading subset of primary-key fields, not a DynamoDB partition-key/
sort-key pair. Same shard values co-locate records; high-cardinality tenant IDs
usually distribute, but a dominant tenant needs deliberate bucketing. A bucket in
the shard key changes query fan-out and atomicity, so quantify both. A secondary
index does not give separate capacity or a new transaction boundary. Parent/child
tables share shard keys and parent capacity; hierarchy is an explicit alternative,
not a requirement to put every entity in one JSON table.

Example [DDL](../assets/orders.ddl): fixed identity/status/time/decimal fields,
flexible `document JSON`, primary `SHARD(tenantId),orderId`, 30-day default TTL.
This is ephemeral demonstration data; durable financial orders may require no TTL.
NoSQL schema normalization and SQL dialect are not Oracle Database relational SQL.
Tenant predicates are application controls, not IAM-enforced per-row isolation;
separate tables/compartments/principals if the threat model requires that boundary.

[Cloud concepts](https://docs.oracle.com/en/cloud/paas/nosql-cloud/dtddt/cloud-concepts.html)
and [modeling](https://docs.oracle.com/en/database/other-databases/nosql-database/23.1/nsdev/table-modelling-and-design1.html), checked 2026-11-08.
