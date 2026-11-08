# Cloud Firestore data modeling and operations

Load this for Firestore data-model, query, index, transaction, location,
backup, TTL or security-boundary questions. Every bracketed ID resolves in
[evidence-ledger.md](evidence-ledger.md). Text without an ID is labeled
inference or recommendation.

Scope: Standard edition, Native mode, unless stated. Firestore is a document
database: do not transfer DynamoDB, Cosmos DB or Bigtable key or capacity
concepts to it.

## Edition first

| Question | Standard | Enterprise | Evidence |
| --- | --- | --- | --- |
| MongoDB-compatible drivers? | no | yes ("Firestore with MongoDB compatibility" mode) | [FS-E15] |
| Single-field indexes | automatic | not automatic; fully customizable | [FS-E10] [FS-E15] |
| Server-library transaction default | PESSIMISTIC (locks read documents) | OPTIMISTIC | [FS-E07] |

In Enterprise, the mode (Native or MongoDB compatibility) is chosen at database
creation [FS-E15]. Confirm the edition before
giving any index or concurrency advice, because the defaults invert.
*(recommendation)*

## Limits that shape the model

- A document is at most **1 MiB**, subcollections nest at most **100** levels,
  and a document carries at most **40,000 index entries** [FS-E01]. Unbounded
  arrays or maps inside one document hit the size limit and the index-entry
  limit. Model growing lists as subcollections or separate documents.
  *(recommendation)*
- Composite indexes: 200 per database without billing, 1,000 with billing
  [FS-E02]. Every composite index is a budget decision.
- Requests are at most 10 MiB, and transactions time out at 270 seconds with a
  60-second idle expiry [FS-E01] [FS-E03]. Long-running read-modify-write logic
  belongs outside a transaction. *(recommendation)*

## Write throughput and hotspots

- Ramp new collections with **500/50/5**: start at no more than 500 ops/s and
  increase 50% every 5 minutes [FS-E04].
- Use automatic document IDs. They are scattered and should not hotspot
  [FS-E05]. Avoid monotonically increasing document IDs. *(recommendation)*
- There is **no fixed per-document write rate**. It "depends highly on the
  workload" [FS-E05]. Do not quote a hard number. Counters and other
  single-document hot spots need sharding or aggregation designs, validated by
  load test. *(recommendation)*

## Consistency and transactions

- Transactions are serializable, ordered by commit time [FS-E06].
- Concurrency: mobile/web SDKs are always optimistic. Server libraries follow
  the database setting, which defaults to pessimistic on Standard edition and
  optimistic on Enterprise [FS-E07]. Pessimistic transactions lock what they
  read, so long or wide server-side transactions on Standard edition create
  contention. *(inference)*

## Queries and indexes

- Single-field indexes are automatic on Standard edition. Composite indexes are
  never created automatically, and exemptions cut storage and write cost
  [FS-E10]. Exempt large or never-queried fields. *(recommendation)*
- Standard edition limits [FS-E09]:
  - at most 30 disjunctions (fixed)
  - `not-in` takes up to 10 values and cannot combine with `!=`
  - at most one `array-contains` per disjunction
- A query design that needs more fan-out than these limits allow needs a
  different data model, not a workaround. *(recommendation)*

## Security boundary

- Firebase security rules are evaluated for **mobile/web SDK** traffic. **Server
  client libraries bypass rules entirely** and are governed by IAM [FS-E08].
- A backend service account with broad IAM roles can read and write
  everything regardless of rules. Review IAM for every server path, and never
  treat rules as protection against a compromised backend. *(inference)*

## Location, backup and recovery

- Location is **permanent** after provisioning [FS-E11]. Choose multi-region
  for regional-outage survival, regional for lower cost and write latency
  [FS-E11].
- PITR is **off by default**. When enabled it keeps one version per minute for
  7 days. Without it, you can read (not export) data from only the past hour
  [FS-E12]. Treat "PITR not enabled" as a finding on production databases.
  *(recommendation)* PITR has no free tier and needs billing enabled, reads or
  exports in the PITR window are billed as reads, and up to one day of PITR
  storage may be charged even if PITR is disabled within a day [FS-E17].
- Scheduled backups allow one daily and one weekly schedule, keep backups up to
  14 weeks in the source location, and restore to a **new** database [FS-E13].
  The restore plan must cover repointing clients. *(inference)*

## TTL

TTL deletes typically within 24 hours, and expired documents still appear in
queries until deleted [FS-E14]. Filter expired documents in queries. Never use
TTL for exact-time deletion obligations. *(recommendation)*

## Not covered (unverified)

Pricing, server library package names and Datastore mode were not retrieved
[FS-E16]. Fetch the official page or label the answer an assumption.
