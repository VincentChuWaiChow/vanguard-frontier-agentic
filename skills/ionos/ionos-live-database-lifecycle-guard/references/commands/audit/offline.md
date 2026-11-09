# audit --mode offline

Inputs are the repository (IaC, application config, client code) and sanitized
exports supplied by the user. No cloud CLI, network call, credential or customer
data. Scope is IONOS DBaaS MongoDB only. Expectations come from
[the MongoDB operations reference](../../mongodb-operations.md); a missing match is
UNVERIFIED, not a pass.

```bash
git rev-parse --short HEAD
find . -type f \( -name '*.tf' -o -name '*.json' -o -name '*.yaml' -o -name '*.yml' \) -not -path './node_modules/*' | head -50
```

## Backup and restore

- Backups exist only in Business and Enterprise; retention is the last seven days and there is no on-demand snapshot call [IONOS-MDB-E03] [IONOS-MDB-E07].
- Destroying a cluster deletes every snapshot, so deletion runbooks need an export plan [IONOS-MDB-E06].
- Point in time recovery and offsite backup are Enterprise only; offsite location is set at creation [IONOS-MDB-E08] [IONOS-MDB-E14] [IONOS-MDB-E15].
- Restore is in place on the same cluster and cannot be reverted once confirmed [IONOS-MDB-E11] [IONOS-MDB-E12].

```bash
rg -n -i 'backup|snapshot|point.?in.?time|offsite|restore' --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml' --glob '*.md'
```

## Disaster recovery and replication

- Replica counts must be odd; the documented ceiling conflicts between pages, so verify the contract limit [IONOS-MDB-E16] [IONOS-MDB-E17].
- Playground is a single instance with no replica failover [IONOS-MDB-E22].
- Weakened per-operation write concern undoes the default `majority` with `j: true` [IONOS-MDB-E18].
- Clients must reconnect after the weekly maintenance window [IONOS-MDB-E21].

```bash
rg -n -i 'instances?\b|replica|writeConcern|w=1|journal|maintenance.?window' --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml' --glob '*.js' --glob '*.ts' --glob '*.py'
```

## Cost

- Billing follows edition, template size (Business) or per-node CPU, RAM and storage times instances and shards (Enterprise) [IONOS-MDB-E23] [IONOS-MDB-E27].
- Sharded config servers are not billed; backup and traffic are left out of the creation estimate [IONOS-MDB-E24] [IONOS-MDB-E26].
- Never copy prices into repository docs.

```bash
rg -n -i 'template|edition|shards?\b|cores|\bram\b|storage.?size|storage.?type' --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml'
```

## Monitoring

- The documented metrics path targets Enterprise clusters with the `ionos_dbaas_mongodb_` prefix [IONOS-MDB-E28] [IONOS-MDB-E29].
- Alerts should include replication lag and unexpected elections, with 70 percent usage thresholds [IONOS-MDB-E30] [IONOS-MDB-E31] [IONOS-MDB-E32].
- No alarm resource exists, so alert rules live in your own Grafana or Prometheus setup.

```bash
rg -n -i 'ionos_dbaas_mongodb|replication_lag|member_state|oplog_window|prometheus|grafana' --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml'
```

## Performance

- Driver pools must stay below the RAM-derived connection limit [IONOS-MDB-E35].
- Sharding is Enterprise only and needs a good shard key [IONOS-MDB-E39].
- Per-instance ceilings and IOPS caps depend on edition and disk size [IONOS-MDB-E36] [IONOS-MDB-E37].

```bash
rg -n -i 'maxPoolSize|minPoolSize|shard.?key|storage.?type|ssd' --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml' --glob '*.js' --glob '*.ts' --glob '*.py'
```

## IAM

- Database users are managed through the IONOS API only, at most 100 per cluster [IONOS-MDB-E41] [IONOS-MDB-E42].
- Application accounts should hold per-database `read` or `readWrite`; flag the `AnyDatabase` roles and `dbAdmin` [IONOS-MDB-E43].
- Control-plane access relies on the DCD group privilege Access and manage DBaaS [IONOS-MDB-E44].

```bash
rg -n -i 'readAnyDatabase|readWriteAnyDatabase|dbAdminAnyDatabase|dbAdmin|readWrite|clusterMonitor' --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml'
```

## Tagging

- Tagging for MongoDB clusters is undocumented; do not accept tag-based access control or billing allocation as proof.

```bash
rg -n -i 'tags?\s*[=:]' --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml'
```

## Encryption

- TLS applies to client and instance traffic in all editions; flag clients that disable verification [IONOS-MDB-E45].
- At-rest encryption is documented for Business and Enterprise data volumes; customer-managed keys are unconfirmed [IONOS-MDB-E46] [IONOS-MDB-E47] [IONOS-MDB-E48].

```bash
rg -n -i 'tlsInsecure|tlsAllowInvalid|tls=false|ssl=false|tlsCAFile' --glob '*.env*' --glob '*.json' --glob '*.yaml' --glob '*.yml' --glob '*.js' --glob '*.ts' --glob '*.py'
```

## Network isolation

- Clusters attach to a private LAN with one IP or subnet per instance; avoid the reserved ranges `172.16.0.0/12`, `192.168.230.0/24` and `10.208.0.0/12` [IONOS-MDB-E50] [IONOS-MDB-E51].
- Enterprise clusters are reachable only through private LANs; clients still resolve public DNS names [IONOS-MDB-E49] [IONOS-MDB-E52].

```bash
rg -n -i 'lan|cidr|172\.16\.|192\.168\.230\.|10\.208\.' --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml'
```

## Audit logging

- The Activity Log is contract-level and does not name MongoDB; confirm coverage before relying on it [IONOS-MDB-E53].
- mongod logs have 30-day retention and are an operational log, not a data audit [IONOS-MDB-E33] [IONOS-MDB-E34].

```bash
rg -n -i 'activity.?log|mongod.?log|logging.?service|audit' --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml'
```
