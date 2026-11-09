# Design

Capture service/API, workload, authenticated tenant, row/retention sizes, request
rates, latency/staleness objectives and source of truth. Produce an access-pattern
matrix: full primary key / shard-bounded range / secondary index / explicitly
priced scatter query. Pick shard keys for distribution and atomicity, not vanity
single-table parity. State multi-tenant enforcement, schema/JSON tradeoffs,
continuation and version contracts. Load only selected features; provisioning and
one language adapter load only when code/IaC is requested. Return alternatives,
small proposed artifacts and PASS/MISS/UNVERIFIED; do not create a table.
