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

- Snapshot schedule set in IaC or API payloads: `is_snapshot_schedule_enabled`, `snapshot_schedule_frequency_hours`, `snapshot_schedule_retention_days`; compare retention with the documented 365-day default [SCW-MDB-E02] [SCW-MDB-E03].
- Restore runbooks must treat a restore as a new, separately billed Instance, not an in-place rollback [SCW-MDB-E04].
- Engine upgrade runbooks must stop writes first, because the upgrade does not synchronize data [SCW-MDB-E22].
- Logical backups (MongoDB dump and restore tools) must use TLS and keep dumps out of the repository [SCW-MDB-E06].

```bash
rg -n -i 'snapshot_schedule|snapshot.?retention|dump.?restore|restore.?point' --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml' --glob '*.sh'
```

## Disaster recovery and replication

- Production-class deployments should request a 3-node replica set; a 1-node set has no high availability [SCW-MDB-E07] [SCW-MDB-E08].
- Snapshots stay in the original location, so a runbook that claims a cross-Region copy needs other evidence [SCW-MDB-E01].
- `mongosync` is a migration tool, not a DR mechanism [SCW-MDB-E26].
- RPO, RTO and failover timing claims have no provider figure; mark them UNVERIFIED.

```bash
rg -n -i 'replica.?set|node.?count|mongosync|failover|rpo|rto' --glob '*.tf' --glob '*.json' --glob '*.md'
```

## Cost

- Look for the billing dimensions in config: node type, replica set size, volume size and IOPS setting [SCW-MDB-E09] [SCW-MDB-E10].
- Volumes grow but never shrink, so check for oversized volume requests [SCW-MDB-E13].
- Custom metric or log push to Cockpit is billable [SCW-MDB-E12].
- Orphaned snapshots outlive the Instance until retention expires; list them in the runbook cleanup [SCW-MDB-E05].

```bash
rg -n -i 'node.?type|volume.?size|iops|replica.?set' --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml'
```

## Monitoring

- Cockpit integrates MongoDB metrics but not logs; alert rules must not assume log ingestion [SCW-MDB-E11].
- Dashboards or agents pushing through an external path are billable [SCW-MDB-E12].
- No provider-named metric identifiers exist in the reference; treat copied PostgreSQL or MySQL metric names as defects.

```bash
rg -n -i 'cockpit|grafana|alertmanager|prometheus' --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml'
```

## Performance

- Check the IOPS setting (5k to 15k) and volume size against the stated workload [SCW-MDB-E10].
- Block Storage Low Latency needs new-generation node types and PAR or AMS regions [SCW-MDB-E14].
- Client pools and timeouts are engine-level; no provider throttling signal is documented, so load-test evidence is required.

```bash
rg -n -i 'iops|low.?latency|maxPoolSize|serverSelectionTimeoutMS' --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml' --glob '*.js' --glob '*.ts' --glob '*.py'
```

## IAM

- Control-plane rules should scope MongoDB permission sets to a Project, not the whole Organization; the permission-set names `MongoDBReadOnly` and `MongoDBFullAccess` are unconfirmed, so verify them in the IAM console [SCW-MDB-E16].
- Database users should hold per-database roles; flag global `db_admin` and `sync` outside migrations [SCW-MDB-E17].
- Credentials must come from a secret store, never from committed files.

```bash
rg -n -i 'MongoDBFullAccess|MongoDBReadOnly|db_admin|read_write|\bsync\b' --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml'
```

## Tagging

- The operations reference records tagging as UNVERIFIED for Managed MongoDB; do not treat a tag in IaC as proof of tag-based access control.
- Flag IaC that depends on tags for IAM scoping or cost allocation without a live check.

```bash
rg -n -i 'tags?\s*[=:]' --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml'
```

## Encryption

- Connection strings must use TLS with the downloaded certificate (`tlsCAFile`) and never disable certificate checks [SCW-MDB-E18].
- Encryption at rest and customer-managed keys are undocumented for this product; do not accept claims of either without a Scaleway source.

```bash
rg -n -i 'tlsCAFile|tlsInsecure|tlsAllowInvalid|tls=false|ssl=false' --glob '*.env*' --glob '*.json' --glob '*.yaml' --glob '*.yml' --glob '*.js' --glob '*.ts' --glob '*.py'
```

## Network isolation

- Prefer Private Network attachment; an Instance attaches to one Private Network in the same Region [SCW-MDB-E19] [SCW-MDB-E20].
- Public connectivity is visible by default; require a written reason for it [SCW-MDB-E21].
- After an engine upgrade the new Instance has a public endpoint by default, so check upgrade runbooks for an exposure review [SCW-MDB-E22].

```bash
rg -n -i 'private.?network|public.?(endpoint|connectivity)|mongodb\+srv' --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml'
```

## Audit logging

- Audit Trail covers MongoDB control-plane calls only, and the integration is listed for fr-par only [SCW-MDB-E23] [SCW-MDB-E24].
- Check for a retention or export plan: exports cover the past 90 days to Object Storage [SCW-MDB-E25].
- Data-plane auditing of queries or logins is undocumented; mark it UNVERIFIED.

```bash
rg -n -i 'audit.?trail|audit.?log|object.?storage' --glob '*.tf' --glob '*.json' --glob '*.yaml' --glob '*.yml'
```
