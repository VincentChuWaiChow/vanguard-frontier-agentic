# audit --mode live

Read only for a requested live audit; do not load the offline sibling. Apply the
shared [audit](../audit.md) checks and selected feature guides. This workflow can
collect real configuration evidence **after scoped read authorization**; selecting
the mode alone collects nothing and supplies no permission.

## Target and authorized metadata collection

Confirm the exact account/principal, Region, table/index, selected features, time
window and permitted metadata. Use an already-authorized exposed AWS tool or an
existing CLI/profile; do not install tools, create keys or inspect credentials.
If access is unavailable, report the missing live evidence. Do not fabricate a live
result from a template or silently substitute an offline audit.

The following is a bounded metadata-read example, **not automatically executed**.
Run from the skill directory only after approval. Set the expected full STS
principal ARN from the reviewed target; a different session/role fails closed.
Outputs use a private temporary directory. No customer item is read or printed.

```bash
set -euo pipefail
: "${AWS_PROFILE:?Approved profile}" "${AWS_REGION:?Approved Region}" \
  "${TABLE_NAME:?Approved table}" "${EXPECTED_ACCOUNT_ID:?Approved account}" \
  "${EXPECTED_PRINCIPAL_ARN:?Approved full STS principal ARN}"
umask 077
audit_dir=$(mktemp -d)
aws_target=(aws --profile "$AWS_PROFILE" --region "$AWS_REGION" --no-cli-pager --no-cli-auto-prompt --output json)
"${aws_target[@]}" sts get-caller-identity > "$audit_dir/identity.json"
python3 - "$audit_dir/identity.json" "$EXPECTED_ACCOUNT_ID" "$EXPECTED_PRINCIPAL_ARN" <<'PY'
import json
import sys
identity = json.load(open(sys.argv[1]))
if identity["Account"] != sys.argv[2] or identity["Arn"] != sys.argv[3]:
    raise SystemExit("Identity mismatch; stop before table discovery")
PY
"${aws_target[@]}" dynamodb describe-table --table-name "$TABLE_NAME" > "$audit_dir/table.json"
```

Collect additional **selected-feature metadata only** through its existing guide:
TTL/PITR settings and recovery window; key identity/transition metadata; stream or
replica/index state; scoped resource policy/tags; approved metric windows. A guide
may contain mutations too: do not execute them in audit. Do not enumerate unrelated
accounts/tables/keys or assume every permission/configuration is visible to this role.

## Evidence and hard boundaries

- Record actual commands/tools, principal/Region/resource, observation timestamps,
  read failures and sanitized artifact locations. Compare readback with repo intent
  only if that local source is in scope; selected live config is authoritative for
  observed state, not proof of application behavior.
- Never GetItem/Query/Scan customer data, invoke embeddings/generation, run load,
  enable Contributor Insights, create alarms, change capacity/policies/features,
  remediate findings or perform cleanup as part of this metadata-only audit.
- Table metadata cannot prove effective KMS/IAM access, successful restore/failover,
  cross-tenant isolation, recall or end-to-end performance. Report these separately
  as UNVERIFIED unless a distinct authorized test supplies evidence.
- Return selected-scope findings with exact observed evidence and pending
  verification/approval. A successful DescribeTable or selector output is not an
  all-features healthy verdict or completed runtime audit.

Command contracts checked through Context7:
[GetCallerIdentity](https://docs.aws.amazon.com/cli/latest/reference/sts/get-caller-identity.html),
[DescribeTable](https://docs.aws.amazon.com/cli/latest/reference/dynamodb/describe-table.html).
