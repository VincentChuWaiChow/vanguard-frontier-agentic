# backup --method pitr

Read only this selected method after the shared recovery target gate.


After the shared target gate and authorized metadata scope:

```bash
"${aws_target[@]}" dynamodb describe-continuous-backups --table-name "$TABLE_NAME" \
  > "$recovery_dir/pitr-before.json"
```

Require PointInTimeRecoveryStatus=ENABLED, the configured RecoveryPeriodInDays
(1–35), EarliestRestorableDateTime and LatestRestorableDateTime. ContinuousBackupsStatus
alone is insufficient. Latest is typically about five minutes behind now, not a
zero-loss guarantee. Enabling/increasing a window does not create earlier history;
reducing it removes older recoverability. Do not change retention during inspection.

**Optional separately approved settings mutation**, through the current owner:

```bash
: "${RECOVERY_DAYS:?Reviewed recovery period, 1-35 days}"
spec=$(python3 - "$RECOVERY_DAYS" <<'PY'
import json
import sys
days = int(sys.argv[1])
if not 1 <= days <= 35:
    raise SystemExit("PITR recovery period must be 1-35 days")
print(json.dumps({"PointInTimeRecoveryEnabled": True, "RecoveryPeriodInDays": days}))
PY
)
"${aws_target[@]}" dynamodb update-continuous-backups --table-name "$TABLE_NAME" \
  --point-in-time-recovery-specification "$spec"
"${aws_target[@]}" dynamodb describe-continuous-backups --table-name "$TABLE_NAME" \
  > "$recovery_dir/pitr-after.json"
```

Read back actual status/window; request success is not proof the desired history
exists. Never disable/re-enable PITR or reset settings to repair missing history.

Source: [PITR](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/PointInTimeRecovery_Howitworks.html).
