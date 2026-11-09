# audit --mode offline

No cloud CLI, network call, credential or customer data. Inputs are the repository
(IaC, application config, client code) and sanitized exports supplied by the user.
Commands are local only. Check against [the operations reference](../../operations.md).

## Backup and restore

- Check the retention period is set deliberately (1 to 35 days) rather than left at the
  default of 1 day. [DOCDB-E02]
- Look for manual snapshot or copy automation that keeps a restore point beyond
  cluster deletion; automatic snapshots go with the cluster.
- Check restore runbooks set the cluster parameter group again and add instances after a
  point-in-time restore. [DOCDB-E04]

```bash
rg -n -i 'backup.?retention|preferred.?backup|deletion.?protection|final.?snapshot|skip.?final' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml' --glob '*.bicep'
```

## Disaster recovery and replication

- Check the instance count and Availability Zone spread: failover needs at least two
  instances. [DOCDB-E09]
- Check global cluster definitions (`GlobalClusterIdentifier`, secondary Regions) and
  that instance classes are not db.t3, db.t4g or db.r4. [DOCDB-E06]
- Check client connection strings and failover runbooks, and that lag is alarmed.

```bash
rg -n -i 'global.?cluster|replication.?source|instance.?class|promotion.?tier|availability.?zone' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml' --glob '*.bicep'
```

## Cost

- Check retention days and manual snapshot counts against the billed backup storage
  dimension. [DOCDB-E05]
- Check instance classes and replica counts against the billing dimensions. [DOCDB-E12]
- Billing figures are not in the repository; mark them UNVERIFIED without an export.

```bash
rg -n -i 'instance.?class|storage.?type|serverless|backup.?retention' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml' --glob '*.bicep'
```

## Monitoring

- Check alarms on `CPUUtilization`, `FreeableMemory`, `DatabaseConnections` and
  `VolumeBytesUsed` in the `AWS/DocDB` namespace. [DOCDB-E13]
- Check alarms on the low-memory throttling metrics and replica lag metrics. [DOCDB-E15]
- Check notification targets and missing-data treatment.

```bash
rg -n 'AWS/DocDB|CPUUtilization|FreeableMemory|DatabaseConnections|LowMemNumOperationsThrottled|ReplicaLag' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml' --glob '*.bicep'
```

## Performance

- Compare instance-class connection and cursor quotas with application pool sizes. [DOCDB-E16]
- Check client code for connection reuse, timeouts and cursor handling.
- Check dashboards cover `DatabaseCursorsTimedOut` and the low-memory throttle metrics.

```bash
rg -n -i 'maxPoolSize|max.?pool|minPoolSize|socketTimeout|serverSelectionTimeout|retryWrites' \
  --glob '*.js' --glob '*.ts' --glob '*.py' --glob '*.java' --glob '*.go' --glob '*.json' --glob '*.yaml'
```

## IAM

- Check control-plane policies use `rds:` actions scoped as far as resource-level
  permissions allow, and avoid broad managed policies for runtime roles. [DOCDB-E19]
- Check for `AmazonDocDBFullAccess` attached outside administration. [DOCDB-E18]
- Check database user roles are least privilege, and users created before March 26, 2020
  are reviewed. [DOCDB-E20]

```bash
rg -n -i 'AmazonDocDB|"rds:|rds:\*|docdb-elastic|iam.?auth' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml' --glob '*.bicep'
```

## Tagging

- Check clusters, instances and snapshots carry the required tags (at most 10 per
  resource). [DOCDB-E22]
- Check tag-conditioned IAM policies account for partial ABAC support on instance-based
  clusters. [DOCDB-E17]

```bash
rg -n -i 'tags' --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml' --glob '*.bicep' \
  | rg -i 'docdb|documentdb'
```

## Encryption

- Check `StorageEncrypted` is true and `KmsKeyId` names a symmetric key; CLI-created
  clusters default to unencrypted and the setting cannot change later. [DOCDB-E24]
- Check the cluster parameter group keeps `tls` enabled. [DOCDB-E25]
- Check clients connect with TLS and a CA bundle.

```bash
rg -n -i 'storage.?encrypted|kms.?key|\btls\b|tlsCAFile|ssl_ca' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml' --glob '*.bicep' --glob '*.py' --glob '*.js'
```

## Network isolation

- Check the subnet group is private, security groups restrict ingress to application
  sources, and no public path to the cluster is defined. [DOCDB-E26]
- Check for an interface VPC endpoint for the management API where required. [DOCDB-E27]

```bash
rg -n -i 'db.?subnet.?group|vpc.?security.?group|0\.0\.0\.0/0|vpc.?endpoint|privatelink' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml' --glob '*.bicep'
```

## Audit logging

- Check the `audit_logs` cluster parameter is enabled and `EnabledCloudwatchLogsExports`
  includes the audit log; auditing is off by default. [DOCDB-E29]
- Check a CloudTrail trail covers the control-plane calls. [DOCDB-E28]
- Check log group retention and access.

```bash
rg -n -i 'audit_logs|enabled.?cloudwatch.?logs.?exports|cloudwatch.?logs.?exports|cloudtrail' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml' --glob '*.bicep'
```
