# AWS CLI v2 command cookbook

These are **examples, not an automation script**. Run only the authorized section,
not the entire page. Use the table's IaC owner for lasting changes; do not introduce
CLI drift into a CloudFormation-managed table. No command runs when loading this
skill. Shell snippets are Bash; DynamoDB expressions/typed JSON are language-neutral.

## Command groups by domain

Use only the requested group; each guide has target prerequisites, permission
boundaries, read versus mutation labels, examples and primary sources.

| Domain | Guidance and commands |
|---|---|
| Core table/data operations | This page: inspection, Get/Query, pagination, conditional CRUD, index creation and TTL |
| Selective workflow / audit / performance / feature plans | [Offline mode selector and workflow contract](README.md) |
| Data types / native vectors | [Typed item, vector table and SearchVectors examples](../data-types-and-vectors.md) |
| Embedding / AWS integrations | [Load only for Titan, native/S3 vector and KB CLI commands](embedding.md) |
| Content freshness | [June/December local reminder and review procedure](../maintenance.md) |
| IAM actions and table/GSI/LSI/stream resource scopes | [IAM mapping, validation and simulation](../iam-and-access.md#domain-iam-action-to-resource-mapping) |
| Tags and ABAC | [Table vs stream tags and authorization boundaries](../iam-and-access.md#domain-tagging-and-abac) |
| Resource-based / cross-account access | [Policy read/change preconditions](../iam-and-access.md#domain-resource-based-and-cross-account-access) |
| Encryption/KMS | [Key selection and readback](../encryption.md) |
| Streams and consumers | [Stream setup, bounded reads and mappings](../streams-and-dax.md#domain-dynamodb-streams) |
| DAX caching | [Cache decision, inspection and reviewed cluster creation](../streams-and-dax.md#domain-dax-caching) |
| PITR and restore | [Recovery-window checks and new-table restore](../recovery-and-replicas.md#domain-point-in-time-recovery) |
| Multi-Region replicas | [Same-account current-version MREC example and MRSC boundaries](../recovery-and-replicas.md#domain-current-version-multi-region-replicas) |
| CloudWatch and notifications | [Table/index/stream/cache/replica metrics and alarms](../monitoring.md) |

## Target and synthetic sample data

Run from the skill directory. Use an existing authorized profile; do not create or
read credentials. Confirm the returned role/account, Region and resource before
any data read. Each mutation needs separate approval, cost and rollback review.
`acme/c42/o100` are synthetic fixtures; replace them only with authorized data.

```bash
set -euo pipefail
: "${AWS_PROFILE:?}" "${AWS_REGION:?}" "${TABLE_NAME:?}"
aws_target=(aws --profile "$AWS_PROFILE" --region "$AWS_REGION" --no-cli-pager --output json)
ddb() { "${aws_target[@]}" dynamodb "$@"; }
"${aws_target[@]}" sts get-caller-identity
key='{"PK":{"S":"TENANT#acme#CUSTOMER#c42"},"SK":{"S":"ORDER#2026-01-02T10:00:00.000Z#o100"}}'
order='{"PK":{"S":"TENANT#acme#CUSTOMER#c42"},"SK":{"S":"ORDER#2026-01-02T10:00:00.000Z#o100"},"entityType":{"S":"Order"},"orderId":{"S":"o100"},"status":{"S":"OPEN"},"version":{"N":"1"},"GSI1PK":{"S":"TENANT#acme#STATUS#OPEN"},"GSI1SK":{"S":"2026-01-02T10:00:00.000Z#o100"},"totalMinor":{"N":"1250"},"currency":{"S":"USD"}}'
```

Do not put raw records or cursors in shared logs, tickets or shell history. A
customer's real identifiers require authorization even for read-only commands.

## Inspect configuration (metadata reads)

```bash
ddb describe-table --table-name "$TABLE_NAME" \
  --query 'Table.{State:TableStatus,Keys:KeySchema,GSIs:GlobalSecondaryIndexes,LSIs:LocalSecondaryIndexes,SSE:SSEDescription}'
ddb describe-time-to-live --table-name "$TABLE_NAME"
ddb describe-continuous-backups --table-name "$TABLE_NAME"
```

`--query` here is a client-side JMESPath output selector, **not** a database key
condition, IAM restriction or read-cost optimization.

## Get a known order and list orders (authorized data reads)

GetItem requires **both** PK and SK; it cannot read an index or locate an order by
orderId alone. Strong consistency is optional on the base table, not on a GSI.

```bash
ddb get-item --table-name "$TABLE_NAME" --key "$key" \
  --consistent-read --return-consumed-capacity TOTAL
page=$(ddb query --table-name "$TABLE_NAME" \
  --cli-input-json file://assets/query-input.json --no-paginate)
printf '%s\n' "$page"
last_key=$(printf '%s' "$page" | python3 -c \
  'import json,sys; print(json.dumps(json.load(sys.stdin).get("LastEvaluatedKey") or {}))')
if [[ "$last_key" != '{}' ]]; then
  ddb query --table-name "$TABLE_NAME" \
    --cli-input-json file://assets/query-input.json \
    --exclusive-start-key "$last_key" --no-paginate
fi
```

This fetches at most **two service pages**, not the whole result set. Continue only
within an approved request/deadline budget. A nonempty LastEvaluatedKey can accompany
zero matches and does not guarantee more matches. `--no-paginate` stops CLI automatic
page draining; `--limit` caps evaluated items in one service request. A service page
can stop at 1 MB. CLI `--max-items`/`--starting-token`/NextToken are a different
pagination layer: never interchange them with LastEvaluatedKey/ExclusiveStartKey.

## Query the sparse GSI and optional LSI

```bash
ddb query --table-name "$TABLE_NAME" --index-name GSI1 \
  --key-condition-expression '#gpk = :open' \
  --expression-attribute-names '{"#gpk":"GSI1PK"}' \
  --expression-attribute-values '{":open":{"S":"TENANT#acme#STATUS#OPEN"}}' \
  --no-consistent-read --no-scan-index-forward --limit 50 --no-paginate \
  --return-consumed-capacity INDEXES
```

The default table has **no LSI**. Only query LSI1 on an independently approved table
created with the optional LSI asset below, containing order `LSI1SK` attributes.
This is not an LSI retrofit to the default/existing table.

```bash
: "${LSI_TABLE_NAME:?Select the approved table that already has LSI1}"
ddb query --table-name "$LSI_TABLE_NAME" --index-name LSI1 \
  --key-condition-expression '#pk = :customer AND begins_with(#ls, :updated)' \
  --expression-attribute-names '{"#pk":"PK","#ls":"LSI1SK"}' \
  --expression-attribute-values '{":customer":{"S":"TENANT#acme#CUSTOMER#c42"},":updated":{"S":"UPDATED#"}}' \
  --consistent-read --no-scan-index-forward --limit 50 --no-paginate \
  --return-consumed-capacity INDEXES
```

Both query examples return one service page; handle its cursor as above. Review
[LSI vs GSI](../lsi-vs-gsi.md) before choosing an index. Neither filter expressions nor
request projection expressions reduce units for data already read.

## Conditional create and close (data mutations; each separately approved)

These operate on the synthetic key above, not an entire partition. A failed
condition is a conflict—not a reason to remove the condition or retry endlessly.

```bash
ddb put-item --table-name "$TABLE_NAME" --item "$order" \
  --condition-expression 'attribute_not_exists(PK)' --return-consumed-capacity TOTAL
ddb update-item --table-name "$TABLE_NAME" --key "$key" \
  --update-expression 'SET #st = :closed, #v = #v + :one REMOVE GSI1PK, GSI1SK' \
  --condition-expression '#st = :open AND #v = :expected' \
  --expression-attribute-names '{"#st":"status","#v":"version"}' \
  --expression-attribute-values '{":open":{"S":"OPEN"},":closed":{"S":"CLOSED"},":one":{"N":"1"},":expected":{"N":"1"}}' \
  --return-consumed-capacity TOTAL
```

The close changes version 1 to 2 and removes both sparse GSI keys atomically. Index
visibility may lag; consumers conditionally recheck current state. For an LSI table,
every order write also maintains `LSI1SK=UPDATED#<fixed-width UTC time>#<orderId>`
in that **same update**; these default-table commands deliberately omit that field.
Handle timeout-after-commit by reconciliation/operation identity, not blind replay.

## Delete one fixture (destructive; separate exact-item approval)

This is not normal cleanup for production/financial records. Check retention/holds
and backups first. This predicate expects the fixture was closed at version 2.

```bash
ddb delete-item --table-name "$TABLE_NAME" --key "$key" \
  --condition-expression '#st = :closed AND #v = :expected' \
  --expression-attribute-names '{"#st":"status","#v":"version"}' \
  --expression-attribute-values '{":closed":{"S":"CLOSED"},":expected":{"N":"2"}}'
```

## New table, optional LSI, or new GSI (choose the authorized path)

For complete default table creation/TTL/PITR, use [Provisioning](../provisioning.md).
The optional asset is a **new-table alternative**, not a change to the default:

```bash
: "${LSI_TABLE_NAME:?Select the approved NEW table name}"
ddb create-table --table-name "$LSI_TABLE_NAME" \
  --cli-input-json file://assets/create-table-with-lsi.json
ddb wait table-exists --table-name "$LSI_TABLE_NAME"
```

It retains the default GSI and adds LSI1 (`PK`, `LSI1SK`). Configure/read back TTL
and PITR separately using the approved LSI table name, as in Provisioning. No
command can add/delete an LSI on an existing table; rebuilding/migrating needs its
own plan. CLI-managed tables have no CloudFormation Retain policy.

For a **CLI-owned on-demand table only**, this optional GSI2 has a new partition key
`GSI2PK`. Obtain index/backfill-cost approval, inspect existing names/types/quotas,
and populate compatible keys through an approved write/backfill plan. It does not
automatically make an order-ID lookup work for items missing GSI2PK.

```bash
ddb update-table --table-name "$TABLE_NAME" \
  --attribute-definitions AttributeName=GSI2PK,AttributeType=S \
  --global-secondary-index-updates '[{"Create":{"IndexName":"GSI2","KeySchema":[{"AttributeName":"GSI2PK","KeyType":"HASH"}],"Projection":{"ProjectionType":"KEYS_ONLY"}}}]'
ddb describe-table --table-name "$TABLE_NAME" \
  --query 'Table.GlobalSecondaryIndexes[?IndexName==`GSI2`].{State:IndexStatus,Backfilling:Backfilling}'
```

Do not route reads until GSI2 itself is ACTIVE and backfill has finished; a
table-exists waiter is not proof of GSI readiness. Monitor index throttling and
base-write back-pressure; retain the old read path for staged cutover/rollback.
Provisioned tables require separately sized index throughput, not this request.

## Backup/expiry configuration (metadata mutations)

```bash
ddb update-continuous-backups --table-name "$TABLE_NAME" \
  --point-in-time-recovery-specification PointInTimeRecoveryEnabled=true
ddb update-time-to-live --table-name "$TABLE_NAME" \
  --time-to-live-specification Enabled=true,AttributeName=expiresAt
```

Read back the statuses with the inspection commands; TTL can remain ENABLING.
TTL is delayed cleanup, never an exact expiry or authorization mechanism. Do not
set a default expiry on orders. Encryption/key selection and readback commands are
in [Encryption](../encryption.md); that is a separate reviewed change, not a side effect
of adding an index. No key/table deletion commands are provided here.

Primary command references: [GetItem](https://docs.aws.amazon.com/cli/latest/reference/dynamodb/get-item.html),
[Query](https://docs.aws.amazon.com/cli/latest/reference/dynamodb/query.html),
[PutItem](https://docs.aws.amazon.com/cli/latest/reference/dynamodb/put-item.html),
[UpdateItem](https://docs.aws.amazon.com/cli/latest/reference/dynamodb/update-item.html),
[UpdateTable](https://docs.aws.amazon.com/cli/latest/reference/dynamodb/update-table.html).
