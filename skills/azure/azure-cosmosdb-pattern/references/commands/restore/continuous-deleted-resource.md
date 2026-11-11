# Continuous restore of a deleted DB/container into the SAME account

This method is only for an actual deleted database/container in an existing
continuous-backup NoSQL account. It isn't item rollback or an overwrite of a live
resource. Confirm Delete event/resource RID, source instance, time before deletion
inside observed recoverable window, region, exact original name and currently
absent target. Any existing target/name race is a hard stop; never delete to fit.

Azure CLI docs require 2.58.0+ for in-account restore; verify installed core/help
and support/preview state of selected command without auto-installing extensions.
Choose ONE approved operation, not both:

```bash
# Deleted database: restores child containers too; broader blast radius.
az cosmosdb sql database restore --subscription "$AZ_SUBSCRIPTION" \
  --resource-group "$RESOURCE_GROUP" --account-name "$ACCOUNT_NAME" \
  --name "$DELETED_DATABASE" --restore-timestamp "$RESTORE_TIMESTAMP" --disable-ttl true
```

```bash
# Deleted container only; database exists.
az cosmosdb sql container restore --subscription "$AZ_SUBSCRIPTION" \
  --resource-group "$RESOURCE_GROUP" --account-name "$ACCOUNT_NAME" \
  --database-name "$DATABASE_NAME" --name "$DELETED_CONTAINER" \
  --restore-timestamp "$RESTORE_TIMESTAMP" --disable-ttl true
```

Although service may default omitted timestamp to latest deleted instance, this
workflow requires explicit UTC timestamp/identity to avoid restoring the wrong
generation. Prove source/destination/deletion scope before approval. No more than three
restore operations can be active at once on the same account [CP-E354]; check
for in-flight restores before starting another. Preserve keys
and permissions while async restore runs. Inspect InAccount restore activity and
resource state, then indexes/TTL/RUs/security/app invariants and measured RPO/RTO;
no acceptance from a waiter alone and no automatic cutover/cleanup.

## Required token / consumer recovery exception

Restored resources are treated as new resources. Invalidate old session and query
continuation tokens; refresh/restart SDK clients and restart queries against the
restored resource. Do not replay old continuation tokens as though the original
resource lifetime continued.

Restart change-feed listeners from the beginning for the restored resource.
Only its **post-restore lifetime** events are available; events from before the
deletion aren't propagated/restored into that feed. Restarting does NOT recreate
lost pre-deletion history. Preserve old checkpoints as evidence but do not blindly
reuse them for this new lifetime. Design idempotent replay, downstream business/
projection reconciliation, explicit tombstone/delete/TTL-gap recovery, and separate
rebuild from validated restored state if required. Test duplicate/stale effects
and missing-history scenarios before resuming consumers/cutover.

Sources: [same-account recovery](https://learn.microsoft.com/azure/cosmos-db/how-to-restore-in-account-continuous-backup),
[restored lifetime/token/feed semantics](https://learn.microsoft.com/azure/cosmos-db/restore-in-account-continuous-backup-introduction),
[resource model](https://learn.microsoft.com/azure/cosmos-db/restore-in-account-continuous-backup-resource-model).
