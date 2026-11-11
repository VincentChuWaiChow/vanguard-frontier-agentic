# Refactor

1. Capture current sanitized contract, API/SDK version, partition/id, indexes,
   throughput, ETag/session behavior, tenant enforcement and existing IaC owner.
2. Classify code-only, compatible index change or data-model migration. A new
   partition key or incompatible vector policy can require a new container; never
   drop/recreate or invoke createIfNotExists to hide drift.
3. Preserve public adapter behavior. Parameterize queries; handle pagination,
   cancellation/RU/retry bounds and 409/412 explicitly. No speculative ORM rewrite.
4. For migration: define source watermark, versioned backfill, idempotency,
   deletes/TTL reconciliation, dual-write divergence, validation, approved cutover
   and rollback window. Change feed latest-version alone misses deletions.
5. Deliver minimal diff, negative tests, compatibility/readback checks and risks.
   No backfill, deployment, writes or load without separate explicit authorization.
