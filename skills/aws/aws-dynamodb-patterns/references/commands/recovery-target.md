# Shared backup/restore target gate

Load only with backup or restore. Selector output is a plan, not permission to run
these examples. Confirm approved source account/principal, Region, table, method,
RPO/RTO, costs, destination and allowed actions first. Use existing IaC/role/CLI;
never inspect credentials or install tooling to bypass access restrictions.

Examples are same-account, same-Region. Run from the skill directory and only the
chosen method, not every block/page. All outputs are private local metadata. This
gate verifies source identity before table discovery; it changes no AWS setting.
Use the source's plain table name for TABLE_NAME in these examples; its full ARN
is read from DescribeTable for backup identity checks.
This setup assumes an existing source; do not use it unchanged for deleted-source
recovery. The restore command describes the separately reviewed SYSTEM/backup-ARN
path using preserved source identity instead of live DescribeTable.

```bash
set -euo pipefail
: "${AWS_PROFILE:?Approved profile}" "${AWS_REGION:?Approved Region}" \
  "${TABLE_NAME:?Approved source table}" "${EXPECTED_ACCOUNT_ID:?Approved account}" \
  "${EXPECTED_PRINCIPAL_ARN:?Approved full caller ARN}"
umask 077
recovery_dir=$(mktemp -d)
aws_target=(aws --profile "$AWS_PROFILE" --region "$AWS_REGION" --no-cli-pager --no-cli-auto-prompt --output json)
"${aws_target[@]}" sts get-caller-identity > "$recovery_dir/identity.json"
python3 - "$recovery_dir/identity.json" "$EXPECTED_ACCOUNT_ID" "$EXPECTED_PRINCIPAL_ARN" <<'PY'
import json
import sys
identity = json.load(open(sys.argv[1]))
if identity["Account"] != sys.argv[2] or identity["Arn"] != sys.argv[3]:
    raise SystemExit("Identity mismatch; stop before source discovery")
PY
"${aws_target[@]}" dynamodb describe-table --table-name "$TABLE_NAME" > "$recovery_dir/source-table.json"
```

Review source ARN/status, encryption/indexes, restore dependencies and quotas.
Named-backup checks must match the observed source TableArn, not just its friendly
name. Cross-account/Region, AWS Backup vaults/immutability or global-table recovery
need their own reviewed design/permissions; do not infer support from this example.
KMS key/grant access must survive backup retention and the entire restore; old
backups may depend on older keys. Do not disable/delete a key as backup cleanup.

PITR, on-demand backup and replication are different protections. A replica can
copy bad writes/deletes; PITR is not automatic failover. An available backup is
not a tested restore. Record business RPO (acceptable data loss) and end-to-end RTO
(including detection, decision, restore, validation and routing), then rehearse
with separately authorized nonproduction targets.
