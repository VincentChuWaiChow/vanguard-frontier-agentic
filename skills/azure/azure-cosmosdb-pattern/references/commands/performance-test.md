# Bounded performance plan — not a load runner

Selector defaults: local, 30 seconds, 500 requests, concurrency 2, 1000 RU budget;
all caps must be positive. Production is deliberately unsupported. Selection
never launches traffic, creates data or changes throughput.

Specify exact endpoint/tenant/account/container, nonproduction authorization,
synthetic dataset/cardinality/hot-key distribution, operation mix/item/vector
sizes, index/SDK/connection mode, consistency and warm/cold states. Local emulator
results aren't Azure latency/availability/cost proof.

Before a separately approved run enforce duration, total attempts INCLUDING
retries, concurrency, requests/sec, per-operation deadline, overall RU and spend
budget, cancellation, cleanup ownership and 429/error/latency stop thresholds.
Unknown RU must stop, not be counted as zero; one request can cross a RU budget
before its charge is known. Reserve that overshoot and Azure billing uncertainty.

Measure p50/p95/p99/end-to-end latency, charges by operation, normalized RU by
partition, throttling/retries, diagnostics/activity IDs and success/error counts.
Vector tests also fix recall ground truth, tenant filters, index population/build
state, nDCG/recall@k and freshness. No self-match-only quality claim.

Deliver an executable plan only if requested, always with separately reviewed
enforcement and exact run approval. No SDK install or cloud traffic on load.
