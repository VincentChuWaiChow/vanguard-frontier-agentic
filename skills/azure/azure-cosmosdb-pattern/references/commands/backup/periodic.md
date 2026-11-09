# Periodic protection

Periodic snapshots are automatic, with support-assisted restore; no user-named
on-demand snapshot equivalent. Default is a backup about every four hours with
the latest two copies retained (nominal eight hours). Actual completion can lag
the configured interval; treat RPO as observed, not exactly interval seconds.

Account policy bounds: interval **60–1440 minutes**, retention at least **twice
the interval** and at most **720 hours**. Redundancy choices Geo/Local/Zone depend
on region/support and residency requirements. Read actual policy before changes.
Changing redundancy affects future backups, can take twice the interval and
immediately removes access to older backups per current docs; require explicit
recovery-loss review. Don't silently choose Geo across residency boundaries.

Prepare support access, source/subscription/resource IDs, incident window/UTC,
required data scope, keys/default identity and new-account destination. For an
incident, promptly escalate; current docs advise contacting support/increasing
retention to at least seven days within eight hours to preserve usable snapshots.
Retention change is still a mutation needing approval; no automatic change here.

Deleted-container/database snapshot retention rules can differ from the live
account's nominal window (docs describe retention for 30 days). Verify exact
source/snapshot availability with Support; don't make every corruption recoverable
for 30 days or claim timestamp-precise PITR. Shared-throughput backup/restore has
database-scope implications. Record RPO/RTO/support-delay uncertainty.

Sources: [periodic](https://learn.microsoft.com/azure/cosmos-db/periodic-backup-restore-introduction),
[interval/retention/redundancy](https://learn.microsoft.com/azure/cosmos-db/periodic-backup-modify-interval-retention).
