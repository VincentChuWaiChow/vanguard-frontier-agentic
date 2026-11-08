# Create the worked-model table

Choose **one** owner: CloudFormation **or** direct CLI, not both for the same new
table. These are examples to inspect and adapt, not instructions to execute during
skill authoring. Any live AWS call needs authorized target/data scope; creation
additionally needs approval of the exact change and recurring charges.

Both paths define string PK/SK, string GSI1PK/GSI1SK, GSI1 with INCLUDE projection,
on-demand billing, deletion protection, TTL and PITR. The CLI creates the same keys,
index and billing first, then configures TTL/PITR through separate APIs.

For these ordinary table/GSI examples, only key attributes belong in
`AttributeDefinitions`. Do **not** add `status`,
`orderId`, `totalMinor` or every application field there. They are item attributes,
not key declarations. Sparse membership depends on which items supply GSI keys.
Native vector indexes are different: their HASH **and INLINE_FILTER** SearchSchema
attributes need scalar declarations; the embedding list itself does not. See
[data types and vectors](data-types-and-vectors.md) instead of extending this
ordinary-index rule to vector indexes.

For operational commands and the optional **new-table** LSI alternative, see
[Commands](commands/aws-cli.md) and [LSI vs GSI](lsi-vs-gsi.md). For deliberate AWS-managed
or customer-managed KMS selection, use [Encryption](encryption.md). The default
assets still use an AWS-owned key and no LSI; do not silently change either.

## CloudFormation

Use [table.cfn.yaml](../assets/table.cfn.yaml). Essential excerpt (not a standalone
template; the asset includes index attribute definitions and preservation policies):

```yaml
Type: AWS::DynamoDB::Table
Properties:
  BillingMode: PAY_PER_REQUEST
  DeletionProtectionEnabled: true
  KeySchema:
    - AttributeName: PK
      KeyType: HASH
    - AttributeName: SK
      KeyType: RANGE
  TimeToLiveSpecification:
    AttributeName: expiresAt
    Enabled: true
  PointInTimeRecoverySpecification:
    PointInTimeRecoveryEnabled: true
```

After target/cost approval, create an **unexecuted** change set, inspect it, then
obtain approval before executing that exact change-set ARN:

```bash
# Run from this skill's directory. All variables are explicit approved values.
: "${AWS_PROFILE:?}" "${AWS_REGION:?}" "${STACK_NAME:?}" "${TABLE_NAME:?}"
aws --profile "$AWS_PROFILE" --region "$AWS_REGION" sts get-caller-identity
cfn-lint assets/table.cfn.yaml                          # local check only
aws --profile "$AWS_PROFILE" --region "$AWS_REGION" cloudformation deploy \
  --template-file assets/table.cfn.yaml --stack-name "$STACK_NAME" \
  --parameter-overrides TableName="$TABLE_NAME" --no-execute-changeset
# Inspect the generated change set. Use its exact ARN for subsequent approved
# describe-change-set / execute-change-set / wait stack-create-complete calls.
# An update uses the matching stack-update waiter instead. No implicit execution.
```

`Retain`/`UpdateReplacePolicy: Retain` deliberately keep billable data/resources.
The named table may prevent replacement; inspect changes rather than forcing it.
Deletion protection also blocks normal table deletion; disabling it is a separate
destructive decision, not a default cleanup step. DynamoDB encrypts at rest by
default; this example leaves the AWS-owned default key. Customer-managed KMS keys
need separate key policy/IAM, availability, lifecycle and cost review.

## AWS CLI v2

Inspect [create-table.json](../assets/create-table.json) and
[create-table.sh](../assets/create-table.sh). Minimal creation shape, **not executed**:

```bash
: "${AWS_PROFILE:?}" "${AWS_REGION:?}" "${TABLE_NAME:?}"
aws --profile "$AWS_PROFILE" --region "$AWS_REGION" dynamodb create-table \
  --table-name "$TABLE_NAME" --cli-input-json file://assets/create-table.json
aws --profile "$AWS_PROFILE" --region "$AWS_REGION" dynamodb wait table-exists \
  --table-name "$TABLE_NAME"
aws --profile "$AWS_PROFILE" --region "$AWS_REGION" dynamodb update-time-to-live \
  --table-name "$TABLE_NAME" \
  --time-to-live-specification 'Enabled=true,AttributeName=expiresAt'
aws --profile "$AWS_PROFILE" --region "$AWS_REGION" dynamodb update-continuous-backups \
  --table-name "$TABLE_NAME" \
  --point-in-time-recovery-specification 'PointInTimeRecoveryEnabled=true'
```

The complete shell example adds an account guard and interactive target check.
Its confirmation prompt is not a substitute for approval of role/Region/resource.
It fails on an existing table and on any failed step. If TTL/PITR fails **after**
creation, leave the table intact, read back actual state, and resume only the
approved missing configuration; do not rerun creation, auto-delete or adopt a
different table. A create timeout is also an unknown outcome: reconcile first.

TTL may report ENABLING before ENABLED; explicitly verify final status when TTL
is required. Check table and GSI ACTIVE, key/projection/billing, deletion protection,
PITR ENABLED and TTL attribute/status. A successful create response or table waiter
does not prove all later settings or application access. Do not auto-delete test
resources; disclose retained cost and propose separately approved cleanup.

The example deliberately excludes LSIs, Streams, Global Tables, alarms, workload
IAM, tags and on-demand maximum/warm-throughput tuning. Add only when justified;
it is not a complete production landing zone. On-demand has workload/partition
limits and is not a cost ceiling. Refresh service quotas and prices for the Region.

Sources: [CloudFormation table](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-dynamodb-table.html),
[CLI CreateTable](https://docs.aws.amazon.com/cli/latest/reference/dynamodb/create-table.html),
[PITR](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/PointInTimeRecovery.html).
