# Selected-feature metrics

Namespace `oci_nosql`; scope to approved `resourceId`(table OCID), `tableName`,
and replica dimension where applicable. Do not copy CloudWatch dimensions.

| Selected focus | Metrics / evidence | Interpretation |
|---|---|---|
| Core/index capacity | ReadUnits,WriteUnits; SDK consumed capacity, client p95/p99 | query/index/consistency/row size and retry amplification |
| Throttle/hot shard | ReadThrottleCount,WriteThrottleCount,StorageThrottleCount,MaxShardSizeUsagePercent | correlate with target limits and skew; total table average can hide a hot shard |
| Storage/TTL | StorageGB plus application expiry validation | asynchronous/coarse storage metric is not proof every row expired |
| replicas | replica lag metric/dimension as published for deployed region | lag/readiness is not backup RPO |
| recovery | application export manifest age/failures/restore drill duration | custom evidence; do not invent native Backup/PITR metrics |
| embeddings | app model generation failures, source freshness, deletion lag, recall | not NDCS-native vector metrics |

Example MQL intent: `ReadThrottleCount[1m]{resourceId="APPROVED_TABLE_OCID"}.sum()>0`.
Check actual dimension casing in discovered metric definitions before applying an
alarm. Use explicit windows/statistics/missing-data behavior and separately verified
Notifications delivery. Storage metrics may lag; metric silence is not health.
No alarm/notification is created by this skill.

[Published metrics](https://docs.oracle.com/en/cloud/paas/nosql-cloud/mgygg/), checked 2026-11-08.
