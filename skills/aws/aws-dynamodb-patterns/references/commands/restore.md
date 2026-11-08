# restore

Plan and validate recovery into a **new table**, never overwrite/delete the source.
PITR is the default; on-demand uses an exact native backup ARN. Load this file,
the [target gate](recovery-target.md) and selected feature detail, not backup's
creation procedure. Selector output performs no restore or cutover. Read only the selected method:
[pitr](restore/pitr.md) or [on-demand](restore/on-demand.md), never both.

```text
/aws-dynamodb-patterns restore --method pitr
/aws-dynamodb-patterns restore --method on-demand --features pitr,encryption
```

Real local selector: `python3 scripts/select_mode.py restore --method pitr`.

## Source, recovery point and new destination

After the shared target gate, approve the exact new target and recovery point,
data scope, KMS/IAM/dependent rights, costs and acceptance/rollback plan. The CLI
examples are same-account/Region. Source remains authoritative until an explicitly
approved application cutover; it continues receiving writes unless separately
quiesced. Account for legitimate writes after the selected recovery point.

These runnable examples inspect an **existing source**. A missing/deleted source
needs a source-independent variant, not recreation of the original or a fabricated
DescribeTable response. Verify deletion separately from permission/network errors.
For a table deleted with PITR enabled, AWS documents a SYSTEM backup retained for
35 days that recovers its pre-deletion state. Identify the exact available backup
using approved inventory and archived source ARN/table ID; generate a reviewed
RestoreTableFromBackup variant without requiring live source-table metadata.
That is not a promise of arbitrary historical PITR points after deletion. Keep the
same caller/KMS/new-target/acceptance gates and verify expiry before relying on it.

```bash
: "${RESTORE_TABLE_NAME:?Approved distinct NEW table}"
if [[ "$RESTORE_TABLE_NAME" == "$TABLE_NAME" ]]; then
  printf '%s\n' 'Restore target must differ from source.' >&2
  exit 64
fi
```

Restore APIs create a new table [DP-E410]; existing targets must fail, not be adopted. No
delete/recreate/rename shortcut. Stop on a failed restore request; do not run a
waiter against an unrelated existing table. After a timeout, reconcile exact target
identity/status before any retry; errors do not establish nonexistence.
A waiter timeout can leave a valid restore still running: inspect that same target,
not a new restore request, before deciding the next bounded wait or escalation.

## Validate recovery before cutover (either method)

Run only after a successful chosen restore request, not a failed/unknown attempt:

```bash
"${aws_target[@]}" dynamodb wait table-exists --table-name "$RESTORE_TABLE_NAME"
"${aws_target[@]}" dynamodb describe-table --table-name "$RESTORE_TABLE_NAME" \
  > "$recovery_dir/restored-table.json"
```

| Gate | Required evidence |
|---|---|
| Infrastructure | Actual target identity, schema/index readiness, billing/throughput, encryption and deletion protection; no fixed restore-duration promise |
| Re-establish settings | Explicitly inspect/reapply required PITR, TTL, tags, Streams/consumers, IAM/resource policy, auto scaling, alarms and app routing; these are not guaranteed inherited |
| Correctness/security | Authorized representative data invariants, deleted/expired records, positive and tenant-negative access, dependencies and business reconciliation; counts alone are insufficient |
| Business continuity | Measured recovery point/loss vs RPO and full drill elapsed time vs RTO, including validation and routing, not just the AWS waiter |
| Cutover/rollback | Separate routing/write authorization, controlled traffic and observability, current source/old path, reconciliation or reverse synchronization after new writes |

Restored global-table data does not automatically restore its replica topology.
Vector/index/stream dependencies need feature-specific verification. New writes
can make the old source stale; retaining it alone is not rollback. No automatic
source, backup, table, index or KMS-key deletion/cleanup. Report PLAN ONLY if no
restore ran, and UNVERIFIED for untested recovery/IAM/application acceptance.

Sources: [PITR restore](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/PointInTimeRecovery.Tutorial.html),
[RestoreTableFromBackup](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_RestoreTableFromBackup.html),
[restore tutorial](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Restore.Tutorial.html).
[Deleted-source SYSTEM backup](https://docs.aws.amazon.com/cli/latest/reference/dynamodb/describe-backup.html).
