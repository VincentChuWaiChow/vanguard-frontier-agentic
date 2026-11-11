# audit --mode live

Commands are examples for an operator holding read-only authorization; the skill does
not run them. Check against [the operations reference](../../operations.md). All blocks
below share one shell session started from the setup block.

## Before any command

- Confirm the approved principal, AWS account, Region and exact keyspace and table.
  Stop on any mismatch; never switch target to make a command work.
- Use a read-only role. Write output to a private temp directory (`umask 077`). Never
  print keys, connection strings or customer data.
- Control-plane metadata only. No CQL shell, no row reads. Reading table metadata needs
  Select on the table and the system keyspace. [KS-E902]

```bash
set -euo pipefail
: "${AWS_PROFILE:?Approved profile}" "${AWS_REGION:?Approved Region}" \
  "${EXPECTED_ACCOUNT_ID:?Approved account}" "${KS_KEYSPACE:?Approved keyspace}" \
  "${KS_TABLE:?Approved table}"
umask 077
audit_dir=$(mktemp -d)
aws_target=(aws --profile "$AWS_PROFILE" --region "$AWS_REGION" --no-cli-pager --output json)
"${aws_target[@]}" sts get-caller-identity > "$audit_dir/identity.json"   # [KS-E901]
jq -e --arg a "$EXPECTED_ACCOUNT_ID" '.Account == $a' "$audit_dir/identity.json" >/dev/null \
  || { echo "Account mismatch; stop" >&2; exit 1; }
```

## Backup and restore

Point-in-time recovery status and earliest restorable time, from the table description.

```bash
: "${KS_KEYSPACE:?Approved keyspace}" "${KS_TABLE:?Approved table}"
"${aws_target[@]}" keyspaces get-table --keyspace-name "$KS_KEYSPACE" --table-name "$KS_TABLE" \
  > "$audit_dir/table.json"   # [KS-E902]
jq '{status, pointInTimeRecovery}' "$audit_dir/table.json"
```

## Disaster recovery and replication

Replication strategy and Regions of the keyspace, and replica settings of the table.

```bash
: "${KS_KEYSPACE:?Approved keyspace}"
"${aws_target[@]}" keyspaces get-keyspace --keyspace-name "$KS_KEYSPACE" \
  > "$audit_dir/keyspace.json"   # [KS-E903]
"${aws_target[@]}" keyspaces list-tables --keyspace-name "$KS_KEYSPACE" \
  > "$audit_dir/tables.json"   # [KS-E904]
jq '.replicaSpecifications' "$audit_dir/table.json"
```

## Cost

**No live command:** billing data lives in the AWS Billing and Cost Management console
and its billing APIs, not in the Keyspaces resource API. The capacity settings read in
the Backup and restore section (`capacitySpecification`) size the cost drivers.

## Monitoring

Alarms defined in the `AWS/Cassandra` namespace (filtered client-side).

```bash
"${aws_target[@]}" cloudwatch describe-alarms \
  --query "MetricAlarms[?Namespace=='AWS/Cassandra'].{name:AlarmName,metric:MetricName,state:StateValue}" \
  > "$audit_dir/alarms.json"   # [KS-E906]
```

## Performance

**No live command:** throttling and latency are CloudWatch metric datapoints, which this
audit does not pull. Capacity mode comes from `capacitySpecification` in the table
description above; compare it with the partition and connection limits offline.

## IAM

**No live command:** Keyspaces has no resource-based policy to read, and reviewing the
principal's identity policies needs IAM read calls outside this skill's cited command
set. Review policies in the repository (offline).

## Tagging

Tags on the table, by ARN (take the ARN from the table description).

```bash
KS_TABLE_ARN=$(jq -r '.resourceArn' "$audit_dir/table.json")
"${aws_target[@]}" keyspaces list-tags-for-resource --resource-arn "$KS_TABLE_ARN" \
  > "$audit_dir/tags.json"   # [KS-E905]
```

## Encryption

Encryption settings of the table (key type; key identifier for a customer managed key).

```bash
jq '.encryptionSpecification' "$audit_dir/table.json"   # [KS-E902]
```

## Network isolation

Interface VPC endpoints for Keyspaces in the approved Region (filtered client-side).

```bash
"${aws_target[@]}" ec2 describe-vpc-endpoints \
  --query "VpcEndpoints[?ends_with(ServiceName,'.cassandra')||ends_with(ServiceName,'.cassandra-fips')]" \
  > "$audit_dir/vpc-endpoints.json"   # [KS-E907] [KS-E910]
```

## Audit logging

Trails in the Region and the event selectors of an approved trail.

```bash
: "${TRAIL_NAME:?Approved CloudTrail trail name or ARN}"
"${aws_target[@]}" cloudtrail describe-trails > "$audit_dir/trails.json"   # [KS-E908]
"${aws_target[@]}" cloudtrail get-event-selectors --trail-name "$TRAIL_NAME" \
  > "$audit_dir/selectors.json"   # [KS-E909]
```
