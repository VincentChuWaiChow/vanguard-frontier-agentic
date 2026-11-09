# Azure Cosmos DB for MongoDB operations

Operational coverage for **Azure Cosmos DB for MongoDB (RU-based account)**. Every
section uses the same heading and states one status. Cite evidence rows from
[the evidence ledger](evidence-ledger.md) as `[COSMOS-Exx]`.
`tests/validate-nosql-operations.py` enforces this structure.

Scope and naming: this file covers the request-unit (RU) based Azure Cosmos DB
account that exposes the MongoDB wire protocol [COSMOS-E203]. The vCore-based
offering, formerly "vCore-based Azure Cosmos DB for MongoDB", is now **Azure
DocumentDB (with MongoDB compatibility)**, a separate service with dedicated
vCore-based clusters [COSMOS-E201] [COSMOS-E202]. Microsoft points new
MongoDB Query Language needs and MongoDB migrations to Azure DocumentDB
[COSMOS-E204]. Nothing below applies to Azure DocumentDB unless stated, and
nothing here is inferred from the API for NoSQL: rows were chosen only where a
page names MongoDB or is an account-level Azure Cosmos DB page.

Statuses:

- `VERIFIED`: the provider documents the capability; every statement below cites a
  VERIFIED row.
- `PARTIAL`: some of it is documented and some is not, or it is limited (by tier,
  mode, Region or API); say which part is which.
- `UNSUPPORTED`: provider documentation states the capability does not exist. Cite it.
- `UNVERIFIED`: no official page settled it in this pass. Say what was searched.
- `NOT APPLICABLE`: the capability does not apply to this service. Say why.

## Backup and restore

**Status:** PARTIAL

- Every account has automatic backups; there are two modes, periodic (default for
  existing accounts) and continuous with 7-day, 30-day and 35-day tiers
  [COSMOS-E205] [COSMOS-E206] [COSMOS-E218].
- Periodic mode: full backup every 4 hours, latest two kept by default,
  interval as low as one hour, retention up to a month, restore by support
  request, always into a new single-region account in the source write region
  [COSMOS-E207] [COSMOS-E208] [COSMOS-E210].
- Continuous mode: point-in-time restore within the tier; stored locally
  redundant (zone-redundant where the region has zones) and not geo-resistant
  unless another region is added [COSMOS-E214] [COSMOS-E215]. Restore cannot
  target an account with lower RU/s or fewer partitions [COSMOS-E217].
- Documentation conflict: the continuous-backup article describes restore into a
  new or an existing account but also states it always restores into a new
  account; confirm in the portal before relying on in-place restore
  [COSMOS-E212].
- Not restored: firewall, virtual network, data-plane RBAC and private endpoint
  settings, source regions, stored procedures, triggers, UDFs, RBAC assignments
  [COSMOS-E211]; periodic mode also loses account keys, TTL-expired documents,
  analytical data and materialized views [COSMOS-E208].
- MongoDB specific: continuous-backup accounts cannot create a unique index on an
  existing collection; create it with the collection [COSMOS-E213].
- Backups are platform managed, encrypted with Microsoft-managed keys and not
  alterable or deletable under the retention policy [COSMOS-E219] [COSMOS-E220].
  Restore rights: CosmosdbBackupOperator, owner or contributor for periodic
  [COSMOS-E209]; CosmosBackupOperator and CosmosRestoreOperator built-in roles
  [COSMOS-E253].

## Disaster recovery and replication

**Status:** PARTIAL

- Add regions from the Replicate data globally pane; Microsoft recommends region
  pairs for business continuity and says each added region has a cost
  [COSMOS-E221].
- The driver treats the write region as primary and other regions as read
  replicas; the write concern maps to the account default consistency level
  [COSMOS-E226]. Multi-region writes are available and described as active-active
  [COSMOS-E222].
- Failover and RPO figures come from the account-level Reliability article, which
  is framed around the API for NoSQL and carries no MongoDB marker: service-managed
  failover is Microsoft-decided and can take an hour or more, with possible loss of
  unreplicated writes [COSMOS-E223]. RPO for two or more regions: under 15 minutes
  for Session, Consistent prefix and Eventual, K and T for Bounded staleness
  (minimum 100,000 operations or 300 seconds), 0 for Strong [COSMOS-E224]
  [COSMOS-E225]. Treat these as account-level figures not confirmed for the
  MongoDB API: do not promise them to a MongoDB workload without that confirmation.
- Unverified for this API: a MongoDB-specific availability SLA figure and RTO.
  Searched the MongoDB pages of Microsoft Learn; the SLA text on those pages
  describes the API for NoSQL only.
- vCore difference: Azure DocumentDB is a separate service with its own
  replication model; its disaster recovery is not covered here [COSMOS-E201].

## Cost

**Status:** PARTIAL

Billing dimensions only. Pricing: <https://azure.microsoft.com/pricing/details/cosmos-db/>
and the calculator <https://azure.microsoft.com/pricing/calculator/>.

- Throughput: provisioned throughput is billed hourly at the maximum RU/s set in
  the hour, in 100 RU/s steps [COSMOS-E227]. Capacity modes are provisioned and
  serverless; serverless availability for the MongoDB API was not separately
  confirmed [COSMOS-E231].
- Storage: data plus indexes, per GB per hour [COSMOS-E228].
- Replication: each added region carries its own throughput and storage, and
  replication egress is billed at data transfer rates [COSMOS-E229] [COSMOS-E230].
- Backup: periodic keeps two backups free and charges extra copies [COSMOS-E209];
  continuous 30-day and 35-day tiers charge backup storage, and every restore is
  charged [COSMOS-E216].
- Estimating: the Cost Estimator documents MongoDB operation mapping
  [COSMOS-E232]; per-operation charge is readable with getLastRequestStatistics
  [COSMOS-E233].

## Monitoring

**Status:** PARTIAL

- Azure Monitor metrics include MongoRequests, MongoRequestCharge,
  NormalizedRUConsumption, TotalRequestUnits, ProvisionedThroughput, DataUsage,
  IndexUsage, DocumentCount, ReplicationLatency and ServiceAvailability
  [COSMOS-E234].
- Resource logs: MongoRequests (table CDBMongoRequests), PartitionKeyRUConsumption
  and PartitionKeyStatistics for hot partition and storage outliers
  [COSMOS-E235] [COSMOS-E236].
- Kusto samples exist for top RU requests, throttling (429 or 16500) and
  timeouts (50) [COSMOS-E237].
- Alerts: Azure Monitor alerts apply, and the partitioning article recommends one
  on logical partition size nearing 20 GB; no MongoDB-specific recommended alert
  list was found [COSMOS-E238].

## Performance

**Status:** VERIFIED

- Throttling returns error 16500 (TooManyRequests); remedies are more RU/s,
  client retry, or server-side retry [COSMOS-E239] [COSMOS-E242].
- Error 50 means the 60-second limit was exceeded; retrying a write can duplicate
  it. Multi-document transactions have a 5-second limit [COSMOS-E240] [COSMOS-E241].
- Each logical and physical partition is capped at 10,000 RU/s, and a logical
  partition holds up to 20 GB; choose a key that spreads consumption evenly
  [COSMOS-E243] [COSMOS-E244] [COSMOS-E245].
- Floor of 400 RU/s on manual throughput [COSMOS-E246]. API limits: 60-second
  operations, 16 MB documents with enablement, 6-level index nesting
  [COSMOS-E247]. Use a 2-3 minute driver idle timeout [COSMOS-E248].

## IAM

**Status:** VERIFIED

- Data plane: built-in RBAC for MongoDB, enabled with the
  EnableMongoRoleBasedAccessControl capability [COSMOS-E249]. Scope with custom
  roles (privileges on a database and collection) assigned to users with
  SCRAM-SHA-256 [COSMOS-E250].
- Control plane: Cosmos DB Operator (no data or keys), DocumentDB Account
  Contributor and Cosmos DB Account Reader Role [COSMOS-E251] [COSMOS-E252].
- Restore roles: CosmosBackupOperator and CosmosRestoreOperator [COSMOS-E253].
- Set disableKeyBasedMetadataWriteAccess so account keys cannot change
  resources [COSMOS-E254].

## Tagging

**Status:** PARTIAL

- Resource tags are set at account creation to categorize resources and view
  consolidated billing [COSMOS-E255].
- Tags are not customer data: keep sensitive information out of names and tags
  [COSMOS-E256].
- Unverified: tag-based access control (ABAC) and a tag limit for this resource.
  Searched Learn for MongoDB account tags; no governing page found.

## Encryption

**Status:** PARTIAL

- At rest: always on, AES-256, Microsoft-managed keys, no switch [COSMOS-E257].
  In transit: encrypted over the network [COSMOS-E258].
- Customer-managed keys add a second layer using Azure Key Vault, one key per
  account [COSMOS-E259]. The vault needs soft delete and purge protection and
  cannot be private-endpoint-only [COSMOS-E260].
- Documentation conflict: the encryption FAQ says CMK is for new accounts at
  creation, while the CMK article references enabling it on existing accounts
  [COSMOS-E261].
- Restoring a CMK continuous account needs a user-assigned identity [COSMOS-E262].
- Unverified: minimum TLS version for the MongoDB endpoint.

## Network isolation

**Status:** VERIFIED

- An account is internet-reachable with a valid token until an IP firewall is set;
  Data Explorer clients must then be allow-listed [COSMOS-E263].
- Private endpoints support MongoDB accounts on server version 3.6 or higher;
  clients need SNI or the appName parameter [COSMOS-E265] [COSMOS-E266].
- Set publicNetworkAccess to Disabled at creation to block public traffic
  [COSMOS-E267]. Adding endpoints later causes about five minutes of downtime
  unless rules are added first [COSMOS-E268].
- The Mongo shell in Data Explorer does not work with virtual network access
  [COSMOS-E264].

## Audit logging

**Status:** VERIFIED

- Control plane: enable the ControlPlaneRequests diagnostic category
  [COSMOS-E269]; Mongo database and collection create, update, throughput and
  delete operations are listed [COSMOS-E270].
- Data plane: the MongoRequests category records requests with RU charge,
  duration and error code; the command text is logged only if opted in
  [COSMOS-E235] [COSMOS-E271].
- Restrict key-based metadata writes so changes are attributable to Azure roles
  [COSMOS-E254].
