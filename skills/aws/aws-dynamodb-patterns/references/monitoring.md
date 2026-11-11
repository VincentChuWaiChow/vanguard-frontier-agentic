# CloudWatch monitoring by component

Start with user-visible latency/error objectives, not a dashboard of averages.
Distinguish table/index throttling, SDK retry/network time, stream consumer lag,
replication delay and cache misses. Commands below are examples; metric reads need
authorized account/Region scope and alarm writes need separate reviewed approval.
No alarm delivery, endpoint reachability or production health is proved here.

```bash
set -euo pipefail
: "${AWS_PROFILE:?}" "${AWS_REGION:?}" "${TABLE_NAME:?}"
: "${METRIC_START_UTC:?Explicit ISO-8601 start}" "${METRIC_END_UTC:?Explicit ISO-8601 end}"
: "${GSI_NAME:?Approved observed GSI name, not an assumed default}"
aws_target=(aws --profile "$AWS_PROFILE" --region "$AWS_REGION" --no-cli-pager --output json)
# Metadata preflight: stop before index metrics/alarm writes if the GSI is absent or not ACTIVE.
index_state=$("${aws_target[@]}" dynamodb describe-table --table-name "$TABLE_NAME" \
  --query Table.GlobalSecondaryIndexes)
printf '%s' "$index_state" | python3 -c \
  'import json,sys; indexes=json.load(sys.stdin); ok=any(i.get("IndexName")==sys.argv[1] and i.get("IndexStatus")=="ACTIVE" for i in (indexes or [])); sys.exit(0 if ok else "Selected GSI is absent or not ACTIVE")' "$GSI_NAME"
```

## Domain: table and index health

| AWS/DynamoDB signal | Correct dimension set / statistic | What not to infer |
|---|---|---|
| ConsumedReadCapacityUnits / ConsumedWriteCapacityUnits | TableName; add GlobalSecondaryIndexName for a GSI; Sum divided by seconds for a rate | Period sum is not units/second; table-only does not aggregate every GSI |
| ReadThrottleEvents / WriteThrottleEvents | TableName; add GlobalSecondaryIndexName for GSI; Sum | Balanced table does not exclude a hot index or partial-batch throttling |
| ThrottledRequests / SystemErrors | TableName **and Operation**; Sum | SystemErrors with only TableName is not the matching series |
| SuccessfulRequestLatency | TableName + Operation; Average or p95/p99 | Service latency excludes client/network/retry time |
| UserErrors | Regional/account aggregate, no table dimensions; Sum | Not a per-table metric by inventing a TableName dimension |
| OnlineIndexPercentageProgress / OnlineIndexThrottleEvents | Documented table/index dimensions; inspect published series | A table ACTIVE waiter does not prove an online index ready |

Use documented metric dimensions exactly; extra/missing dimensions select a
different or nonexistent series rather than filtering a metric. `list-metrics`
helps discover published series but an absent series may be inactive/sparse, not
proof the metric or table doesn't exist. Inspect throttle reasons and per-key/index
signals; don't diagnose every event as insufficient total provisioned capacity.

### Commands: discover and query table/GSI metrics

```bash
"${aws_target[@]}" cloudwatch list-metrics --namespace AWS/DynamoDB \
  --metric-name ReadThrottleEvents --dimensions Name=TableName,Value="$TABLE_NAME" --no-paginate
"${aws_target[@]}" cloudwatch get-metric-statistics --namespace AWS/DynamoDB \
  --metric-name ReadThrottleEvents --dimensions Name=TableName,Value="$TABLE_NAME" \
  --start-time "$METRIC_START_UTC" --end-time "$METRIC_END_UTC" --period 60 --statistics Sum
"${aws_target[@]}" cloudwatch get-metric-statistics --namespace AWS/DynamoDB \
  --metric-name WriteThrottleEvents \
  --dimensions Name=TableName,Value="$TABLE_NAME" Name=GlobalSecondaryIndexName,Value="$GSI_NAME" \
  --start-time "$METRIC_START_UTC" --end-time "$METRIC_END_UTC" --period 60 --statistics Sum
"${aws_target[@]}" cloudwatch get-metric-statistics --namespace AWS/DynamoDB \
  --metric-name SuccessfulRequestLatency \
  --dimensions Name=TableName,Value="$TABLE_NAME" Name=Operation,Value=Query \
  --start-time "$METRIC_START_UTC" --end-time "$METRIC_END_UTC" --period 60 --extended-statistics p99
"${aws_target[@]}" cloudwatch get-metric-statistics --namespace AWS/DynamoDB \
  --metric-name SystemErrors --dimensions Name=TableName,Value="$TABLE_NAME" Name=Operation,Value=Query \
  --start-time "$METRIC_START_UTC" --end-time "$METRIC_END_UTC" --period 60 --statistics Sum
```

No datapoints is not a positive health verdict. Verify time bounds, metric
publication/retention granularity, namespace, Region, dimensions and sparse-event
behavior. Contributor Insights can reveal hot keys but is opt-in/cost-bearing and
can expose key values; handle activation and access as a separate privacy review.

## Domain: replica, stream consumer and cache monitoring

| Component | Namespace / metric / dimensions | Interpretation |
|---|---|---|
| MREC replication | AWS/DynamoDB / ReplicationLatency / TableName + ReceivingRegion, queried in source Region; Average or Maximum | Observed lag, not a failover/RPO guarantee or MRSC transaction latency metric |
| Lambda stream consumer | AWS/Lambda / IteratorAge / FunctionName; Maximum | Alert well before 24-hour record retention; also inspect Errors/Throttles Sum and failure destinations |
| DAX | AWS/DAX / ItemCacheHits, ItemCacheMisses, QueryCacheHits, QueryCacheMisses / ClusterId (or observed ClusterId+NodeId); Sum | Compute hits/(hits+misses), guard zero denominator; separate item/query caches |

DAX emits sparse/nonzero signals; an idle cache is not a proven healthy cache.
Include DAX CPU/memory pressure, errors and throttles, plus application-level
cache staleness and fallback load. Check actual published dimension sets before
composing dashboards; do not combine a node series with an aggregate cluster one.

```bash
: "${REPLICA_REGION:?Destination Region}" "${FUNCTION_NAME:?Consumer function}" "${DAX_CLUSTER_NAME:?Cluster ID}"
"${aws_target[@]}" cloudwatch get-metric-statistics --namespace AWS/DynamoDB \
  --metric-name ReplicationLatency \
  --dimensions Name=TableName,Value="$TABLE_NAME" Name=ReceivingRegion,Value="$REPLICA_REGION" \
  --start-time "$METRIC_START_UTC" --end-time "$METRIC_END_UTC" --period 60 --statistics Maximum
"${aws_target[@]}" cloudwatch get-metric-statistics --namespace AWS/Lambda \
  --metric-name IteratorAge --dimensions Name=FunctionName,Value="$FUNCTION_NAME" \
  --start-time "$METRIC_START_UTC" --end-time "$METRIC_END_UTC" --period 60 --statistics Maximum
"${aws_target[@]}" cloudwatch get-metric-statistics --namespace AWS/DAX \
  --metric-name ItemCacheHits --dimensions Name=ClusterId,Value="$DAX_CLUSTER_NAME" \
  --start-time "$METRIC_START_UTC" --end-time "$METRIC_END_UTC" --period 60 --statistics Sum
```

## Domain: alarms, notifications and permission boundaries

The following creates/updates one named GSI alarm and can overwrite an existing
alarm's settings. Inspect the current definition first, review its owner and exact
delta, and keep a rollback definition. Threshold/window here is illustrative, not
a workload-calibrated SLO. Wire an approved SNS topic and test subscriptions/topic
policy and actual delivery separately; an AlarmActions ARN alone proves nothing.

```bash
: "${ALARM_NAME:?Reviewed alarm name}" "${SNS_TOPIC_ARN:?Approved notification topic}"
"${aws_target[@]}" cloudwatch describe-alarms --alarm-names "$ALARM_NAME"
# Approved alarm mutation; two of two one-minute periods with at least one event.
"${aws_target[@]}" cloudwatch put-metric-alarm --alarm-name "$ALARM_NAME" \
  --namespace AWS/DynamoDB --metric-name WriteThrottleEvents \
  --dimensions Name=TableName,Value="$TABLE_NAME" Name=GlobalSecondaryIndexName,Value="$GSI_NAME" \
  --statistic Sum --period 60 --evaluation-periods 2 --datapoints-to-alarm 2 \
  --threshold 0 --comparison-operator GreaterThanThreshold \
  --treat-missing-data ignore --alarm-actions "$SNS_TOPIC_ARN" --actions-enabled
"${aws_target[@]}" cloudwatch describe-alarms --alarm-names "$ALARM_NAME"
```

Current primary sources **disagree**: the CloudWatch user guide describes a default
of ignore with an override, while PutMetricAlarm's API reference says AWS/DynamoDB
always ignores missing data even when another setting is supplied. Do not claim
an override is guaranteed or silently label one source obsolete. This example
explicitly uses `ignore`, consistent with both descriptions: missing data can leave
the alarm in a stale state, including ALARM. It is not a heartbeat detector or an
automatic all-clear policy. Design a verified recovery/healthy signal and test
OK/ALARM/INSUFFICIENT_DATA transitions and notification delivery in the target
environment before relying on a different missing-data setting.

Metric queries use cloudwatch:ListMetrics/GetMetricStatistics/GetMetricData, not
dynamodb:GetItem. Account/metric read actions often lack table-level ARN scoping;
check CloudWatch SAR rather than using a DynamoDB ARN in their Resource field.
Alarm configuration uses cloudwatch:PutMetricAlarm/DescribeAlarms with appropriate
alarm resources; SNS delivery/topic administration is a separate service/policy.
Lambda logs and stream processing likewise have distinct role and resource scopes.
Keep these operational rights out of ordinary item CRUD roles.

Sources: [DynamoDB metrics/dimensions](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/metrics-dimensions.html),
[global-table monitoring](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/V2globaltables_HowItWorks.html),
[DAX metrics](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/dax-metrics-dimensions-dax.html),
[Lambda metrics](https://docs.aws.amazon.com/lambda/latest/dg/monitoring-metrics-types.html),
[missing-data behavior](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/alarms-and-missing-data.html),
[PutMetricAlarm API conflict](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_PutMetricAlarm.html),
[CloudWatch IAM](https://docs.aws.amazon.com/service-authorization/latest/reference/list_amazoncloudwatch.html).
