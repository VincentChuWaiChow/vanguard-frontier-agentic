# Bounded performance plan — no execution

Require local or explicit nonproduction target, synthetic/non-sensitive dataset,
known partition/tenant skew, warm/cold phases, selected SQL/index and consistency.
Selector defaults: 30 seconds, 500 requests, 2 concurrency; all positive. Also require
rate ceiling, total attempts including retries, RU/WU/storage/cost cap, timeout,
p95/p99/error/throttle stop rules, owner and emergency stop. Never infer execution
approval from this route; no production choice exists. Test hot shard, wide row,
empty continuation page, version conflict and SDK retry interaction. Capture
consumed units, latency distribution and sample size; local/fake results cannot
prove OCI capacity/SLA. External ANN tests add tenant isolation, recall@k and stale
source/deletion recall metrics; ordinary index latency is not vector quality.
