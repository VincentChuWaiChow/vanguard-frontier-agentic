# restore --method pitr

Read only this method after target and distinct-new-table guards; complete the
shared restore acceptance checks afterward.


Use a timezone-qualified RESTORE_AT, ideally a verified clean point **before** the
incident, not blindly latest. Confirm source PITR is enabled and compare actual
bounds. The example expects CLI v2 ISO8601 metadata; unsupported formats fail closed.

```bash
: "${RESTORE_AT:?Approved timestamp with timezone, e.g. 2026-11-08T01:00:00Z}"
"${aws_target[@]}" dynamodb describe-continuous-backups --table-name "$TABLE_NAME" \
  > "$recovery_dir/pitr-window.json"
python3 - "$recovery_dir/pitr-window.json" "$RESTORE_AT" <<'PY'
from datetime import datetime
import json
import sys

def aware(value):
    result = datetime.fromisoformat(value.replace("Z", "+00:00"))
    if result.tzinfo is None or result.utcoffset() is None:
        raise SystemExit("Recovery timestamps need an explicit timezone")
    return result

pitr = json.load(open(sys.argv[1]))["ContinuousBackupsDescription"]["PointInTimeRecoveryDescription"]
if pitr["PointInTimeRecoveryStatus"] != "ENABLED":
    raise SystemExit("PITR is not enabled")
if not aware(pitr["EarliestRestorableDateTime"]) <= aware(sys.argv[2]) <= aware(pitr["LatestRestorableDateTime"]):
    raise SystemExit("Requested point is outside the observed recovery window")
PY
# Separately approved billable restore; never combine timestamp and latest flags.
"${aws_target[@]}" dynamodb restore-table-to-point-in-time \
  --source-table-name "$TABLE_NAME" --target-table-name "$RESTORE_TABLE_NAME" \
  --restore-date-time "$RESTORE_AT"
```

Only if the chosen clean point is explicitly latest, use --use-latest-restorable-time
**instead of** --restore-date-time. Enabling PITR now cannot recover an earlier
incident outside the existing window.

Source: [PITR tutorial](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/PointInTimeRecovery.Tutorial.html).
