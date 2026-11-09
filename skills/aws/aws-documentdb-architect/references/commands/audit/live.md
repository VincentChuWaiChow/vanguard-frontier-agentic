# audit --mode live

Commands are examples for an operator holding read-only authorization; the skill does
not run them. Check against [the operations reference](../../operations.md). All blocks
below share one shell session started from the setup block.

## Before any command

- Confirm the approved principal, AWS account, Region and exact cluster. Stop on any
  mismatch; never switch target to make a command work.
- Use a read-only role. Write output to a private temp directory (`umask 077`). Never
  print keys, connection strings or customer data.
- Control-plane metadata only. No database shell, no data reads.

```bash
set -euo pipefail
: "${AWS_PROFILE:?Approved profile}" "${AWS_REGION:?Approved Region}" \
  "${EXPECTED_ACCOUNT_ID:?Approved account}" "${DOCDB_CLUSTER_ID:?Approved cluster identifier}"
umask 077
audit_dir=$(mktemp -d)
aws_target=(aws --profile "$AWS_PROFILE" --region "$AWS_REGION" --no-cli-pager --output json)
"${aws_target[@]}" sts get-caller-identity > "$audit_dir/identity.json"   # [DOCDB-E901]
jq -e --arg a "$EXPECTED_ACCOUNT_ID" '.Account == $a' "$audit_dir/identity.json" >/dev/null \
  || { echo "Account mismatch; stop" >&2; exit 1; }
```

## Backup and restore

Retention, encryption flag and deletion protection of the cluster, and its snapshots.

```bash
: "${DOCDB_CLUSTER_ID:?Approved cluster identifier}"
"${aws_target[@]}" docdb describe-db-clusters --db-cluster-identifier "$DOCDB_CLUSTER_ID" \
  --query 'DBClusters[].{id:DBClusterIdentifier,retention:BackupRetentionPeriod,window:PreferredBackupWindow,deletionProtection:DeletionProtection}' \
  > "$audit_dir/cluster-backup.json"   # [DOCDB-E902]
"${aws_target[@]}" docdb describe-db-cluster-snapshots --db-cluster-identifier "$DOCDB_CLUSTER_ID" \
  --snapshot-type manual > "$audit_dir/manual-snapshots.json"   # [DOCDB-E903]
```

## Disaster recovery and replication

Instances, Availability Zone spread and global cluster membership.

```bash
: "${DOCDB_CLUSTER_ID:?Approved cluster identifier}"
"${aws_target[@]}" docdb describe-db-instances \
  --filters "Name=db-cluster-id,Values=$DOCDB_CLUSTER_ID" > "$audit_dir/instances.json"   # [DOCDB-E904]
"${aws_target[@]}" docdb describe-db-clusters --db-cluster-identifier "$DOCDB_CLUSTER_ID" \
  --query 'DBClusters[].{members:DBClusterMembers,multiAZ:MultiAZ,azs:AvailabilityZones}' \
  > "$audit_dir/cluster-topology.json"   # [DOCDB-E902]
: "${DOCDB_GLOBAL_CLUSTER_ID:?Approved global cluster identifier, only if one is in scope}"
"${aws_target[@]}" docdb describe-global-clusters \
  --global-cluster-identifier "$DOCDB_GLOBAL_CLUSTER_ID" > "$audit_dir/global.json"   # [DOCDB-E905]
```

## Cost

**No live command:** billing data lives in the AWS Billing and Cost Management console
and its billing APIs, not in the DocumentDB resource API. Use the resource commands in
the other sections only to size cost drivers (instance classes, retention, snapshots).

## Monitoring

Alarms defined in the `AWS/DocDB` namespace (filtered client-side).

```bash
"${aws_target[@]}" cloudwatch describe-alarms \
  --query "MetricAlarms[?Namespace=='AWS/DocDB'].{name:AlarmName,metric:MetricName,state:StateValue}" \
  > "$audit_dir/alarms.json"   # [DOCDB-E906]
```

## Performance

**No live command:** performance signals are CloudWatch metric datapoints, which this
audit does not pull; instance classes come from `describe-db-instances` above. Compare
them with the quotas page as an offline step.

## IAM

**No live command:** IAM policy review needs `iam` read calls outside this skill's
cited command set, and DocumentDB has no resource-based policy to read. Review the
IAM policies in the repository (offline) or the principal's attached policies in the
approved IAM review process.

## Tagging

Tags on the cluster, by ARN.

```bash
: "${DOCDB_CLUSTER_ARN:?Approved cluster ARN}"
"${aws_target[@]}" docdb list-tags-for-resource --resource-name "$DOCDB_CLUSTER_ARN" \
  > "$audit_dir/tags.json"   # [DOCDB-E907]
```

## Encryption

Encryption flag and KMS key identifier on the cluster, and the `tls` setting in its
cluster parameter group.

```bash
: "${DOCDB_CLUSTER_ID:?Approved cluster identifier}" "${DOCDB_PARAMETER_GROUP:?Approved cluster parameter group}"
"${aws_target[@]}" docdb describe-db-clusters --db-cluster-identifier "$DOCDB_CLUSTER_ID" \
  --query 'DBClusters[].{encrypted:StorageEncrypted,kms:KmsKeyId}' \
  > "$audit_dir/cluster-encryption.json"   # [DOCDB-E902]
"${aws_target[@]}" docdb describe-db-cluster-parameters \
  --db-cluster-parameter-group-name "$DOCDB_PARAMETER_GROUP" \
  --query "Parameters[?ParameterName=='tls'||ParameterName=='audit_logs']" \
  > "$audit_dir/parameters.json"   # [DOCDB-E908]
```

## Network isolation

Subnet group, security groups on the cluster and DocumentDB-related VPC endpoints.

```bash
: "${DOCDB_SUBNET_GROUP:?Approved subnet group name}"
"${aws_target[@]}" docdb describe-db-subnet-groups --db-subnet-group-name "$DOCDB_SUBNET_GROUP" \
  > "$audit_dir/subnet-group.json"   # [DOCDB-E909]
"${aws_target[@]}" docdb describe-db-clusters --db-cluster-identifier "$DOCDB_CLUSTER_ID" \
  --query 'DBClusters[].VpcSecurityGroups' > "$audit_dir/cluster-sg.json"   # [DOCDB-E902]
"${aws_target[@]}" ec2 describe-vpc-endpoints \
  --query "VpcEndpoints[?ends_with(ServiceName,'.rds')]" \
  > "$audit_dir/vpc-endpoints.json"   # [DOCDB-E910] [DOCDB-E912]
```

## Audit logging

Log exports on the cluster, the `audit_logs` parameter (read above) and CloudTrail trails.

```bash
: "${DOCDB_CLUSTER_ID:?Approved cluster identifier}"
"${aws_target[@]}" docdb describe-db-clusters --db-cluster-identifier "$DOCDB_CLUSTER_ID" \
  --query 'DBClusters[].EnabledCloudwatchLogsExports' > "$audit_dir/log-exports.json"   # [DOCDB-E902]
"${aws_target[@]}" cloudtrail describe-trails > "$audit_dir/trails.json"   # [DOCDB-E911]
```
