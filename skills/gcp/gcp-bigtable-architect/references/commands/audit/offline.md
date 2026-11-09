# audit --mode offline

Inputs are the repository (IaC, application config, client code) and sanitized
exports supplied by the user. Use local tools only: no cloud CLI, network call,
credential or customer data. Field names follow the Bigtable Admin API resources
[BT-E914] [BT-E915] [BT-E916] [BT-E917] [BT-E918]. An absent setting in the
repository does not prove the live setting; record it as UNVERIFIED. Expectations
come from [the operations reference](../../operations.md).

## Backup and restore

- Check tables for an automated backup policy (`automatedBackupPolicy`) and compare
  its retention with the recovery objective [BT-E916].
- Check any scheduled or on-demand backup tooling for expiry (`expireTime`) and for
  `hotToStandardTime` if hot backups are used [BT-E918].
- Check the restore runbook: a restore creates a new table, and garbage collection
  policies must be set before new writes (operations reference, Backup and restore).
- Check `deletionProtection` on production tables [BT-E916].

```bash
rg -n -i 'automated.?backup|expire.?time|hot.?to.?standard|deletion.?protection' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml' --glob '*.md'
```

## Disaster recovery and replication

- Check how many clusters an instance defines and in which zones; replication needs
  more than one cluster (operations reference, Disaster recovery and replication).
- Check app-profile routing: `multiClusterRoutingUseAny` versus `singleClusterRouting`
  decides automatic versus manual failover [BT-E917].
- Check that code which needs single-row transactions does not run under a
  multi-cluster routing profile (`allowTransactionalWrites` applies to single-cluster
  routing) [BT-E917].
- Check cross-region recovery steps in the runbook against backup copies.

```bash
rg -n -i 'multi.?cluster.?routing|single.?cluster.?routing|allow.?transactional.?writes|app.?profile' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.py' --glob '*.java' --glob '*.go'
```

## Cost

- Check autoscaling bounds (`clusterAutoscalingConfig`, `autoscalingLimits`,
  `autoscalingTargets`) against fixed `serveNodes` [BT-E915].
- Check `defaultStorageType` (SSD or HDD) and replicated clusters, which each hold
  their own storage copy (operations reference, Cost).
- Check backup retention and the number of scheduled copies for storage growth.
- Check data-access audit log enablement, which has a cost impact.

```bash
rg -n -i 'autoscaling|serve.?nodes|default.?storage.?type|node.?scaling.?factor' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml'
```

## Monitoring

- Check for alerting policies on storage utilization and CPU against the thresholds
  in the operations reference (Monitoring, Performance).
- Check dashboards or IaC for hottest-node CPU, replication latency and error rate.
- Check that monitoring is defined in code, not only in the console.

```bash
rg -n -i 'storage_utilization|bigtable\.googleapis\.com/cluster|cpu_load|replication.?latency' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml'
```

## Performance

- Check row-key construction in client code for leading timestamps or sequential
  IDs (operations reference, Performance; design guide).
- Check for read-modify-write or check-and-mutate calls that assume multi-row atomicity.
- Check `columnFamilies` garbage collection (`gcRule`) and max-version or max-age
  settings [BT-E916].
- Check client connection reuse and warm-up code.

```bash
rg -n -i 'row.?key|gc.?rule|max.?versions|max.?age|ReadModifyWrite|CheckAndMutate' \
  --glob '*.tf' --glob '*.py' --glob '*.java' --glob '*.go' --glob '*.ts'
```

## IAM

- Check role bindings for `roles/bigtable.admin` granted to applications; prefer
  `roles/bigtable.user`, `roles/bigtable.reader` or `roles/bigtable.viewer` at
  table level (operations reference, IAM).
- Check for `allUsers` or `allAuthenticatedUsers` members.
- Check for service-account key files committed to the repository.
- Check that no project-wide grant duplicates a narrower table-level grant.

```bash
rg -n -i 'roles/bigtable\.|allUsers|allAuthenticatedUsers|iam.?member|iam.?binding' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml'
```

## Tagging

- Check that instances carry resource tags (`tags`) where conditional IAM depends on
  them, and labels (`labels`) for billing grouping [BT-E914].
- Check that tag keys and values are not secrets or customer data.

```bash
rg -n -i '\btags\b|\blabels\b|tag.?binding' --glob '*.tf' --glob '*.json' --glob '*.yaml'
```

## Encryption

- Check clusters for `encryptionConfig` with `kmsKeyName` when CMEK is required;
  CMEK is set at cluster creation and is immutable (operations reference, Encryption)
  [BT-E915].
- Check that the key region matches each cluster's region.
- Check that backup and restore destinations are CMEK-protected when the source is.

```bash
rg -n -i 'encryption.?config|kms.?key.?name|cmek' --glob '*.tf' --glob '*.json' --glob '*.yaml'
```

## Network isolation

- Check IaC for VPC Service Controls perimeters that include `bigtable.googleapis.com`
  (operations reference, Network isolation).
- Check for Private Service Connect or Private Google Access configuration used by
  clients.
- Check client config for hard-coded public endpoints.

```bash
rg -n -i 'bigtable\.googleapis\.com|bigtableadmin\.googleapis\.com|service.?perimeter|private.?google.?access|psc' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml'
```

## Audit logging

- Check that Data Access audit logs are configured for `bigtable.googleapis.com`
  (DATA_READ, DATA_WRITE, ADMIN_READ) as well as the default Admin Activity logs
  (operations reference, Audit logging).
- Check for log sinks or retention that carry those logs, and for exempted principals.

```bash
rg -n -i 'audit.?config|log.?type|DATA_READ|DATA_WRITE|ADMIN_READ|exempted.?members' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml'
```
