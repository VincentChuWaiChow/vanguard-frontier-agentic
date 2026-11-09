# Migrator import to a new table

Review [import config](../../../assets/migrator-import.json) and only the exact
manifest/object versions. Rewrite the saved schema to the approved NEW table name
with one DDL statement per line; do not blindly execute source-name DDL or infer
that `table` renames every saved statement. Include only reviewed create/index DDL;
no DROP or IF NOT EXISTS. Keep `overwrite:false`, `abortOnError:true`, explicit TTL
handling. Intended command after approval: `runMigrator --config reviewed-import.json`.
An existing destination is a stop, even if inserts would be non-overwriting.
Wait for table/index work requests, independently read ACTIVE state/schema/limits,
then validate counts/per-key hashes/types/tenant queries/expiry and failed/skipped
rows. Restore IAM/tags/alarms/key/topology separately. Resume traffic only after a
separately approved application cutover and rollback rehearsal; waiter alone fails.
