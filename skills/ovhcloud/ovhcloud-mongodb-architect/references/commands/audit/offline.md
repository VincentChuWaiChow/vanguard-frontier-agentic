# audit --mode offline

Inputs are the repository (IaC, application config, client code) and sanitized
exports supplied by the user. No cloud CLI, network call, credential or customer
data. Expectations come from [the operations reference](../../operations.md); a
missing match is UNVERIFIED, not a pass.

```bash
git rev-parse --short HEAD
find . -type f \( -name '*.tf' -o -name '*.json' -o -name '*.yaml' -o -name '*.yml' \) -not -path './node_modules/*' | head -50
```

## Backup and restore

- The `backups` attribute sets the backup region and daily time; for MongoDB the regions array takes one element [OVH-MDB-E05].
- Retention differs by plan: Production keeps 7 days, Advanced keeps 30 days with point-in-time recovery for 24 hours [OVH-MDB-E02] [OVH-MDB-E03].
- Restore runbooks must name fork (new service) versus in-place restore, and recreate IP restrictions and users after a fork [OVH-MDB-E07] [OVH-MDB-E08] [OVH-MDB-E09].
- Logical backups must use the `backup` and `restore` roles and keep dumps out of the repository [OVH-MDB-E10].

```bash
rg -n -i 'backups?\b|backup.?(time|region)|point.?in.?time|\bfork\b' --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml' --glob '*.md'
```

## Disaster recovery and replication

- High availability starts at 3 nodes; a 1-node-style setup is not HA [OVH-MDB-E22].
- Region choice decides the zone model: 3-AZ regions survive a zone loss, 1-AZ regions may not survive a region outage [OVH-MDB-E15] [OVH-MDB-E16].
- Cross-region recovery is a restore of a remote-region backup into a fork; flag any claim of continuous cross-region replication as UNVERIFIED [OVH-MDB-E05] [OVH-MDB-E08].
- Check that stated SLA figures match the plan [OVH-MDB-E20] [OVH-MDB-E21].

```bash
rg -n -i 'nodes?\b|node.?count|region|availability.?zone|\bsla\b' --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml'
```

## Cost

- Billing dimensions are compute, storage and backup; look for sizing choices (flavor, node count, storage) in config [OVH-MDB-E27] [OVH-MDB-E29].
- Storage is billed on allocated capacity, and the resize rule leaves at least 20% free [OVH-MDB-E29] [OVH-MDB-E50].
- The Discovery plan is end of life; flag services still declared on it [OVH-MDB-E35].
- Never copy unit prices into the repository docs.

```bash
rg -n -i 'flavor|node.?flavor|storage|\bplan\b|discovery' --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml'
```

## Monitoring

- Prometheus export is enabled per service and follows its IP restrictions and private-network reachability [OVH-MDB-E41] [OVH-MDB-E42].
- Alerts should cover CPU, memory, disk I/O, replication lag and slow queries; disk-fill emails are automatic [OVH-MDB-E45] [OVH-MDB-E46] [OVH-MDB-E48].
- Logs forwarding to Logs Data Platform is optional [OVH-MDB-E44].

```bash
rg -n -i 'prometheus|alert|logs.?data.?platform|replication.?lag|oplog' --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml'
```

## Performance

- Client read preference and write concern defaults are primary and majority; `w=1` trades durability for latency [OVH-MDB-E26] [OVH-MDB-E77].
- Analytics queries belong on an Analytics node, an Advanced feature [OVH-MDB-E23] [OVH-MDB-E25].
- Storage growth plans must keep headroom, since disk full makes MongoDB refuse writes [OVH-MDB-E49] [OVH-MDB-E50].

```bash
rg -n -i 'readPreference|w=1|writeConcern|analytics' --glob '*.env*' --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml' --glob '*.js' --glob '*.ts' --glob '*.py'
```

## IAM

- Database users should hold narrow built-in roles per database; flag `root`, `userAdmin` and `dbAdmin` on application accounts [OVH-MDB-E56] [OVH-MDB-E59].
- Do not manage technical users such as `admin@admin`; shell-created users are removed by the automation [OVH-MDB-E57].
- Control-plane access should rely on OVHcloud IAM policies; MongoDB-specific action names are undocumented [OVH-MDB-E58].

```bash
rg -n -i 'userAdmin|dbAdmin|readWrite|clusterMonitor|\broot\b|roles?\b' --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml'
```

## Tagging

- IAM policy conditions can use resource tags account-wide, but a MongoDB service carrying tags is unconfirmed; do not accept tag-based access control for it without live evidence [OVH-MDB-E61].

```bash
rg -n -i 'tags?\s*[=:]' --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml'
```

## Encryption

- Customer-managed keys are not offered; reject designs that depend on them [OVH-MDB-E64].
- Field level encryption with customer keys is Advanced only [OVH-MDB-E65].
- Clients must use TLS 1.2 or 1.3 with certificate verification left on [OVH-MDB-E66] [OVH-MDB-E67].

```bash
rg -n -i 'tlsInsecure|tlsAllowInvalid|tls=false|ssl=false|tlsCAFile' --glob '*.env*' --glob '*.json' --glob '*.yaml' --glob '*.yml' --glob '*.js' --glob '*.ts' --glob '*.py'
```

## Network isolation

- No outside connection is accepted until authorised IPs are set; flag `0.0.0.0/0` [OVH-MDB-E69] [OVH-MDB-E74].
- Private mode needs a vRack network you own; switching between public and private causes downtime and clears IP restrictions when moving to public [OVH-MDB-E71] [OVH-MDB-E72].
- Use DNS, not a pinned IP, because the service IP changes [OVH-MDB-E73].

```bash
rg -n -i '0\.0\.0\.0/0|ip.?restriction|vrack|private.?network|subnet' --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml'
```

## Audit logging

- Control-plane calls are logged by the account audit and API-call logs; check the retention and review plan [OVH-MDB-E75].
- Data-plane MongoDB auditing has no documented enablement procedure; mark it UNVERIFIED [OVH-MDB-E76].
- Service logs hold only the last 1000 lines [OVH-MDB-E43].

```bash
rg -n -i 'audit|logs.?data.?platform|log.?forward' --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml'
```
