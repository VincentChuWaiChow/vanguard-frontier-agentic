# restore --method on-demand

Read only this method after the target and distinct-new-table guards.

## Exact backup/source validation

Use target gate + new-destination guard first. Match the backup's source `TableId`
as well as its ARN [DP-E701] [DP-E702]: a table deleted and recreated under the same
name keeps the ARN but gets a new `TableId`. Do not select the newest backup by
name alone, infer readiness from CreateBackup, or mix this with the PITR block.

```bash
: "${BACKUP_ARN:?Approved exact native backup ARN}"
"${aws_target[@]}" dynamodb describe-backup --backup-arn "$BACKUP_ARN" > "$recovery_dir/backup-status.json"
python3 - "$recovery_dir/backup-status.json" "$recovery_dir/source-table.json" <<'PY'
import json
import sys
backup = json.load(open(sys.argv[1]))["BackupDescription"]
table = json.load(open(sys.argv[2]))["Table"]
origin = backup["SourceTableDetails"]
# A recreated table reuses the name-based ARN; only TableId is per incarnation.
if not table.get("TableId") or origin.get("TableId") != table["TableId"]:
    raise SystemExit("Backup TableId missing or from another incarnation of this table")
if origin["TableArn"] != table["TableArn"] or backup["BackupDetails"]["BackupStatus"] != "AVAILABLE":
    raise SystemExit("Backup source/status is not approved and AVAILABLE")
PY
# Separately approved billable restore into a NEW target.
"${aws_target[@]}" dynamodb restore-table-from-backup \
  --backup-arn "$BACKUP_ARN" --target-table-name "$RESTORE_TABLE_NAME"
```

Source: [RestoreTableFromBackup](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_RestoreTableFromBackup.html).
