# Amazon DocumentDB operations

Operational coverage for Amazon DocumentDB (with MongoDB compatibility), instance-based
clusters. Elastic clusters are mentioned only where the documentation distinguishes
them. Every section uses the same heading and states one status. Cite evidence rows from
[the evidence ledger](evidence-ledger.md) as `[DOCDB-Exx]`.
`tests/validate-nosql-operations.py` enforces this structure.

Statuses:

- `VERIFIED`: the provider documents the capability; every statement below cites a
  VERIFIED row.
- `PARTIAL`: some of it is documented and some is not, or it is limited (by tier,
  mode, Region or API); say which part is which.
- `UNSUPPORTED`: provider documentation states the capability does not exist. Cite it.
- `UNVERIFIED`: no official page settled it in this pass. Say what was searched.
- `NOT APPLICABLE`: the capability does not apply to this service. Say why.

Do not transfer Amazon DynamoDB behaviour to this service: DocumentDB is a cluster of
instances over a shared storage volume, not a request-unit or capacity-unit table service.

## Backup and restore

**Status:** VERIFIED

- Continuous backup to service-managed Amazon S3 buckets with a retention period of 1 to
  35 days; you have no access to the backup files, and the backup process does not
  impact cluster performance. [DOCDB-E01]
- Automatic snapshots cannot be disabled, cannot be manually deleted, are removed with
  the cluster and cannot be shared directly (copy first, then share the copy);
  restore to any point within the retention window. [DOCDB-E02]
- Manual snapshots are on demand, full, survive cluster deletion, are capped at 100 per
  Region and can be shared. [DOCDB-E02]
- Every restore (snapshot or point in time) creates a new cluster, never an existing
  one. An encrypted snapshot cannot be restored to an unencrypted cluster. [DOCDB-E03]
- A point-in-time restore keeps the source configuration except the cluster parameter
  group, which resets to the default group; set it again after the cluster is
  available. With the AWS CLI, instances are added separately. [DOCDB-E04]
- Backup storage is billed from continuous backups and manual snapshots outside the
  retention window; track `TotalBackupStorageBilled`, `SnapshotStorageUsed` and
  `BackupRetentionPeriodStorageUsed`. [DOCDB-E05]

## Disaster recovery and replication

**Status:** PARTIAL

The in-Region mechanics and the global cluster topology are documented. The two pages
that state the global cluster RTO word it differently, so the figure is not a guarantee.

- In-Region high availability uses one primary and up to 15 replicas spread across at
  least two Availability Zones; replicas are eventually consistent. [DOCDB-E10]
- In-Region failover is automatic, promotes a replica by failover tier and typically
  completes within 30 seconds; it needs at least two instances and the page applies to
  instance-based clusters, not elastic or global clusters. [DOCDB-E09]
- Global clusters add up to 10 read-only secondary Regions with asynchronous replication;
  unsupported on db.t3, db.t4g and db.r4 instances. [DOCDB-E06]
- Documented global RPO is typically seconds and depends on lag; the RTO wording
  conflicts between pages (under a minute versus minutes). [DOCDB-E07] [DOCDB-E08]
- Managed global failover requires matching engine versions and unreplicated data may
  be missing on the new primary; check `GlobalClusterReplicationLag` first. [DOCDB-E11]

## Cost

**Status:** VERIFIED

Billing dimensions only (what is billed and how it scales). Never write unit prices
or currency amounts: they change by Region and over time. Link the official pricing
page and calculator.

- Billed dimensions: on-demand instance hours, database storage, I/O (standard or
  I/O-Optimized configuration), backup storage beyond the free allotment, serverless
  capacity units, replicated write I/O to each global secondary Region, and data
  transfer such as cross-AZ traffic from applications. [DOCDB-E12]
- Backup storage scales with retention days and the manual snapshots you keep outside
  the window; free backup storage equals one day of cluster volume. [DOCDB-E05]
- Official pricing page: <https://aws.amazon.com/documentdb/pricing/>. [DOCDB-E12]

## Monitoring

**Status:** VERIFIED

- Metrics are published to Amazon CloudWatch in the `AWS/DocDB` namespace and the metric
  tables apply to instance-based and elastic clusters. [DOCDB-E13]
- Named metrics: `CPUUtilization`, `FreeableMemory`, `DatabaseConnections`,
  `DatabaseCursorsTimedOut`, `VolumeBytesUsed`, `DBInstanceReplicaLag` and
  `DBClusterReplicaLagMaximum`. [DOCDB-E13]
- Recommended practice is to monitor memory, CPU, connections and storage and alarm as
  usage approaches capacity. [DOCDB-E14]
- Global clusters expose `GlobalClusterReplicationLag` for secondary clusters. [DOCDB-E11]

## Performance

**Status:** PARTIAL

Documented: low-memory throttling metrics and per-instance quotas. Not found in the
pages retrieved: request-level throttling error codes and hot-key or partition guidance
comparable to a key-partitioned service, so none is stated here.

- Requests throttled by low available memory are reported by
  `LowMemNumOperationsThrottled`, `LowMemThrottleQueueDepth` and
  `LowMemThrottleMaxQueueDepth`. [DOCDB-E15]
- Per-instance-type connection, cursor and open-transaction quotas, and cluster quotas
  (for example 15 replicas), are tabulated on the quotas page. [DOCDB-E16]
- Track `DatabaseConnections`, `DatabaseCursorsTimedOut` and `FreeableMemory` against
  those quotas. [DOCDB-E13] [DOCDB-E14]

## IAM

**Status:** VERIFIED

- Control plane: IAM identity-based policies using shared `rds:` actions with only
  partial resource-level permission support. [DOCDB-E19]
- Resource-based policies are not supported. [DOCDB-E17]
- AWS managed policies: `AmazonDocDBFullAccess`, `AmazonDocDBReadOnlyAccess`,
  `AmazonDocDBConsoleFullAccess`, `AmazonDocDBElasticReadOnlyAccess` and
  `AmazonDocDBElasticFullAccess`. [DOCDB-E18]
- Data plane: database users with role-based access control and built-in roles such as
  `read` and `readWrite`; review users created before March 26, 2020, which received
  broad roles. [DOCDB-E20]
- IAM identity authentication for database users is password-less, available only for
  instance-based version 5.0, and excludes the primary user. [DOCDB-E21]

## Tagging

**Status:** VERIFIED

- Tags are supported on clusters, instances, snapshots, cluster snapshots, parameter
  groups, cluster parameter groups, security groups and subnet groups; at most 10 per
  resource. [DOCDB-E22]
- Tags can drive IAM policy conditions and cost tracking; a tag on a snapshot is not
  reflected in the bill. [DOCDB-E22]
- Tag-based access control (ABAC) is only partially supported for instance-based
  clusters but supported for elastic clusters. [DOCDB-E17]

## Encryption

**Status:** VERIFIED

- At rest: AES-256 with AWS KMS keys covering storage, indexes, logs, automated backups
  and snapshots. [DOCDB-E23]
- Console-created clusters default to encrypted, CLI-created clusters default to
  unencrypted; the setting and the KMS key cannot change after creation; only symmetric
  KMS keys are supported. [DOCDB-E24]
- In transit: TLS is enabled by default for new clusters and is governed by the `tls`
  parameter in a non-default cluster parameter group. [DOCDB-E25]

## Network isolation

**Status:** VERIFIED

- Clusters are deployed within an Amazon VPC; access is from the same VPC or from other
  VPCs through peering. [DOCDB-E26]
- An interface VPC endpoint (AWS PrivateLink) for the DocumentDB API keeps management
  calls off the internet; instance-based clusters do not require it and elastic clusters
  do not support it. [DOCDB-E27]

## Audit logging

**Status:** VERIFIED

- Control-plane calls are recorded by AWS CloudTrail, logged as Amazon RDS API calls
  because of shared technology. [DOCDB-E28]
- Data-plane auditing is off by default and opt-in through the `audit_logs` parameter;
  it records DDL, DML, authentication, authorization and user-management events to
  CloudWatch Logs, and CloudWatch Logs usage is charged. [DOCDB-E29]
