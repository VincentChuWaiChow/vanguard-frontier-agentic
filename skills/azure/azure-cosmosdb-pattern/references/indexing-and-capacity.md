# Indexing / RU capacity

NoSQL containers default to consistent indexing of JSON properties. This isn't a
GSI/LSI system: no per-index provisioned throughput, projection table or separate
index query resource. Index policy affects write RUs, storage and query execution.

| Need | Native mechanism | Validation |
|---|---|---|
| Equality/range/order by one scalar | Range index on path | Query metrics, indexed paths and type/order |
| ORDER BY multiple properties | Matching composite index | Match path sequence and ASC/DESC directions; reverse all directions may work, not arbitrary mix |
| Geospatial | Spatial index/type | GeoJSON semantics and regional/query cost |
| Native vectors | Vector embedding policy AND vector index | Same concrete path; selected type/dimensions constraints |
| Exclude large unused properties | includedPaths/excludedPaths | More-specific path precedence; don't accidentally exclude queried fields |

Use parameterized SQL, selective projections and routed partition keys. Queries
can scan even when syntax is valid; don't call a cross-partition query a point read.
Index transformations are asynchronous: confirm progress and query behavior before
cutover; adding index policy isn't measured optimization by itself.

RU is normalized operation work, not requests per second. Item size/indexes/query
shape/consistency/regions all matter. Dedicated manual throughput example uses
400 RU/s; autoscale has a configured maximum and dynamic minimum; serverless bills
consumption and has different limits. Database shared throughput and per-container
throughput are distinct, not additive synonyms. Never set manual and autoscale
max simultaneously or apply provisioned-throughput commands to serverless.

Choose minimum/capacity from current service rules and observed workload, not a
blanket “400 fits production.” Autoscale doesn't fix a hot key. Inspect normalized
RU by partition, 429 rates, Retry-After, per-operation charges, request/diagnostic
latency and index utilization; capture consistency/session and connection mode.
Global replication creates regional charges; backup/vector/index/CMK choices may
add costs. Refresh price/region assumptions before any budget claim.

Sources: [index policy](https://learn.microsoft.com/azure/cosmos-db/index-policy),
[manage indexes](https://learn.microsoft.com/azure/cosmos-db/how-to-manage-indexing-policy),
[request units](https://learn.microsoft.com/azure/cosmos-db/request-units),
[throughput](https://learn.microsoft.com/azure/cosmos-db/set-throughput).
