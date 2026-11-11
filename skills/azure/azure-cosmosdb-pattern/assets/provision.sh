#!/usr/bin/env bash
# Illustrative BILLABLE CLI alternative to ARM, not executed on skill load.
# Caller must independently verify globally new account name and absent resources.
set -euo pipefail
: "${AZ_SUBSCRIPTION:?explicit subscription required}" "${RESOURCE_GROUP:?existing group required}"
: "${ACCOUNT_NAME:?new account required}" "${LOCATION:?explicit region required}"
: "${APPROVED_TARGET_ID:?exact subscription/group/account creation approval required}"
target_id="/subscriptions/$AZ_SUBSCRIPTION/resourceGroups/$RESOURCE_GROUP/providers/Microsoft.DocumentDB/databaseAccounts/$ACCOUNT_NAME"
[[ "$APPROVED_TARGET_ID" == "$target_id" ]] || { echo 'approval target mismatch' >&2; exit 2; }
[[ "$ACCOUNT_NAME" =~ ^[a-z0-9][a-z0-9-]{1,42}[a-z0-9]$ ]] || { echo 'invalid account name' >&2; exit 2; }
asset_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
az_scope=(--subscription "$AZ_SUBSCRIPTION" --only-show-errors --output json)
# Fail closed on existing target OR any unclassified read failure. No adoption.
if probe="$(az cosmosdb show "${az_scope[@]}" --resource-group "$RESOURCE_GROUP" --name "$ACCOUNT_NAME" 2>&1)"; then
  echo 'target exists; refuse create/update/adoption' >&2; exit 2
else
  [[ "$probe" == 'ERROR: (ResourceNotFound)'* ]] || { echo 'target lookup failed; absence unproven' >&2; exit 2; }
fi
# Absence is a preflight observation, not a cloud-side atomic create-only guarantee.
# Stop on concurrent-owner/name drift; never retry by deleting or adopting.
az cosmosdb create "${az_scope[@]}" --resource-group "$RESOURCE_GROUP" --name "$ACCOUNT_NAME" \
  --kind GlobalDocumentDB --locations "regionName=$LOCATION" failoverPriority=0 isZoneRedundant=False \
  --default-consistency-level Session --backup-policy-type Continuous --continuous-tier Continuous7Days \
  --enable-automatic-failover false --enable-multiple-write-locations false \
  --disable-local-auth true --public-network-access DISABLED --minimal-tls-version Tls12
az cosmosdb sql database create "${az_scope[@]}" --resource-group "$RESOURCE_GROUP" \
  --account-name "$ACCOUNT_NAME" --name ordersdb
az cosmosdb sql container create "${az_scope[@]}" --resource-group "$RESOURCE_GROUP" \
  --account-name "$ACCOUNT_NAME" --database-name ordersdb --name orders \
  --partition-key-path /tenantId --partition-key-version 2 --throughput 400 --ttl -1 \
  --idx "@$asset_dir/indexing-policy.json"
# No waiter/cutover/data request/network opening/role grant/cleanup is implicit.
