# audit --mode offline

Inputs are the repository (IaC, application config, client code) and sanitized
exports supplied by the user. No cloud CLI, network call, credential or customer
data. Expectations come from [the operations reference](../../operations.md).
Patterns are case-insensitive to cover API casing and snake_case.

## Backup and restore

- Look for Cloud Backup policy and vault definitions that cover Tablestore: backup
  cycle, interval, incremental flag and retention.
- Check that nothing assumes secondary indexes are backed up, and that restore runbooks
  delete and recreate indexes first.
- Check that a KMS key used for backup is protected from disabling or deletion.

```bash
rg -n -i 'backup.?(plan|vault|policy)|retention|incremental|ots_backup' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml'
```

## Disaster recovery and replication

- Check the redundancy type per instance (zone-redundant or locally redundant) and
  whether the Region supports it.
- Check Global Table definitions: mode, regions, and that search or secondary indexes
  are not assumed to replicate.

```bash
rg -n -i 'multi.?az|zone.?redundan|global.?table|replica' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml'
```

## Cost

- Check instance type, payment type and reserved throughput settings against the
  workload shape in the operations reference.
- Look for TTL, Max Versions and index definitions that add storage and read cost.

```bash
rg -n -i 'instance.?(type|specification)|payment.?type|reserved|vcu|time.?to.?live|max.?versions|ttl' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml'
```

## Monitoring

- Check CloudMonitor alert rules on the Tablestore namespace with headroom below
  throttling limits, and an alert contact group.
- Check that client code retries the retryable error codes and does not retry the
  non-retryable ones.

```bash
rg -n -i 'acs_ots_new|InstanceCU|ReadCU|contact.?group|OTSServerBusy|OTSRowOperationConflict|OTSCapacityUnitExhausted' .
```

## Performance

- Check partition key design for sequential values and for key-value size growth.
- Check batch sizes against BatchWriteRow and BatchGetRow limits and the SDK retry
  configuration.

```bash
rg -n -i 'BatchWriteRow|BatchGetRow|retry|partition.?key' \
  --glob '*.java' --glob '*.py' --glob '*.go' --glob '*.js' --glob '*.ts'
```

## IAM

- Check RAM policies for wildcard `ots:*` actions and resources, and for use of
  AliyunOTSFullAccess where read-only or write-only would do.
- Check instance policies for principals, source IP and TLS version conditions.

```bash
rg -n -i 'ots:\*|AliyunOTS(Full|ReadOnly|WriteOnly)Access|instance.?policy|"Principal"' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml'
```

## Tagging

- Check that instances carry owner, environment and cost-center tags, and that keys
  do not start with `aliyun` or `acs:`.
- Do not assume table-level tags exist; tags apply to instances.

```bash
rg -n -i 'tags?\b|resource.?group' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml'
```

## Encryption

- Check table definitions for the encryption type chosen at creation, since it cannot
  be changed afterwards, and for BYOK role and key references.
- Check clients use TLS and a signature version that does not send the secret.

```bash
rg -n -i 'encrypt|kms|byok|tls' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml'
```

## Network isolation

- Check the network type and ACL settings: Internet access disabled where possible,
  VPC bound before use.
- Check client endpoints use the VPC endpoint rather than the public one.

```bash
rg -n -i 'network.?(type|acl)|vpc|tablestore\.aliyuncs\.com|internet' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml' --glob '*.env*'
```

## Audit logging

- Check ActionTrail trails for management events and data events, delivery targets
  and retention beyond the default.
- Check the Simple Log Service audit log setting and retention.

```bash
rg -n -i 'actiontrail|trail|sls|log.?project|audit.?log' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml'
```
