# audit --mode offline

No OCI CLI, network call, credential or customer data. Inputs are the repository
(Terraform, DDL, IAM policy statements, client code) and sanitized exports supplied
by the user. Commands are local only. Expectations come from
[the operations reference](../../operations.md). Set `ROOT` to the reviewed
repository path before running the examples.

## Backup and restore

- Backup is the customer's job (NoSQL Database Migrator export to Object Storage).
  Find the scheduled export job, its source table, bucket and sink table.
- Check the restore path targets a scratch table and what happens when the sink
  table exists with a different schema.
- Look for a recovery runbook and evidence of a tested restore; the bucket holding
  the export needs its own access and retention controls.
- Do not treat a Global Active replica as a backup.

```bash
: "${ROOT:?Approved repository path}"
rg -n -i 'migrator|object.?storage|backup|restore|export' "$ROOT" --glob '*.tf' --glob '*.json' --glob '*.yml' --glob '*.yaml' --glob '*.sh'
```

## Disaster recovery and replication

- Find whether any table is a Global Active table: replicas, regions, frozen
  schema, and which regions need ALTER rights to add or drop replicas.
- Check the application has its own region redirect logic; no service-driven
  failover switch is documented.
- Check schema changes account for the drop-replicas, alter, re-add sequence.
- Confirm the code does not assume ABSOLUTE consistency across regions.

```bash
: "${ROOT:?Approved repository path}"
rg -n -i 'replica|is_multi_region|schema frozen|multi.?region|failover' "$ROOT" --glob '*.tf' --glob '*.sql' --glob '*.json' --glob '*.py' --glob '*.java' --glob '*.go'
```

## Cost

- Compare the declared capacity mode and limits with the access patterns: in the
  Terraform `oci_nosql_table` resource these are `table_limits` with
  `capacity_mode`, `max_read_units`, `max_write_units` and `max_storage_in_gbs`.
  [OCI-NOSQL-E914]
- Look for ABSOLUTE reads, conditional puts and query-based updates or deletes that
  multiply read units, and for secondary indexes that add write units.
- Check compartment quotas for read units, write units and table size.
- Billing amounts are out of scope; never write prices.

```bash
: "${ROOT:?Approved repository path}"
rg -n -i 'table_limits|capacity_mode|max_read_units|max_write_units|max_storage_in_gbs|ABSOLUTE|IfAbsent|IfPresent' "$ROOT" --glob '*.tf' --glob '*.java' --glob '*.py' --glob '*.go' --glob '*.js' --glob '*.ts' --glob '*.cs'
rg -n -i 'nosql quota|read-unit-count|write-unit-count|table-size-gb' "$ROOT" --glob '*.tf' --glob '*.txt' --glob '*.md'
```

## Monitoring

- Look for alarms on the `oci_nosql` namespace: `MaxShardSizeUsagePercent` (about
  90 percent), `ReadThrottleCount`, `WriteThrottleCount`, `StorageThrottleCount`
  and Replica Lag.
- Check notification topics and owners exist for those alarms.
- Check rules on the NoSQL table events (create, drop, alter, index, replica).
- Check the client logs the read and write units returned by each call.

```bash
: "${ROOT:?Approved repository path}"
rg -n -i 'oci_nosql|MaxShardSizeUsagePercent|ThrottleCount|ReplicaLag|Replica Lag|com\.oraclecloud\.nosql' "$ROOT" --glob '*.tf' --glob '*.json' --glob '*.yml' --glob '*.yaml'
```

## Performance

- Check shard key cardinality in the DDL: a low-cardinality shard key causes hot
  shards, and child tables share the root shard key.
- Check index count (limit of 5 per table) and row size against the documented
  limits; check DDL operation rate.
- Check the client handles throttling exceptions with backoff rather than tight retry.
- Check consistency per read matches the requirement, since ABSOLUTE costs double.

```bash
: "${ROOT:?Approved repository path}"
rg -n -i 'shard\s*\(|primary key|create index|create table' "$ROOT" --glob '*.sql' --glob '*.ddl' --glob '*.tf' --glob '*.json'
rg -n -i 'consistency|throttl|retry' "$ROOT" --glob '*.java' --glob '*.py' --glob '*.go' --glob '*.js' --glob '*.ts' --glob '*.cs'
```

## IAM

- Compare policy statements with `nosql-tables`, `nosql-rows` and `nosql-indexes`
  and their cumulative verbs (inspect, read, use, manage). Application principals
  should hold `nosql-rows` rights only.
- Check scoping to a compartment and to one table with `target.nosql-table.name`
  or `target.nosql-table.id`; flag broad `manage` grants.
- Check dynamic groups and rules for instance or resource principals.
- Check who holds ALTER in every region of a Global Active table.

```bash
: "${ROOT:?Approved repository path}"
rg -n -i 'nosql-(tables|rows|indexes)|target\.nosql-table|manage all-resources|dynamic.?group' "$ROOT" --glob '*.tf' --glob '*.json' --glob '*.txt' --glob '*.md'
```

## Tagging

- Check `freeform_tags` and `defined_tags` on the `oci_nosql_table` resource.
  [OCI-NOSQL-E914]
- Tags are not replicated, so each regional replica needs its own tags.
- Tag-based access control is not documented for NoSQL; do not accept a policy that
  relies on tags for isolation.

```bash
: "${ROOT:?Approved repository path}"
rg -n -i 'freeform_tags|defined_tags|oci_nosql_table' "$ROOT" --glob '*.tf'
```

## Encryption

- Default is an Oracle-managed master key. Customer-managed keys apply only in a
  dedicated environment; check the key reference and that it is an AES-256
  symmetric, single-version key in an OCI KMS vault.
- Check clients use HTTPS, TLS 1.2 or later and no plain endpoint overrides.
- Check any migrator backup bucket is also encrypted and access controlled.

```bash
: "${ROOT:?Approved repository path}"
rg -n -i 'cmek|kms|master.?key|dedicated|http://|tls' "$ROOT" --glob '*.tf' --glob '*.json' --glob '*.py' --glob '*.java' --glob '*.go' --glob '*.js' --glob '*.ts' --glob '*.cs'
```

## Network isolation

- No NDCS-specific private endpoint or public-endpoint disable setting is
  documented; do not accept a claim of one.
- Look for a service gateway, route rules to it, and a private subnet for clients.
- Check security lists or network security groups for the client subnet and that
  clients use the regional service endpoint over TLS.

```bash
: "${ROOT:?Approved repository path}"
rg -n -i 'service.?gateway|route.?table|route.?rules|security.?list|network.?security.?group|private.?endpoint' "$ROOT" --glob '*.tf' --glob '*.json'
```

## Audit logging

- OCI Audit records supported control-plane calls; confirm retention or export of
  audit logs and that DDL activity is reviewable.
- Row-level reads and writes are not documented as audited; check the application
  logs data access itself without logging row content or secrets.
- Check rules and notifications on NoSQL table events.

```bash
: "${ROOT:?Approved repository path}"
rg -n -i 'oci_audit|audit|com\.oraclecloud\.nosql|events?.?rule' "$ROOT" --glob '*.tf' --glob '*.json' --glob '*.yml' --glob '*.yaml'
git -C "$ROOT" log --oneline -n 20 -- . ':(glob)**/*.tf'
```
