# audit --mode offline

Inputs are the repository (IaC, Firebase config, security rules, client code) and
sanitized exports supplied by the user. Use local tools only: no cloud CLI, network
call, credential or customer data. Field names follow the Firestore Admin API
Database and BackupSchedule resources [FS-E910] [FS-E911]. An absent setting in the
repository does not prove the live setting; record it as UNVERIFIED. Expectations
come from [Native operations](../../firestore-operations.md) and
[MongoDB operations](../../firestore-mongodb-operations.md); audit only the mode in use.

## Backup and restore

- Native mode: check for point-in-time recovery (`pointInTimeRecoveryEnablement`) and
  for backup schedules (`dailyRecurrence`, `weeklyRecurrence`, `retention`) [FS-E910]
  [FS-E911]; compare retention with the recovery objective.
- MongoDB compatibility: same fields; also check the restore runbook for the TTL
  policies a backup does not carry (MongoDB operations, Backup and restore).
- Both: a restore always creates a new database, so check that runbooks and client
  config can point at a new database ID.
- Check `deleteProtectionState` on production databases [FS-E910].

```bash
rg -n -i 'point.?in.?time.?recovery|pitr|daily.?recurrence|weekly.?recurrence|backup.?schedule|delete.?protection' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml' --glob '*.md'
```

## Disaster recovery and replication

- Both modes: check `locationId` for regional versus multi-region against the
  availability target; the location cannot be changed after creation [FS-E910].
- Check runbooks do not promise an RTO or RPO for zone or region outages, which the
  operations references do not document.
- Check that the cross-database copy or export plan, if any, uses the PITR-clone or
  backup-restore path described in the operations reference.

```bash
rg -n -i 'location.?id|multi.?region|nam5|eur3|nam7|database.?edition' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.md'
```

## Cost

- Native mode: check query shapes that scan many index entries or read whole
  collections, and index exemptions that cut storage (Native operations, Cost).
- MongoDB compatibility: billing is in read, write and event units; check change
  streams and index counts (MongoDB operations, Cost).
- Both: check TTL policies, retention of backups and PITR (no free tier).

```bash
rg -n -i 'ttl|index.?exemption|fieldOverrides|change.?stream|\.count\(|limit\(' \
  --glob '*.tf' --glob '*.json' --glob '*.ts' --glob '*.js' --glob '*.py'
```

## Monitoring

- Both modes: check for alerting policies on request latency and error rate in IaC.
- Native mode: check for `api/request_count` and the document operation metrics.
- MongoDB compatibility: check for `api/billable_read_units` and the other billing
  metrics named in the MongoDB operations reference.
- Check that monitoring is defined in code, not only in the console.

```bash
rg -n -i 'firestore\.googleapis\.com|request_latencies|request_count|billable_(read|write)_units|alert.?policy' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml'
```

## Performance

- Native mode: check bulk loaders against the 500/50/5 ramp rule, and monotonic
  indexed values (timestamps, counters) that hotspot (Native operations, Performance).
- MongoDB compatibility: check driver settings for TLS and disabled retryable writes;
  do not apply the 500/50/5 rule (MongoDB operations, Performance).
- Both: check transaction code against the 270-second limit and contention retries.

```bash
rg -n -o -i 'retry.?writes|tls=|ssl=|serverTimeout|runTransaction|BulkWriter|batch.?write' \
  --glob '*.ts' --glob '*.js' --glob '*.py' --glob '*.java' --glob '*.go' --glob '*.env*'
```

## IAM

- Both modes: check bindings of `roles/datastore.user`, `roles/datastore.owner`,
  `roles/datastore.backupsAdmin` and `roles/datastore.restoreAdmin`; prefer the
  narrowest role on the specific database, with IAM Conditions.
- Native mode: check Firebase Security Rules files for open `allow read, write: if true`
  patterns; server client libraries bypass rules.
- MongoDB compatibility: check which SCRAM users exist in code and that each has a role.
- Check for service-account key files committed to the repository.

```bash
rg -n -i 'roles/datastore\.|allow (read|write)[^;]*: *if true|allUsers|scram|firestore\.rules' \
  --glob '*.tf' --glob '*.json' --glob '*.rules' --glob '*.yaml'
```

## Tagging

- Both modes: check that databases carry resource tags (`tags`) where conditional
  IAM depends on them, set at creation [FS-E910].
- Check that tag keys and values are not secrets or customer data.

```bash
rg -n -i '\btags\b|tag.?binding|tag.?value' --glob '*.tf' --glob '*.json' --glob '*.yaml'
```

## Encryption

- Both modes: CMEK is set only at database creation and the key location must match
  the database; check `cmekConfig` and `kmsKeyName` [FS-E910].
- MongoDB compatibility: the CMEK quota is allowlisted; check access was requested.
- Both: check client config for plaintext transport (non-TLS connection strings).
  The second command lists files only; never print the matching line, which can
  hold a connection string with credentials.

```bash
rg -n -o -i 'cmek.?config|kms.?key.?name' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.env*' --glob '*.ts' --glob '*.py'
rg -l -i 'mongodb://|tls=false|ssl=false' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.env*' --glob '*.ts' --glob '*.py'
```

## Network isolation

- Both modes: check for VPC Service Controls perimeters that include
  `firestore.googleapis.com` (operations references, Network isolation).
- MongoDB compatibility: check for Private Google Access and the private DNS zone for
  the restricted VIP; do not assume Native mode has the same path.
- Check client config for hard-coded public endpoints.

```bash
rg -n -i 'firestore\.googleapis\.com|service.?perimeter|private.?google.?access|restricted\..*goog' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml'
```

## Audit logging

- Both modes: check that Data Access audit logs are configured for
  `datastore.googleapis.com` (it covers `firestore.googleapis.com`) as well as the
  default Admin Activity logs.
- Check log sinks, retention and exempted principals.
- Check that bulk import, bulk delete and TTL activity is not assumed to be audited
  per write (Native operations, Audit logging).

```bash
rg -n -i 'audit.?config|datastore\.googleapis\.com|DATA_READ|DATA_WRITE|ADMIN_READ|exempted.?members' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml'
```
