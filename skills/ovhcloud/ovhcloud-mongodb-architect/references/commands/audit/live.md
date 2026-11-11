# audit --mode live

Commands are examples for an operator holding read-only authorization; the skill
does not run them. They use the official `ovhcloud` CLI, which sends signed API
requests with the operator's own configured credentials; the skill never handles
them. They read control-plane metadata only. Expectations come from
[the operations reference](../../operations.md).

## Before any command

- Confirm the approved principal, OVHcloud account, Public Cloud project, region and
  exact service ID. Stop on any mismatch; never switch target to make a command work.
- Use a read-only principal. Write output to a private directory (`umask 077`). Never
  print keys, connection strings or customer data; the service object can contain
  connection details, so review saved output before sharing.

```bash
set -euo pipefail
umask 077
: "${OVH_CLOUD_PROJECT_SERVICE:?Approved Public Cloud project ID}"
: "${OVH_MDB_SERVICE_ID:?Approved MongoDB service (cluster) ID}"
OUT="$(mktemp -d)"
ovhcloud cloud project get "$OVH_CLOUD_PROJECT_SERVICE" --output json > "$OUT/project.json" # [OVH-MDB-E903]
ovhcloud cloud managed-database get "$OVH_MDB_SERVICE_ID" --cloud-project "$OVH_CLOUD_PROJECT_SERVICE" --output json > "$OUT/service-target.json" # [OVH-MDB-E901] [OVH-MDB-E904] [OVH-MDB-E905]
jq -e --arg s "$OVH_MDB_SERVICE_ID" '.id == $s' "$OUT/service-target.json" >/dev/null \
  || { echo "Service mismatch; stop" >&2; exit 1; }
```

The service is read through the approved project, so a match confirms it belongs
there. The project response fields and the calling principal are not documented on
the CLI pages; the project read only confirms access, so confirm the principal in
the Console before running.

## Backup and restore

```bash
ovhcloud cloud managed-database backup list "$OVH_MDB_SERVICE_ID" --cloud-project "$OVH_CLOUD_PROJECT_SERVICE" > "$OUT/backups.txt" # [OVH-MDB-E902]
```

## Disaster recovery and replication

```bash
ovhcloud cloud managed-database get "$OVH_MDB_SERVICE_ID" --cloud-project "$OVH_CLOUD_PROJECT_SERVICE" --output json > "$OUT/service.json" # [OVH-MDB-E901] [OVH-MDB-E904]
```

Read plan, node count and region from the saved file with local `jq` after listing
its keys; do not paste it unredacted.

## Cost

**No live command:** billing lives in the OVHcloud billing console and invoices, not in the managed-database commands; the CLI pages list no cost command.

## Monitoring

**No live command:** the Observability charts and Prometheus export are Console and service settings; no `ovhcloud cloud managed-database` metrics command is documented. Searched the CLI managed-database command index.

## Performance

```bash
ovhcloud cloud managed-database get "$OVH_MDB_SERVICE_ID" --cloud-project "$OVH_CLOUD_PROJECT_SERVICE" --output json > "$OUT/service-sizing.json" # [OVH-MDB-E901] [OVH-MDB-E904]
```

Compare flavor, node count and storage in the saved file with the operations reference.

## IAM

```bash
ovhcloud cloud managed-database user list "$OVH_MDB_SERVICE_ID" --cloud-project "$OVH_CLOUD_PROJECT_SERVICE" > "$OUT/db-users.txt" # [OVH-MDB-E901]
```

## Tagging

**No live command:** no page states that a MongoDB service carries tags, and the managed-database CLI commands show none. Searched the CLI managed-database command index and the operations reference.

## Encryption

**No live command:** at-rest and in-transit encryption are platform properties; the managed-database CLI commands expose no encryption setting. `ovhcloud cloud managed-database certificate` is not used because it reads certificate material.

## Network isolation

```bash
ovhcloud cloud managed-database get "$OVH_MDB_SERVICE_ID" --cloud-project "$OVH_CLOUD_PROJECT_SERVICE" --output json > "$OUT/service-network.json" # [OVH-MDB-E901] [OVH-MDB-E904]
```

Check the saved file for private-network and IP restriction fields if present; the
CLI page does not enumerate the response fields.

## Audit logging

**No live command:** account API-call and audit-log routes are signed OVHcloud API calls (see the operations reference [OVH-MDB-E75]), and no `ovhcloud` command for them was found. Run them only through an approved API client.
