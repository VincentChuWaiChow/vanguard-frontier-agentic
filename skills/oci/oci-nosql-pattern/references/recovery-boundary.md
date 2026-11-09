# Recovery boundary and evidence

Oracle's responsibility model assigns table backup/restore to the customer using
Migrator/Object Storage. Reviewed NDCS APIs do not establish a customer-native PITR
or named snapshot workflow; do not invent one from Oracle Database/on-prem docs.
Provider-managed durability or internal CMEK backups do not establish user-selected
recovery points. This package supports planned Migrator or app-owned SDK exports.

For either method record exact source identity/schema, export start/end, quiescence,
TTL policy, row counts/failures/skips, object versions/checksums, tool version, keys,
retention and an approved NEW destination. The [example manifest](../assets/recovery-manifest.example.json)
is intentionally incomplete and rejected until populated with real evidence.
`check_recovery.py` checks sanitized caller-supplied consistency only; it never calls
OCI or verifies actual target absence/object bytes. New name alone is not absence.

No cross-row snapshot is promised while writes run. Keep DML/DDL quiesced; otherwise
label a fuzzy interval and cannot promise exact-point recovery. Timestamp-filtered
exports miss deletes and may race updates; incremental/delta recovery requires an
independent ordered change/tombstone ledger, reconciliation and tested replay, not
a modification-time query alone. Multiple-table exports need coordinated quiescence.

RPO is the age/coverage of the last verified consistent export at incident time,
not a fixed advertised number. RTO includes access/key recovery, data transfer,
new-table/index readiness, metadata reconstruction, application validation and
approved cutover. Measure both in drills. The usable recovery window is bounded
by actually retained readable object versions, TTL/deletion policy, keys, schema
compatibility and capacity; expiration can make an older export unusable. Recreate
IAM/tags/alarms/replica topology as separately reviewed settings. Never accept a
work-request waiter alone as restore success.

[Responsibility model](https://docs.oracle.com/en/cloud/paas/nosql-cloud/dtddt/features-of-oracle-nosql-database-cloud-service.html),
[Migrator limitations](https://docs.oracle.com/en/cloud/paas/nosql-cloud/dtddt/using-oracle-nosql-database-migrator.html), checked 2026-11-08.
