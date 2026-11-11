# IONOS Cloud DBaaS MongoDB operations

Operational coverage for IONOS Cloud Database as a Service (DBaaS) MongoDB, managed
through the Data Center Designer (DCD) and the `api.ionos.com/databases/mongodb` API
(v1). Every section uses the same heading and states one status. Cite evidence rows from
[the evidence ledger](evidence-ledger.md) as `[IONOS-MDB-Exx]`.
`tests/validate-nosql-operations.py` enforces this structure.

Statuses:

- `VERIFIED`: the provider documents the capability; every statement below cites a
  VERIFIED row.
- `PARTIAL`: some of it is documented and some is not, or it is limited (by tier,
  mode, Region or API); say which part is which.
- `UNSUPPORTED`: provider documentation states the capability does not exist. Cite it.
- `UNVERIFIED`: no official page settled it in this pass. Say what was searched.
- `NOT APPLICABLE`: the capability does not apply to this service. Say why.

Edition scope: IONOS documents three editions, Playground, Business and Enterprise.
Playground is a single-instance sandbox, Business is a single replica set on
templates, and Enterprise adds flexible sizing, sharding, offsite backup, point in time
recovery and the BI Connector. Every section below says which edition a capability
applies to. Nothing here is taken from IONOS PostgreSQL or MariaDB DBaaS; those are
separate services with separate documentation.

Documentation conflicts found in this pass (rows with ledger status CONFLICT) are
named where they matter: Playground lifetime [IONOS-MDB-E01], edition change
[IONOS-MDB-E02], backup retention and on-demand backups [IONOS-MDB-E10], replica
counts [IONOS-MDB-E17] and per-instance ceilings [IONOS-MDB-E38]. Where pages
disagree, treat the stricter reading as authoritative until live API evidence says
otherwise.

## Backup and restore

**Status:** PARTIAL

- Edition: backups exist only in Business and Enterprise; Playground has backups
  disabled [IONOS-MDB-E07].
- Automatic snapshots: one at cluster creation, one after each restore, a base
  snapshot every 24 hours and a full snapshot every Sunday [IONOS-MDB-E04]. Retention
  is the last seven days [IONOS-MDB-E03].
- Retention conflict: the Benefits list claims configurable retention and on-demand
  backups [IONOS-MDB-E10]. The backup page and the limitations page document seven
  days and no on-demand call. Plan for seven days and no on-demand snapshot.
- Storage and loss risk: snapshots live in an IONOS object storage bucket in the
  database's region, with a documented fallback to `eu-central-2` [IONOS-MDB-E05].
  Destroying a cluster deletes every snapshot [IONOS-MDB-E06], so this is not a
  protection against cluster deletion.
- Enterprise only: point in time recovery to an oplog timestamp [IONOS-MDB-E08], with
  a DCD look-back of 1 to 24 hours [IONOS-MDB-E09]. Offsite backup to a second
  location is Enterprise only [IONOS-MDB-E14]; the location can only be set at
  creation [IONOS-MDB-E15].
- Restore target: in place on the same cluster using only that cluster's snapshots
  [IONOS-MDB-E11]. A snapshot restores only into the same or an older MongoDB patch
  version [IONOS-MDB-E13]. The database is unavailable during restore, and a confirmed
  restore cannot be reverted [IONOS-MDB-E12]. Restore into a new cluster is not
  documented on the pages read.
- Not documented: what a restore excludes (users, oplog, configuration), cross-cluster
  restore and any restore-test feature. Confirm with live snapshot listing, not docs.

## Disaster recovery and replication

**Status:** PARTIAL

- Topology: Business and Enterprise run multiple instances on different physical
  hosts with automatic replication and failure handling [IONOS-MDB-E19]. Only odd
  member counts are allowed; one primary takes writes and secondaries may serve reads
  [IONOS-MDB-E16].
- Instance counts conflict: the High Availability page says 3 replicas outside
  Playground, while the FAQ and the Resource Allocation table list up to 7 instances
  for Enterprise [IONOS-MDB-E17]. Verify the count your contract allows.
- Playground is a single instance and has no replica failover [IONOS-MDB-E22].
- Durability: default write concern is `majority` with `j: true`; data not replicated
  to a majority can be lost on primary failure and rollback [IONOS-MDB-E18]. Weakening
  the per-operation write concern weakens this.
- Planned interruption: the weekly four-hour maintenance window can disconnect
  clients, so applications must reconnect [IONOS-MDB-E21].
- Failover is described as automatic failure handling; the page gives no election time.
  The only stated SLA is enterprise-grade support response [IONOS-MDB-E20]. No RPO, RTO
  or availability percentage is documented on the pages read.
- Multi-region replication and cross-region failover were not found in the MongoDB
  pages (searched the overview, high availability, backup and FAQ pages). Do not assume
  them. The only documented cross-location copy is Enterprise offsite backup
  [IONOS-MDB-E14].

## Cost

**Status:** PARTIAL

Billing dimensions only (what is billed and how it scales). Never write unit prices
or currency amounts: they change by Region and over time. Link the official pricing
page and calculator.

- Playground: the first instance is free and further instances are billed
  [IONOS-MDB-E25].
- Business: billed by template size (XS up to 4XL_S), where each template fixes vCPU,
  RAM and storage; price varies by edition, template and contract region
  [IONOS-MDB-E27].
- Enterprise: per-node CPU, RAM and storage multiplied by instance count and shard
  count [IONOS-MDB-E23]. The three config servers of a sharded cluster are not billed
  [IONOS-MDB-E24].
- Backup and traffic are billing variables left out of the creation-time estimate
  [IONOS-MDB-E26]. The documented dimensions do not state a backup-storage unit or a
  data-transfer rule on the MongoDB pages; read the price list for those.
- Official price list: <https://docs.ionos.com/cloud/support/general-information/price-list>
  (a MongoDB section with a backup line exists there; amounts are deliberately not
  copied). No calculator page was cited.

## Monitoring

**Status:** PARTIAL

- Edition: the documented metrics path is written for Enterprise clusters
  [IONOS-MDB-E28]. No equivalent Business or Playground monitoring page was found,
  although the features page lists monitoring tools for Business.
- Metrics use the `ionos_dbaas_mongodb_` prefix, for example `cpu_rate5m`,
  `memory_available_bytes`, `storage_available_bytes`, `connections_count`, `health`,
  `instance_count`, `member_state` and `oplog_window_seconds` [IONOS-MDB-E29].
- Replication: `replication_lag_seconds` and `replication_headroom_seconds`
  [IONOS-MDB-E30], and `replication_last_election_timestamp_seconds` to detect
  unexpected failovers [IONOS-MDB-E31].
- Recommended alert levels: 70 percent of CPU, RAM or storage, with scaling planned
  before sustained usage reaches 85 percent [IONOS-MDB-E32].
- Logs: mongod logs are available per cluster through the API with 30-day retention
  [IONOS-MDB-E33]; collection cannot be changed [IONOS-MDB-E34].
- No alarm resource is documented, so alerts are built in your own Grafana or
  Prometheus setup.

## Performance

**Status:** PARTIAL

- Connection ceiling: RAM sets the hard connection limit, (RAM in GB minus 2) / 2
  thousand, and the server rejects new connections beyond it [IONOS-MDB-E35]. Size
  driver pools below the limit; MongoDB drivers provide pooling and IONOS provides no
  separate pooler.
- Storage: Enterprise write IOPS scale with disk size to a cap of 30,000 at 600 GB or
  more; a sharded cluster multiplies the cap by shard count [IONOS-MDB-E36]. The FAQ
  recommends SSD Premium for production and sizing RAM to the working set
  [IONOS-MDB-E40].
- Hard ceilings: per-instance limits are 31 cores, 230 GB RAM and 4 TB storage
  [IONOS-MDB-E37]. One page quotes a lower per-instance limit [IONOS-MDB-E38]; the
  limit also depends on contract quota.
- Hot keys: sharding is Enterprise only and needs a good shard key to spread writes
  [IONOS-MDB-E39]. The pages give no per-key throughput limit or hot-shard signal.
- Throttling: no throttling error code or request-unit style limit is documented. The
  only documented rejection is the connection limit above. Edition choice, not a
  throughput setting, is the main performance lever.

## IAM

**Status:** PARTIAL

- Control plane: a DCD group privilege called Access and manage DBaaS governs
  creating and managing clusters [IONOS-MDB-E44]. No finer-grained built-in control
  plane roles or per-cluster policies were found.
- Data plane: users are managed only through the IONOS API and never inside the
  database [IONOS-MDB-E41]. A cluster holds at most 100 users [IONOS-MDB-E42].
- Least privilege: assign only `read`, `readWrite`, `dbAdmin` (per database),
  `readAnyDatabase`, `readWriteAnyDatabase`, `dbAdminAnyDatabase` or
  `clusterMonitor`. `AnyDatabase` roles are granted on `admin`, and `config` and
  `local` are excluded for the per-database roles [IONOS-MDB-E43].
- Prefer a per-database `read` or `readWrite` user for applications and keep admin
  roles out of application accounts.
- Not documented: custom roles, LDAP or token-based database authentication.

## Tagging

**Status:** UNVERIFIED

- Searched: the MongoDB overview, set-up and update how-tos, the cluster create and
  patch API pages and the llms.txt index for the docs site. None mentions resource tags
  on a MongoDB cluster. The only tagging pages found are for object storage buckets and
  objects, which are a different service.
- Absence is not proof: do not assume tag support, tag-based access control or tag
  propagation to billing for MongoDB clusters. Check the live cluster API response.

## Encryption

**Status:** PARTIAL

- In transit: TLS secures instance-to-instance and client-to-cluster traffic in all
  three editions [IONOS-MDB-E45].
- At rest, Business: data is encrypted at rest on directly attached NVMe devices
  [IONOS-MDB-E46].
- At rest, Enterprise: the data volume is encrypted at rest; the boot volume is not
  mentioned [IONOS-MDB-E47]. Playground at-rest encryption is not stated.
- Customer-managed keys: the Benefits list mentions optional customer-managed keys,
  but the creation UI and API show no key setting [IONOS-MDB-E48]. Treat as
  unconfirmed and do not promise key control.
- Key rotation, key scope and key deletion behaviour are not documented.

## Network isolation

**Status:** PARTIAL

- Placement: a cluster attaches to a data center and a private LAN you choose, with
  one private IP or subnet per instance for Business and Enterprise [IONOS-MDB-E50].
- Enterprise clusters are reachable only through private LANs [IONOS-MDB-E49]. The
  same restriction is not stated for Business or Playground on that page, though the
  creation steps use a private LAN for every edition.
- Clients must resolve public DNS entries to connect, even over the private LAN
  [IONOS-MDB-E52].
- Reserved ranges that cannot be used: `172.16.0.0/12`, `192.168.230.0/24`,
  `10.208.0.0/12` [IONOS-MDB-E51].
- Not documented: a public endpoint, IP allow lists, or private link style endpoints.
  Absence of a public-access toggle should be confirmed against the live cluster.

## Audit logging

**Status:** PARTIAL

- Control plane: the IONOS Activity Log is a contract-level, read-only API that lists
  user logins, provisioning, configuration changes and data access [IONOS-MDB-E53]. It
  does not name MongoDB; confirm it records MongoDB cluster operations on a test
  cluster before relying on it.
- Data plane: mongod logs are retrievable per cluster with 30-day retention
  [IONOS-MDB-E33] and cannot be reconfigured [IONOS-MDB-E34]. This is an operational
  log, not a documented audit log of reads and writes.
- Database audit (MongoDB auditing) is not documented for this service.
