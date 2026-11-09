# audit --mode live

Commands are examples for an operator holding read-only authorization; the skill
does not run them. They read control-plane metadata only and never open a MongoDB
connection. Expectations come from [the operations reference](../../operations.md).

## Before any command

- Confirm the approved principal, Scaleway Organization, Project, Region and exact
  Database Instance. Stop on any mismatch; never switch target to make a command work.
- Use a read-only principal. Write output to a private directory (`umask 077`). Never
  print keys, connection strings or customer data; review saved output before sharing.

```bash
umask 077
: "${SCW_ORGANIZATION_ID:?Approved Scaleway Organization ID}"
: "${SCW_PROJECT_ID:?Approved Scaleway Project ID}"
: "${SCW_REGION:?Approved region, for example fr-par}"
: "${SCW_MDB_INSTANCE_ID:?Approved Managed MongoDB Instance ID}"
OUT="$(mktemp -d)"
# Show the active profile values and keep only the identity fields [SCW-MDB-E903]
scw config info | grep -i -E 'organization|project|region|profile'
```

## Backup and restore

```bash
scw mongodb snapshot list instance-id="$SCW_MDB_INSTANCE_ID" region="$SCW_REGION" > "$OUT/snapshots.txt" # [SCW-MDB-E901]
```

## Disaster recovery and replication

```bash
scw mongodb instance get "$SCW_MDB_INSTANCE_ID" region="$SCW_REGION" > "$OUT/instance.txt" # [SCW-MDB-E901]
```

Read the node count and status from the saved output; do not paste it unredacted
because it can include endpoints.

## Cost

**No live command:** billing data lives in the Scaleway billing console and billing API, not in the `scw mongodb` commands; the CLI page lists no cost command.

## Monitoring

**No live command:** Cockpit metrics and alert rules are not exposed by the `scw mongodb` commands. Searched the `scw mongodb` reference page; no Cockpit command is documented there.

## Performance

```bash
scw mongodb instance list project-id="$SCW_PROJECT_ID" region="$SCW_REGION" > "$OUT/instances.txt" # [SCW-MDB-E901]
```

Compare node type and volume settings in the saved output with the operations reference.

## IAM

```bash
scw mongodb user list instance-id="$SCW_MDB_INSTANCE_ID" region="$SCW_REGION" > "$OUT/db-users.txt" # [SCW-MDB-E901]
scw iam policy list organization-id="$SCW_ORGANIZATION_ID" > "$OUT/iam-policies.txt" # [SCW-MDB-E904]
```

## Tagging

```bash
scw mongodb instance list project-id="$SCW_PROJECT_ID" region="$SCW_REGION" > "$OUT/instances-tags.txt" # [SCW-MDB-E901]
```

The CLI page documents tags as a create argument and a list filter, which the
operations reference does not record. Check the saved output for tags, and treat the
operations reference status as UNVERIFIED until a tagged Instance confirms it.

## Encryption

**No live command:** the documented `scw mongodb` commands expose no encryption-at-rest or key setting; `scw mongodb instance get-certificate` is excluded because it reads a certificate. Encryption at rest is undocumented for this product.

## Network isolation

```bash
scw mongodb instance get "$SCW_MDB_INSTANCE_ID" region="$SCW_REGION" > "$OUT/instance-network.txt" # [SCW-MDB-E901]
```

Look for public versus Private Network endpoints in the saved output.

## Audit logging

```bash
scw audit-trail event list organization-id="$SCW_ORGANIZATION_ID" project-id="$SCW_PROJECT_ID" resource-type=mongodb_instance region="$SCW_REGION" # [SCW-MDB-E902]
```
