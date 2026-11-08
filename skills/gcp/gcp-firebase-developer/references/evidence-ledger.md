# Evidence ledger — Cloud Firestore

Every material claim in [firestore.md](firestore.md) cites one row below as
`[FS-Exx]`. `tests/validate-skill-references.py` fails the build if a citation
has no row, a row is never cited, or a source is not an allow-listed Google
documentation host.

Scope: **Cloud Firestore**, Standard edition and Native mode unless a row says
Enterprise edition or MongoDB compatibility. Not Firebase Realtime Database,
not Datastore mode. `VERIFIED` = the section was retrieved on the retrieval
date and supports the claim as worded; it is documentation verification, not a
behavioral test. Limits change: re-fetch before relying on a number.

| ID | Claim | Source | Section | Retrieved | Status |
| --- | --- | --- | --- | --- | --- |
| FS-E01 | Limits: a document is at most 1 MiB; subcollections nest at most 100 deep; a document has at most 40,000 index entries (single-field plus composite); an API request is at most 10 MiB. | <https://firebase.google.com/docs/firestore/quotas> | Collections, documents, and fields; Indexes; Writes and transactions | 2026-11-08 | VERIFIED |
| FS-E02 | A database has at most 200 composite indexes without billing enabled and 1,000 with billing enabled. | <https://firebase.google.com/docs/firestore/quotas> | Indexes | 2026-11-08 | VERIFIED |
| FS-E03 | A transaction has a 270-second time limit with a 60-second idle expiration. | <https://firebase.google.com/docs/firestore/quotas> | Writes and transactions | 2026-11-08 | VERIFIED |
| FS-E04 | Ramp traffic to a new collection with the "500/50/5" rule: start at most 500 operations per second, then increase by 50% every 5 minutes. | <https://firebase.google.com/docs/firestore/best-practices> | Ramping up traffic | 2026-11-08 | VERIFIED |
| FS-E05 | Automatic document IDs use a scatter algorithm and should not hotspot on writes; the maximum update rate of a single document depends highly on the workload (no fixed figure is given). | <https://firebase.google.com/docs/firestore/best-practices> | Hotspots | 2026-11-08 | VERIFIED |
| FS-E06 | Firestore guarantees serializable isolation of transactions, serialized by commit time. | <https://firebase.google.com/docs/firestore/transaction-data-contention> | Serializable isolation | 2026-11-08 | VERIFIED |
| FS-E07 | Mobile/web SDKs always use optimistic concurrency; server client libraries follow the database setting, which defaults to PESSIMISTIC on Standard edition (transactions lock the documents they read) and OPTIMISTIC on Enterprise edition. | <https://firebase.google.com/docs/firestore/transaction-data-contention> | Concurrency controls; Concurrency mode defaults | 2026-11-08 | VERIFIED |
| FS-E08 | Server client libraries exist for C#, Go, Java, Node.js, PHP, Python and Ruby; their requests are not evaluated against Firebase security rules and are secured with IAM, whereas mobile/web SDKs rely on security rules and Firebase Auth. | <https://firebase.google.com/docs/firestore/client/libraries> | Server client libraries; Mobile and web SDKs | 2026-11-08 | VERIFIED |
| FS-E09 | Standard edition query limits: at most 30 disjunctions in disjunctive normal form (fixed); `not-in` takes at most 10 values and cannot combine with `!=`; at most one `array-contains` per disjunction, not combinable with `array-contains-any` in the same disjunction. | <https://firebase.google.com/docs/firestore/query-data/queries> | Limitations | 2026-11-08 | VERIFIED |
| FS-E10 | Standard edition builds single-field indexes for each field automatically (collection-group-scope automatic indexes are not maintained by default); composite indexes are never created automatically; index exemptions reduce storage cost and can improve write performance. | <https://firebase.google.com/docs/firestore/query-data/index-overview> | Single-field indexes; Composite indexes; Index exemptions | 2026-11-08 | VERIFIED |
| FS-E11 | A database's location cannot be changed after provisioning; multi-region locations store replicas in several regions and keep serving through the loss of a region; regional locations cost less and give lower write latency. | <https://firebase.google.com/docs/firestore/locations> | Locations | 2026-11-08 | VERIFIED |
| FS-E12 | PITR is disabled by default; once enabled it retains one version per minute for 7 days; without PITR, documents can still be read (not exported) at any timestamp within the past hour. | <https://firebase.google.com/docs/firestore/pitr> | PITR overview | 2026-11-08 | VERIFIED |
| FS-E13 | Scheduled backups: at most one daily and one weekly schedule per database, retention up to 14 weeks, stored in the source database's location, restored to a new database. | <https://firebase.google.com/docs/firestore/backups> | Back up and restore data | 2026-11-08 | VERIFIED |
| FS-E14 | TTL deletion is not instantaneous: expired documents still appear in queries and lookups until deleted, typically within 24 hours of expiry. | <https://firebase.google.com/docs/firestore/ttl> | TTL policies | 2026-11-08 | VERIFIED |
| FS-E15 | Enterprise edition offers two modes chosen at database creation, Native mode (Core and Pipeline operations) and Firestore with MongoDB compatibility; in Enterprise, indexing is fully customizable and single-field indexes are not created automatically. Standard edition does not support MongoDB compatibility. | <https://firebase.google.com/docs/firestore/editions> | Editions overview | 2026-11-08 | VERIFIED |
| FS-E16 | Pricing, server library package names and Datastore-mode behavior were not retrieved in this pass. | <https://firebase.google.com/docs/firestore/pricing> | Understand Cloud Firestore billing | 2026-11-08 | UNVERIFIED |
