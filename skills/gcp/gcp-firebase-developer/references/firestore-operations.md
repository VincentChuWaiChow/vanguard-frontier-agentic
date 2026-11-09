# Firestore (Native mode) operations

Operational coverage for Firestore in Native mode, Standard edition (Firestore API;
not Datastore mode, not Realtime Database, not Firestore with MongoDB
compatibility). Every section uses the same heading and states one status. Cite
evidence rows from [the evidence ledger](evidence-ledger.md) as `[FS-Exx]`.
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

- Two recovery tools exist. Scheduled backups: at most one daily and one weekly
  schedule per database, retention up to 14 weeks, stored in the source database's
  location. [FS-E13] Point-in-time recovery (PITR) is a separate setting, disabled
  by default, that keeps one version per minute for 7 days; without PITR you can
  still read (not export) documents from the past hour. [FS-E12]
- Backups need billing enabled on the project. [FS-E150] Deleting the source
  database does not delete its backups. [FS-E102]
- A restore always writes into a new database. [FS-E103] An in-place restore means
  deleting the source database and restoring under the same ID: it needs downtime
  and the original database is permanently lost once it starts. [FS-E151] [FS-E106]
- Not restored: backups exclude database TTL policies and Firebase Security Rules
  [FS-E101], and a restore does not bring back App Engine search data or blob
  entities from the `(default)` database. [FS-E149] Reapply TTL policies after a
  restore. [FS-E104]
- Whole-database PITR recovery is a clone or a PITR export; a clone reaches up to
  seven days back with PITR enabled. [FS-E111] A single document or subset is
  recovered with a stale read at a past timestamp, then written back. [FS-E12]
- Cost of recovery: a restore is billed on backup size [FS-E105]; PITR storage has
  no free tier and needs billing, and stale reads or exports inside the PITR window
  incur read costs. [FS-E17]
- IAM for recovery is split across predefined roles, for example
  `roles/datastore.backupsAdmin` and `roles/datastore.restoreAdmin`. [FS-E159]
  [FS-E131]

## Disaster recovery and replication

**Status:** PARTIAL

- Verified, replication design: regional databases replicate synchronously across at
  least three zones [FS-E107]; multi-region databases replicate synchronously across
  five zones in three regions (two serving regions and one witness region).
  [FS-E108]
- Verified, no operator action: replication is handled by Firestore without extra
  configuration or provisioning. [FS-E109] Data keeps being served through the loss
  of a zone (regional) or of a whole region (multi-region). [FS-E11]
- Verified, availability figures: 99.999 percent for multi-region and 99.99 percent
  for regional databases. [FS-E152]
- Verified, location is permanent: it cannot be changed after provisioning, so
  choose regional versus multi-region up front. [FS-E11]
- Verified, data-disaster objectives: the PITR page states a recovery time objective
  of 0 and a recovery point objective of 1 minute for surgical write-back. [FS-E110]
  A PITR export can archive the database to Cloud Storage past the 14-week backup
  maximum. [FS-E153]
- Unconfirmed: the DR page states availability figures but no RTO or RPO for a
  zone or region outage, and does not describe a failover step (automatic or
  manual). Treat failover as service-managed without a documented objective.

## Cost

**Status:** VERIFIED

- Official pricing: <https://cloud.google.com/firestore/pricing> (also
  <https://firebase.google.com/docs/firestore/pricing>); estimate with the Google
  Cloud pricing calculator at <https://cloud.google.com/products/calculator>.
- Standard edition bills per document read, written and deleted. [FS-E117]
- Reads also bill index entries scanned: one read per batch of up to 1000 index
  entries, with documented exceptions. [FS-E116] Aggregation queries bill on index
  entries read, with a minimum of one document read. [FS-E154]
- Storage is billed including metadata and index overhead [FS-E112]; index
  exemptions reduce storage cost. [FS-E10]
- Backup storage is billed when backups are enabled [FS-E114]; a restore is billed on
  backup size [FS-E105]; PITR storage is billed separately and has no free tier.
  [FS-E17]
- TTL deletes, PITR data, backup data, restore operations and clone operations carry
  no free usage and require billing. [FS-E113]
- Network bandwidth used by requests is billed. [FS-E115]

## Monitoring

**Status:** VERIFIED

- Service is Cloud Monitoring (with the console usage and monitoring dashboards).
  Service runtime metric `api/request_count` carries response code class and gRPC
  status, so it yields the error rate. [FS-E118] `api/request_latencies` gives
  latency distributions. [FS-E155] Service runtime metrics are sampled every 30
  minutes, which smooths the data. [FS-E119]
- Document metrics `document/read_ops_count`, `document/write_ops_count` and
  `document/delete_ops_count` show read-heavy versus write-heavy load. [FS-E120]
- TTL: `document/ttl_deletion_count` counts TTL deletes, which are excluded from
  `document/delete_ops_count`. [FS-E156]
- `index/write_count` against `document/write_ops_count` shows index fanout.
  [FS-E157]
- Documented alert example: a p99 latency policy over a 5 minute rolling window that
  fires when latency stays above 250 ms for 5 minutes. [FS-E121]
- The usage dashboard is an estimate; the billing report wins on discrepancy.
  [FS-E122]

## Performance

**Status:** VERIFIED

- Ramp traffic on a new collection with the 500/50/5 rule; automatic document IDs
  are scattered and should not hotspot on writes. [FS-E04] [FS-E05]
- `ABORTED` can come from traffic that exceeds 500/50/5 or hits hot-spots
  [FS-E123], and from contention between overlapping transactions. [FS-E158]
- `RESOURCE_EXHAUSTED` occurs when the free tier quota is exceeded without billing
  enabled. [FS-E124] `DEADLINE_EXCEEDED` means an operation exceeded the 60 second
  default deadline. [FS-E125]
- Sustained high rates on one document can hotspot, and indexed timestamps or other
  monotonic values can hotspot at high rates. [FS-E127] [FS-E128]
- Write and transaction latency grows with the number of splits involved. [FS-E129]
- Limits that shape performance: 1 MiB documents and 10 MiB requests [FS-E01]; a
  270-second transaction limit with a 60-second idle expiration [FS-E03]; indexed
  field values over 1,500 bytes fail with `INVALID_ARGUMENT`. [FS-E126]
- Server client libraries use the database's concurrency mode, PESSIMISTIC by
  default on Standard edition. [FS-E07]

## IAM

**Status:** VERIFIED

- Two access paths: server client libraries are secured with IAM and bypass
  Firebase security rules, while mobile and web SDKs rely on security rules and
  Firebase Auth. [FS-E08]
- Data-plane role `roles/datastore.user` grants read/write on data and is meant for
  application developers and service accounts. [FS-E130]
- Control-plane duties have narrower roles: backups (`roles/datastore.backupsAdmin`)
  [FS-E159], restore (`roles/datastore.restoreAdmin`, which also allows creating
  databases) [FS-E131].
- Scope least privilege by granting the narrowest role on the specific database and
  using IAM Conditions for time-bound or resource-bound access. [FS-E133]
- Role changes can take up to 5 minutes to apply because IAM permissions are cached.
  [FS-E132]
- Tags can drive conditional IAM grants. [FS-E134]

## Tagging

**Status:** VERIFIED

- Resource tags (key-value pairs attached through tag bindings) are supported on
  Firestore databases. [FS-E134]
- Tags can be set when the database is created and attached to existing databases.
  [FS-E160] [FS-E136]
- Attaching tags to Firestore databases requires the Datastore Owner role
  (`roles/datastore.owner`). [FS-E135]
- Tag-based access control: tags can conditionally allow or deny policies, for
  example conditional IAM role grants. [FS-E134]

## Encryption

**Status:** VERIFIED

- At rest: all data is encrypted before it is written to disk with no setup. [FS-E137]
- In transit: TLS protects data over the Internet. [FS-E138]
- Customer-managed keys (CMEK, Cloud KMS) are available by access request. [FS-E161]
- CMEK restrictions: enable only when creating a database [FS-E139]; the key cannot
  be changed afterwards but can be rotated, enabled and disabled [FS-E140]; the key
  location must match the database location. [FS-E143]
- With CMEK, data in transit or memory and database metadata stay under Google
  default encryption. [FS-E141]
- If the key becomes unavailable, calls fail with `FAILED_PRECONDITION` within 10
  minutes. [FS-E142]

## Network isolation

**Status:** PARTIAL

- Verified: VPC Service Controls perimeters. Restricting the Firestore API service
  also restricts the bundled Datastore and Key Visualizer API services. [FS-E144]
- Verified: for full egress protection, import and export must use the Firestore
  service agent. [FS-E145]
- Unverified: private endpoints for Native mode (Private Google Access or Private
  Service Connect guidance) and a setting that disables public access. Searched the
  Firestore Native docs set (VPC Service Controls, regional endpoints, locations,
  security pages); no page covers them. Do not infer them from the MongoDB
  compatibility mode, which has its own Private Google Access page.

## Audit logging

**Status:** VERIFIED

- Cloud Audit Logs for Firestore track administrative and data-access operations.
  [FS-E162]
- Methods needing `ADMIN_WRITE` permissions produce Admin Activity logs. [FS-E147]
  Methods needing `DATA_READ`, `DATA_WRITE` or `ADMIN_READ` produce Data Access logs,
  so data-plane reads and writes are auditable. [FS-E163]
- Configure audit logging with service name `datastore.googleapis.com`, which covers
  `firestore.googleapis.com` too. [FS-E146]
- Gap: individual writes from import, bulk delete and TTL are not audit logged.
  [FS-E148]
