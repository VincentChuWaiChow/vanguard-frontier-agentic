# Oracle NoSQL Database Cloud Service operations

Operational coverage for Oracle NoSQL Database Cloud Service (NDCS) on OCI. This is
not Autonomous Database and not on-premises Oracle NoSQL Database. Every section uses the
same heading and states one status. Cite evidence rows from
[the evidence ledger](evidence-ledger.md) as `[OCI-NOSQL-Exx]`.
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

- Documented (VERIFIED): backup and restore are the customer's job. Oracle supplies
  the NoSQL Database Migrator, which can export a table to OCI Object Storage and
  restore a table from those files. [OCI-NOSQL-E30] [OCI-NOSQL-E101]
- Oracle's own example copies a table to a JSON file in Object Storage for periodic
  backups, run from Cloud Shell. The schedule is yours to build. [OCI-NOSQL-E102]
- Restore target: the migrator writes into a sink table. If the sink table exists with
  a different schema the migration fails; with the same schema, rows with matching
  primary keys are overwritten when the overwrite setting is true. [OCI-NOSQL-E102]
- Not documented (UNVERIFIED): any native on-demand, scheduled or point-in-time
  backup, retention limit, or statement of what a restore leaves out (indexes, TTL,
  IAM policies, tags). Searched the Features, Plan, Global Active, Monitor, Policies,
  migrator, What's New and release-notes pages. [OCI-NOSQL-E126]
- Design consequence: treat any exported file set as the only recovery copy, secure
  the Object Storage bucket, and test a restore into a scratch table.
  Global Active replicas are redundancy, not backups: a bad write replicates. [OCI-NOSQL-E28]

## Disaster recovery and replication

**Status:** PARTIAL

- Inside a region, data is stored across multiple Availability Domains, or Fault
  Domains in single-AD regions, so a single AD or FD loss leaves data reachable.
  [OCI-NOSQL-E103]
- Cross-region: Global Active tables replicate active-active and asynchronously;
  concurrent conflicts resolve by latest timestamp. [OCI-NOSQL-E09]
- Failover mode: when a region is unavailable, the application requests are redirected
  to a region holding a replica, where reads and writes continue; the failed
  region resynchronizes on return. No service-driven automatic failover switch is
  documented, so the redirect is application or DNS logic you own. [OCI-NOSQL-E104]
- ACID transactions and ABSOLUTE consistency are local to one region; data replicated
  from another region is read as eventually consistent. [OCI-NOSQL-E10]
- Lag is observable (Replica Lag metric) but the pages searched state no RPO or RTO
  figure (UNVERIFIED); do not promise one. [OCI-NOSQL-E105] [OCI-NOSQL-E114] [OCI-NOSQL-E126]
- Limits: a dedicated hosted environment does not support Global Active tables, and a
  schema change after replicas exist needs all replicas dropped first.
  [OCI-NOSQL-E30] [OCI-NOSQL-E26]

## Cost

**Status:** VERIFIED

Billing dimensions only (what is billed and how it scales). Official pricing page:
<https://www.oracle.com/database/nosql/pricing/> and the Cost Estimator on the Oracle
Cloud website. Never write unit prices or currency amounts here.

- Capacity modes: Provisioned Capacity (fixed read/write throughput and storage) or
  On-Demand Capacity (Oracle manages read/write capacity, storage still limited); a
  dedicated hosted environment is a third, flat-fee option. [OCI-NOSQL-E01] [OCI-NOSQL-E107]
- Dimensions: read units, write units and GB of storage per month. A write unit covers
  up to 1 KB per second, index updates consume write units, and ABSOLUTE reads consume
  twice the read units of EVENTUAL. [OCI-NOSQL-E15] [OCI-NOSQL-E03] [OCI-NOSQL-E108]
- Hidden consumers: conditional puts, query-based updates and deletes consume read
  units as well as write units. [OCI-NOSQL-E25]
- Replication: a Global Active table adds a second component, incoming replicated
  writes at each regional replica, charged on writes consumed. [OCI-NOSQL-E106]
  [OCI-NOSQL-E25]
- Dedicated hosted environment: one monthly price for the whole environment instead
  of per-unit charges. [OCI-NOSQL-E107]
- Autoscale or on-demand invoices reflect actual consumption; log the units returned
  by each API call to reconcile. [OCI-NOSQL-E108]
- Guardrails: compartment quotas on read units, write units and table size, and
  per-table limits (including at most four capacity decreases per 24 hours).
  [OCI-NOSQL-E116] [OCI-NOSQL-E23]

## Monitoring

**Status:** VERIFIED

- Service: OCI Monitoring; namespace `oci_nosql`; dimensions `resourceId` and
  `tableName`. [OCI-NOSQL-E110]
- Metrics: `ReadUnits`, `WriteUnits`, `StorageGB`, `ReadThrottleCount`,
  `WriteThrottleCount`, `StorageThrottleCount`, `MaxShardSizeUsagePercent`, and
  Replica Lag (extra dimension `replica`, milliseconds). [OCI-NOSQL-E110] [OCI-NOSQL-E114]
- Recommended alarm: `MaxShardSizeUsagePercent` at about 90 percent, notified through
  OCI Notifications. Alarm on the throttle counts too (an inference from the
  documented meaning of those metrics). [OCI-NOSQL-E111] [OCI-NOSQL-E112]
- Events: table actions (create, drop, alter, index, replica, compartment move) emit
  OCI Events for rules and notifications. [OCI-NOSQL-E115]
- Caveat: StorageGB is truncated and refreshed with delay; under 1 GB can show as 0.
  [OCI-NOSQL-E110]

## Performance

**Status:** VERIFIED

- Throttling signal: a throttling exception usually means the provisioned throughput
  was exceeded; if frequent, raise Read Units, Write Units or storage. Watch the three
  throttle-count metrics. [OCI-NOSQL-E112] [OCI-NOSQL-E110]
- Hot shards: data is sharded on the shard key. Low-cardinality keys cause hot
  shards. [OCI-NOSQL-E02] [OCI-NOSQL-E05]
- Storage hotspot: when `MaxShardSizeUsagePercent` reaches 100, writes stop even if
  allocated storage is not used up. Pick a high-cardinality shard key and limit child
  tables, since a hierarchy shares one shard key. [OCI-NOSQL-E113] [OCI-NOSQL-E08]
- Limits that bind: per-table read/write units (provisioned and on-demand maxima),
  5 indexes per table, 512 KB per row, 4 DDL operations per minute per region.
  [OCI-NOSQL-E23] [OCI-NOSQL-E24]
- ABSOLUTE consistency costs double the read units. [OCI-NOSQL-E03]

## IAM

**Status:** VERIFIED

- Model: OCI IAM policies (groups, dynamic groups, compartments). Three NoSQL resource
  types: `nosql-tables` (table DDL and limits), `nosql-rows` (data) and `nosql-indexes`.
  Verbs are cumulative: inspect, read, use, manage. [OCI-NOSQL-E117]
- Control plane versus data plane: `nosql-tables` USE adds ALTER (UpdateTable,
  change TableLimits); MANAGE adds create and drop. `nosql-rows` READ allows get and
  SELECT, USE adds insert and update, MANAGE adds delete. [OCI-NOSQL-E29] [OCI-NOSQL-E13]
- Least privilege: scope statements to a compartment and to one table with
  `target.nosql-table.name` or `target.nosql-table.id`. Give applications `nosql-rows`
  rights only; keep `nosql-tables` MANAGE with administrators. [OCI-NOSQL-E117] [OCI-NOSQL-E29]
- Workloads on OCI use instance or resource principals, which need a dynamic group
  and policy first. [OCI-NOSQL-E118]
- Global Active tables replicate changes regardless of user permission, and adding or
  dropping replicas needs ALTER in every region involved. [OCI-NOSQL-E28]

## Tagging

**Status:** PARTIAL

- Supported (VERIFIED): OCI tags attach to a Global Active table, and tags are not
  replicated, so each region's replica carries its own. [OCI-NOSQL-E119]
- Table events carry `freeformTags` in their payload. [OCI-NOSQL-E115]
- Not documented (UNVERIFIED): tag-based access control. The policy reference lists
  only `target.nosql-table.id` and `target.nosql-table.name` as NoSQL variables, so
  name- or OCID-based scoping is the documented route. [OCI-NOSQL-E117]
- Defined-tag namespaces, tag defaults and cost-tracking behaviour for NoSQL tables
  were not confirmed in this pass. [OCI-NOSQL-E126]

## Encryption

**Status:** VERIFIED

- At rest: data is encrypted on disk with AES 256 under an Oracle-managed master key
  by default. [OCI-NOSQL-E120] [OCI-NOSQL-E35]
- Customer-managed keys: CMEK only in a dedicated environment. The key lives in an OCI
  KMS vault, must be AES-256 symmetric, and only single-version keys are supported;
  backups of that environment in Object Storage use the same key. [OCI-NOSQL-E35]
- Key state changes are logged by OCI Audit, and an `EncryptionKeyStatus` metric
  supports alarms. [OCI-NOSQL-E122]
- In transit: HTTPS; Oracle states connections require TLS 1.2 and wallet-based
  authentication. [OCI-NOSQL-E120] [OCI-NOSQL-E121]

## Network isolation

**Status:** PARTIAL

- Documented (VERIFIED): a service gateway gives a VCN private access to supported
  Oracle services over the Oracle network fabric, without internet traffic. An Oracle
  reference architecture uses it for NDCS. [OCI-NOSQL-E123] [OCI-NOSQL-E34]
- Not documented (UNVERIFIED): an NDCS-specific private endpoint, a setting that
  disables the public endpoint, or IP allow-listing. Earlier and current searches found
  none. Do not claim them. [OCI-NOSQL-E22] [OCI-NOSQL-E126]
- Workaround to design for: private subnet, service gateway route, IAM scoping and
  TLS; the endpoint stays a public regional name otherwise. This is judgment, not a
  documented guarantee. [OCI-NOSQL-E121]
- Dedicated hosted environments get their own endpoint name, but the page does not
  describe it as private. [OCI-NOSQL-E30]

## Audit logging

**Status:** PARTIAL

- Control plane (VERIFIED): Oracle states it logs DDL API calls on NoSQL tables and
  makes the logs available to you; OCI Audit records supported public API calls as
  events. [OCI-NOSQL-E124] [OCI-NOSQL-E125]
- Events service adds create, drop, alter, index and replica events for rules and
  alerts. [OCI-NOSQL-E115]
- Data plane (UNVERIFIED): no page states whether row operations (get, put, query)
  appear in OCI Audit. Do not assume row-level auditing; log access in the
  application. [OCI-NOSQL-E125] [OCI-NOSQL-E126]
- CMEK key state changes are audited in dedicated environments. [OCI-NOSQL-E122]
