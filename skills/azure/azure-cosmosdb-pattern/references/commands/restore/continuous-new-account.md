# Continuous recovery into a NEW account

Apply recovery-target contract first. Bind exact source/restorable instance,
region present at UTC point, dynamic earliest/latest bounds, destination
subscription/group/new account and approved scope. Same-subscription only; a
precreated account isn't the destination. Accidental item corruption normally
uses this method, not an in-place overwrite.

CLI example is an **illustrative approved operation**, never automatically run:

```bash
az cosmosdb restore --subscription "$AZ_SUBSCRIPTION" \
  --resource-group "$TARGET_RESOURCE_GROUP" --account-name "$SOURCE_ACCOUNT" \
  --target-database-account-name "$TARGET_ACCOUNT" --location "$LOCATION" \
  --restore-timestamp "$RESTORE_TIMESTAMP" --disable-ttl true \
  --disable-local-auth true --public-network-access DISABLED
```

The CLI uses source account name, not a source-instance argument. Enumerate and
bind its exact restorable instance first; if reused names are ambiguous, stop and
use reviewed ARM `createMode: Restore` / `restoreParameters.restoreSource` full
restorable-resource-ID workflow rather than guessing. No hidden name fallback.
For partial restore use documented repeated `--databases-to-restore`
`name=DatabaseName collections=collection1 collection2` CLI shorthand, NOT ARM
JSON. Verify installed help and bind the exact approved subset before execution.
A subset of containers in shared-throughput database can't be restored;
use whole database. CMK requires reviewed `--assign-identity`/`--default-identity`
and working key access, not the service-managed example copied unchanged.

The offline recovery_guard.py helper deliberately supports **full-account only**
for this method: explicit matching restore_scope/approved_restore_scope are
required and any subset fields fail. Partial restore remains a reviewed manual
workflow; it must never silently expand to all-account recovery.

Track async provisioning/error status and re-read restoreParameters source/time
and destination identity; don't retry uncertain outcomes. Recovered data/index
properties/consistency can be restored, but throughput may not match point in
time. Reconcile firewall/VNet/private endpoints, data/control RBAC, other regions,
stored procedures/triggers/UDFs, diagnostics and application dependencies; current
docs say these settings aren't all restored. Keep public network disabled while
repairing connectivity through the approved existing network workflow.

Explicit `--disable-ttl true` is the example's reviewed preservation choice, not
a permanent production setting. Validate/re-enable expiry deliberately later.
Index rebuilding, application/security/domain validation and measured RPO/RTO must
pass before separately approved cutover. Waiter success alone isn't recovery.

Sources: [restore command/workflow](https://learn.microsoft.com/azure/cosmos-db/restore-account-continuous-backup),
[what is/isn't restored](https://learn.microsoft.com/azure/cosmos-db/continuous-backup-restore-introduction),
[CLI restore](https://learn.microsoft.com/cli/azure/cosmosdb#az-cosmosdb-restore).
