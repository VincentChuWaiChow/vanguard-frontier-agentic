# Application-owned SDK import alternative

Create a separately approved new table with reviewed DDL/capacity/index/security;
conditional insert-only writes (`putIfAbsent`/equivalent), never upsert over existing
data. Verify manifest hashes/types first, preserve original absolute expiry rather
than restart TTL, and record already-expired rows as deliberately excluded. Bound
retry attempts, reject version conflicts, and reconcile expected keys/duplicates/
counts. A resumed importer needs an explicit checkpoint/idempotency design; a
collision is not permission to overwrite. Rebuild selected metadata/dependencies;
validate application queries and measured RPO/RTO before approved cutover. This is
a design contract, not a shipped turnkey backup/restore implementation.
