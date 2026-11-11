# DynamoDB encryption: choose who controls the key

Encryption at rest is always on. Distinguish three choices; “AWS does the
encryption” does not identify which one is configured. LSI/GSI data uses the table
key—there is no separate index-encryption option. This does not replace workload
authorization, TLS, data minimization or application/client-side encryption needs.

## Decision matrix

| Choice | Ownership/control | When it fits | Cost/lifecycle tradeoff |
|---|---|---|---|
| AWS-owned key (default) | DynamoDB owns it in its service account; no customer key policy/lifecycle | Standard service encryption without a customer KMS-control requirement | No additional encryption charge; no customer key inventory/control |
| AWS-managed KMS key `alias/aws/dynamodb` | In your account; AWS controls policy/lifecycle/rotation; you can inspect policy/use in CloudTrail | Need account-visible service KMS key and auditability, not custom policy/control | KMS usage charges/quotas apply; do not claim this is the free default |
| Customer-managed KMS key | Your existing symmetric encryption key; you own policy, grants, rotation and availability | A concrete customer key-policy, separation-of-duties or lifecycle requirement | KMS charges and operational duties; revocation, disablement or deletion can break access/recovery |

Default to the simplest choice satisfying the stated requirement—not a blanket
“CMKs are always more secure” or compliance certification. A CMK creates a new
availability dependency. Record data sensitivity, required control/audit evidence,
key owner, allowed principals/services, rotation, restore and incident ownership.
Refresh Region-specific KMS pricing before estimating costs; a KMS operation is
not performed for every DynamoDB item access because table keys are cached.

## CloudFormation (choose one Properties fragment)

The shipped [table.cfn.yaml](../assets/table.cfn.yaml) deliberately omits
SSESpecification and therefore uses the **AWS-owned** key. Omission is not a
claim of unencrypted storage. For an explicit AWS-owned selection:

```yaml
SSESpecification:
  SSEEnabled: false
```

For the AWS-managed DynamoDB KMS key, omit KMSMasterKeyId:

```yaml
SSESpecification:
  SSEEnabled: true
  SSEType: KMS
```

For a customer-managed key, first add this template-level parameter (not inside
the table's Properties). Supply the reviewed, existing key's ARN at deployment:

```yaml
Parameters:
  KmsKeyArn:
    Type: String
    Description: Existing approved symmetric KMS encryption key ARN in the table Region
```

Then use this table Properties fragment:

```yaml
SSESpecification:
  SSEEnabled: true
  SSEType: KMS
  KMSMasterKeyId: !Ref KmsKeyArn
```

These are mutually exclusive fragments, not standalone templates or permission
to update a stack. Use a reviewed change set; do not change the default asset or
create a key/policy as an incidental prerequisite. An alias is resolved when set;
retargeting that alias later does **not** switch the table's encryption key.

## AWS CLI v2 equivalents (billable mutations; choose one)

Use the command cookbook's target/approval rules. Each create is an **alternative
for a new table**, not three sequential creates with the same name. The base
asset is unchanged; explicit CLI parameters select the encryption variant.

```bash
set -euo pipefail
: "${AWS_PROFILE:?}" "${AWS_REGION:?}" "${TABLE_NAME:?}"
aws_target=(aws --profile "$AWS_PROFILE" --region "$AWS_REGION" --no-cli-pager --output json)
# AWS-owned: no SSE override; encryption remains on.
"${aws_target[@]}" dynamodb create-table --table-name "$TABLE_NAME" \
  --cli-input-json file://assets/create-table.json
```

```bash
# AWS-managed alias/aws/dynamodb, not customer-managed.
"${aws_target[@]}" dynamodb create-table --table-name "$TABLE_NAME" \
  --cli-input-json file://assets/create-table.json \
  --sse-specification Enabled=true,SSEType=KMS
```

```bash
: "${KMS_KEY_ARN:?Select the approved existing customer-managed key ARN}"
"${aws_target[@]}" dynamodb create-table --table-name "$TABLE_NAME" \
  --cli-input-json file://assets/create-table.json \
  --sse-specification "Enabled=true,SSEType=KMS,KMSMasterKeyId=$KMS_KEY_ARN"
```

Wait for table creation and configure/read back TTL/PITR as in Provisioning. For a
**CLI-owned existing table**, after separately approved key-transition review, use
one of these alternatives. Keep CloudFormation-owned tables in CloudFormation.

```bash
# Switch to the AWS-managed key.
"${aws_target[@]}" dynamodb update-table --table-name "$TABLE_NAME" \
  --sse-specification Enabled=true,SSEType=KMS
```

```bash
# Switch to the reviewed customer-managed key.
: "${KMS_KEY_ARN:?}"
"${aws_target[@]}" dynamodb update-table --table-name "$TABLE_NAME" \
  --sse-specification "Enabled=true,SSEType=KMS,KMSMasterKeyId=$KMS_KEY_ARN"
```

Switching to AWS-owned is also possible with `Enabled=false`; it changes key
ownership, **not** encryption to plaintext. Require the same approval/policy review
before weakening a required customer-controlled-key posture.

## Inspect actual key identity and transition completion (authorized metadata reads)

```bash
"${aws_target[@]}" dynamodb describe-table --table-name "$TABLE_NAME" \
  --query 'Table.{State:TableStatus,SSE:SSEDescription}'
"${aws_target[@]}" kms describe-key --key-id "$KMS_KEY_ARN" \
  --query 'KeyMetadata.{Arn:Arn,Manager:KeyManager,State:KeyState,Usage:KeyUsage,Spec:KeySpec}'
```

For AWS-managed, inspect `alias/aws/dynamodb`/the resolved table key ARN instead;
that alias may not exist until first use. For customer-managed, compare the table's
resolved KMSMasterKeyArn to the approved ARN and verify KeyManager=CUSTOMER,
KeyState=Enabled, KeyUsage=ENCRYPT_DECRYPT and a symmetric encryption KeySpec.
`SSEType=KMS` by itself does not distinguish AWS-managed from customer-managed.
Absent SSEDescription can be the AWS-owned default, not missing encryption.
For a KMS transition, require SSEDescription.Status=ENABLED with the expected new
ARN; do not infer completion from an update response/table waiter alone.

## Permissions, recovery and non-negotiable stops

- Verify Region/key compatibility and effective caller IAM **plus** key policy and
  grants authorizing DynamoDB on the caller's behalf. A table-only IAM permission
  is not proof of usable KMS access. Follow AWS's policy/grant conditions (including
  service/account scope), not a blanket wildcard `kms:*` grant or new access key.
- Preserve original key access through transition, and beyond it for any dependent
  old on-demand backups: they retain their original key and require it for restore.
  Inventory those dependencies and actually test restore before retiring a key.
- Automatic rotation of the **same** KMS key is not switching to a different key
  ARN; neither alias retargeting nor table switching migrates old backups.
- Disabling/deleting a customer key or removing required policy/grants can block
  table access. A deleted key can make dependent data/backups irrecoverable. Do
  not provide or run disable/schedule-deletion cleanup as part of this workflow.
- Verify Global Tables version/replica key settings, Streams, DAX, exports/backups
  and monitoring separately; “table uses my CMK” is not end-to-end encryption proof
  for every adjunct service. DAX does not support a customer-managed key for its
  cluster encryption. SDK Get/Put/Query shapes do not change for server-side KMS.

Sources: [encryption choices/cost](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/EncryptionAtRest.html),
[CLI key selection](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/encryption.tutorial.html),
[key usage, policies and backup dependencies](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/encryption.usagenotes.html),
[CloudFormation SSE specification](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-properties-dynamodb-table-ssespecification.html).
