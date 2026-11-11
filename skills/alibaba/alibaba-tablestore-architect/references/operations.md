# Alibaba Cloud Tablestore operations

Operational coverage for Alibaba Cloud Tablestore (Wide Column model, international
site). Every section uses the same heading and states one status. Cite evidence rows
from [the evidence ledger](evidence-ledger.md) as `[TS-Exx]`.
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

**Status:** PARTIAL

- Backup is delivered by Cloud Backup plans: the policy sets the backup cycle, first
  start time, interval and whether incremental backup is used, and plans can be run
  manually as full or incremental backups. [TS-E101]
- The first backup job is full and later jobs are incremental by default. [TS-E105]
- Only data tables and search indexes are backed up; secondary indexes and time series
  tables are not, and encryption settings are not backed up (vault tables are not
  encrypted). Each backup job creates a tunnel in Tablestore. [TS-E20]
- Backups can be replicated cross-region (off by default); traffic shaping is off by
  default, so cap backup traffic or run in off-peak hours. [TS-E102]
- Restore goes to the source instance or another instance in the same region, one
  table (plus search indexes) per job. A missing destination table is created from
  the backup; rows with the same primary key are overwritten and other rows are
  untouched. [TS-E104]
- Restore cannot target a table that has a secondary index (delete, restore,
  recreate), and a disabled or deleted KMS key blocks restore. [TS-E105]
- Not documented in the pages reviewed: a maximum retention value. The lifecycle
  setting accepts a retention period and an optional keep-at-least-one rule, which is
  the unconfirmed part. [TS-E101]

## Disaster recovery and replication

**Status:** VERIFIED

- Zone-redundant storage (ZRS) states an RTO of 0 and an RPO of 0 for a zone
  failure, with strong consistency and transparent failover. [TS-E107]
- LRS (single zone) versus ZRS: the page lists durability of 11 nines versus 12 nines
  and availability of 99.9 percent versus 99.99 percent. In ZRS regions new instances
  use ZRS by default and this cannot be changed; each instance has one redundancy
  type and search indexes use ZRS there. [TS-E108]
- Global Table replicates data tables across regions in active-passive or
  active-active mode with eventual consistency and last-write-wins conflict
  resolution. A user starts the disaster recovery switchover; the RPO is typically a
  few seconds but no latency guarantee is given. [TS-E109]
- Global Table limits: data tables only (no search or secondary indexes, no time
  series), up to 10 regions, listed China regions, not across countries, and table
  properties such as TTL cannot be modified. [TS-E110]
- Backup replication to another region is a separate, Cloud Backup based option.
  [TS-E102]

## Cost

**Status:** VERIFIED

Billing dimensions only (what is billed and how it scales). No unit prices or currency
amounts appear here: they change by Region and over time. Official page:
[Billing overview](https://www.alibabacloud.com/help/en/tablestore/table-store-service/product-overview/billing-overview/).

- Pay-as-you-go, hourly, starting after a table is created. Dimensions: data
  storage, reserved and pay-as-you-go read and write throughput, global table
  replication traffic, and outbound Internet traffic. [TS-E111]
- Throughput is measured in capacity units covering a row of up to 4 KB, rounded up;
  reserved throughput exists only on High-performance instances and on-demand is
  priced higher than reserved. [TS-E10] [TS-E11]
- Replication traffic is billed per replica to the source instance; inbound and
  internal traffic is free, failed requests still generate outbound traffic, and
  cross-region access counts as Internet access. [TS-E112]
- Features add dimensions: search index storage plus reserved and metered reads,
  secondary index storage and reads, extra versions and TTL timestamps consuming
  storage, billed reads for SQL scans, DTS and compute engines. [TS-E113]
- Tunnel Service is free but data reads while consuming are billed. [TS-E19]
- Backup adds Cloud Backup storage capacity, and restore adds Tablestore write
  throughput. KMS encryption adds KMS charges. [TS-E106] [TS-E125]

## Monitoring

**Status:** PARTIAL

- Monitoring runs through CloudMonitor: namespace `acs_ots_new`, dimensions
  `instanceName` and `tableName`, covering instances, tables, indexes, search indexes
  and time series tables. Console views are limited to 30 days. [TS-E114]
- Named metrics in the docs: InstanceCU, InstanceReadCU, InstanceCount2xxNumber
  (instance); AverageLatency, Count2xxNumber, ReadCU (table or index); VCU,
  RequestCount, InstanceNet in the CloudMonitor console. [TS-E115]
- Metrics are averages over a statistical period; throttling depends on
  instantaneous usage, so alert with headroom. [TS-E116]
- Alert rules are created in CloudMonitor and need an alert contact group; viewing
  needs the Tablestore and CloudMonitor read policies. [TS-E117] [TS-E114]
- Unconfirmed: provider-recommended alert thresholds are not stated on the pages
  reviewed. [TS-E117]

## Performance

**Status:** PARTIAL

- Retryable signals: OTSServerBusy, OTSPartitionUnavailable, OTSTimeout,
  OTSServerUnavailable (all 503) and OTSRowOperationConflict (409). OTSCapacityUnitExhausted
  (403) and OTSFlowControl (429) are listed as not retryable. [TS-E118]
- A new table starts with one partition; rebalancing can make partitions briefly
  unavailable (hundreds of milliseconds, seconds under load). Use SDK retry and ask
  support to pre-split. [TS-E119]
- A single partition key value cannot be split and should stay under 10 GB; a
  sequential partition key is a tail hotspot that splitting cannot fix. [TS-E02]
  [TS-E03]
- Request-size limits: 4 MB per write request, 200 rows per BatchWriteRow, 100 rows
  per BatchGetRow. [TS-E12]
- Throttling follows instantaneous usage, not the monitored average. [TS-E116]
- Unconfirmed: per-partition throughput ceilings were not found in the pages
  reviewed.

## IAM

**Status:** VERIFIED

- Identity-based RAM policies: explicit deny wins, then allow, then implicit deny.
  Built-in system policies are AliyunOTSFullAccess, AliyunOTSReadOnlyAccess and
  AliyunOTSWriteOnlyAccess. [TS-E120]
- Scope custom policies with `ots:` actions and ARNs of the form
  `acs:ots:<region>:<account>:instance/<name>/table/<table>`, plus conditions on source
  IP, time and secure transport. [TS-E120]
- Control plane: the RAM reference lists instance-management actions such as
  `ots:CreateInstance`, `ots:UpdateInstance`, `ots:DeleteInstance`,
  `ots:BindInstance2Vpc` and `ots:UpdateInstancePolicy`. [TS-E121]
- Other layers: Control Policies (deny only), Network ACLs and per-instance Instance
  Policies; deny wins on conflict. [TS-E122] [TS-E14]
- Cloud Backup uses the service-linked role AliyunServiceRoleForHbrOtsBackup.
  [TS-E103]

## Tagging

**Status:** PARTIAL

- Tags are key-value pairs on Tablestore instances (single or batch). Keys and values
  are at most 128 characters and cannot start with `aliyun` or `acs:`. Table-level
  tags are not described on that page. [TS-E123]
- Tag-based access control is only partly confirmed: the RAM reference defines the
  condition key `ots:tag`, but its action table lists no condition key against any
  action, so which actions honour it is unconfirmed. [TS-E124]

## Encryption

**Status:** VERIFIED

- Encryption at rest is disabled by default and is chosen per table at creation; it
  cannot be disabled afterwards and is limited to listed regions. [TS-E127] [TS-E125]
- Methods: KMS CMK-based (default CMK, no KMS instance purchase) or BYOK with a
  software-protected key you manage in KMS; KMS charges apply. [TS-E125]
- BYOK needs a RAM role that Tablestore assumes, allowing `kms:Decrypt` and
  `kms:GenerateDataKey`. [TS-E126]
- In transit all client-server traffic uses TLS; TLS 1.2 or later is recommended and
  the version can be restricted by policy. [TS-E128]
- Cloud Backup does not carry encryption settings into the vault. [TS-E20]

## Network isolation

**Status:** PARTIAL

- Each instance has public, VPC and classic network endpoints; Internet access has
  higher latency and the VPC endpoint is preferred. [TS-E129] [TS-E18]
- A Network ACL limits access to console plus bound VPC, or bound VPC only; the
  Custom type blocks Internet access for new instances until enabled. [TS-E129]
- A VPC must be bound to the instance first, and an Instance Policy and Network ACL
  must both be satisfied. [TS-E130]
- Control Policies can restrict creation to instances without public access, and the
  `ots:AllowInstanceInternetAccess` condition key exists. [TS-E122] [TS-E124]
- Unconfirmed: a separate private-link or service-gateway style endpoint service was
  not found in the pages reviewed; only VPC endpoints and ACLs are documented here.

## Audit logging

**Status:** VERIFIED

- Control plane: ActionTrail records Tablestore management events, keeps them 90
  days by default, and needs a trail to SLS, OSS or MaxCompute for longer retention.
  [TS-E131]
- Data plane: ActionTrail data events cover PutRow, GetRow, UpdateRow, DeleteRow,
  BatchWriteRow, GetRange, Search and SQLQuery, delivered to Simple Log Service.
  [TS-E132]
- The Tablestore Audit Log page uses Simple Log Service; default maximum retention is
  7 days and can be changed, and log alert rules are available. [TS-E133]
