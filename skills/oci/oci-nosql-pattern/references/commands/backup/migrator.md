# Migrator export

Use the Oracle NoSQL Database Migrator cloud→Object Storage JSON path, not on-prem
`kv export`/snapshot utilities. Review [export config](../../../assets/migrator-export.json)
only for this method; replace all placeholders. Instance-principal auth requires
an already approved OCI compute identity and least-privilege policies; do not set
up auth here. Pin/review the existing Migrator release. Intended command after
approval is `runMigrator --config reviewed-export.json` (not executed by this skill).
Quiesce DML/DDL, document the window; Migrator is not snapshot isolation. Use a
unique empty prefix, preserve schema/TTL, stop on any failed/skipped row, record
immutable object identities/hashes and verify a new-table restore drill. Do not
reuse an old prefix or treat process exit/log success as recoverability proof.
