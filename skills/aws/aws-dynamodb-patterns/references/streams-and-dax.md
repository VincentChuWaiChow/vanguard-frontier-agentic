# Change streams and DAX caching

These are separate domains: a stream is a retained change log, DAX is an optional
cache service. Neither adds correctness guarantees to an inadequate table model.
Run only selected, approved examples, not this whole page. No command is executed
by loading the skill. Inspect current state before changing it.

```bash
set -euo pipefail
: "${AWS_PROFILE:?}" "${AWS_REGION:?}" "${TABLE_NAME:?}"
aws_target=(aws --profile "$AWS_PROFILE" --region "$AWS_REGION" --no-cli-pager --output json)
```

## Domain: DynamoDB Streams

This covers native DynamoDB Streams, not Kinesis Data Streams change-data-capture
destinations; that is a separate integration with different APIs and permissions.

Choose image shape deliberately: KEYS_ONLY, NEW_IMAGE, OLD_IMAGE, or
NEW_AND_OLD_IMAGES. More images increase sensitive payload exposure and processing
work. Streams retains records for 24 hours. Ordering is per item, not a global
order or an order across all items with the same partition key. A stream record
appearing once does not make a Lambda consumer exactly-once: Lambda delivery is
at least once; use idempotent processing, checkpoints, bounded retries/failure
destinations and recovery from poison records.

Changing image type requires disabling/re-enabling and creates a new stream ARN.
Do not perform that as an implicit update: assess replay gaps, mappings, stream
policies/tags, consumer permissions and checkpoint reset first. Streams on global
tables has mode-specific constraints; do not disable an MREC replication stream.

### Commands: inspection and separately approved enablement

```bash
"${aws_target[@]}" dynamodb describe-table --table-name "$TABLE_NAME" \
  --query 'Table.{Stream:StreamSpecification,Arn:LatestStreamArn}'
# Only if currently disabled and this image choice is approved.
"${aws_target[@]}" dynamodb update-table --table-name "$TABLE_NAME" \
  --stream-specification StreamEnabled=true,StreamViewType=NEW_AND_OLD_IMAGES
"${aws_target[@]}" dynamodb describe-table --table-name "$TABLE_NAME" \
  --query 'Table.{Stream:StreamSpecification,Arn:LatestStreamArn}'
```

Use the actual LatestStreamArn once stream creation is complete; do not reuse an
old incarnation. These commands manually inspect **one shard**, not a complete
consumer. Read only an authorized bounded payload; do not log raw customer images.

```bash
: "${STREAM_ARN:?Current authorized stream ARN}" "${SHARD_ID:?Observed shard ID}"
"${aws_target[@]}" dynamodbstreams describe-stream --stream-arn "$STREAM_ARN" \
  --limit 10 --no-paginate
"${aws_target[@]}" dynamodbstreams get-shard-iterator --stream-arn "$STREAM_ARN" \
  --shard-id "$SHARD_ID" --shard-iterator-type TRIM_HORIZON
: "${SHARD_ITERATOR:?Actual unexpired iterator from the previous response}"
"${aws_target[@]}" dynamodbstreams get-records \
  --shard-iterator "$SHARD_ITERATOR" --limit 25
```

TRIM_HORIZON begins at the oldest retained record; LATEST deliberately skips
existing retained records and can miss changes during consumer setup. Follow
NextShardIterator, checkpoint only completed processing, honor throttling, discover
new shards and process parent shards before children. An empty records page is
not necessarily completion. Do not share/log opaque iterator tokens.

### Permissions and managed consumers

CLI namespace is `dynamodbstreams`, but IAM actions remain `dynamodb:DescribeStream`,
`dynamodb:GetShardIterator`, `dynamodb:GetRecords` on the **stream ARN**. See the
[scoped stream policy](../assets/stream-read-policy.json). ListStreams is a separate
discovery action without resource-level scope, not implied by those rights.
Enablement uses dynamodb:UpdateTable on the table. Table CRUD does not grant stream
consumption and stream rights do not grant writes to an output table.

For an existing correctly configured Lambda function, mapping setup uses the
operator's lambda:CreateEventSourceMapping and applicable tagging rights; it does
**not itself pass an execution role**. Creating a function or assigning/changing
its execution role is a separate operation that can require iam:PassRole on that
exact role. Keep these operator rights separate from execution-role stream/log
permissions and any KMS/sink access. Verify function Region/network, batch size/window, partial
batch failure handling, retry/record age, failure destination and concurrency.
The managed AWSLambdaDynamoDBExecutionRole is a starting point, not proof of the
right sink permissions or least-privilege resource scoping. Do not create a new
mapping blindly: duplicate consumers can duplicate side effects.

```bash
: "${FUNCTION_NAME:?Approved existing Lambda function}"
"${aws_target[@]}" lambda list-event-source-mappings \
  --function-name "$FUNCTION_NAME" --event-source-arn "$STREAM_ARN"
```

Monitor IteratorAge, failures and throttles well before the retention limit;
see [Monitoring](monitoring.md). Stream tags/ABAC are independent of table tags;
see [IAM and access](iam-and-access.md).

## Domain: DAX caching

Add DAX only for measured read pressure/latency and an acceptable staleness budget.
It is a regional VPC service with its own client, node costs, IAM and networking—not
a table flag or universal SDK endpoint substitution.

- Item cache covers GetItem/BatchGetItem. Writes through DAX update its item cache;
  bypass writes, another Region's writes and other clusters can leave it stale.
- Query/Scan cache stores result sets by request parameters. **Even writes through
  DAX do not invalidate those cached result sets**; choose query TTL deliberately.
- Strong and transactional reads pass to DynamoDB and are not cached. A GSI cannot
  become strongly consistent through DAX. Item TTL=0 and query TTL=0 have different
  meanings; verify documented behavior rather than treating zero as a universal
  cache-disable setting. Neither TTL is DynamoDB item expiration.
- Check client/version/TLS support for the requested language. AWS documents Java,
  .NET, Node.js, Python and Go clients [DP-E453]; do not promise native Rust/PHP/Kotlin support
  merely because those languages have DynamoDB SDKs. A JVM integration can evaluate
  the Java DAX client explicitly. Node v3 client shape has its own restrictions.
- Benchmark cold/warm hit rates, p95/p99, staleness, outage fallback and connection
  recovery. Fallback directly to DynamoDB can cause a capacity/cost surge; do not
  add an unbounded retry/fallback loop. Count the whole multi-node cost, not savings
  inferred from successful creation or a high hit rate without workload context.

### Commands: inspect and optionally create a reviewed cluster

```bash
: "${DAX_CLUSTER_NAME:?Approved cluster name}"
"${aws_target[@]}" dax describe-clusters --cluster-names "$DAX_CLUSTER_NAME"
```

The next command creates **three billable nodes**. It is an optional creation
example only, not a default recommendation to deploy. Review node availability,
HA/AZ placement, approved private subnet group/security group, service-role trust,
KMS requirements and client reachability first. Keep TLS enabled and do not open
security groups to the internet to resolve connectivity problems.

```bash
: "${DAX_NODE_TYPE:?Approved supported size}" "${DAX_SUBNET_GROUP:?Existing reviewed subnet group}"
: "${DAX_SECURITY_GROUP_ID:?Existing reviewed private SG}" "${DAX_ROLE_ARN:?Reviewed DAX service role}"
"${aws_target[@]}" dax create-cluster --cluster-name "$DAX_CLUSTER_NAME" \
  --node-type "$DAX_NODE_TYPE" --replication-factor 3 \
  --subnet-group-name "$DAX_SUBNET_GROUP" --security-group-ids "$DAX_SECURITY_GROUP_ID" \
  --iam-role-arn "$DAX_ROLE_ARN" --sse-specification Enabled=true \
  --cluster-endpoint-encryption-type TLS
"${aws_target[@]}" dax describe-clusters --cluster-names "$DAX_CLUSTER_NAME"
```

Caller data-plane rights such as dax:GetItem/dax:Query target the cluster ARN.
The cluster service role trusts dax.amazonaws.com and separately needs DynamoDB
rights on the required tables/indexes; setup needs dax:CreateCluster and reviewed
iam:PassRole on that exact role. Inspect the DAX SAR for action-specific resource
support; do not grant `dax:*` plus `dynamodb:*`. A broad service role can become
a privilege-expansion path if application identities can use its cluster.

DAX cluster encryption is separate from the table's KMS selection; a table CMK
does not give the cluster CMK support. Confirm the cluster status/TLS endpoint and
perform authorized client tests; metadata alone does not prove cache use, hit rate
or restored consistency. Tagging/alarms on the cluster are also separate resources.

Sources: [Streams behavior](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Streams.html),
[Streams CLI examples](https://docs.aws.amazon.com/code-library/latest/ug/cli_2_dynamodb-streams_code_examples.html),
[Lambda consumers](https://docs.aws.amazon.com/lambda/latest/dg/with-ddb.html),
[DAX consistency](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/DAX.consistency.html),
[DAX clients](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/DAX.client.html),
[DAX access control](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/DAX.access-control.html),
[CreateCluster](https://docs.aws.amazon.com/cli/latest/reference/dax/create-cluster.html).
