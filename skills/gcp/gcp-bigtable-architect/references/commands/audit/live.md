# audit --mode live

Commands are examples for an operator holding read-only authorization; the skill
does not run them. They read Bigtable control-plane metadata only. Expectations come
from [the operations reference](../../operations.md).

## Before any command

- Confirm the approved principal, project, instance, clusters and tables. Stop on any
  mismatch; never switch target to make a command work. Bigtable instance IDs and
  cluster locations come from the approved scope, not from discovery.
- Use a read-only role. Write output to a private directory (`umask 077`). Never
  print keys, connection strings or customer data. Do not read table rows.

Identity check, using the active account and the active configuration [BT-E901] [BT-E902]:

```bash
set -euo pipefail
: "${GCP_PROJECT:?Approved project ID}" "${BT_INSTANCE:?Approved instance ID}" \
  "${BT_TABLE:?Approved table ID}" "${EXPECTED_ACCOUNT:?Approved principal}"
umask 077
audit_dir=$(mktemp -d)
gcloud auth list --filter=status:ACTIVE --format="value(account)" > "$audit_dir/account.txt"
gcloud config list --format=json > "$audit_dir/config.json"
grep -qxF "$EXPECTED_ACCOUNT" "$audit_dir/account.txt" || { echo "Principal mismatch; stop" >&2; exit 1; }
jq -e --arg p "$GCP_PROJECT" '.core.project == $p' "$audit_dir/config.json" >/dev/null \
  || { echo "Project mismatch; stop" >&2; exit 1; } # core/project property [BT-E902]
```

The blocks below assume the variables and `audit_dir` from this block.

## Backup and restore

Backups on the approved instance, and the table's replication and schema metadata
[BT-E906] [BT-E907]:

```bash
gcloud bigtable backups list --instance="$BT_INSTANCE" --project="$GCP_PROJECT" --format=json > "$audit_dir/backups.json"
gcloud bigtable instances tables describe "$BT_TABLE" --instance="$BT_INSTANCE" --view=full --project="$GCP_PROJECT" --format=json > "$audit_dir/table.json"
```

## Disaster recovery and replication

Clusters (locations, state) and app profiles (routing policy) of the instance
[BT-E904] [BT-E905]:

```bash
gcloud bigtable clusters list --instances="$BT_INSTANCE" --project="$GCP_PROJECT" --format=json > "$audit_dir/clusters.json"
gcloud bigtable app-profiles list --instance="$BT_INSTANCE" --project="$GCP_PROJECT" --format=json > "$audit_dir/app-profiles.json"
```

## Cost

**No live command:** spend lives in Cloud Billing reports and the billing export,
not in the Bigtable resource API. Searched the gcloud Bigtable reference; no cost
command exists. Node counts and storage types come from `clusters.json` in the
section above, and spend needs the billing console or an approved billing export.

## Monitoring

Alerting policies in the project; filter for Bigtable metrics in the output [BT-E913]:

```bash
gcloud monitoring policies list --project="$GCP_PROJECT" --format=json > "$audit_dir/alert-policies.json"
```

## Performance

**No live command:** CPU, hot tablets and Key Visualizer data come from Cloud
Monitoring metrics and the Key Visualizer console, not from a gcloud Bigtable
command. Searched the gcloud Bigtable reference; none returns them. Use the
metrics views named in the operations reference with an approved Monitoring read.

## IAM

Instance, table and project policies [BT-E908] [BT-E909] [BT-E910]:

```bash
gcloud bigtable instances get-iam-policy "$BT_INSTANCE" --project="$GCP_PROJECT" --format=json > "$audit_dir/instance-iam.json"
gcloud bigtable instances tables get-iam-policy "$BT_TABLE" --instance="$BT_INSTANCE" --project="$GCP_PROJECT" --format=json > "$audit_dir/table-iam.json"
gcloud projects get-iam-policy "$GCP_PROJECT" --format=json > "$audit_dir/project-iam.json"
```

## Tagging

Tag bindings on the instance; set the parent to the instance's full resource name
from the approved scope [BT-E912] [BT-E903]:

```bash
: "${INSTANCE_FULL_RESOURCE_NAME:?Approved full resource name of the instance}"
gcloud resource-manager tags bindings list --parent="$INSTANCE_FULL_RESOURCE_NAME" --effective --format=json > "$audit_dir/tag-bindings.json"
gcloud bigtable instances describe "$BT_INSTANCE" --project="$GCP_PROJECT" --format=json > "$audit_dir/instance.json"
```

## Encryption

Cluster metadata carries the encryption configuration; the table `encryption` view
returns only the encryption-related fields [BT-E904] [BT-E907]:

```bash
gcloud bigtable clusters list --instances="$BT_INSTANCE" --project="$GCP_PROJECT" --format=json > "$audit_dir/clusters-encryption.json"
gcloud bigtable instances tables describe "$BT_TABLE" --instance="$BT_INSTANCE" --view=encryption --project="$GCP_PROJECT" --format=json > "$audit_dir/table-encryption.json"
```

## Network isolation

**No live command:** VPC Service Controls perimeters and Private Service Connect
endpoints belong to the Access Context Manager and VPC APIs, whose reference pages
were not retrieved in this pass. Bigtable has no per-instance public-endpoint
setting to read.

## Audit logging

Recent audit log entries for the Bigtable service; the format keeps only time,
method and caller, not request payloads [BT-E911]:

```bash
gcloud logging read 'protoPayload.serviceName="bigtable.googleapis.com"' --project="$GCP_PROJECT" --freshness=7d --limit=50 --format="value(timestamp,protoPayload.methodName,protoPayload.authenticationInfo.principalEmail)" > "$audit_dir/audit-log.txt"
```
