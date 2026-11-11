# audit --mode offline

No cloud CLI, network call, credential or customer data. Inputs are the repository
(IaC, CQL schema files, application config, client code) and sanitized exports
supplied by the user. Commands are local only. Check against
[the operations reference](../../operations.md).

## Backup and restore

- Check `point_in_time_recovery` (API field `pointInTimeRecovery`) is enabled on every
  table that needs it; disabling then re-enabling erases backup history. [KS-E02]
- Check restore runbooks re-apply tags, IAM policies, alarms and CDC streams, which a
  restore does not bring back. [KS-E04]
- Check customer managed keys stay accessible to Keyspaces, or restore fails. [KS-E23]

```bash
rg -n -i 'point.?in.?time.?recovery|pitr|RESTORE' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml' --glob '*.cql' --glob '*.bicep'
```

## Disaster recovery and replication

- Check keyspace replication strategy and replica Regions (`replicaSpecifications`)
  for multi-Region keyspaces. [KS-E06]
- Check PITR is enabled on all replicas of a multi-Region table. [KS-E05]
- Check application failover is client-driven (for example DNS health checks); there is
  no secondary promotion step. [KS-E07]

```bash
rg -n -i 'replication.?strategy|replica.?specification|replication.?specification|regions' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml' --glob '*.cql'
```

## Cost

- Check the capacity mode per table (`capacitySpecification`: on-demand or provisioned)
  matches the workload. [KS-E09]
- Check for PITR, TTL, CDC streams and multi-Region replicas, which add billing
  dimensions. [KS-E09]
- Billing figures are not in the repository; mark them UNVERIFIED without an export.

```bash
rg -n -i 'capacity.?specification|throughput.?mode|PAY_PER_REQUEST|PROVISIONED|read.?capacity|write.?capacity|auto.?scaling' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml' --glob '*.cql'
```

## Monitoring

- Check alarms on `ReadThrottleEvents`, `WriteThrottleEvents`, `SystemErrors` and
  `UserErrors` in the `AWS/Cassandra` namespace. [KS-E11]
- Check multi-Region keyspaces alarm on `ReplicationLatency`. [KS-E11]
- Check notification ownership through Amazon SNS. [KS-E13]

```bash
rg -n 'AWS/Cassandra|ReadThrottleEvents|WriteThrottleEvents|SystemErrors|UserErrors|ReplicationLatency' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml'
```

## Performance

- Check client retry and backoff for `Read_Timeout` and `Write_Timeout`. [KS-E14]
- Check partition-key design against the per-partition limits. [KS-E15]
- Check driver connection counts against the per-connection request limit. [KS-E16]

```bash
rg -n -i 'Read_Timeout|Write_Timeout|NoHostAvailable|retry.?policy|pool.?size|connection.?per|core.?connection|local.?size' \
  --glob '*.java' --glob '*.py' --glob '*.go' --glob '*.js' --glob '*.ts' --glob '*.conf' --glob '*.yaml'
```

## IAM

- Check policies use the `cassandra:` prefix with resource ARNs for keyspaces and
  tables, and `cassandra:Select` on the `system*` keyspace only where needed. [KS-E17]
- Check for broad `AmazonKeyspacesFullAccess` on runtime roles. [KS-E19]
- Check tag condition keys (`aws:ResourceTag`, `aws:RequestTag`, `aws:TagKeys`). [KS-E20]

```bash
rg -n -i 'cassandra:|AmazonKeyspaces|aws:ResourceTag|aws:RequestTag|aws:TagKeys' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml'
```

## Tagging

- Check keyspaces, tables and streams carry the required tags. [KS-E21]
- Check restore automation re-applies tags (for example `WITH TAGS`). [KS-E04]

```bash
rg -n -i 'tags|WITH TAGS' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml' --glob '*.cql' \
  | rg -i 'keyspace|cassandra|table'
```

## Encryption

- Check `encryptionSpecification` type: `AWS_OWNED_KMS_KEY` or
  `CUSTOMER_MANAGED_KMS_KEY` with a key ARN, and key policy access for Keyspaces. [KS-E22]
- Check clients use TLS 1.2 or later and a trust store; plaintext connections are not
  accepted. [KS-E24] [KS-E28]

```bash
rg -n -i 'encryption.?specification|kms.?key|CUSTOMER_MANAGED_KMS_KEY|AWS_OWNED_KMS_KEY|tls|ssl|truststore' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml' --glob '*.java' --glob '*.py' --glob '*.conf'
```

## Network isolation

- Check an interface VPC endpoint (PrivateLink) exists for the VPC and its security
  group allows port 9142 from application sources only. [KS-E25]
- Check endpoint policies restrict access. [KS-E25]

```bash
rg -n -i 'vpc.?endpoint|privatelink|9142|cassandra\.' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml' --glob '*.bicep'
```

## Audit logging

- Check a CloudTrail trail covers Keyspaces DDL calls. [KS-E26]
- Check data event selection for the `Cassandra table` resource type where DML audit is
  required. [KS-E27]

```bash
rg -n -i 'cloudtrail|event.?selector|data.?resource' \
  --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml'
```
