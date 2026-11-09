# Azure Cosmos DB for NoSQL operations

Operational coverage for Azure Cosmos DB for NoSQL (the NoSQL API only; MongoDB,
Cassandra, Gremlin and Table API behavior is not covered). Every section uses the
same heading and states one status. Cite evidence rows from
[the evidence ledger](evidence-ledger.md) as `[COSMOS-Exx]`.
`tests/validate-nosql-operations.py` enforces this structure.

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

- Two account-level backup modes exist: periodic and continuous. Continuous offers 7-day, 30-day and 35-day retention tiers, and a continuous account cannot be switched back to periodic. [COSMOS-E101]
- Periodic is the default for existing accounts; restores are requested through the support team. [COSMOS-E102]
- Periodic mode takes a full backup every 4 hours and keeps only the latest two by default; interval and retention are configurable. [COSMOS-E103]
- Periodic restore always creates a new account in the source write region and can cover any mix of containers, shared-throughput databases or the whole account. [COSMOS-E104]
- Periodic restore or retention changes need the CosmosdbBackupOperator role, owner or contributor. [COSMOS-E106]
- Continuous account restore always targets a new account; deleted containers and databases can be restored into a new or an existing account. [COSMOS-E107] [COSMOS-E108]
- Not restored: firewall, virtual network, data plane RBAC and private endpoint settings (both modes); periodic also omits account keys; continuous also omits source regions, stored procedures, triggers, UDFs, RBAC assignments and partial shared-throughput databases. Reapply these after restore. [COSMOS-E105] [COSMOS-E109]
- Continuous backups are taken in every region of the account, zone redundant where availability zones are enabled. [COSMOS-E110]
- Analytical store data is not part of backups or restores for Synapse Link accounts. [COSMOS-E112]

## Disaster recovery and replication

**Status:** VERIFIED

- Zone redundancy spreads replicas across availability zones and is set per region, not account-wide. Zone failover and recovery are fully managed by Microsoft. [COSMOS-E113] [COSMOS-E114]
- Single-write-region multi-region accounts: service-managed failover is automatic but can take one hour or more; a manual failover is triggered by changing the region holding failoverPriority=0. [COSMOS-E115] [COSMOS-E118]
- Per-partition automatic failover (PPAF) fails individual partitions over to a secondary region for single-write-region accounts. [COSMOS-E117]
- Multi-write-region accounts route to healthy regions through SDK configuration with no manual intervention. [COSMOS-E116]
- Documented RPO for multi-region accounts depends on the consistency level (Session, Consistent prefix and Eventual under 15 minutes; Strong zero); a single-region account has RPO under 240 minutes. Strong is unavailable with multiple write regions. [COSMOS-E17] [COSMOS-E18]
- A serverless account is single-region and cannot add regions while in serverless mode. [COSMOS-E10] For geo-replication, plan the one-time, irreversible in-place conversion to provisioned throughput (no duration SLA; management operations are blocked while it runs), then evaluate region addition and its billing; the migration steps are in [Consistency, capacity and security](consistency-capacity-security.md). [COSMOS-E20]
- Availability SLAs vary by configuration (zone redundancy, multi-region); read the current SLA document for figures. [COSMOS-E120]

## Cost

**Status:** PARTIAL

Billing dimensions only (what is billed and how it scales). Never write unit prices
or currency amounts: they change by Region and over time. Official pricing:
<https://azure.microsoft.com/pricing/details/cosmos-db/autoscale-provisioned/>;
calculator: <https://azure.microsoft.com/pricing/calculator/>.

- Meters: database operations (RU/s provisioned or consumed) and consumed storage, plus extra charges for backup storage, analytical storage, availability zones and multi-region writes. [COSMOS-E121]
- Provisioned throughput bills hourly; autoscale bills the highest RU/s reached each hour with a floor of 0.1 x Tmax. [COSMOS-E122] [COSMOS-E13]
- Serverless bills per request unit consumed and storage; it is single-region while in serverless mode. [COSMOS-E10]
- Backup: periodic mode includes two copies, extra copies bill as backup storage; continuous 30-day and 35-day tiers bill backup storage monthly, and every continuous restore is billed. [COSMOS-E123] [COSMOS-E111]
- Zone-redundant regions carry a premium, waived for multi-region-write accounts and autoscale collections. [COSMOS-E119]
- Reserved capacity is an upfront RU commitment; diagnostic logs to Log Analytics bill separately by ingested volume. [COSMOS-E124] [COSMOS-E125]
- Not confirmed in this pass: a dedicated data-transfer or cross-region egress meter. Check the pricing page before estimating multi-region transfer.

## Monitoring

**Status:** PARTIAL

- Platform metrics for `Microsoft.DocumentDB/DatabaseAccounts` flow to Azure Monitor, including NormalizedRUConsumption, TotalRequests, TotalRequestUnits, ProvisionedThroughput, AutoscaleMaxThroughput, DataUsage, IndexUsage, ReplicationLatency and ServerSideLatencyDirect/Gateway. [COSMOS-E126]
- Use Normalized RU Consumption by PartitionKeyRangeId to find hot partitions. [COSMOS-E127]
- Alerting is Azure Monitor alerts; the pages read give sample queries and point to the Azure Monitor Baseline Alerts site but no Cosmos-specific thresholds, so alert thresholds are a team decision. [COSMOS-E128]
- Microsoft treats 1-5 percent of requests returning 429 with acceptable latency as healthy; use it as a starting alert reference. [COSMOS-E131]

## Performance

**Status:** VERIFIED

- 429 (request rate too large) means operations exceeded the provisioned RUs within a second; variants cover data-operation RUs, metadata requests and transient service errors. [COSMOS-E129] [COSMOS-E130]
- SDKs retry 429s automatically, typically up to nine times, so Azure Monitor can show 429s the app never sees. [COSMOS-E131]
- Each logical partition is capped at 10,000 RU/s and 20 GB; exceeding a physical partition's throughput rate-limits operations. [COSMOS-E12]
- Hot partitions come from keys such as date-keyed IoT writes or one dominant tenant; hierarchical partition keys help when one key value would exceed 20 GB. [COSMOS-E132] [COSMOS-E12]
- Autoscale does not rate-limit within its range, but shared-database autoscale lets one container's burst throttle others. [COSMOS-E13]
- Serverless gives no predictable throughput or latency guarantee. [COSMOS-E19]

## IAM

**Status:** VERIFIED

- Control plane (account keys and connection strings, backup and restore, database and container management) is governed by Azure RBAC; built-in roles include Cosmos DB Operator. [COSMOS-E15] [COSMOS-E133]
- Data plane (items, queries, change feed, stored procedures) uses Cosmos DB native RBAC with the built-in Data Reader and Data Contributor roles; no `notDataActions`, and it is not manageable in the portal. [COSMOS-E14]
- Least privilege: assign data roles at the narrowest scope, and disable key auth with `disableLocalAuth=true` so only Microsoft Entra ID identities connect. [COSMOS-E14] [COSMOS-E16]
- Azure Policy is enforced at the resource provider, so SDK management calls that bypass it ignore policy; do not rely on policy alone. [COSMOS-E134]

## Tagging

**Status:** PARTIAL

- Resource tags can be set on the account at creation and on update, and the portal describes them for categorization and consolidated billing. [COSMOS-E136] [COSMOS-E137]
- Not confirmed: whether containers or databases carry their own tags, and whether tag-based access control (ABAC) applies to Cosmos DB data or control-plane access. The pages read do not say; treat both as unverified.

## Encryption

**Status:** VERIFIED

- At rest: always on, no switch, AES-256 in every region; databases, media attachments and backups are encrypted. [COSMOS-E138] [COSMOS-E139]
- Customer-managed keys are a second layer over service-managed keys, held in Azure Key Vault with one key per account, and can be enabled on new or existing accounts. [COSMOS-E140] [COSMOS-E143]
- Key Vault restrictions: Soft Delete and Purge Protection are required, and the vault must be public-reachable or allow trusted Microsoft services, so a private-endpoint-only vault cannot host the key. [COSMOS-E141] [COSMOS-E142]
- Continuous-backup accounts with customer-managed keys currently support only a user-assigned managed identity at creation. [COSMOS-E144]
- In transit: minimum TLS 1.2 service-wide (per-account `minimalTlsVersion`), with TLS 1.3 supported since March 31, 2025. [COSMOS-E145] [COSMOS-E146]

## Network isolation

**Status:** VERIFIED

- Private Link exposes the account through a private endpoint in a virtual network subnet; filtering is at application level and public DNS still resolves the endpoint. [COSMOS-E147] [COSMOS-E148]
- Accounts can also use an IP firewall and virtual network service endpoints. [COSMOS-E150]
- Setting `publicNetworkAccess` to Disabled takes precedence over the allow-from-Azure-datacenters option. [COSMOS-E149]
- Built-in Azure Policy definitions audit or deny accounts with public network access. [COSMOS-E135]
- Firewall, virtual network and private endpoint settings are not restored by backup restore; recreate them. [COSMOS-E105] [COSMOS-E109]

## Audit logging

**Status:** VERIFIED

- Control-plane operations are logged through the ControlPlaneRequests diagnostic category. [COSMOS-E153]
- Disable key-based metadata write access first, so SDK clients using account keys cannot change resources outside the audited path. [COSMOS-E154]
- Data-plane operations (for example delete, insert, readFeed) are captured per request as resource logs in the DataPlaneRequests category. [COSMOS-E151] [COSMOS-E152]
- Built-in Azure Policy definitions deploy diagnostic settings to Log Analytics, Storage or Event Hubs. [COSMOS-E135]
- Diagnostic logs bill separately in Log Analytics; enable them deliberately. [COSMOS-E125]
