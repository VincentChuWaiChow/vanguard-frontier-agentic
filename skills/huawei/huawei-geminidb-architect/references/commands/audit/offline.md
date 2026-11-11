# audit --mode offline

No cloud CLI, network call, credential or customer data. Inputs are the repository
(IaC, application config, client code) and sanitized exports supplied by the user.
Commands are local only. Checks follow [the operations reference](../../operations.md).
Patterns are case-insensitive and cover API and snake_case spellings; adapt them to
the IaC provider in use rather than assuming an attribute name.

## Backup and restore

- Automated backup policy present with `keep_days` and `start_time` set deliberately, not left to a default [GDB-E102].
- Manual-backup and restore runbooks: restore creates a new instance, never in place [GDB-E106].
- Incremental/PITR, table-level and cross-region backup need a service ticket; flag IaC that assumes them [GDB-E104] [GDB-E110].
- Cross-region backup is unsupported with disk encryption [GDB-E112].

```bash
rg -n -i 'backup_strategy|keep.?days|start.?time|backup.?(policy|period)|recycle' --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.md' .
```

## Disaster recovery and replication

- Nodes spread across three AZs where DR matters [GDB-E120].
- Intra-region and cross-region DR are OBT and ticket-gated; check dependence on them [GDB-E115] [GDB-E119].
- Clients handle a DR switch by moving to the new connection address [GDB-E117].
- No RPO/RTO figure is documented; flag any hard-coded target as unsupported [GDB-E122].

```bash
rg -n -i 'availability.?zone|\bazs?\b|dual.?active|disaster|failover|contact.?points|connection.?address' --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.properties' .
```

## Cost

- Billing mode (prepaid vs pay-per-use) and node specification are explicit [GDB-E124] [GDB-E123].
- Storage autoscaling has a capped storage limit [GDB-E125].
- Backup space above the free allowance, EIP and NAT gateway resources are accounted for [GDB-E10] [GDB-E126].

```bash
rg -n -i 'pay.?mode|charging|period|auto.?(expansion|scal)|disk.?auto|flavor|spec.?code|bandwidth|\beip\b' --glob '*.tf' --glob '*.json' --glob '*.yaml' .
```

## Monitoring

- Cloud Eye alarms on namespace `SYS.NoSQL` for storage, CPU, memory, latency and pending tasks [GDB-E127] [GDB-E128].
- Event alarms for faulty instance/node, primary-standby switch and backup failure [GDB-E129].
- BigKey/HotKey events alarmed [GDB-E130]; slow query timeout set [GDB-E131].

```bash
rg -n -i 'SYS\.NoSQL|cassandra[0-9]{3}_|nosql[0-9]{3}_|NoSQLFaulty|slow_query_log|alarm' --glob '*.tf' --glob '*.json' --glob '*.yaml' .
```

## Performance

- Client code and models keep partitions under 100,000 rows or 100 MB and rows within 64 KB [GDB-E05] [GDB-E04].
- Prefer TTL over mass deletes; no reliance on transactions [GDB-E07] [GDB-E06].
- Node count at most 50 with N/2 spare headroom [GDB-E08].

```bash
rg -n -i 'using ttl|default_time_to_live|\bdelete from\b|\bblob\b|batch|if not exists' --glob '*.cql' --glob '*.java' --glob '*.py' --glob '*.go' --glob '*.ts' .
```

## IAM

- Identity uses `GeminiDB ReadOnlyAccess` for review and avoids blanket FullAccess [GDB-E09] [GDB-E134].
- Deny on `nosql:instance:delete` alongside broad grants [GDB-E134].
- Autoscaling agency actions are granted deliberately [GDB-E136].

```bash
rg -n -i 'GeminiDB (FullAccess|ReadOnlyAccess)|nosql:[a-z]+:[a-z*]+|gaussdbfornosql|"Action"|"Effect"' --glob '*.tf' --glob '*.json' --glob '*.yaml' .
```

## Tagging

- Tags set at creation, at most 20 per instance, one value per key [GDB-E138] [GDB-E139].
- No access decision relies on tags alone [GDB-E140].

```bash
rg -n -i 'tags?\b|tag_key|tag_value' --glob '*.tf' --glob '*.json' --glob '*.yaml' .
```

## Encryption

- Disk encryption selected at purchase with a DEW key; cannot be changed later [GDB-E141].
- Key lifecycle controls prevent disable or deletion of an in-use key [GDB-E142].
- SSL enabled in client config, TLS 1.2 or later [GDB-E143].

```bash
rg -n -i 'disk.?encrypt|kms|cmk|\bssl\b|\btls\b|truststore|enable_ssl' --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.properties' --glob '*.java' --glob '*.py' .
```

## Network isolation

- No EIP bound to nodes; prefer a NAT gateway [GDB-E144] [GDB-E146].
- Security group inbound rules are narrow and not 0.0.0.0/0 [GDB-E145] [GDB-E147].
- Non-default port used [GDB-E147].

```bash
rg -n -i 'security_group_id|security.?group|0\.0\.0\.0/0|\beip\b|public.?ip|vpc_id|subnet_id|\bport\b' --glob '*.tf' --glob '*.json' --glob '*.yaml' .
```

## Audit logging

- CTS tracker configured for control-plane operations; console keeps seven days [GDB-E149] [GDB-E150].
- LTS log shipping referenced where audit/slow logs are required [GDB-E152].
- Do not claim data-plane audit; none is documented for Cassandra [GDB-E151].

```bash
rg -n -i '\bcts\b|tracker|\blts\b|log.?group|log.?stream|audit' --glob '*.tf' --glob '*.json' --glob '*.yaml' .
```
