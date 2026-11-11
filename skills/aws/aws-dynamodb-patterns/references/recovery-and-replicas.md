# PITR recovery and multi-Region replicas

These are independent protections. Replication also copies accidental writes and
deletes; it is not a historical backup. PITR is not an automatic failover plan.
Examples are generated guidance only: restore/replica creation is a billable
mutation requiring exact target, data-residency, IAM/KMS and rollback approval.
Follow the existing IaC owner rather than creating CLI drift.

```bash
set -euo pipefail
: "${AWS_PROFILE:?}" "${AWS_REGION:?}" "${TABLE_NAME:?}"
aws_target=(aws --profile "$AWS_PROFILE" --region "$AWS_REGION" --no-cli-pager --output json)
```

## Domain: point-in-time recovery

PITR's recovery period is configurable from 1–35 days. Check the actual
PointInTimeRecoveryStatus, RecoveryPeriodInDays, EarliestRestorableDateTime and
LatestRestorableDateTime; the latest is typically about five minutes behind now.
ContinuousBackupsStatus=ENABLED alone does not establish PITR enablement. A new
setting does not manufacture earlier history; disabling/re-enabling resets it.
Reducing the retention window is not documented to reduce PITR price.

### Commands: inspect, enable, restore to a NEW table

```bash
"${aws_target[@]}" dynamodb describe-continuous-backups --table-name "$TABLE_NAME"
# Approved settings change, not an item mutation or restore test.
"${aws_target[@]}" dynamodb update-continuous-backups --table-name "$TABLE_NAME" \
  --point-in-time-recovery-specification PointInTimeRecoveryEnabled=true,RecoveryPeriodInDays=35
"${aws_target[@]}" dynamodb describe-continuous-backups --table-name "$TABLE_NAME"
```

Select a point inside the returned bounds; the following deliberately uses the
latest restorable point, not wall-clock now. For a chosen timestamp, use
`--restore-date-time` **instead of** `--use-latest-restorable-time`.

```bash
: "${RESTORE_TABLE_NAME:?Approved distinct NEW target table name}"
if [[ "$RESTORE_TABLE_NAME" == "$TABLE_NAME" ]]; then
  printf '%s\n' 'Restore target must be a new, distinct table.' >&2
  exit 64
fi
"${aws_target[@]}" dynamodb restore-table-to-point-in-time \
  --source-table-name "$TABLE_NAME" --target-table-name "$RESTORE_TABLE_NAME" \
  --use-latest-restorable-time
"${aws_target[@]}" dynamodb wait table-exists --table-name "$RESTORE_TABLE_NAME"
"${aws_target[@]}" dynamodb describe-table --table-name "$RESTORE_TABLE_NAME"
```

Never overwrite/delete/recreate the source to simulate an in-place restore.
Reconcile an unknown create/restore outcome before retrying. The restored table
is independent, including when the source was a global-table replica; rejoining
or forming a new global table is another approved design/operation.

### Permissions and recovery acceptance

DescribeContinuousBackups/UpdateContinuousBackups target the source table. Restore
uses dynamodb:RestoreTableToPointInTime on the relevant table resources; review
source and new target ARNs plus operation/dependent rights from the current SAR,
and KMS key/grant access throughout recovery. On-demand backup recovery is a
different RestoreTableFromBackup path that also involves the backup ARN. Do not
infer restore authority from GetItem/CreateTable alone or use wildcard admin as
a diagnostic shortcut.

Inspect the actual restored schema, indexes, encryption, billing/throughput and
deletion protection; do not assume table settings equal the historical data point.
Re-establish tags, IAM/resource policy, PITR, TTL, Streams/consumers, auto scaling,
Contributor Insights, alarms and application routing as required. Existing backups
can still depend on older KMS keys. Test representative positive/negative access,
data invariants and recovery duration before a separately approved cutover. Keep
the source/old route available for rollback; a table waiter proves neither data
parity nor application acceptance.

## Domain: current-version multi-Region replicas

Scope the example to **same-account, current-version MREC**. Inspect table status,
existing GlobalTableVersion/consistency mode/replica descriptions, source Region,
destination name conflicts, service quotas, replicated write load, KMS keys,
data-residency policy and resource/SCP denies before adding a replica. Modern
replicas use UpdateTable; do not use legacy CreateGlobalTable/UpdateGlobalTable
commands as a generic shortcut. MREC can begin from a populated regional source;
do not pre-create an unrelated empty table with the destination name.

### Commands: inspect both Regions and optionally add ONE MREC replica

```bash
: "${REPLICA_REGION:?Approved distinct destination Region}"
if [[ "$REPLICA_REGION" == "$AWS_REGION" ]]; then
  printf '%s\n' 'Replica Region must differ from source Region.' >&2
  exit 64
fi
"${aws_target[@]}" dynamodb describe-table --table-name "$TABLE_NAME"
# Approved source-Region mutation; the builder avoids malformed interpolated JSON.
replica_update=$(python3 -c 'import json,sys; print(json.dumps([{"Create":{"RegionName":sys.argv[1]}}]))' "$REPLICA_REGION")
"${aws_target[@]}" dynamodb update-table --table-name "$TABLE_NAME" \
  --replica-updates "$replica_update"
"${aws_target[@]}" dynamodb describe-table --table-name "$TABLE_NAME" \
  --query 'Table.{State:TableStatus,Replicas:Replicas,Consistency:MultiRegionConsistency}'
aws --profile "$AWS_PROFILE" --region "$REPLICA_REGION" --no-cli-pager --output json \
  dynamodb describe-table --table-name "$TABLE_NAME"
```

A destination read during creation may not succeed yet; use bounded readback with
explicit partial-state handling, not a blind retry of replica creation. Require all intended replicas
ACTIVE and verify data/authorization/failover behavior independently.

### Replica permission and settings boundaries

Current global-table security docs require UpdateTable on the source; destination
table rights include CreateTable, CreateTableReplica, Query, Scan, UpdateItem,
PutItem, GetItem, DeleteItem and BatchWriteItem. If the replication service-linked
role does not exist, its creation needs iam:CreateServiceLinkedRole restricted to
the documented service. These are deployment/replication rights, **not** a blanket
application-runtime policy. Verify resource-policy/SCP denies and regional KMS
authorizations; do not block required service-linked-role paths inadvertently.

MREC is asynchronous with per-item conflict resolution. Strong reads in one Region
can still miss a write from another Region. Transactions do not replicate as an
atomic group. TTL synchronization and replicated-delete charges differ from backup
protection. Verify PITR, tags, deletion protection, resource policies, Streams
consumers, encryption keys and alarms in **each** Region; do not infer them from
replicated data. Inspect settings-replication mode where available rather than
blindly reusing an older “all settings copy” or “nothing copies” checklist.
DAX caches are regional and not invalidated by replication into their base table.

### MRSC is a separate architecture, not this command with a new label

MRSC needs exactly three participating Regions (three replicas or two plus a
witness), an empty starting table and no writes during conversion. Mode cannot be
changed after creation; additional replicas cannot be added later. TTL, LSI and
transaction APIs are unsupported; GSI reads remain eventual. Witness creation has
its own permissions. Do not apply the MREC add-one command to an MRSC table.

Current core docs permit eligible Regions across continents; some best-practices
pages retain older Region-set restrictions. That source conflict was detected in
research: use the current core guide and verify exact intended availability before
deployment. Do not hard-code the stale restriction or promise arbitrary Regions.
Multi-account global tables have a separate resource-policy/settings contract;
this same-account command is not a multi-account onboarding procedure.

Sources: [PITR behavior/window](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/PointInTimeRecovery_Howitworks.html),
[PITR tutorial](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/PointInTimeRecovery.Tutorial.html),
[restore settings](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/pointintimerecovery_restores.html),
[global-table behavior](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/V2globaltables_HowItWorks.html),
[replica tutorial](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/V2globaltables.tutorial.html),
[replication permissions](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/globaltables-security.html),
[multi-account differences](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/V2globaltables_MA_HowItWorks.html).
