# Periodic restore — Azure Support, not self-service PITR

Do not issue `az cosmosdb restore` against periodic backups or invent a named
backup ARN/ID/download API. Promptly prepare a Support request; no case is opened
automatically by this skill. Confirm eligible support plan and escalation owner.

Supply subscription, original account/resource IDs/API, precise incident UTC
window, affected DB/container scope, source write region, requested snapshot/new
account name/resource group and preservation/retention/key dependencies. Do not
recreate a deleted source or precreate the restore target. Periodic recovery
creates a new account; exact snapshot availability/time and scope are confirmed
with Support, not promised by requested timestamp.

If source still exists, review approved retention increase promptly to avoid
overwriting recovery points (docs advise within eight hours). Never execute a
retention/redundancy change automatically or revoke old keys. Shared-throughput
database scope and current region restrictions need explicit Support confirmation.

Track case and asynchronous operation state; capture resulting source/snapshot/
destination identity. Reconcile partition/index/TTL/throughput/consistency, IAM,
CMK/default identity, private endpoints/firewall/DNS, replication, server-side
functions, consumers/diagnostics and application endpoints. Validate data/domain
invariants, tenant denials, dependencies and measured RPO/RTO before separately
approved cutover. Retain source/rollback; no delete/cleanup commands here.

Sources: [request restore](https://learn.microsoft.com/azure/cosmos-db/periodic-backup-request-data-restore),
[periodic behavior](https://learn.microsoft.com/azure/cosmos-db/periodic-backup-restore-introduction).
