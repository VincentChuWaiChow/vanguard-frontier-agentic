# Azure Cosmos DB for Apache Cassandra operations

Operational coverage for **Azure Cosmos DB for Apache Cassandra** (the CQL API of
an Azure Cosmos DB account). Every section uses the same heading and states one
status. Cite evidence rows from [the evidence ledger](evidence-ledger.md) as
`[COSMOS-Exx]`. `tests/validate-nosql-operations.py` enforces this structure.

Scope and naming: this is the Cassandra API on the Azure Cosmos DB engine, billed
in request units [COSMOS-E301]. It is **not** Azure Managed Instance for Apache
Cassandra, a separate first-party service that hosts pure open-source Cassandra
clusters (virtual machine scale sets in your virtual network, instance-based
pricing, nodetool-style operations) [COSMOS-E302] [COSMOS-E303]. Its metrics and
diagnostics are a different surface [COSMOS-E304]. Nothing below applies to the
managed instance service, and nothing is inferred from the API for NoSQL or
MongoDB.

Statuses:

- `VERIFIED`: the provider documents the capability; every statement below cites a
  VERIFIED row.
- `PARTIAL`: some of it is documented and some is not, or it is limited (by tier,
  mode, Region or API); say which part is which.
- `UNSUPPORTED`: provider documentation states the capability does not exist. Cite it.
- `UNVERIFIED`: no official page settled it in this pass. Say what was searched.
- `NOT APPLICABLE`: the capability does not apply to this service. Say why.

## Backup and restore

**Status:** VERIFIED

- Continuous backup with point-in-time restore is **not supported** for the API
  for Cassandra; Microsoft lists it for the NoSQL, MongoDB, Gremlin and Table APIs
  only [COSMOS-E305].
- Periodic backup is the default mode for existing accounts; interval can be as
  low as one hour and retention up to a month [COSMOS-E306]. Default is a full
  backup every 4 hours with the latest two kept; deleted containers or
  shared-throughput databases keep snapshots for 30 days [COSMOS-E307].
- Restore is by request to the Azure Cosmos DB team and always creates a new
  single-region account in the source write region; backups are taken only in the
  write region and replicated by geo-redundant storage [COSMOS-E308].
- Not restored: account keys, firewall, virtual network, data-plane RBAC and
  private endpoint settings, regions, stored procedures, triggers, UDFs, RBAC
  assignments, TTL-expired documents, analytical data, materialized views
  [COSMOS-E309].
- Restore rights belong to CosmosdbBackupOperator, owners and contributors; two
  backups are free [COSMOS-E310]. Restores cannot target fewer partitions or lower
  RU/s, and analytical data is excluded [COSMOS-E311].
- Backups are platform managed and immutable under the retention policy
  [COSMOS-E312].

## Disaster recovery and replication

**Status:** PARTIAL

- Regions are added at the account level; CREATE KEYSPACE replication options are
  ignored, and each region keeps four replicas that cannot be modified
  [COSMOS-E313].
- Single-write-region accounts can keep strong consistency with service-managed
  failover across regions [COSMOS-E315]; accounts with multiple write regions
  cannot use strong consistency, and no distributed system gives RPO and RTO of
  zero [COSMOS-E316]. Write consistency follows the account default
  [COSMOS-E314].
- Lightweight transactions are not supported with multiple region writes
  [COSMOS-E317].
- The account-level Reliability article is framed around the API for NoSQL and
  has no Cassandra marker: service-managed failover is Microsoft-decided, may take
  an hour or more and may lose unreplicated writes [COSMOS-E318]; RPO depends on
  consistency level [COSMOS-E319]. Treat these as account-level statements not
  confirmed for the Cassandra API: do not promise them to a Cassandra workload
  without that confirmation.
- Unverified for this API: an availability SLA figure and RTO specific to the
  Cassandra API.
- Backups are not geo-resilient beyond the periodic GRS copy [COSMOS-E308].

## Cost

**Status:** VERIFIED

Billing dimensions only. Pricing: <https://azure.microsoft.com/pricing/details/cosmos-db/>
and the calculator <https://azure.microsoft.com/pricing/calculator/>.

- Throughput: provisioned RU/s billed hourly at the hour maximum in 100 RU/s
  steps [COSMOS-E320]; set per keyspace (shared) or per table with a 400 RU/s
  minimum [COSMOS-E323]; autoscale available via CQL [COSMOS-E324].
- Storage: data and indexes per GB per hour [COSMOS-E321]; physical size can
  differ from native Cassandra [COSMOS-E357].
- Replication: each added region carries throughput and storage, and replication
  egress is billed [COSMOS-E322].
- Backup: periodic keeps two free and charges extra copies [COSMOS-E310].
- Sizing: profile requests to estimate RUs [COSMOS-E325].

## Monitoring

**Status:** PARTIAL

- Metrics: CassandraRequests, CassandraRequestCharges and
  CassandraConnectionClosures plus shared metrics such as NormalizedRUConsumption
  and ProvisionedThroughput [COSMOS-E326].
- Resource logs: CDBCassandraRequests for data plane operations and
  CDBPartitionKeyRUConsumption for hot keys [COSMOS-E327] [COSMOS-E328]; the
  monitoring reference lists Cassandra error codes [COSMOS-E329].
- 429s retried by server-side retry appear in the Metrics pane [COSMOS-E331].
- Alerts: Azure Monitor alerts apply generically [COSMOS-E330]; no
  Cassandra-specific provider-recommended alert set was found.

## Performance

**Status:** VERIFIED

- Throttling surfaces as OverloadedException or 429 [COSMOS-E332] [COSMOS-E337].
- Server-side retry is an account-level setting (capability
  DisableRateLimitingResponses, up to 15 minutes) with a 60-second retry window;
  set client read time-out above it [COSMOS-E333] [COSMOS-E334].
- Limits: 20 GB per logical and 30 GB per physical partition, replication factor 4
  [COSMOS-E335]; 10,000 RU/s per partition, so choose evenly distributed keys
  [COSMOS-E336].
- gc_grace_seconds must be zero [COSMOS-E338].

## IAM

**Status:** VERIFIED

- Control plane: Azure RBAC for provisioning, key rotation, metrics and obtaining
  read-write or read-only passwords [COSMOS-E339]; built-in roles Cosmos DB
  Operator (no data or keys) [COSMOS-E341], DocumentDB Account Contributor and
  Cosmos DB Account Reader Role [COSMOS-E342].
- Data plane: the API does **not** support roles for CRUD activities, so access is
  by account-key credentials rather than per-keyspace roles [COSMOS-E340].
- Limit blast radius with firewall and private endpoints (below) and by restricting
  who can read keys.
- Block key-based metadata writes with disableKeyBasedMetadataWriteAccess
  [COSMOS-E356].

## Tagging

**Status:** PARTIAL

- Resource tags are set on the account at creation for categorization and billing
  views [COSMOS-E343].
- Unverified: tag-based access control and tag support below account level.
  Searched Learn for Cassandra account tags and the keyspace resource template.

## Encryption

**Status:** PARTIAL

- At rest: always on, AES-256, Microsoft-managed keys, no switch [COSMOS-E344].
  In transit: encrypted over the network [COSMOS-E345].
- Customer-managed keys add a second layer using Azure Key Vault [COSMOS-E358].
  The vault needs soft delete and purge protection and cannot be reachable only by
  private endpoints [COSMOS-E347].
- With CMK, id, ttl, _ts, _etag, _rid, _self, _attachments and _epk are reserved
  field names on Cassandra tables [COSMOS-E346].
- Unverified: minimum TLS version for the Cassandra endpoint.

## Network isolation

**Status:** VERIFIED

- IP firewall rules apply; Data Explorer client IPs must be allow-listed
  [COSMOS-E348]. The Cassandra shell in Data Explorer does not work with virtual
  network access [COSMOS-E349].
- Private endpoints support the Cassandra API type [COSMOS-E350].
- Set publicNetworkAccess to Disabled at creation to block public traffic
  [COSMOS-E351]; adding endpoints later needs rules first to avoid about five
  minutes of downtime [COSMOS-E352].

## Audit logging

**Status:** VERIFIED

- Control plane: enable ControlPlaneRequests [COSMOS-E353]; keyspace and table
  create, update and throughput events are logged [COSMOS-E354].
- Data plane: CDBCassandraRequests records requests [COSMOS-E327]; full-text
  query logging includes parameter values (PII), so enable deliberately
  [COSMOS-E355].
- Use disableKeyBasedMetadataWriteAccess so changes are attributable to Azure roles
  [COSMOS-E356].
