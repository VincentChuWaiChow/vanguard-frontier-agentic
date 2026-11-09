# Recovery overview — choose one native method lazily

Platform-managed backups are distinct from replication, change feed and custom
exports. No named-backup object/create/download interface is exposed. Don't
invent AWS Backup/PITR APIs. Backup is present as periodic or continuous mode;
continuous can't be switched back to periodic. Current continuous documentation
lists **7, 30 and 35 days**; tool/API support for the newest tier varies.

| Need | Selected next path (load only that path) |
|---|---|
| Review continuous policy/window | commands/backup/continuous.md |
| Review scheduled snapshots | commands/backup/periodic.md |
| Recover corrupted items/account at time | commands/restore/continuous-new-account.md |
| Recover deleted DB/container into same account | commands/restore/continuous-deleted-resource.md |
| Recover periodic snapshot | commands/restore/periodic-support.md |

Select methods from observed policy and incident, not a renamed AWS command.
An existing continuous account can recover **deleted** resources in-account;
that doesn't authorize overwriting an existing container or reverting individual
items in place. Soft-delete preview is another lifecycle feature, not assumed
enabled or a replacement for backup; research separately if explicitly requested.

Define business RPO/RTO, observed backup readiness/timestamp, protected data/settings,
keys/identities/IAM, new target or deleted-resource boundary and actual drill
evidence. Continuous async capture in steady state is documented within about
100 seconds, not a universal zero-RPO guarantee. RTO depends on data/service/index/
network and application recovery; measure it, don't promise a fixed duration.

Sources: [backup modes](https://learn.microsoft.com/azure/cosmos-db/online-backup-and-restore),
[continuous](https://learn.microsoft.com/azure/cosmos-db/continuous-backup-restore-introduction),
[periodic](https://learn.microsoft.com/azure/cosmos-db/periodic-backup-restore-introduction).
