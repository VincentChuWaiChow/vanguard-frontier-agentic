# backup --method on-demand

Read only this selected method; do not load PITR settings instructions.

## Approved billable backup creation

Use the shared target gate first. Choose a deliberate backup name and record the
returned ARN; creation is asynchronous, not an exact request-instant consistent
snapshot. AWS documents changes around the request's ±1-minute boundary may be
included/excluded. Do not assume cross-table/Region transactional consistency.

```bash
: "${BACKUP_NAME:?Approved native on-demand backup name}"
"${aws_target[@]}" dynamodb create-backup --table-name "$TABLE_NAME" \
  --backup-name "$BACKUP_NAME" > "$recovery_dir/backup-create.json"
BACKUP_ARN=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["BackupDetails"]["BackupArn"])' "$recovery_dir/backup-create.json")
"${aws_target[@]}" dynamodb describe-backup --backup-arn "$BACKUP_ARN" > "$recovery_dir/backup-status.json"
python3 - "$recovery_dir/backup-status.json" "$recovery_dir/source-table.json" <<'PY'
import json
import sys
backup = json.load(open(sys.argv[1]))["BackupDescription"]
source = json.load(open(sys.argv[2]))["Table"]["TableArn"]
if backup["SourceTableDetails"]["TableArn"] != source:
    raise SystemExit("Backup source mismatch")
if backup["BackupDetails"]["BackupStatus"] != "AVAILABLE":
    raise SystemExit("Backup not AVAILABLE; inspect the recorded ARN within the approved deadline")
PY
```

One DescribeBackup is shown, not an unbounded waiter. If still creating, perform
bounded status checks on that exact ARN; do not create more backups to poll.
Reconcile timeout/unknown outcomes before retrying creation. Preserve ARN/time,
source identity, encryption/retention ownership and evidence. No automatic expiry,
backup deletion or AWS Backup vault policy is created by this mode.

Sources: [CreateBackup](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_CreateBackup.html), [DescribeBackup](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_DescribeBackup.html).
