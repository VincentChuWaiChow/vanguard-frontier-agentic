# audit --mode offline

No cloud CLI, network call, credential or customer data. Inputs are the repository
(IaC, application config, client code) and sanitized exports supplied by the user.
Commands are local only. Checks follow [the operations reference](../../dds-operations.md).
Patterns are case-insensitive and cover API and snake_case spellings; adapt them to
the IaC provider in use rather than assuming an attribute name.

## Backup and restore

- Automated backup policy via `backup_strategy` with `keep_days` (1 to 732) and `start_time` [HW-DDS-E102].
- Incremental backup availability by version and vCPUs; restore runbook distinguishes in-place overwrite from new-instance restore [HW-DDS-E104] [HW-DDS-E106].
- PITR/oplog sizing for the intended window [HW-DDS-E108].
- Recycle bin and backup-download policy not assumed disabled [HW-DDS-E112].

```bash
rg -n -i 'backup_strategy|keep.?days|start.?time|backup.?(policy|period|method)|recycle' --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.md' .
```

## Disaster recovery and replication

- Mode is replica set or sharding, not single node, where HA is required [HW-DDS-E119].
- Nodes spread across AZs where the Region supports it [HW-DDS-E117] [HW-DDS-E118].
- Clients use replica-set connection strings with automatic reconnect [HW-DDS-E120].
- Cross-region DR depends on cross-region backups [HW-DDS-E114].

```bash
rg -n -i 'ReplicaSet|Sharding|\bsingle\b|availability.?zone|replicaSet=|readPreference|retryWrites|cross.?region' --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.properties' .
```

## Cost

- Spec, node count, storage and billing mode are explicit [HW-DDS-E125].
- Snapshot backup via CBR, cross-region backup, EIP, audit policy and LTS reporting each add charges [HW-DDS-E103] [HW-DDS-E127] [HW-DDS-E129] [HW-DDS-E167].

```bash
rg -n -i 'pay.?mode|charging|period|flavor|spec.?code|volume|bandwidth|\beip\b|cbr|snapshot' --glob '*.tf' --glob '*.json' --glob '*.yaml' .
```

## Monitoring

- Cloud Eye alarms on namespace `SYS.DDS` for connections, CPU, memory, disk, replication lag and oplog window [HW-DDS-E131] [HW-DDS-E133] [HW-DDS-E135].
- Event alarm for `planDeleteKmsKey` on encrypted instances [HW-DDS-E134].

```bash
rg -n -i 'SYS\.DDS|mongo0[0-9]{2}_|planDeleteKmsKey|repl_lag|oplog|alarm' --glob '*.tf' --glob '*.json' --glob '*.yaml' .
```

## Performance

- Shard key is not monotonic; hashed sharding considered [HW-DDS-E01].
- Connection pool caps respect connection usage limits [HW-DDS-E140].
- Storage headroom so the instance does not go read-only [HW-DDS-E136].

```bash
rg -n -i 'shardCollection|shard.?key|maxPoolSize|max.?connections|minPoolSize' --glob '*.js' --glob '*.py' --glob '*.java' --glob '*.go' --glob '*.ts' --glob '*.yaml' --glob '*.properties' .
```

## IAM

- Prefer `DDS ManageAccess` or `DDS ReadOnlyAccess` over `DDS FullAccess` [HW-DDS-E02].
- Condition keys `dds:Encrypted` and `dds:BackupEnabled` used to enforce encryption and backups [HW-DDS-E143].
- Data-plane users use `read`/`readWrite` roles, not `rwuser` [HW-DDS-E145] [HW-DDS-E146].

```bash
rg -n -i 'DDS (FullAccess|ReadOnlyAccess|ManageAccess)|dds:Encrypted|dds:BackupEnabled|rwuser' --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.js' --glob '*.py' .
```

## Tagging

- Tags set at creation, at most 20 per instance [HW-DDS-E148] [HW-DDS-E149].
- No access decision relies on tags alone [HW-DDS-E150].

```bash
rg -n -i 'tags?\b|tag_key|tag_value' --glob '*.tf' --glob '*.json' --glob '*.yaml' .
```

## Encryption

- Disk encryption selected at purchase (`disk_encryption_id` in the API) and cannot change later [HW-DDS-E151].
- Backup encryption is separate and ticket-gated; do not enable both [HW-DDS-E153] [HW-DDS-E154].
- `ssl` enabled and clients use TLS 1.2 [HW-DDS-E155].

```bash
rg -n -i 'disk_encryption_id|disk.?encrypt|kms|cmk|\bssl\b|\btls\b|tlsCAFile|tls=true' --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.properties' --glob '*.py' --glob '*.js' .
```

## Network isolation

- No Internet EIP on the instance; prefer a public gateway if public access is required [HW-DDS-E156] [HW-DDS-E157].
- `security_group_id` rules are narrow and not 0.0.0.0/0 [HW-DDS-E158] [HW-DDS-E160].
- Non-default port; cluster EIPs bind only to mongos [HW-DDS-E159].

```bash
rg -n -i 'security_group_id|security.?group|0\.0\.0\.0/0|\beip\b|public.?ip|vpc_id|subnet_id|\bport\b' --glob '*.tf' --glob '*.json' --glob '*.yaml' .
```

## Audit logging

- CTS tracker for control-plane operations [HW-DDS-E162] [HW-DDS-E163].
- Database audit policy (`keep_days`, `audit_scope`, `audit_types`) set where required; disabled by default [HW-DDS-E164].
- Audit logs retained more than 180 days; LTS reporting if needed; accept the performance and fee impact [HW-DDS-E165] [HW-DDS-E166] [HW-DDS-E167].

```bash
rg -n -i 'auditlog|audit.?(policy|scope|types)|\bcts\b|tracker|\blts\b|log.?group' --glob '*.tf' --glob '*.json' --glob '*.yaml' .
```
