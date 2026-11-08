# restore --method on-demand

Read only this method after the target and distinct-new-table guards.

## Exact backup/source validation

Use target gate + new-destination guard first. Do not select the newest backup by
name alone, infer readiness from CreateBackup, or mix this with the PITR block.

```bash
: "${BACKUP_ARN:?Approved exact native backup ARN}"
"${aws_target[@]}" dynamodb describe-backup --backup-arn "$BACKUP_ARN" > "$recovery_dir/backup-status.json"
python3 - "$recovery_dir/backup-status.json" "$recovery_dir/source-table.json" <<'PY'
import json
import sys
backup = json.load(open(sys.argv[1]))["BackupDescription"]
source = json.load(open(sys.argv[2]))["Table"]["TableArn"]
if backup["SourceTableDetails"]["TableArn"] != source or backup["BackupDetails"]["BackupStatus"] != "AVAILABLE":
    raise SystemExit("Backup source/status is not approved and AVAILABLE")
PY
# Separately approved billable restore into a NEW target.
"${aws_target[@]}" dynamodb restore-table-from-backup \
  --backup-arn "$BACKUP_ARN" --target-table-name "$RESTORE_TABLE_NAME"
```

Source: [RestoreTableFromBackup](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_RestoreTableFromBackup.html).
