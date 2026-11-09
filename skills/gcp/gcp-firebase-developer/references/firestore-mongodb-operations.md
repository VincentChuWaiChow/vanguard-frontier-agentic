# Firestore with MongoDB compatibility operations

Operational coverage for Firestore with MongoDB compatibility (Enterprise edition,
MongoDB-compatible API and drivers). It is a separate mode from Native mode: the
sections below cite MongoDB-compatibility pages only and do not assume Native-mode
features apply. Every section uses the same heading and states one status. Cite
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

**Status:** PARTIAL

- Verified, scheduled backups: at most one daily and one weekly schedule per
  database [FS-E203], retention up to 14 weeks [FS-E204], stored in the source
  database's location. [FS-E281] Billing must be enabled. [FS-E206]
- Verified, restore target: a restore writes into a new database. [FS-E205] An
  in-place restore means deleting the database and restoring under the same ID; it
  needs downtime and the original is permanently lost once it starts. [FS-E209]
  [FS-E208]
- Verified, PITR: disabled by default, minute granularity, 7 days of data. [FS-E210]
  [FS-E211] Recover a subset by reading at a past timestamp and writing back, or the
  whole database by clone or export of a whole-minute timestamp within seven days.
  [FS-E212] PITR storage has no free tier. [FS-E230]
- Verified, not restored: backups exclude TTL policies, so reapply them after a
  restore. [FS-E202] [FS-E207]
- Unconfirmed: whether SCRAM user credentials, IAM bindings, change streams or saved
  queries are carried into a restored database. The backup and in-place restore
  pages list only the TTL exclusion; verify these on a restored test database.

## Disaster recovery and replication

**Status:** PARTIAL

- Verified, replication design: regional databases replicate synchronously across at
  least three zones [FS-E213]; multi-region databases across five zones in three
  regions (two serving, one witness). [FS-E214]
- Verified, no operator action: replication is automatic and needs no extra
  configuration or provisioning. [FS-E216]
- Verified, availability figures: 99.999 percent for multi-region and 99.99 percent
  for regional. [FS-E215]
- Verified, locations: multi-region locations eur3, nam5 and nam7 exist, and the
  location cannot be changed after provisioning. [FS-E219]
- Verified, data-disaster objectives: PITR write-back has a stated recovery time
  objective of 0 and recovery point objective of 1 minute [FS-E217]; a clone reaches
  seven days back with PITR enabled, one hour without. [FS-E218]
- Unconfirmed: no RTO or RPO is stated for a zone or region outage and no failover
  step (automatic or manual) is described; treat failover as service-managed.

## Cost

**Status:** VERIFIED

- Official pricing: <https://cloud.google.com/firestore/enterprise/pricing>;
  estimate with the Google Cloud pricing calculator at
  <https://cloud.google.com/products/calculator>. Enterprise edition is billed in
  units, not per document as in Standard edition.
- Read units: data processed (documents or indexes) in 4 KiB tranches. [FS-E220]
- Write units: data processed in 1 KiB tranches [FS-E221]; index entries written are
  charged as write units. [FS-E225]
- Event units: data included in each event, in 4 KiB tranches. [FS-E222] Change
  stream storage is billed on captured events and retention. [FS-E228]
- Real-time update units relate to Native-mode client libraries, not to MongoDB
  compatibility. [FS-E223]
- Storage includes metadata and index overhead [FS-E224]; backup storage [FS-E227];
  restore on backup size [FS-E279]; PITR storage with no free tier. [FS-E230]
- Managed deletes, PITR data, backup data, restore and clone operations carry no free
  usage. [FS-E226] Network bandwidth is billed. [FS-E229]

## Monitoring

**Status:** VERIFIED

- Service is Cloud Monitoring; monitored resource `firestore.googleapis.com/Database`
  breaks down by project, location and database ID. [FS-E231]
- `api/request_latencies` gives latency distributions. [FS-E232]
- Document metrics `document/read_ops_count`, `document/write_ops_count` and
  `document/delete_ops_count` show read-heavy versus write-heavy load. [FS-E233]
- Billing metrics `api/billable_read_units`, `api/billable_write_units` and
  `document/billable_managed_delete_write_units` exclude administrator operations.
  [FS-E234]
- `index/write_count` against `document/write_ops_count` shows index fanout.
  [FS-E235] TTL metrics `document/ttl_deletion_count` and
  `document/ttl_expiration_to_deletion_delays` track TTL. [FS-E236]
- Predefined dashboards include request latencies, response codes and query stats.
  [FS-E237]
- Documented alert example: p99 latency over a 5 minute rolling window above 250 ms
  for 5 minutes. [FS-E238] The usage dashboard is an estimate. [FS-E239]

## Performance

**Status:** PARTIAL

- Verified, error signals: `Aborted (112)` means contention [FS-E240];
  `DeadlineExceeded (262)` means the 60 second default deadline passed [FS-E241];
  `InvalidArgument (2)` flags an index entry over 7.5 KiB. [FS-E242]
- Verified, hotspots: sustained high rates on one document and monotonic indexed
  values such as timestamps can hotspot. [FS-E243] [FS-E244] Ramping too fast can
  cause high latency or deadline errors. [FS-E245]
- Verified, limits: transactions 270 seconds with 60-second idle expiration [FS-E246];
  at most 40,000 index entries per document and 8 MiB of index entries. [FS-E247]
- Verified, client: drivers must use TLS and disable retryable writes. [FS-E248]
- Not documented for this mode: the numeric 500/50/5 ramp rule and a quota-based
  `RESOURCE_EXHAUSTED` signal appear only in Native-mode pages. Do not carry them over.

## IAM

**Status:** VERIFIED

- Control plane uses IAM roles; data-plane MongoDB methods map to `datastore.*`
  permissions (for example, `GetMore` needs the permissions of the call that created
  the cursor). [FS-E249]
- Clients authenticate with SCRAM, Compute Engine or Cloud Run service accounts, or
  the Google Auth Library. [FS-E280]
- Data access role `roles/datastore.user` is for developers and service accounts.
  [FS-E250] A new SCRAM credential has no permissions until a role is granted.
  [FS-E254]
- Narrow control-plane roles: `roles/datastore.userCredsAdmin` [FS-E251] and
  `roles/datastore.restoreAdmin`. [FS-E255]
- Least privilege: grant roles per database with IAM Conditions [FS-E253]; role
  changes take up to 5 minutes. [FS-E252]

## Tagging

**Status:** VERIFIED

- Resource tags are supported on MongoDB-compatible databases. [FS-E256]
- Tags can be set at creation and attached to existing databases. [FS-E258]
  [FS-E259]
- Attaching tags requires `roles/datastore.owner`. [FS-E257]
- Tag-based access control: tags can drive conditional allow or deny policies, such
  as conditional IAM grants. [FS-E256]

## Encryption

**Status:** VERIFIED

- At rest: all data is encrypted before it is written to disk with no setup.
  [FS-E260]
- In transit: TLS protects data over the Internet [FS-E261], and drivers must connect
  with TLS. [FS-E248]
- CMEK restrictions: only on new databases at creation [FS-E262]; the key cannot be
  changed but can be rotated, enabled and disabled [FS-E263]; the KMS key location
  must match the database [FS-E265]; the default CMEK database quota per project is 0
  because the feature is allowlisted. [FS-E264]
- With CMEK, data in transit or memory and database metadata stay under Google
  default encryption. [FS-E266]

## Network isolation

**Status:** PARTIAL

- Verified, VPC Service Controls: restricting the Firestore API service also
  restricts the bundled Datastore and Key Visualizer API services [FS-E267];
  import and export need the Firestore service agent for full egress protection.
  [FS-E273]
- Verified, private access: a VM without an external IP reaches the service through
  Private Google Access on its subnet [FS-E269]; the domains support only the MongoDB
  protocol on port 443. [FS-E270]
- Verified, restricted VIP: needs connectivity to the service VIP domain, which is
  VPC Service Controls compliant [FS-E268]; the public `restricted.firestore.goog`
  records cannot be used, so create a private DNS zone. [FS-E271]
- Verified, default path: the default domain uses public IP addresses. [FS-E272]
- Unverified: Private Service Connect endpoints and a setting that disables public
  access. Searched the MongoDB-compatibility docs index, VPC Service Controls and
  Private Google Access pages; none covers them.

## Audit logging

**Status:** VERIFIED

- Audit logging uses service name `datastore.googleapis.com`, covering
  `firestore.googleapis.com`. [FS-E274]
- `ADMIN_WRITE` methods produce Admin Activity logs. [FS-E275] `DATA_READ`,
  `DATA_WRITE` and `ADMIN_READ` methods produce Data Access logs. [FS-E276]
- MongoDB data-plane methods (Find, Aggregate, Insert, Update, Delete and others) are
  listed as `DATA_READ` or `DATA_WRITE`, so they can be audited. [FS-E277]
- The caller is identified through the `AuthenticationInfo` field and caller IP in
  request metadata. [FS-E278]
