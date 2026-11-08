# Bounded performance testing

```text
/aws-dynamodb-patterns performance-test --features vectors --environment nonproduction
```

```bash
python3 scripts/select_mode.py performance-test --features core,indexes \
  --environment local --duration-seconds 30 --max-requests 1000 --max-concurrency 4
```

Read this command only, not audit or feature-plan siblings. These positive limits
are requested plan caps, not an enforcing harness or authorization to run it.

Load only for performance-test mode. Start with a test plan; the selector prints
requested caps and guide paths, **not measurements or an executable load engine**.
Reuse the target application's SDK/language and established harness. If no suitable
harness exists, generate/review a minimal one as a separate implementation task;
do not install or launch a benchmark tool silently.

## Execution gate

Before a run, specify and review:

| Item | Required evidence |
|---|---|
| Target | Exact local endpoint or authorized isolated nonproduction account/Region/table/index; identity, IaC owner and no production routing |
| Dataset | Synthetic/minimized fixtures, item bytes, cardinality, largest tenant/hottest base and index keys; setup/cleanup key namespace |
| Operations | Read/write/query/transaction mix, payloads, consistency, index projections, filters, SDK retry/timeouts and connection reuse |
| Limits | Duration, total logical requests AND SDK attempts, arrival-rate cap, concurrency, per-request/deadline bounds; harness-enforced hard stop |
| Safety | Cost ceiling, throttle/error/latency stop thresholds, interruption/cancellation, authorized fixture writes and exact cleanup scope |
| Acceptance | Predeclared correctness, client latency/throughput/error, freshness and cost thresholds; vector relevance/recall where applicable |

The router's 30 seconds / 1,000 logical requests / four workers are **planning
defaults only**, not proven safe rates or complete limits. SDK retries can multiply
service calls. Missing target, rate/attempt/cost/stop bounds or matching authorization
blocks a live run. A nonproduction flag is not identity verification or consent.
Never use a destructive reset or whole-table cleanup to simplify the experiment.

Local mode can exercise injected clients/emulators and harness invariants. Report
the implemented capabilities/version: local timing does not reproduce AWS network,
partition/capacity/KMS/global/vector behavior. Passing unit tests is not a throughput
or cloud-latency benchmark. Production is not a generic mode target; it needs a
separate high-impact review outside this command.

## Measurement plan

1. Pin code/SDK/configuration, fixture seed, target settings and test time. Record
   warmup separately and use repeatable, bounded trials. Include representative
   hot-key/skew and burst cases, not only uniform random keys.
2. Measure **client end-to-end logical-operation** p50/p95/p99 with monotonic timing,
   completions, failures/timeouts, attempts/backoff and cancellation. Also capture
   offered versus completed throughput and scheduler/backlog delays; do not hide
   overload by counting only successes or excluding queued requests.
3. SuccessfulRequestLatency covers successful activity **inside DynamoDB**, not
   network/client/retry time. Keep it separate from client SLOs. Shell/aws-process
   timing is a smoke measurement, not representative of a pooled SDK client.
4. Collect relevant consumed capacity, throttles/throttle reasons, unprocessed batch
   items, errors and selected table/index metrics. Capacity Sum / period_seconds is
   average units/second; table-only dimensions exclude GSI consumption. Minute
   averages can hide spikes; a partial throttled batch need not increment the
   top-level ThrottledRequests metric. Do not enable Contributor Insights as an
   implicit test step: it changes settings, can incur charges and exposes key data.
5. For vectors, fix model/configuration/dimensions and labeled ground truth. Compute
   recall@k against the **authorized filtered candidate set**, result counts and
   update/delete-to-search visibility lag. Measure embedding, search, authorized
   source fetch and generation separately; a self-match is not recall evidence.
   Native vector request-byte metrics/cost must not be priced as ordinary Query
   RCU/WCU; request returned capacity and verify current vector billing semantics.
6. Compare against the predeclared thresholds and baseline under comparable load.
   A single trial cannot establish stability; report trial counts, sample sizes,
   variance, failures and limitations rather than fabricated confidence or pass@k.

Stop immediately at the reviewed limit/threshold, isolate failed writes and resolve
unknown outcomes before retrying. Preserve sanitized results. Clean only explicitly
approved fixture keys/resources; cleanup is not permission to delete a table,
index, key, backup, vector bucket or unrelated items.

## Report

Return the plan versus actual enforced limits, environment/evidence level, dataset
and operation mix, raw sample/histogram artifact locations, client and service
metrics separately, retries/throttles/errors, correctness/security/recall/freshness,
observed cost and unmeasured components. Mark thresholds PASS/MISS/UNVERIFIED.
If no run occurred, say **PLAN ONLY — no performance result**. If thresholds were
not declared, label the run exploratory, not passing. Do not claim real cost,
performance improvement or tenant isolation from a router, mock or checklist.

Primary sources checked 2026-11-08 through Context7/AWS documentation MCP:
[latency boundaries](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/TroubleshootingLatency.html),
[metric/statistic/dimension semantics](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/metrics-dimensions.html),
[hot-key constraints](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/throttling-key-range-limit-exceeded-mitigation.html),
[vector metrics](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/VectorSearchMonitoring.html).
Use the selected [monitoring guide](../monitoring.md) only when actual metric commands
are needed; do not eagerly load every operational reference.
