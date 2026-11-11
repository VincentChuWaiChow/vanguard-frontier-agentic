# Regional resilience isn't logical recovery

Choose single-write/multi-read versus multi-write deliberately; configure region
priority, availability zones where supported, client preferred regions/discovery,
consistency, conflict resolution and failover testing through existing IaC owner.
Strong consistency and multi-region writes aren't interchangeable/simultaneously
supported choices. Measure client end-to-end latency and region failure behavior,
not only service-side metrics.

Replication propagates bad writes/deletes and isn't PITR. Continuous backups are
taken in account regions; recovery point requires the source resource/region to
have existed at that time. Restored new accounts don't automatically recreate
all original regions/network/roles. Multi-write hub conflict confirmation affects
restorable mutations/time; use actual region-specific latest restorable timestamp.

Define business RPO/RTO, planned/unplanned failover ownership, client/session-token
behavior, dependency/network/DNS resilience and rollback. No region addition,
failover, conflict-policy change or recovery drill runs from this guide.

Sources: [global distribution](https://learn.microsoft.com/azure/cosmos-db/distribute-data-globally),
[multi-region writes](https://learn.microsoft.com/azure/cosmos-db/multi-region-writes),
[continuous recovery](https://learn.microsoft.com/azure/cosmos-db/continuous-backup-restore-introduction).
