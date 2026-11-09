# audit --mode live

Commands are examples for an operator holding read-only authorization; the skill
does not run them. Scope is IONOS DBaaS MongoDB only. They read control-plane
metadata only and never open a MongoDB connection. Expectations come from
[the MongoDB operations reference](../../mongodb-operations.md).

## Before any command

- Confirm the approved principal, IONOS contract, data center location and exact
  cluster ID. Stop on any mismatch; never switch target to make a command work.
- Use a read-only principal and keep `IONOS_TOKEN` in the environment; never echo
  or print it. Write output to a private directory (`umask 077`). The cluster
  columns below omit the connection string (URL); never print keys, connection
  strings or customer data.
- The documented identity command is `ionosctl config whoami` [IONOS-MDB-E905]. It
  is described in prose, not in a command block, because the gate's read-verb list
  has no word matching `whoami`.

```bash
umask 077
: "${IONOS_TOKEN:?Approved IONOS token in the environment}"
: "${IONOS_MDB_CLUSTER_ID:?Approved MongoDB cluster ID}"
OUT="$(mktemp -d)"
COLS="ClusterId,Name,Edition,Type,Instances,Shards,Health,State,MongoVersion,MaintenanceWindow,Location,DatacenterId,LanId,Cidr,TemplateId,Cores,RAM,StorageSize,StorageType"
ionosctl dbaas mongo cluster list --cols "$COLS" # [IONOS-MDB-E902]
```

## Backup and restore

```bash
ionosctl dbaas mongo snapshot list --cluster-id "$IONOS_MDB_CLUSTER_ID" > "$OUT/snapshots.txt" # [IONOS-MDB-E903]
```

## Disaster recovery and replication

```bash
ionosctl dbaas mongo cluster get --cluster-id "$IONOS_MDB_CLUSTER_ID" --cols ClusterId,Edition,Type,Instances,Shards,Health,State,MongoVersion,Location > "$OUT/cluster-ha.txt" # [IONOS-MDB-E901]
```

## Cost

**No live command:** billing lives in the IONOS billing and contract tools, not in the `dbaas mongo` commands; the CLI pages list no cost command.

## Monitoring

**No live command:** metrics are exposed through the documented monitoring tutorial and your Grafana or Prometheus setup, not through an `ionosctl dbaas mongo` command. `ionosctl dbaas mongo logs list` is excluded because it returns server log lines that can hold customer data.

## Performance

```bash
ionosctl dbaas mongo cluster get --cluster-id "$IONOS_MDB_CLUSTER_ID" --cols ClusterId,TemplateId,Cores,RAM,StorageSize,StorageType,Instances,Shards > "$OUT/cluster-sizing.txt" # [IONOS-MDB-E901]
```

## IAM

```bash
ionosctl dbaas mongo user list --cluster-id "$IONOS_MDB_CLUSTER_ID" --cols Username,CreatedBy,Roles > "$OUT/db-users.txt" # [IONOS-MDB-E904]
```

## Tagging

**No live command:** the `dbaas mongo` cluster pages list no tag column or option, and the operations reference records tagging as UNVERIFIED. Searched the `cluster get`, `cluster list` and operations pages.

## Encryption

**No live command:** the `dbaas mongo cluster get` page documents no encryption or key column; encryption is a platform property described in the operations reference.

## Network isolation

```bash
ionosctl dbaas mongo cluster get --cluster-id "$IONOS_MDB_CLUSTER_ID" --cols ClusterId,DatacenterId,LanId,Cidr > "$OUT/cluster-network.txt" # [IONOS-MDB-E901]
```

## Audit logging

**No live command:** the Activity Log is a contract-level IONOS API, and no `ionosctl dbaas mongo` audit command is documented. Searched the `dbaas mongo` command index for audit and activity commands.
