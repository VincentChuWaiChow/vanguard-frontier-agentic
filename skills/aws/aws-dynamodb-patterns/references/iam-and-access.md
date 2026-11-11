# IAM, tags and resource-based access

## Target and authority

Examples are **not executed**. Each read still needs authorized account/Region/data
scope; IAM/tag/policy changes require separate reviewed approval. Run selected Bash
blocks from the skill directory, not this whole page. Do not change access to make
a diagnostic succeed. These synthetic policy assets must be rendered offline to
the intended partition, Region, owner table/index/stream and caller principal.

```bash
set -euo pipefail
: "${AWS_PROFILE:?}" "${AWS_REGION:?}" "${TABLE_NAME:?}" "${TABLE_ARN:?}"
aws_target=(aws --profile "$AWS_PROFILE" --region "$AWS_REGION" --no-cli-pager --output json)
"${aws_target[@]}" sts get-caller-identity
```

The table ARN is `arn:<partition>:dynamodb:<region>:<owner>:table/<table>`.
Both GSI and LSI use `<table-arn>/index/<name>`; Streams use
`<table-arn>/stream/<label>`. A table ARN alone is not a wildcard for those children.

## Domain: IAM action-to-resource mapping

Remove the `dynamodb:` prefix below only for readability, not in actual policies.
Distinguish service API/CLI names from IAM actions; they are **not** one-to-one.

| Application/API operation | Actual IAM permission | Resource scope |
|---|---|---|
| GetItem / PutItem / UpdateItem / DeleteItem | Matching action | Table, not an index |
| BatchGetItem / BatchWriteItem | Matching batch action | Every participating table |
| Query / Scan table or named GSI/LSI | Query / Scan | Table plus relevant index ARNs; same ARN pattern for both index types |
| Native vector similarity search | SearchVectors | Named vector index ARN, following its dedicated IAM example; not ordinary Query or FGAC tenant-key conditions |
| TransactGetItems | GetItem | Every target table |
| TransactWriteItems | PutItem, UpdateItem, DeleteItem and/or ConditionCheckItem according to its components | Every target table; same-account/same-Region transaction boundary |
| PartiQL reads / writes | PartiQLSelect / PartiQLInsert / PartiQLUpdate / PartiQLDelete | Select: table/relevant index; writes: table |
| DescribeTable / CreateTable / UpdateTable / DeleteTable | Matching action | Table; keep administration out of runtime roles |
| DescribeTimeToLive / UpdateTimeToLive | Matching action | Table |
| DescribeContinuousBackups / UpdateContinuousBackups | Matching action | Table; PITR is not implied by CRUD access |
| DescribeStream / GetShardIterator / GetRecords | Matching action, still **dynamodb:** | Stream ARN, not just table or index ARN |
| ListTables / ListStreams / DescribeLimits / DescribeEndpoints | Matching action | `Resource: "*"` where SAR lists no resource-level support; separate statement if genuinely needed |
| TagResource / UntagResource / ListTagsOfResource | Matching action | Table **or stream**, not directly an index |
| GetResourcePolicy / PutResourcePolicy / DeleteResourcePolicy | Matching action | Table or stream; owner administration, not cross-account policy-management rights |
| GetAbacStatus / UpdateAbacStatus permissions | Matching IAM action | Account/Region setting; `Resource: "*"`; not ordinary runtime access |

`dynamodb:TransactWriteItems`, `dynamodb:TransactGetItems` and
`dynamodb:ExecuteStatement` are not substitute IAM actions. ConditionCheckItem is
a permission-only action, not a standalone CLI API. Use supported condition keys
such as `dynamodb:EnclosingOperation` when restricting transactional use; verify
each action's SAR entry and dependent actions rather than grouping by name.

[table-index-read-policy.json](../assets/table-index-read-policy.json) allows only
GetItem/BatchGetItem on one table and Query on that table plus GSI1. It does not
grant Scan, writes, metadata discovery, tags, policy administration or Streams.
If a mapper needs DescribeTable, add it explicitly. Add only the selected LSI's
index ARN when that path exists. `/index/*` also permits future indexes.

This is **resource** isolation, not tenant/row authorization. A query filter or
projection does not establish tenant access. For supported actions, test
`dynamodb:LeadingKeys`/attribute conditions against real table/index key shapes;
Scan does not support LeadingKeys. Do not attach a generic tenant condition to
unsupported actions or assume every GSI's key equals the base tenant key.

### Commands: validate the proposed identity policy (read-only service analysis)

```bash
: "${IDENTITY_POLICY_FILE:?Path to the reviewed, target-rendered identity JSON}"
"${aws_target[@]}" accessanalyzer validate-policy \
  --policy-type IDENTITY_POLICY --policy-document "file://$IDENTITY_POLICY_FILE"
"${aws_target[@]}" iam simulate-custom-policy \
  --policy-input-list "$(cat "$IDENTITY_POLICY_FILE")" \
  --action-names dynamodb:GetItem dynamodb:Query \
  --resource-arns "$TABLE_ARN" "$TABLE_ARN/index/GSI1"
```

Review findings and **per-resource** simulator results, including expected denies
such as GetItem on an index. A simulator/valid JSON is not proof of the effective
caller role, SCPs, boundaries, resource policy, KMS access or a live application
request. Add positive/negative tests for every requested path before claiming
least privilege. No IAM policy-attachment command is run here.

## Domain: tagging and ABAC

Table tags are inherited by indexes; indexes cannot be tagged directly. Current
Streams supports **independent tags and ABAC**, does not inherit table tags and
cannot be tagged at stream creation. Tag an existing current stream ARN. Separate
DAX tags and replica configuration must be inspected in their own scopes.

```bash
"${aws_target[@]}" dynamodb list-tags-of-resource --resource-arn "$TABLE_ARN"
# Approved table tag mutation. This is not implicit permission to change access tags.
"${aws_target[@]}" dynamodb tag-resource --resource-arn "$TABLE_ARN" \
  --tags Key=Environment,Value=staging
# Optional removal is also a separately approved mutation.
"${aws_target[@]}" dynamodb untag-resource --resource-arn "$TABLE_ARN" \
  --tag-keys Environment
: "${STREAM_ARN:?Current approved stream ARN, not the table ARN}"
"${aws_target[@]}" dynamodb tag-resource --resource-arn "$STREAM_ARN" \
  --tags Key=Environment,Value=staging
"${aws_target[@]}" dynamodb list-tags-of-resource --resource-arn "$STREAM_ARN"
```

Tag-on-table-create uses CreateTable's `--tags`; review the additional TagResource
permission rather than assuming CreateTable alone covers tagging. Never place
secrets/PII in tags. Eventual tag readback is not immediate authorization proof.

ABAC enablement is independent for tables and Streams and scoped to account/Region.
Audit existing tag-conditioned policies, privilege to mutate authorization tags
and the documented opt-out window before enabling it. Tag changes can become
privilege escalation when tag writers control their own access conditions.
`aws:ResourceTag`, `aws:RequestTag`, `aws:TagKeys` support varies by action/resource;
verify SAR rather than reusing conditions blindly. Disabled ABAC can change how
conditions evaluate; tags alone do not enable it.

Use the currently supported console/service interface to verify each ABAC status.
The inspected CLI 2.37.4 rejects `dynamodb get-abac-status` as an unknown operation;
IAM action names in docs are not evidence that a public CLI operation exists.
Do not fabricate that command or silently upgrade/install tooling to run it.

## Domain: resource-based and cross-account access

Use [table-resource-policy.json](../assets/table-resource-policy.json) as an
illustrative named-external-role read/query policy only. Attach to the **table**;
express index rights in its statements. A stream can have its own resource policy;
an index cannot have an independently attached policy.

Cross-account data access needs the owner resource policy **and** caller identity
policy, without applicable explicit deny. Address the owner's full table ARN,
not a bare name resolved in the caller account. Verify the operation support
matrix: resource-policy support does not imply cross-account support for every
API (e.g. policy administration, TTL/PITR settings and PartiQL differ from data
APIs). AWS-managed-key table encryption does not support this cross-account path;
customer-managed keys require reviewed cross-account KMS authorization.

```bash
# Owner-context read. Preserve the current document/revision before any change.
"${aws_target[@]}" dynamodb get-resource-policy --resource-arn "$TABLE_ARN"
: "${RESOURCE_POLICY_FILE:?Reviewed target-rendered resource policy JSON}"
: "${EXPECTED_REVISION_ID:?Actual observed revision, or confirmed NO_POLICY}"
# Owner-context approved mutation. Never force-remove your own management access.
"${aws_target[@]}" dynamodb put-resource-policy --resource-arn "$TABLE_ARN" \
  --policy "file://$RESOURCE_POLICY_FILE" --expected-revision-id "$EXPECTED_REVISION_ID"
"${aws_target[@]}" dynamodb get-resource-policy --resource-arn "$TABLE_ARN"
```

Use `NO_POLICY` only after confirming no policy exists, not as fallback for access
denied, timeouts or revision conflicts. Revision mismatch/absence may both produce
PolicyNotFoundException. Reconcile first; do not retry a policy write blindly.
Keep prior policy and bounded propagation/readback expectations; successful policy
writes have an update interval restriction. Do not set
`--confirm-remove-self-resource-access` as an error workaround. A new explicit
deny can break global-table replication or administration; review service-linked
role paths too. CloudFormation-managed policy content does not guarantee drift
repair after an out-of-band edit. No deletion/lockout-bypass command is included.

## Separate permission domains

- [Streams/DAX](streams-and-dax.md): application DAX cluster actions, its service
  role's DynamoDB rights and consumer stream rights are different paths.
- [Encryption](encryption.md): KMS key policy/grants are not implied by table IAM.
- [Recovery/replicas](recovery-and-replicas.md): restore/create/replication rights
  and service-linked-role permissions do not belong in a CRUD-only role.
- [CloudWatch](monitoring.md): metric discovery/analysis, alarm writes, SNS delivery
  and Lambda consumer telemetry are different namespaces/resources.

Primary evidence: [service authorization reference](https://docs.aws.amazon.com/service-authorization/latest/reference/list_dynamodb.html),
[table/index example](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/iam-policy-specific-table-indexes.html),
[fine-grained conditions](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/specifying-conditions.html),
[resource-policy APIs/support matrix](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/rbac-iam-actions.html),
[cross-account requirements](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/rbac-cross-account-access.html),
[PutResourcePolicy](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_PutResourcePolicy.html),
[table ABAC](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/attribute-based-access-control.html),
[stream ABAC](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/abac-streams.html),
[stream tags](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Tagging.Streams.html).
