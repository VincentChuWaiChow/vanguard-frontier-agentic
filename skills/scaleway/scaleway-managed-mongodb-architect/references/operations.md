# Scaleway Managed MongoDB® operations

Operational coverage for Scaleway Managed MongoDB® (Database Instances, Scaleway console and
`mongodb/v1` API). Every section uses the same heading and states one status. Cite evidence
rows from [the evidence ledger](evidence-ledger.md) as `[SCW-MDB-Exx]`.
`tests/validate-nosql-operations.py` enforces this structure. Nothing here is transferred
from Scaleway's PostgreSQL/MySQL Managed Database product.

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

- Snapshots are consistent copies of the Block Storage volume and can only be stored in the same location as the original data [SCW-MDB-E01].
- Snapshots are created on demand in the console or by an API schedule (`is_snapshot_schedule_enabled`, `snapshot_schedule_frequency_hours`, `snapshot_schedule_retention_days`) set through the update Instance endpoint [SCW-MDB-E03].
- Limits: up to 100 snapshots per Instance and project (subject to quotas); default retention is 365 days [SCW-MDB-E02].
- Restore always creates a new, separately billed Instance; no in-place restore is documented. The node type may differ, the volume size equals the snapshot, and settings reset to defaults on a node type with less memory [SCW-MDB-E04].
- Snapshots survive deletion of the Instance until their retention expires or you delete them [SCW-MDB-E05].
- Logical backups use `mongodump`/`mongorestore` over TLS and can restore into any database of any Instance [SCW-MDB-E06].
- Engine upgrade creates a new Instance and does not synchronize data from the source, so stop writes first [SCW-MDB-E22].

## Disaster recovery and replication

**Status:** PARTIAL

- A 3-node replica set (1 primary, 2 standby) lets a standby take over if the primary fails [SCW-MDB-E07].
- A 1-node replica set has no redundancy or high availability [SCW-MDB-E08].
- Snapshots stay in the original location, so they are not a cross-Region copy [SCW-MDB-E01].
- `mongosync` (Cluster-to-Cluster) is documented for migration, not as a DR feature [SCW-MDB-E26].
- Not settled by the pages fetched: Availability Zone placement of the three nodes, failover timing, RPO/RTO or an SLA figure, and cross-Region replicas. Treat these as UNVERIFIED and confirm with Scaleway before promising recovery objectives.

## Cost

**Status:** PARTIAL

Billing dimensions only (what is billed and how it scales). Never write unit prices
or currency amounts: they change by Region and over time. Link the official pricing
page and calculator.

- Documented dimensions: node type, replica set configuration (1 or 3 nodes), Block Storage volume size and IOPS setting [SCW-MDB-E09, SCW-MDB-E10].
- An Instance restored from a snapshot is a new billed Instance [SCW-MDB-E04].
- Extra storage and node types beyond the free trial follow standard pricing [SCW-MDB-E09].
- Pushing your own metrics or logs to Cockpit is billable [SCW-MDB-E12].
- Volumes can grow up to 15 TB but cannot shrink, so over-provisioning is a lasting cost [SCW-MDB-E13].
- Not confirmed in the docs fetched: whether snapshot storage or data transfer is billed separately.
- Pricing (prose link only): <https://www.scaleway.com/en/pricing/managed-databases/>.

## Monitoring

**Status:** PARTIAL

- Scaleway Cockpit integrates Managed MongoDB metrics (with data and dashboards); logs are not integrated [SCW-MDB-E11].
- Sending extra metrics or logs to Cockpit through an external path is billable [SCW-MDB-E12].
- Not settled: named metric identifiers and provider-recommended alerts for MongoDB. The Cockpit pages fetched list integration only. Do not copy metric names from the PostgreSQL/MySQL product.

## Performance

**Status:** PARTIAL

- Performance levers documented: node type, IOPS setting of 5k to 15k at creation, and volume size [SCW-MDB-E10].
- Block Storage Low Latency (5k IOPS) needs new-generation node types and is limited to the PAR and AMS regions [SCW-MDB-E14].
- The volume can be increased up to 15 TB and cannot be decreased [SCW-MDB-E13].
- Not settled: throttling signals, error codes, hot-key behaviour and connection limits. Use MongoDB engine documentation for engine-level behaviour and test under load.

## IAM

**Status:** PARTIAL

- Control plane: IAM permission sets `MongoDBReadOnly` and `MongoDBFullAccess` exist for MongoDB [SCW-MDB-E15].
- Scope IAM rules to a Project or the Organization to limit which Instances a principal reaches [SCW-MDB-E16].
- Unconfirmed: the permission-set names and the Project/Organization scope rows could not be re-checked against the live pages (bot protection); confirm them in the IAM console before relying on them.
- Data plane: database users carry MongoDB roles, global (`read`, `read_write`, `sync`, `db_admin`) or per-database; `sync` is global only and aggregates `clusterMonitor`, `backup` and `restore` [SCW-MDB-E17].
- Least privilege: prefer per-database roles over global ones and avoid `sync` and `db_admin` outside migrations [SCW-MDB-E17].

## Tagging

**Status:** UNVERIFIED

- No resource tag support or tag-based access control for Managed MongoDB Instances was found. Searched: the creation wizard, how-to pages, concepts, FAQ and the IAM permission set and policy pages. Do not assume tags from other Scaleway products.

## Encryption

**Status:** PARTIAL

- In transit: connections use a downloadable TLS certificate, via the public `mongodb+srv` and Private Network connection strings [SCW-MDB-E18].
- Not settled: encryption at rest and customer-managed keys. No Managed MongoDB page fetched states either. Do not infer from other Scaleway products.

## Network isolation

**Status:** PARTIAL

- Private Networks isolate the Instance from the internet; the network must be in the same Region [SCW-MDB-E19].
- An Instance attaches to one Private Network at a time; there is no VPC routing support, so no VPC Peering. Public Gateway Static NAT is unsupported (use SSH bastion port forwarding) [SCW-MDB-E20].
- Unconfirmed: the Private Network limits above could not be re-checked against the live page; confirm them before designing around them.
- The wizard lets you choose Private Network, public connectivity, or both; public resources are publicly visible by default [SCW-MDB-E21].
- An engine upgrade creates the new Instance with a public endpoint by default, so re-check exposure afterwards [SCW-MDB-E22].

## Audit logging

**Status:** PARTIAL

- Audit Trail logs MongoDB control-plane calls (instance, snapshot, restore, maintenance, endpoint and user operations) [SCW-MDB-E23].
- MongoDB integration is listed for fr-par only [SCW-MDB-E24].
- Events can be exported for the past 90 days to an Object Storage bucket [SCW-MDB-E25].
- Not settled: auditing of data-plane operations such as queries or logins. The documented events are API calls only.
