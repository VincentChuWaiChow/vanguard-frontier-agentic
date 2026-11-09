# audit --mode live

Commands are examples for an operator holding read-only authorization; the skill
does not run them. They read Firestore control-plane metadata only and never read
documents or collections. Expectations come from
[Native operations](../../firestore-operations.md) or
[MongoDB operations](../../firestore-mongodb-operations.md). The Firestore Admin API
commands below take a database ID; whether every field applies to an Enterprise
(MongoDB-compatible) database is unverified, so read the `databaseEdition` field in
the output before applying a Native-mode expectation.

## Before any command

- Confirm the approved principal, project, database ID, location and mode. Stop on any
  mismatch; never switch target to make a command work.
- Use a read-only role. Write output to a private directory (`umask 077`). Never
  print keys, connection strings, SCRAM credentials or customer data.

Identity check, using the active account and the active configuration [FS-E901] [FS-E902]:

```bash
set -euo pipefail
: "${GCP_PROJECT:?Approved project ID}" "${FS_DATABASE:?Approved database ID}" \
  "${FS_LOCATION:?Approved database location}" "${EXPECTED_ACCOUNT:?Approved principal}"
umask 077
audit_dir=$(mktemp -d)
gcloud auth list --filter=status:ACTIVE --format="value(account)" > "$audit_dir/account.txt"
gcloud config list --format=json > "$audit_dir/config.json"
grep -qxF "$EXPECTED_ACCOUNT" "$audit_dir/account.txt" || { echo "Principal mismatch; stop" >&2; exit 1; }
grep -qF "\"$GCP_PROJECT\"" "$audit_dir/config.json" || { echo "Project mismatch; stop" >&2; exit 1; }
```

The blocks below assume the variables and `audit_dir` from this block.

## Backup and restore

- Native mode: schedules, backups and the PITR setting in the database metadata.
- MongoDB compatibility: the same commands; confirm in the output that they apply.

Sources [FS-E903] [FS-E904] [FS-E905]:

```bash
gcloud firestore databases describe --database="$FS_DATABASE" --project="$GCP_PROJECT" --format=json > "$audit_dir/database.json"
gcloud firestore backups schedules list --database="$FS_DATABASE" --project="$GCP_PROJECT" --format=json > "$audit_dir/schedules.json"
gcloud firestore backups list --location="$FS_LOCATION" --project="$GCP_PROJECT" --format=json > "$audit_dir/backups.json"
```

## Disaster recovery and replication

Location and type of the database come from the metadata read above; the same file
serves both modes [FS-E903]:

```bash
gcloud firestore databases describe --database="$FS_DATABASE" --project="$GCP_PROJECT" --format=json > "$audit_dir/database-location.json"
```

## Cost

**No live command:** spend lives in Cloud Billing reports and the billing export, not
in the Firestore resource API. Searched the gcloud Firestore reference; no cost
command exists. Usage metrics need an approved Monitoring read.

## Monitoring

Alerting policies in the project; filter for Firestore metrics in the output [FS-E909]:

```bash
gcloud monitoring policies list --project="$GCP_PROJECT" --format=json > "$audit_dir/alert-policies.json"
```

## Performance

**No live command:** latency, contention and hotspot signals come from Cloud
Monitoring metrics and client error codes, not from a gcloud Firestore command.
Searched the gcloud Firestore reference; none returns them.

## IAM

Project-level policy; look for the `roles/datastore.*` roles and Conditions [FS-E906].
No gcloud command that reads a database-level IAM policy was retrieved in this pass,
so database-scoped bindings are UNVERIFIED by this audit:

```bash
gcloud projects get-iam-policy "$GCP_PROJECT" --format=json > "$audit_dir/project-iam.json"
```

## Tagging

Tag bindings on the database; set the parent to the database's full resource name
from the approved scope [FS-E908]:

```bash
: "${DATABASE_FULL_RESOURCE_NAME:?Approved full resource name of the database}"
gcloud resource-manager tags bindings list --parent="$DATABASE_FULL_RESOURCE_NAME" --effective --format=json > "$audit_dir/tag-bindings.json"
```

## Encryption

The database metadata carries the CMEK configuration (`cmekConfig`) when CMEK is
enabled, for both modes [FS-E903] [FS-E910]:

```bash
gcloud firestore databases describe --database="$FS_DATABASE" --project="$GCP_PROJECT" --format=json > "$audit_dir/database-encryption.json"
```

## Network isolation

**No live command:** VPC Service Controls perimeters and Private Google Access
belong to the Access Context Manager and VPC APIs, whose reference pages were not
retrieved in this pass. No documented Firestore command lists a database's network
settings.

## Audit logging

Recent audit log entries for the Firestore service name; the format keeps only time,
method and caller, not request payloads [FS-E907]:

```bash
gcloud logging read 'protoPayload.serviceName="datastore.googleapis.com"' --project="$GCP_PROJECT" --freshness=7d --limit=50 --format="value(timestamp,protoPayload.methodName,protoPayload.authenticationInfo.principalEmail)" > "$audit_dir/audit-log.txt"
```
