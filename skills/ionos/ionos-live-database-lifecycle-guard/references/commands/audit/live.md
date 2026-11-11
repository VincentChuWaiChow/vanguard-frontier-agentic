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
- The identity command `ionosctl config whoami` prints the email of the logged-in
  user [IONOS-MDB-E905]; the block stops unless that line equals the approved email.

```bash
set -euo pipefail
umask 077
: "${IONOS_TOKEN:?Approved IONOS token in the environment}"
: "${IONOS_EXPECTED_USER:?Approved principal email}"
: "${IONOS_MDB_CLUSTER_ID:?Approved MongoDB cluster ID}"
: "${IONOS_MDB_LOCATION:?Approved cluster location}"
OUT="$(mktemp -d)"
ionosctl config whoami > "$OUT/whoami.txt" # [IONOS-MDB-E905]
grep -qxF "$IONOS_EXPECTED_USER" "$OUT/whoami.txt" || { echo "Principal mismatch; stop" >&2; exit 1; }
COLS="ClusterId,Name,Edition,Type,Instances,Shards,Health,State,MongoVersion,MaintenanceWindow,Location,DatacenterId,LanId,Cidr,TemplateId,Cores,RAM,StorageSize,StorageType"
ionosctl dbaas mongo cluster list --cols "$COLS" > "$OUT/clusters.txt" # [IONOS-MDB-E902]
grep -F "$IONOS_MDB_CLUSTER_ID" "$OUT/clusters.txt" | grep -wF "$IONOS_MDB_LOCATION" >/dev/null \
  || { echo "Cluster or location mismatch; stop" >&2; exit 1; }
```

The `whoami` page describes its output as the email, not an exact line format; if
the check stops on a correct principal, confirm it by hand rather than loosening it.
The cluster check reads the documented text columns, not a JSON field.

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
