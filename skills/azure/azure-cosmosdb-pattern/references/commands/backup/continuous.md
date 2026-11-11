# Continuous protection

1. On authorized metadata only, read account `backupPolicy.type` and
   `continuousModeProperties.tier`, API/region, source instance identity and
   resource existence times. Current native retention is 7/30/35 days; don't assume
   the CLI/pinned ARM version exposes 35 merely because service docs list it.
2. Capture per-resource/per-region latest restorable timestamp; aggregate recovery
   uses the minimum latest timestamp across required containers. Discover actual
   backup window before selecting a point. Creation/migration/tier-change time
   can shorten usable history; new tier doesn't manufacture older backups.
3. Plan periodic→continuous migration or tier change through existing IaC owner.
   Continuous→periodic is unsupported. Shortening tier can lose older recovery
   points; require approved loss/cost analysis. Continuous backup redundancy is
   platform-controlled (local/zone based on region configuration), not a generic
   periodic backup redundancy switch.
4. Confirm restore/enumeration permissions, CMK/default identity/old versions and
   recovery target/checks. Return observed/unknown window, RPO/RTO gaps and drill
   plan, not “backup healthy” from policy alone. No backup operation runs here.

Authorized metadata example (the installed CLI may need a separately approved
extension for latest-backup-time; never install automatically):

```bash
az cosmosdb sql retrieve-latest-backup-time --subscription "$AZ_SUBSCRIPTION" \
  --resource-group "$RESOURCE_GROUP" --account-name "$ACCOUNT_NAME" \
  --database-name ordersdb --container-name orders --location "$LOCATION"
```

Sources: [continuous introduction](https://learn.microsoft.com/azure/cosmos-db/continuous-backup-restore-introduction),
[latest timestamp](https://learn.microsoft.com/azure/cosmos-db/get-latest-restore-timestamp).
