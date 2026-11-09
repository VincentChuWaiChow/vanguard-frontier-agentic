# Refactor

Inspect supplied code/DDL/IaC and sanitized shape, not production rows. Classify
code-only, additive schema/index, incompatible key/type or regional change. Preserve
existing table owner and public contracts. Append compatible fields; changing shard
or primary key needs a new table and explicit migration/cutover plan. Do not
recreate, drop, stamp ownership or hide a scan fallback. Specify dual-reader/writer
compatibility, bounded backfill, expiry preservation, delete/tombstone handling,
reconciliation and rollback cutoff. A local diff/test is not runtime migration proof.
