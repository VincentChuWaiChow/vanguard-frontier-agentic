# Scoped Azure CLI cookbook — illustrative, never auto-run

Every request pins `--subscription` and `--resource-group` where supported plus
account/database/container. No `az account set`, implicit current subscription,
environment dump or `list-keys`. Commands needing extensions are not permission
to install them. No az was installed during authoring; syntax checked against
official docs and local fake-command tests, not Azure service acceptance.

```bash
az cosmosdb show --subscription "$AZ_SUBSCRIPTION" \
  --resource-group "$RESOURCE_GROUP" --name "$ACCOUNT_NAME"
az cosmosdb sql container show --subscription "$AZ_SUBSCRIPTION" \
  --resource-group "$RESOURCE_GROUP" --account-name "$ACCOUNT_NAME" \
  --database-name ordersdb --name orders
az cosmosdb sql container throughput show --subscription "$AZ_SUBSCRIPTION" \
  --resource-group "$RESOURCE_GROUP" --account-name "$ACCOUNT_NAME" \
  --database-name ordersdb --name orders
```

These are authorized metadata reads only. Check returned account ID/API/region;
settings readback isn't runtime proof. Data CRUD/SQL uses selected SDK/NoSQL REST,
not a fictional `az cosmosdb query/get-item` management command. See matching
adapter only. Creation uses gated assets/provision.sh OR ARM under existing owner.

An approved periodic policy change (never execute during audit):

```bash
az cosmosdb update --subscription "$AZ_SUBSCRIPTION" \
  --resource-group "$RESOURCE_GROUP" --name "$ACCOUNT_NAME" \
  --backup-interval 240 --backup-retention 168
```

Only apply to an observed periodic account after retention/cost/recovery review.
It doesn't switch continuous back to periodic. Load exact recovery method guide
for timestamp/window/restore examples; don't preload all restore methods here.

Sources: [az cosmosdb](https://learn.microsoft.com/cli/azure/cosmosdb),
[container](https://learn.microsoft.com/cli/azure/cosmosdb/sql/container),
[throughput](https://learn.microsoft.com/cli/azure/cosmosdb/sql/container/throughput).
