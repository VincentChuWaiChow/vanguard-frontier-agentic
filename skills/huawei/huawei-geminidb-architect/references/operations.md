# Huawei Cloud GeminiDB operations

Operational coverage for Huawei Cloud GeminiDB, Cassandra API (cluster instances). Every section uses the
same heading and states one status. Cite evidence rows from
[the evidence ledger](evidence-ledger.md) as `[GDB-Exx]`.
`tests/validate-nosql-operations.py` enforces this structure.

Statuses:

- `VERIFIED`: the provider documents the capability; every statement below cites a
  VERIFIED row.
- `PARTIAL`: some of it is documented and some is not, or it is limited (by tier,
  mode, Region or API); say which part is which.
- `UNSUPPORTED`: provider documentation states the capability does not exist. Cite it.
- `UNVERIFIED`: no official page settled it in this pass. Say what was searched.
- `NOT APPLICABLE`: the capability does not apply to this service. Say why.

Scope: the GeminiDB Cassandra API. GeminiDB is multi-model and its APIs differ in backup, DR and
logging features, so nothing below is generalised from another API. The Mongo API has reached End
of Support [GDB-E02]; DDS is covered in `huawei-gaussdb-rds-dba`. Monitoring is Cloud Eye, audit is
CTS, access control is IAM and keys are DEW (KMS), each checked below.

## Backup and restore

**Status:** PARTIAL

Documented and usable: automated and manual backups. Gated or limited: incremental, PITR, table-level and cross-region backup.

- Automated and manual backups are supported; manual backups are full backups kept until deleted, even after the instance is deleted. Backups are OBS-hosted, cannot be downloaded or exported, and full plus incremental backups are enabled by default [GDB-E12] [GDB-E101].
- Automated retention defaults to 7 days with a policy-defined range of 1 to 3,660 days; a full backup starts within one hour of the window [GDB-E102]. CONFLICT: the generic GeminiDB Fault Recovery page says 1 to 35 days; this reference uses the Cassandra page and does not apply 1 to 35 days [GDB-E105].
- Incremental backups run every 15 minutes; differential type restores to a chosen point in time and PITR type to any point. The Cassandra page says enabling incremental or PITR backup needs a service ticket [GDB-E103] [GDB-E104].
- Restore always creates a new, independent instance (no in-place restore), with at least the original nodes and storage, same API, version and parameter group, and a new administrator password [GDB-E106] [GDB-E107] [GDB-E13]. The Mongo API differs and documents restore to an existing instance; that is not applied here [GDB-E108].
- Table-level backup needs a service ticket; database/table PITR restores up to 50 selected objects into a new instance [GDB-E110] [GDB-E109].
- Cross-region backup copies only automated full backups, needs a ticket, is classic-storage only and is unsupported with disk encryption [GDB-E111] [GDB-E112].
- A recycle bin (on by default, 7 days) lets you rebuild a deleted instance [GDB-E113]. Backup uploads cost CPU and bandwidth, so schedule off-peak windows [GDB-E114].

## Disaster recovery and replication

**Status:** PARTIAL

Documented: multi-AZ deployment, intra-region DR and cross-region dual-active DR. Both DR modes are OBT (open beta) and ticket-gated, and no RPO, RTO or availability percentage was found.

- Nodes can be spread evenly across three AZs for DR; the storage layer keeps three copies and the service lists N-1 fault tolerance [GDB-E120] [GDB-E121].
- Intra-region DR pairs one primary with one DR instance. It is in OBT, needs a service ticket, and the DR instance needs more than half the primary nodes and at least three [GDB-E115]. Lucene indexes, views, counter tables and TRUNCATE are not synchronised [GDB-E116].
- Failover is condition-driven rather than manual-only: the DR instance is promoted when a configured faulty-node percentage and optional replication-latency limit are met, and applications must then switch to the new connection address [GDB-E117].
- Cross-region dual-active DR gives bidirectional synchronisation between two instances in different regions [GDB-E118]. It is OBT, classic storage only, needs an empty target with matching or larger specifications and the same administrator password [GDB-E119].
- UNVERIFIED: no numeric RPO, RTO or SLA figure was found in the DR, FAQ or security pages searched; only the availability formula is given [GDB-E122]. The Mongo API SLA no longer applies [GDB-E02].

## Cost

**Status:** VERIFIED

Billing dimensions only. Official pricing: <https://www.huaweicloud.com/intl/en-us/pricing/index.html?tab=detail#/nosql> (GeminiDB Price Calculator, linked from the billing page).

- Billed dimensions are instance specifications (vCPUs and memory times node count), storage space, backup space above the free allowance, and public network bandwidth or traffic [GDB-E123] [GDB-E10].
- Free backup storage equals purchased storage; the excess is billed at OBS-based rates [GDB-E10].
- Modes are yearly/monthly (prepaid) and pay-per-use, and can be switched [GDB-E124] [GDB-E10].
- Storage autoscaling charges for the storage it adds, so cap the storage limit [GDB-E125].
- A public NAT gateway and EIP are separate billed resources, and an EIP kept after unbinding still bills if pay-per-use [GDB-E126].

## Monitoring

**Status:** VERIFIED

Monitoring is Cloud Eye.

- Cassandra metrics go to Cloud Eye in namespace `SYS.NoSQL`, dimensions `cassandra_cluster_id,cassandra_node_id`, 1-minute raw period; examples are `nosql005_disk_usage`, `cassandra001_cpu_usage`, `cassandra002_mem_usage`, `cassandra015_read_latency`, `cassandra016_write_latency` and `cassandra037_pending_write` [GDB-E127].
- Recommended alarms: storage, CPU and memory above 80% for 3 periods; read or write latency above 900 ms; pending read or write tasks above 3,000; all Major [GDB-E128].
- Event monitoring in the same namespace covers `NoSQLFaultyDBInstance`, `NoSQLFaultyDBNode`, `NoSQLPrimaryStandbySwitched` and backup, resize and node failures; alarm on backup-failure events [GDB-E129].
- BigKey and HotKey detections surface as events and can be alarmed in Cloud Eye [GDB-E130].

## Performance

**Status:** PARTIAL

Documented: data-model limits and signals. Not found: a throttling error code or provisioned-throughput model (capacity is node-based).

- Partition keys must spread load; a partition over 100,000 rows or 100 MB is a BigKey, and one key above 100,000 accesses per minute is a HotKey [GDB-E05] [GDB-E130].
- Keep rows within 64 KB (key plus value), avoid BLOBs, stay at or below 50 nodes, and prefer TTL over mass deletes to avoid tombstones [GDB-E04] [GDB-E07].
- Size for N/2 spare nodes and per-node CPU at or below 50% under typical load [GDB-E08]. Writes are strongly consistent but transactions are unsupported [GDB-E06].
- Slow query logging defaults to 500 ms (`slow_query_log_timeout_in_ms`) [GDB-E131].
- Backups and SSL both add load: backup upload raises CPU 5% to 15% and per-node upload is 20 to 70 MB/s; SSL raises CPU [GDB-E114] [GDB-E132].
- UNVERIFIED: no documented throttling or rate-limit error codes for Cassandra were found in the pages searched.

## IAM

**Status:** PARTIAL

Control plane is documented; data-plane (CQL role) authorization is not.

- IAM supports role/policy-based and identity policy-based authorization (not interoperable; identity policies recommended) [GDB-E133].
- System policies are `GeminiDB FullAccess` and `GeminiDB ReadOnlyAccess`, project-level [GDB-E09]; the identity-policy equivalent includes `GeminiDBReadOnlyPolicy` with the `gaussdbfornosql` action prefix [GDB-E135].
- Least privilege: grant ReadOnlyAccess for review and use a custom Deny on `nosql:instance:delete` alongside FullAccess [GDB-E134].
- Autoscaling needs extra IAM agency actions or the Security Administrator role, so grant them deliberately [GDB-E136].
- Data plane: the administrator account is `rwuser`; per-user CQL roles were not documented in pages read [GDB-E137].

## Tagging

**Status:** PARTIAL

- Instances support tags via Tag Management Service: up to 20 per instance, one value per key [GDB-E138].
- Key up to 128 and value up to 255 characters; settable at creation and searchable [GDB-E139].
- UNVERIFIED: tag-based access control for GeminiDB was not found; identity policies mention only service-specific condition keys [GDB-E140].

## Encryption

**Status:** PARTIAL

Key service is DEW (KMS).

- Disk encryption is chosen at purchase using a key from Data Encryption Workshop; status and key cannot change afterwards [GDB-E141].
- The key must not be disabled, deleted or frozen while in use or the database becomes unavailable; shared keys need an agency [GDB-E142].
- Disk encryption excludes cross-region backups [GDB-E112].
- In transit, SSL is optional per instance and supports TLS 1.2 or later only [GDB-E143].
- UNVERIFIED: whether encryption at rest is on by default in every console flow, and key-type restrictions, were not stated on the pages read.

## Network isolation

**Status:** PARTIAL

- Instances are private-network-only by default; public access needs an EIP on a node or a public NAT gateway address [GDB-E144].
- Security groups restrict inbound TCP; one security group can be associated with only one instance [GDB-E145].
- Prefer a public NAT gateway (one-way DNAT) to a node EIP, available for classic storage only [GDB-E146].
- Best practice: no Internet EIP, internal network behind firewalls, non-default port [GDB-E147].
- UNVERIFIED: no VPC endpoint (VPCEP) path for Cassandra was found [GDB-E148].

## Audit logging

**Status:** PARTIAL

Audit is CTS for the control plane.

- CTS records Cassandra management operations such as create, delete, backup, tag and parameter changes [GDB-E149]; the console retains seven days [GDB-E150].
- The security overview mentions management and data traces, but the Cassandra CTS page lists management operations only [GDB-E151].
- Best practices say audit, error and slow query logs can be sent to LTS, but no Cassandra procedure for enabling audit logs was found [GDB-E152].
