# OVHcloud Public Cloud Databases for MongoDB operations

Operational coverage for OVHcloud Public Cloud Databases for MongoDB (managed MongoDB
replica sets, control plane via OVHcloud Console and API, data plane via the MongoDB
protocol). Every section uses the same heading and states one status. Cite evidence rows
from [the evidence ledger](evidence-ledger.md) as `[OVH-MDB-Exx]`.
`tests/validate-nosql-operations.py` enforces this structure.

Statuses:

- `VERIFIED`: the provider documents the capability; every statement below cites a
  VERIFIED row.
- `PARTIAL`: some of it is documented and some is not, or it is limited (by tier,
  mode, Region or API); say which part is which.
- `UNSUPPORTED`: provider documentation states the capability does not exist. Cite it.
- `UNVERIFIED`: no official page settled it in this pass. Say what was searched.
- `NOT APPLICABLE`: the capability does not apply to this service. Say why.

**Plan differences (read first).** The current MongoDB plans are Production (3 nodes)
and Advanced (3 to 7 nodes, optionally with an analytics node) [OVH-MDB-E01]. The
Discovery plan is at end of life (end of sale 31 August 2026, suspension 31 October
2026, deletion 30 November 2026) [OVH-MDB-E35], and shared OVHcloud pages still use the
older Essential/Business/Enterprise names next to Discovery/Production/Advanced
[OVH-MDB-E21]. Plan-dependent features in this file: backup retention and point-in-time
recovery, SLA, client-side field level encryption and the analytics node. Plan changes
are documented as upgrades; one shared page also mentions downgrades, so treat downgrades
as unconfirmed [OVH-MDB-E37]. Generation labels (Gen 2, Gen 3) were not found on the
MongoDB pages read for this pass and are not used here. All statements below were read on
MongoDB-specific pages, or on shared Public Cloud Databases pages where the text applies to
MongoDB (called out each time); nothing is carried over from the PostgreSQL or MySQL
engines.

## Backup and restore

**Status:** PARTIAL

- Automatic backups run daily on both plans: Production keeps 7 days; Advanced keeps 30
  days with point-in-time recovery (PITR) for the last 24 hours [OVH-MDB-E02]
  [OVH-MDB-E03]. A shared page labels the 24-hour PITR window as "MongoDB Enterprise",
  which this reference reads as Advanced [OVH-MDB-E06].
- MongoDB backups are shipped to a remote region, encrypted, with a documented RPO of 24
  hours; the backup region and daily time can be set through the `backups` attribute and
  the regions array takes one element for MongoDB [OVH-MDB-E04] [OVH-MDB-E05].
- Restore has two paths. Fork creates a new independent service from a backup, a point in
  time (where supported) or the latest state, and lets you change version, region, plan,
  node type, node count and storage [OVH-MDB-E08]. MongoDB also offers an in-place
  restore that rolls back all data and can lose data [OVH-MDB-E07].
- Not restored: a fork does not copy IP restrictions or users, so recreate both before
  use [OVH-MDB-E09].
- Manual logical backup and restore use `mongodump` and `mongorestore` with the `backup`
  and `restore` roles; `mongorestore` merges into existing data. The page header dates it
  to 2022, so re-check before relying on it [OVH-MDB-E10].
- Automated backups cannot be downloaded; restore to a new service and export instead
  [OVH-MDB-E11]. Deletion protection can block accidental service deletion until it is
  switched off [OVH-MDB-E14].
- CONFLICT on what survives deletion: one page says deleting a MongoDB service removes its
  backups in every region, another allows 2 days to 1 month of backup retention after
  termination. Assume backups are gone and export data first [OVH-MDB-E12] [OVH-MDB-E13].

## Disaster recovery and replication

**Status:** PARTIAL

- MongoDB runs as a replica set across multiple nodes and high availability starts at 3
  nodes [OVH-MDB-E22]. The service is available in all 1-AZ and 3-AZ regions
  [OVH-MDB-E17].
- 3-AZ regions spread a multi-node service across three independent availability zones and
  survive the loss of one zone [OVH-MDB-E15]. 1-AZ regions resist node and disk failures but
  may be vulnerable to a complete OpenStack region outage [OVH-MDB-E16].
- Failover after a node failure is automatic, taking a few seconds up to 1 minute, with
  writes discarded until it finishes [OVH-MDB-E18] [OVH-MDB-E19] [OVH-MDB-E51]. These
  statements come from a shared FAQ that covers every engine, not a MongoDB-only page.
- Documented SLA figures: Production 99,90% single AZ and 99,95% multi AZ; Advanced 99,95%
  single AZ and 99,99% multi AZ; none for Essential/Discovery [OVH-MDB-E20] [OVH-MDB-E21].
  No RTO is stated; the only RPO found is the 24-hour backup RPO [OVH-MDB-E04].
- Cross-region recovery is by restoring a remote-region backup into a fork in another
  region [OVH-MDB-E05] [OVH-MDB-E08]. Continuous cross-region replication was not found:
  searched the MongoDB product index, capabilities, FAQ and regions-comparison pages.

## Cost

**Status:** VERIFIED

Billing dimensions only (what is billed and how it scales). Never write unit prices
or currency amounts: they change by Region and over time. Official pages:
[Public Cloud prices](https://www.ovhcloud.com/en-gb/public-cloud/prices/) and the
[MongoDB product page](https://www.ovhcloud.com/en-gb/public-cloud/mongodb/).

- Three billing components: compute (the VM running the service), storage (Block Storage)
  and backups (Object Storage) [OVH-MDB-E27].
- Billing is per second from an hourly reference price; the monthly figure is an estimate
  [OVH-MDB-E28]. Storage is billed on allocated capacity, independent of compute
  [OVH-MDB-E29].
- The new billing model splits compute, storage and backup into separate references, with
  backup following the capacity protected by automated backups; invoices move from two lines
  to three lines after December 2026 [OVH-MDB-E30] [OVH-MDB-E31].
- Ingress and egress traffic are included and unmetered, and the MongoDB license cost is
  included in the plan with no bring-your-own-license option [OVH-MDB-E32] [OVH-MDB-E33].
- The reversibility policy lists cluster size, storage capacity and backups as the billing
  basis and states there is no exit fee [OVH-MDB-E34].
- Switching a suspended Discovery service to Production starts Production billing at once
  [OVH-MDB-E36].

## Monitoring

**Status:** PARTIAL

- The Console Observability tab shows CPU used, disk space used, read and write IOPS, memory
  used and network reception and transmission charts [OVH-MDB-E38]. Logs are kept as the
  last 1000 lines and metrics for 1 calendar year, and both are deleted with the instance
  [OVH-MDB-E43].
- Prometheus export is supported for MongoDB, enabled per service; it follows the service's
  IP restrictions and private-network reachability [OVH-MDB-E41] [OVH-MDB-E42].
- Logs can be forwarded to a Logs Data Platform stream [OVH-MDB-E44].
- With the `clusterMonitor` role, `mongosh` exposes `serverStatus`, `dbStats` and
  `collStats`; Compass adds a real-time Performance tab [OVH-MDB-E39] [OVH-MDB-E40].
- OVHcloud recommends watching CPU, memory, cache activity, disk usage, replication lag,
  read and write tickets, IOPS, latency, connections, opcounters and the oplog window
  [OVH-MDB-E46] [OVH-MDB-E47], and setting alerts on CPU, memory, disk I/O and slow queries
  [OVH-MDB-E48]. Disk-fill emails are sent automatically at a warning mark and a critical
  mark [OVH-MDB-E45].
- Not found: named metric namespaces or a native alarm service for MongoDB. Searched the
  monitoring, Prometheus, operational best practices and capabilities pages.

## Performance

**Status:** PARTIAL

- At disk full MongoDB refuses writes while Admin users can still delete; adding storage or
  upgrading restores normal mode [OVH-MDB-E49]. Storage resize must leave at least 20% free
  [OVH-MDB-E50].
- Read preference defaults to primary and write concern defaults to majority; secondaries
  spread reads and `w=1` lowers latency at the cost of durability [OVH-MDB-E26]
  [OVH-MDB-E77].
- An Analytics node isolates analytical queries from operational ones; it needs at least
  3 non-analytics nodes and a cluster has at most one [OVH-MDB-E54] [OVH-MDB-E23]
  [OVH-MDB-E24], and is documented as an Advanced feature [OVH-MDB-E25].
- Sizing guidance: WiredTiger starts evicting once its cache passes about 80% full
  [OVH-MDB-E53]. Troubleshooting points to CPU and RAM saturation and slow queries
  [OVH-MDB-E55]. Maintenance restarts briefly interrupt single-node services
  [OVH-MDB-E52].
- Not found: throttling error codes or per-collection hot-key behaviour. The service is a
  replica set, so engine-level MongoDB behaviour applies and OVHcloud documents no extra
  quota signal; searched troubleshooting, capabilities and best-practice pages.

## IAM

**Status:** PARTIAL

- Control plane: Console and API access uses two-factor authentication, SSO and OVHcloud IAM
  policies that the customer sets up [OVH-MDB-E58].
- Data plane: database users are created in the Console or API with MongoDB built-in roles
  such as `read`, `readWrite`, `dbAdmin`, `userAdmin`, `clusterMonitor`, `backup`,
  `restore` and `root`; some roles apply on the `admin` database and others on a database
  you choose [OVH-MDB-E56].
- Shell-created users are deleted by the automation, and OVHcloud keeps technical users
  `admin@admin`, `mms-automation@admin` and `backup-*@admin` [OVH-MDB-E57].
- Least privilege: assign narrow roles per user and disable unused accounts [OVH-MDB-E59].
  Passwords are shown once at creation or reset [OVH-MDB-E60].
- Not found: OVHcloud IAM action names specific to MongoDB services. Searched the security
  overview, capabilities and IAM policy pages.

## Tagging

**Status:** PARTIAL

- OVHcloud IAM policies accept conditions on a resource tag, resource name, product type,
  request IP and time [OVH-MDB-E61]. That page is account-wide.
- Not found: any page stating that a MongoDB database service can carry tags or be targeted
  by tag. Searched the MongoDB product index, the Databases bundle, capabilities and the
  security overview. Treat tag-based access control on MongoDB services as unconfirmed.

## Encryption

**Status:** VERIFIED

- At rest, MongoDB node volumes use LUKS2 full-volume encryption with an ephemeral key per
  volume [OVH-MDB-E62]. Backups are encrypted with AES-256 and a random RSA 4096 key, plus
  Object Storage server-side encryption [OVH-MDB-E63].
- Customer-managed keys are not offered, and KMIP is managed by OVHcloud [OVH-MDB-E64].
- Client-side field level encryption with customer-controlled keys is available on Advanced
  plans only [OVH-MDB-E65].
- In transit, MongoDB listens on 27017, refuses unencrypted connections and accepts TLS 1.2
  and TLS 1.3 only, with a certificate from a public authority [OVH-MDB-E66]. Keep
  certificate verification enabled in clients [OVH-MDB-E67].

## Network isolation

**Status:** PARTIAL

- Public and private (vRack) networking are available on both plans; the port is 27017
  [OVH-MDB-E68] [OVH-MDB-E66].
- The service is unreachable until authorised IPs are set, and by default no outside
  connection is accepted; `0.0.0.0/0` opens it to everyone [OVH-MDB-E69] [OVH-MDB-E70].
  Restrict the list to what is needed [OVH-MDB-E74].
- Private-network limits: you must own the network, shared networks are not supported, the
  network cannot be deleted while services run, and subnets need spare addresses
  [OVH-MDB-E71].
- A service can be switched between public and private networks through the API; the
  rebuild causes downtime and moving to public clears IP restrictions [OVH-MDB-E72]. The
  service IP changes periodically, so use DNS [OVH-MDB-E73].
- Not found: private endpoints or service gateways beyond vRack, or a switch that disables
  public access other than choosing the private network mode. Searched the capabilities,
  vRack and network-update pages.

## Audit logging

**Status:** PARTIAL

- Control plane: Console and API interactions are logged and retrievable through account
  API-call and audit-log routes [OVH-MDB-E75].
- Data plane: the best-practices page recommends enabling MongoDB auditing, but no OVHcloud
  procedure for enabling it on the managed service was found [OVH-MDB-E76]. Service logs
  are limited to the last 1000 lines [OVH-MDB-E43].
- Searched the security overview, logs-forwarding, monitoring and best-practices pages.
  Treat data-plane audit as unconfirmed.
