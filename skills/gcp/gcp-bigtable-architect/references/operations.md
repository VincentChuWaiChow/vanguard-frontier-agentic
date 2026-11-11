# Cloud Bigtable operations

Operational coverage for Cloud Bigtable (Enterprise and Enterprise Plus editions, Data and Admin APIs). Every section uses the
same heading and states one status. Cite evidence rows from
[the evidence ledger](evidence-ledger.md) as `[BT-Exx]`.
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

- Backups are per table and stored on one cluster; automated backup creates a daily backup on each cluster, default retention 7 days, up to 90 days for an automated policy; on-demand backups keep up to 90 days (Enterprise) or 365 days (Enterprise Plus); copies keep at most 30 days; up to 150 backups per table per cluster [BT-E14] [BT-E104] [BT-E103].
- Restore always creates a new table (never in place) and then requests are re-routed to it; the backup remains after restore [BT-E14].
- Hot backups (SSD instances only, not tiered-storage tables) speed up restoration; a hot backup can become standard but not the reverse; a copy is always a standard backup and a copy of a copy is not possible [BT-E101].
- Not restored: the new table does not inherit the source table's garbage collection policies, so set them before new writes [BT-E102]. A backup also excludes data not yet replicated to its cluster [BT-E103].
- CMEK-protected backups are pinned to the key version at creation; that version must be enabled to restore, and the destination instance must be CMEK-protected [BT-E106].
- Backups cannot be configured as CMEK resources separately from their cluster (see Encryption) [BT-E125].

## Disaster recovery and replication

**Status:** PARTIAL

- Replication keeps a separate copy of data in each cluster; an instance can span up to 8 regions with at most one cluster per zone [BT-E107].
- Failover is manual with single-cluster routing and automatic with multi-cluster routing to the nearest available cluster [BT-E108] [BT-E06].
- Replication is eventually consistent (typically seconds to minutes) and reads can be stale after failover until replication completes; conflicts resolve by last write wins [BT-E08] [BT-E109].
- Single-row transactions cannot be used with multi-cluster routing, which is the high-availability choice [BT-E06].
- Cross-region recovery from backup copies is documented [BT-E14].
- Not settled: no numeric RPO or RTO was found on the replication, failover or backup pages retrieved; the SLA page (cloud.google.com/bigtable/sla) was not cited, so no availability figure is stated here.

## Cost

**Status:** VERIFIED

Billing dimensions only (what is billed and how it scales). Pricing page: <https://cloud.google.com/bigtable/pricing> (the Google Cloud pricing calculator is linked from it).

- Node compute is billed hourly on the maximum nodes in the hour, one-hour minimum per node; the node type depends on the edition [BT-E111].
- Storage is billed on physical table size (SSD, HDD or infrequent access tiers); every cluster holds its own copy, so replicated instances multiply storage [BT-E110] [BT-E111].
- Network: bandwidth is billed, including replication writes to clusters in other regions, and restore or copy across regions [BT-E111] [BT-E105].
- Backups: backup storage is billed (standard incrementally, hot as the full logical copy); create, copy and restore operations themselves are not charged [BT-E105] [BT-E113].
- Data Boost is billed in serverless processing unit seconds with a per-request minimum and no extra storage or network fee [BT-E112].
- Tiered storage SSD-to-infrequent-access data movement is billed on the compaction SKU [BT-E113].

## Monitoring

**Status:** VERIFIED

- Bigtable exports metrics to Cloud Monitoring, viewable in the console, Metrics Explorer and the Monitoring API; resource type is Cloud Bigtable Cluster [BT-E114].
- Key metrics: average CPU utilization, hottest-node CPU, CPU by app profile/method/table, storage utilization, replication latency, system error rate, read and write latency [BT-E115].
- The documented alert is on `bigtable.googleapis.com/cluster/storage_utilization` at about 70% [BT-E114] [BT-E116].
- Exceeding the per-node storage hard limit makes writes to all clusters in the instance fail [BT-E116].

## Performance

**Status:** VERIFIED

- Keep CPU below 60% for latency-sensitive workloads and below 90% for throughput-oriented ones; autoscaling is recommended [BT-E117] [BT-E12].
- Hot tablets (tablets using a disproportionate share of node CPU) cause latency and replication delay; a list of up to 50 per cluster is available [BT-E118].
- Hotspots from schema design and sequential keys concentrate load and can affect other tables [BT-E119] [BT-E03].
- Cold starts and low QPS raise latency; keep connections warm [BT-E120].
- Row, cell and storage limits drive sizing [BT-E10].

## IAM

**Status:** VERIFIED

- IAM applies at project, instance, cluster, table, backup, authorized view and continuous materialized view levels, with downward inheritance; allUsers and allAuthenticatedUsers cannot be granted [BT-E121].
- Predefined roles: `roles/bigtable.admin`, `roles/bigtable.user`, `roles/bigtable.reader`, `roles/bigtable.viewer` [BT-E13].
- Least privilege: grant at table, backup or authorized view level (write without read, deny restore or delete, read one view) [BT-E122] [BT-E13].

## Tagging

**Status:** VERIFIED

- Resource tags can be attached to Bigtable instances (needs `roles/bigtable.admin` plus tag permissions) and used in conditional IAM allow or deny policies [BT-E123].
- Labels remain the mechanism for grouping instances for automation and billing; tags and labels work independently [BT-E123].

## Encryption

**Status:** PARTIAL

- At rest: encrypted by default with Google default encryption; CMEK through Cloud KMS (including Autokey) is optional [BT-E124].
- CMEK is cluster-level only, key in the cluster's region, immutable once set, no conversion in either direction; inaccessible keys for over 30 days lead to automatic deletion [BT-E125].
- Disabling a key cascades to disable all clusters; disable all keys together [BT-E126].
- Some data stays under default encryption (range-boundary row keys, some timestamps, debug data) [BT-E127].
- In transit: no Bigtable-specific page on transport encryption was retrieved; not asserted here.

## Network isolation

**Status:** PARTIAL

- VPC Service Controls: Bigtable is supported (GA) and perimeters restricting `bigtable.googleapis.com` also restrict the admin service [BT-E128].
- Private Service Connect backends support the global Bigtable APIs [BT-E129].
- Private Google Access can reach Google APIs from private-IP hosts and combines with a VPC Service Controls perimeter [BT-E130].
- Not settled: Bigtable has no documented per-instance toggle to disable public endpoints; none was found.

## Audit logging

**Status:** VERIFIED

- Cloud Audit Logs cover Bigtable under service name `bigtable.googleapis.com`; ADMIN_WRITE methods produce Admin Activity logs [BT-E131].
- Data-plane operations (DATA_READ, DATA_WRITE, ADMIN_READ methods such as readRows and mutateRows) produce Data Access logs, which must be enabled and have cost implications [BT-E131].
