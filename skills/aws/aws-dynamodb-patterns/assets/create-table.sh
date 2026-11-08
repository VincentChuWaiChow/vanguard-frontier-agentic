#!/usr/bin/env bash
# Example only: execute after explicit approval of the printed target and cost.
# Requires AWS CLI v2 and an existing authorized profile; no login/install here.
set -euo pipefail
: "${AWS_PROFILE:?Select the approved existing profile}"
: "${AWS_REGION:?Select the approved Region}"
: "${TABLE_NAME:?Select the approved new table name}"
: "${EXPECTED_ACCOUNT_ID:?Set the approved target account ID}"

asset_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
aws_cmd=(aws --profile "$AWS_PROFILE" --region "$AWS_REGION" --no-cli-pager)

# Identity failure stops execution. The caller reviews the role, not just account.
"${aws_cmd[@]}" sts get-caller-identity
account_id=$("${aws_cmd[@]}" sts get-caller-identity --query Account --output text)
if [[ "$account_id" != "$EXPECTED_ACCOUNT_ID" ]]; then
  printf '%s\n' 'Account mismatch: stopped before table creation.' >&2
  exit 1
fi
printf 'Target: account=%s profile=%s region=%s new-table=%s\n' \
  "$account_id" "$AWS_PROFILE" "$AWS_REGION" "$TABLE_NAME"
read -r -p "Type CREATE ${TABLE_NAME} to create this billable resource: " confirmation
[[ "$confirmation" == "CREATE ${TABLE_NAME}" ]] || exit 1

# Existing name yields ResourceInUseException; do not catch it and adopt the table.
"${aws_cmd[@]}" dynamodb create-table \
  --table-name "$TABLE_NAME" --cli-input-json "file://${asset_dir}/create-table.json"
"${aws_cmd[@]}" dynamodb wait table-exists --table-name "$TABLE_NAME"
"${aws_cmd[@]}" dynamodb update-time-to-live --table-name "$TABLE_NAME" \
  --time-to-live-specification 'Enabled=true,AttributeName=expiresAt'
"${aws_cmd[@]}" dynamodb update-continuous-backups --table-name "$TABLE_NAME" \
  --point-in-time-recovery-specification 'PointInTimeRecoveryEnabled=true'

# Readback is control-plane evidence, not application/runtime proof.
"${aws_cmd[@]}" dynamodb describe-table --table-name "$TABLE_NAME"
"${aws_cmd[@]}" dynamodb describe-time-to-live --table-name "$TABLE_NAME"
"${aws_cmd[@]}" dynamodb describe-continuous-backups --table-name "$TABLE_NAME"
