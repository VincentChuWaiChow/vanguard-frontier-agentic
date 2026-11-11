# Modeling — NoSQL API, not renamed DynamoDB

Cosmos stores JSON items in containers under a database/account. `id` is a string;
uniqueness is `(partition-key value, id)` within a container. Point reads require
both `id` and the exact partition-key value with the correct JSON type. A SQL query
with an id predicate isn't the same priced/routed operation as a point read.

| Pattern | Example route | Choice / tradeoff |
|---|---|---|
| Order by authenticated tenant/id | item(`order-42`, `tenant-a`) | `/tenantId` partition, point read |
| Tenant order history | partition key `tenant-a`, `WHERE c.tenantId=@tenant ORDER BY c.createdAt DESC,c.id ASC` | Composite createdAt DESC/id ASC; paginated |
| Global operational report | cross-partition bounded query/materialized projection | Fan-out/RUs explicit; not an implicit hot-path scan |
| Atomic order + outbox | same tenant logical partition | Transactional batch; bulk isn't atomic |
| Very large/hot tenant | hierarchical `/tenantId`,`/bucket` or explicit tenant-bucket routing | Query/transaction scope changes; evaluate prefixes and complete key tuples |

Start with cardinality, distribution and per-tenant peak RU/s/storage, not only
average traffic. A conventional logical partition is bounded by 20 GB and physical
partition maximum 10,000 RU/s; a hot logical key cannot use all account throughput.
Hierarchical partition keys can have up to three levels and a prefix can span
physical partitions, but complete tuples/transaction bounds still matter. Refresh
limits/account features before production sizing; a tenant key is not a security
boundary by itself. Dedicated containers/accounts improve isolation at cost.

Partition keys are immutable per item; changing them entails delete/create or a
reviewed migration, not an in-place patch. Changing the configured key requires a
new-container/copy strategy appropriate to current features. Keep source live and
rollbackable; never delete it to make a name reusable.

Embed bounded child data when read/updated together and size growth is controlled;
reference unbounded collections, binary/media in Blob Storage, and large source
documents. Avoid unbounded arrays and cross-partition pseudo-joins. Materialize
denormalized views with idempotent consumers and explicit staleness/reconciliation.
Document unique-key policies before container creation: scoped to logical
partition, case-sensitive, no sparse uniqueness; missing values behave as null.

For multi-tenancy, derive tenant from authenticated server context, enforce it in
partition routing AND predicates, validate returned rows, bind continuation tokens
to tenant/query and don't expose account credentials to tenants. Cosmetic SQL
filtering or a partition-key value alone doesn't restrict data-plane authorization.

Sources: [partitioning](https://learn.microsoft.com/azure/cosmos-db/partitioning-overview),
[hierarchical keys](https://learn.microsoft.com/azure/cosmos-db/hierarchical-partition-keys),
[unique keys](https://learn.microsoft.com/azure/cosmos-db/unique-keys),
[limits](https://learn.microsoft.com/azure/cosmos-db/concepts-limits).
