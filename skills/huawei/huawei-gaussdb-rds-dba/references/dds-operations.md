# Huawei Cloud DDS operations

Operational coverage for Huawei Cloud Document Database Service (DDS), the MongoDB-compatible
service, covering cluster and replica set instances (single node is noted only where it differs).
Every section uses the same heading and states one status. Cite evidence rows from
[the evidence ledger](evidence-ledger.md) as `[HW-DDS-Exx]`.
`tests/validate-nosql-operations.py` enforces this structure.

Statuses:

- `VERIFIED`: the provider documents the capability; every statement below cites a
  VERIFIED row.
- `PARTIAL`: some of it is documented and some is not, or it is limited (by tier,
  mode, Region or API); say which part is which.
- `UNSUPPORTED`: provider documentation states the capability does not exist. Cite it.
- `UNVERIFIED`: no official page settled it in this pass. Say what was searched.
- `NOT APPLICABLE`: the capability does not apply to this service. Say why.

Scope: DDS only. GeminiDB (including its End-of-Support Mongo API) is covered in
`huawei-geminidb-architect`, and GeminiDB behaviour is not assumed here. Monitoring is Cloud Eye,
audit is CTS, access control is IAM and keys are DEW (KMS), each checked below.

## Backup and restore

**Status:** PARTIAL

Backup and restore are broadly documented; the cross-region backup policy page itself was not retrieved and cross-region PITR is OBT.

- Automated backup is on by default and uploads to OBS; replica sets also take incremental backups (every 5 minutes), while incremental is on by default except under 4 vCPUs and needs version 3.4+ (replica set) or 4.0+ (cluster) [HW-DDS-E101] [HW-DDS-E104].
- Retention is 1 to 732 days (default 7); manual backups are kept until deleted, even after the instance is deleted, and are billed at OBS rates [HW-DDS-E102] [HW-DDS-E113].
- Methods are physical, snapshot (through CBR, billed as a backup vault) and logical (single node); incremental backup pauses after operations like rename, collMod and user or role changes until the next full backup [HW-DDS-E103] [HW-DDS-E105].
- Unlike GeminiDB Cassandra, DDS restores in place: restoring to the original instance overwrites data and makes the instance unavailable; restoring to a new instance gives an independent instance with the same type, version and engine [HW-DDS-E106] [HW-DDS-E107].
- Point-in-time restore: replica sets (4.0+) to a new or original instance, with oplog above 250 GB/h possibly leaving gaps; clusters need automated plus incremental backup; database or table PITR writes recovered tables as `_bak_timestamp` copies instead of overwriting [HW-DDS-E108] [HW-DDS-E109] [HW-DDS-E111].
- Cross-region backups restore to a new instance in another region [HW-DDS-E114]; cross-region PITR is OBT and needs a service ticket [HW-DDS-E110].
- A recycle bin (default 7 days, cannot be disabled) lets you rebuild a deleted instance [HW-DDS-E112]. Backup files can be downloaded and, for 3.4 and 4.0 replica sets, restored to an on-premises database [HW-DDS-E115] [HW-DDS-E116].

## Disaster recovery and replication

**Status:** PARTIAL

Documented: multi-AZ, automatic failover, read replicas and cross-region backup. No RPO, RTO or availability percentage was found.

- Nodes can be spread across three AZs; a multi-AZ cluster fails over automatically if the primary's AZ fails, but 3-AZ deployment is not available in every Region [HW-DDS-E117] [HW-DDS-E118].
- Failover is automatic: a standby is promoted within 30 seconds of a primary failure, for cluster and replica set instances only (not single node), so clients must reconnect automatically [HW-DDS-E119] [HW-DDS-E120].
- Replication is asynchronous through the oplog, and the lag has no formula; watch the replication lag metric [HW-DDS-E121] [HW-DDS-E122].
- Read replicas sync from the primary and are never elected primary; only cluster read replicas provide HA among themselves [HW-DDS-E123].
- Cross-region DR relies on cross-region backups restored to a new instance [HW-DDS-E114].
- UNVERIFIED: no numeric RPO, RTO or SLA percentage, and no cross-region active-active feature, was found in the pages searched [HW-DDS-E124].

## Cost

**Status:** VERIFIED

Billing dimensions only. Official pricing: <https://www.huaweicloud.com/intl/en-us/pricing/index.html?tab=detail#/dds> (DDS Pricing Details, linked from the billing pages).

- Billed items are instance specifications (vCPUs, memory, node count), storage space, backup space above the free quota, EIP bandwidth and the Monitoring by Seconds function [HW-DDS-E125] [HW-DDS-E126].
- Free backup space equals purchased storage; a frozen instance loses it [HW-DDS-E128].
- Cross-region backup adds a storage charge and a cross-region traffic charge [HW-DDS-E127].
- Snapshot backup through CBR removes the free backup space and bills a backup vault [HW-DDS-E103].
- EIPs bill by bandwidth, traffic or shared bandwidth [HW-DDS-E129]. Log reporting to LTS and the audit policy both incur fees [HW-DDS-E130] [HW-DDS-E167].

## Monitoring

**Status:** VERIFIED

Monitoring is Cloud Eye.

- Metrics use namespace `SYS.DDS` with dimensions `mongodb_instance_id,mongodb_node_id`; examples are `mongo031_cpu_usage`, `mongo032_mem_usage`, `mongo035_disk_usage`, `mongo007_connections_usage`, `mongo026_repl_lag` and `mongo025_repl_oplog_win` [HW-DDS-E131].
- The default interval is 1 minute and can be 5 seconds with the billed Monitoring by Seconds function [HW-DDS-E132].
- Recommended Major alarms: connections, CPU and disk usage above 80% for three periods, memory above 90%, and disk read or write time at or above 0.1 s [HW-DDS-E133].
- Event monitoring in `SYS.DDS` includes `planDeleteKmsKey`, which matters for encrypted instances [HW-DDS-E134].
- Alarm on oplog window and replication lag because PITR coverage and secondaries depend on them [HW-DDS-E135].

## Performance

**Status:** PARTIAL

Documented: capacity, connection, replication and scaling limits. Not found: throttling error codes or a provisioned-throughput model (capacity is instance-class based).

- Shard key choice is permanent; a monotonic key concentrates inserts and hashed sharding spreads data [HW-DDS-E01].
- Storage can only scale up, a nearly full instance goes read-only, and one shard is limited to 5000 GB [HW-DDS-E136].
- Changing instance class can cause one or two switchovers, each interrupting connections for up to 30 seconds [HW-DDS-E137].
- Connection count is a driver of load; cap it and watch connection usage [HW-DDS-E140].
- Oplog is 10% of storage by default and bounds replication and PITR [HW-DDS-E135]. Incremental backup loads the hidden node [HW-DDS-E139]. SSL costs about 5% throughput [HW-DDS-E138].
- UNVERIFIED: no documented rate-limit or throttling error codes were found in the pages searched.

## IAM

**Status:** VERIFIED

- IAM supports role/policy-based and identity policy-based authorization (not interoperable; identity policies recommended) [HW-DDS-E141].
- System policies are `DDS FullAccess`, `DDS ReadOnlyAccess`, `DDS ManageAccess` and the `DDS Administrator` role [HW-DDS-E02]; the identity-policy equivalent includes `DDSReadOnlyPolicy` [HW-DDS-E144].
- Least privilege: prefer `DDS ManageAccess` over FullAccess when deletion is not needed [HW-DDS-E02]; use `dds:Encrypted` and `dds:BackupEnabled` condition keys to require encryption or backups [HW-DDS-E143]. Creating an encrypted instance needs KMS Administrator [HW-DDS-E142].
- Critical operation protection re-authenticates risky actions such as deleting an instance [HW-DDS-E147].
- Data plane is separate from IAM: the default administrator is `rwuser`, built-in roles `read` and `readWrite` are available to clients, plus user-defined roles [HW-DDS-E145] [HW-DDS-E146].

## Tagging

**Status:** PARTIAL

- Instances can be tagged during or after creation [HW-DDS-E148]; up to 20 tags per instance with one value per key [HW-DDS-E149].
- UNVERIFIED: tag-based access control; the DDS IAM pages list only `dds:Encrypted` and `dds:BackupEnabled` as service condition keys [HW-DDS-E150].

## Encryption

**Status:** PARTIAL

Key service is DEW (KMS).

- Disk encryption is chosen when buying the instance with a customer key from Data Encryption Workshop; status and key cannot change afterwards [HW-DDS-E151]. Only AES_256 is supported [HW-DDS-E152].
- Disk encryption does not cover backup data in OBS; backup encryption needs a service ticket, and enabling both makes data unrecoverable [HW-DDS-E153] [HW-DDS-E154].
- A disabled, deleted or frozen key makes the database unavailable, so guard the key [HW-DDS-E168].
- In transit, enabling SSL restricts connections to TLS 1.2 and restarts the instance [HW-DDS-E155].
- UNVERIFIED: whether encryption at rest is on by default; the buy page offers Disabled or Enabled without stating the default.

## Network isolation

**Status:** PARTIAL

- Instances are private-network-only by default; public access uses an EIP or a public gateway [HW-DDS-E156].
- A public gateway (NAT) uses a one-way DNAT rule and is safer than a direct EIP [HW-DDS-E157]. Cluster EIPs bind only to mongos nodes [HW-DDS-E159].
- Security groups restrict access; one instance has one security group by default [HW-DDS-E158].
- Best practice: no Internet EIP, firewalls in front, non-default port [HW-DDS-E160].
- UNVERIFIED: no VPC endpoint (VPCEP) path for DDS was found [HW-DDS-E161].

## Audit logging

**Status:** VERIFIED

Audit is CTS for the control plane, with a separate database audit log for the data plane.

- CTS records DDS operations such as restores for query, audit and backtracking [HW-DDS-E162]; the console keeps seven days [HW-DDS-E163].
- DDS audit logs record operations on databases and collections including reads and writes; they are disabled by default and stored in OBS [HW-DDS-E164].
- Audit logging affects performance and incurs fees; keep logs more than 180 days [HW-DDS-E165] [HW-DDS-E167].
- Log reporting sends audit, error and slow query logs to LTS [HW-DDS-E166].
