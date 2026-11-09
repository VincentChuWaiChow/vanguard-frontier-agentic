# Selected-feature observability

Use Azure Monitor namespace `Microsoft.DocumentDB/databaseAccounts` with exact
account resource ID, database/container/region dimensions. Verify current metric
definitions/available dimensions and aggregation before generating alarms. Do not
collect tenant/item content, raw vectors, session/continuation tokens or secrets.

| Selected domain | Evidence / signal |
|---|---|
| core/indexes/capacity | `TotalRequests` split StatusCode/OperationType/DatabaseName/CollectionName/Region; `TotalRequestUnits`; `NormalizedRUConsumption` Maximum by partition range/physical partition; diagnostic query/index utilization |
| latency | `ServerSideLatencyDirect` or `ServerSideLatencyGateway`, correct mode/operation/region; SDK end-to-end p95/p99/retries separately. Old `ServerSideLatency` is deprecated, not a recommended new alarm. |
| throttling/errors | 429 requests/retry-after/cumulative attempts; 403/404/409/412 classification, SDK substatus/activity ID; bounded retry budget |
| iam/encryption | Entra/data-role/config audit, key availability/expiry/rotation/default identity and denied-access tests; no broad key listing |
| backup/replication | Actual policy/window/latest restorable point, restore errors/index rebuild, region health/failover and measured RPO/RTO; backup policy alone isn't health |
| change-feed | Consumer lag, lease/checkpoint age/restart, poison/retry backlog, deletion reconciliation and duplicate-effect tests |
| vectors/embeddings | Embedding/model-version/dimension failures, vector index build/population, query charges/latency, labeled recall@k/nDCG, freshness and tenant-denial tests |

Alarm thresholds follow measured SLO/workload, not copied numbers. Define window,
statistic/dimensions, missing-data handling, routing/action group, owner, runbook
and notification test. Diagnostics/log analytics cost/retention and PII minimization
are separate choices. Missing metrics or untested notifications are UNVERIFIED.

Source: [current metrics reference](https://learn.microsoft.com/azure/cosmos-db/monitor-reference).
