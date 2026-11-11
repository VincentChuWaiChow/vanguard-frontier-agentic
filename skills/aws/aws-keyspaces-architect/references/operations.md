# Amazon Keyspaces operations

Operational coverage for Amazon Keyspaces (for Apache Cassandra), the serverless
CQL-compatible service. Every section uses the same heading and states one status.
Cite evidence rows from [the evidence ledger](evidence-ledger.md) as `[KS-Exx]`.
`tests/validate-nosql-operations.py` enforces this structure.

Statuses:

- `VERIFIED`: the provider documents the capability; every statement below cites a
  VERIFIED row.
- `PARTIAL`: some of it is documented and some is not, or it is limited (by tier,
  mode, Region or API); say which part is which.
- `UNSUPPORTED`: provider documentation states the capability does not exist. Cite it.
- `UNVERIFIED`: no official page settled it in this pass. Say what was searched.
- `NOT APPLICABLE`: the capability does not apply to this service. Say why.

Do not transfer Amazon DynamoDB behaviour to this service: Keyspaces has its own
capacity modes, CQL surface, quotas and restore rules, described only as documented here.

## Backup and restore

**Status:** PARTIAL

Point-in-time recovery (PITR) is documented in full. Searching the Keyspaces
documentation and AWS Backup search results found no separate on-demand snapshot
feature, so none is described here; that absence is not a claim that none exists.

- PITR gives continuous backups per table; restore to any second since PITR was enabled,
  within the last 35 days; a deleted table with PITR enabled remains restorable for 35
  days; PITR has no performance or availability impact on the base table. [KS-E01]
- The 35-day window cannot be changed, restore is only possible for times when PITR was
  enabled, and disabling then re-enabling PITR erases backup history. [KS-E02]
- A restore always writes a new table and never overwrites an existing one; capacity mode,
  throughput and PITR settings come from the source table at restore time and can be
  overridden. [KS-E03]
- Not restored: CDC streams, IAM policies, CloudWatch metrics and alarms, and tags (add
  tags with `WITH TAGS` in the CQL `RESTORE` statement). [KS-E04]
- Restore can take up to 20 minutes even for an empty table and hours or longer for
  large tables; record real restore times against your RTO. [KS-E05]
- A table encrypted with a customer managed key cannot be restored unless the key is
  accessible to Amazon Keyspaces. [KS-E23]

## Disaster recovery and replication

**Status:** VERIFIED

- Within a Region, data is replicated automatically three times across multiple
  Availability Zones. [KS-E08]
- Multi-Region replication is active-active, one replica table per Region, replicated
  asynchronously (typically under one second) with last-writer-wins conflict
  resolution. [KS-E06]
- Documented figures for multi-Region failover: RTO can be nearly zero, bounded by how
  fast the application detects failure and redirects traffic; RPO is typically
  single-digit seconds. Failover is application-driven (for example Amazon Route 53),
  with no secondary promotion step. [KS-E07]
- PITR restore of a multi-Region table requires PITR on all replicas. [KS-E05]

## Cost

**Status:** VERIFIED

Billing dimensions only (what is billed and how it scales). Never write unit prices
or currency amounts: they change by Region and over time. Link the official pricing
page and calculator.

- Throughput is billed in one of two capacity modes: on-demand per read and write
  request unit, or provisioned read and write capacity units per hour. [KS-E09] [KS-E10]
- Other dimensions: table storage by billable size, PITR backup storage by table size,
  data restored, TTL deletes, writes in every Region of a multi-Region keyspace, change
  data capture streams, data transfer out, and interface VPC endpoint cost billed
  separately under PrivateLink pricing. [KS-E09]
- Track storage cost drivers with the `BillableTableSizeInBytes` metric. [KS-E11]
- Official pricing page: <https://aws.amazon.com/keyspaces/pricing/>. [KS-E09]

## Monitoring

**Status:** VERIFIED

- Metrics go to Amazon CloudWatch in the `AWS/Cassandra` namespace, aggregated per
  minute. [KS-E11] [KS-E12]
- Key metrics: `ConsumedReadCapacityUnits`, `ConsumedWriteCapacityUnits`,
  `ReadThrottleEvents`, `WriteThrottleEvents`, `SystemErrors`, `UserErrors`,
  `SuccessfulRequestLatency` and, for multi-Region keyspaces only, `ReplicationLatency`.
  [KS-E11]
- Alarms notify through Amazon SNS; provisioned tables with Application Auto Scaling get
  alarm pairs created on your behalf. [KS-E13]

## Performance

**Status:** VERIFIED

- Insufficient provisioned capacity returns `Read_Timeout` or `Write_Timeout`, recorded
  in `ReadThrottleEvents` or `WriteThrottleEvents`; repeated errors can appear as driver
  exceptions such as `NoHostAvailableException`. [KS-E14]
- Storage partitions support up to 1000 WCU/WRU and 3000 RCU/RRU per second; exceeding
  that is reported by `StoragePartitionThroughputCapacityExceeded`, and the guidance is
  to revisit the data model to spread traffic across partitions. [KS-E15]
- Each client connection supports up to 3000 CQL requests per second
  (`PerConnectionRequestRateExceeded`); open more connections, noting replicated writes
  in multi-Region keyspaces also count. [KS-E16]

## IAM

**Status:** VERIFIED

- Actions use the `cassandra:` prefix on keyspace, table and stream resources; reading
  system tables needs `cassandra:Select` on the `system*` keyspace resource. [KS-E17]
- Resource-based policies are not supported. [KS-E18]
- AWS managed policies: `AmazonKeyspacesFullAccess`, `AmazonKeyspacesReadOnlyAccess`
  and `AmazonKeyspacesReadOnlyAccess_v2`. [KS-E19]
- Scope least privilege with resource ARNs and tag condition keys. [KS-E17] [KS-E20]

## Tagging

**Status:** VERIFIED

- Tags are supported on keyspaces, tables and streams (console, AWS CLI or CQL) and can
  be activated for cost allocation reports. [KS-E21]
- Tag-based access control is supported through `aws:RequestTag`, `aws:ResourceTag` and
  `aws:TagKeys` condition keys. [KS-E20]
- Tags are not restored by PITR; re-apply them on the restored table. [KS-E04]

## Encryption

**Status:** VERIFIED

- At rest: all table data is encrypted by default with AES-256, using an AWS owned key
  unless you choose a customer managed key. [KS-E22]
- A customer managed key can be set at creation or switched later without downtime and
  also protects table metadata and restorable backups; losing key access blocks restore.
  [KS-E22] [KS-E23]
- In transit: only TLS connections are accepted; TLS 1.2 is required and TLS 1.3
  recommended. [KS-E24] [KS-E28]

## Network isolation

**Status:** PARTIAL

Interface VPC endpoints are documented. No page retrieved describes disabling the
public service endpoint, so that control is not stated.

- Interface VPC endpoints (AWS PrivateLink) provide private connectivity without an
  internet gateway, NAT device, VPN or Direct Connect. [KS-E25]
- The endpoint security group needs an inbound rule for port 9142, and endpoint policies
  can restrict access. [KS-E25]

## Audit logging

**Status:** VERIFIED

- AWS CloudTrail captures DDL and DML API calls for Keyspaces. [KS-E26]
- DML (data plane) logging is opt-in: enable data plane API activity for the
  `Cassandra table` data event type on a trail. [KS-E27]
