# audit --mode live

Commands are examples for an operator holding read-only authorization; the skill
does not run them. They read control-plane metadata only. Compare results with the
operations reference for the API in scope:
[NoSQL](../../operations.md), [MongoDB](../../operations-mongodb.md),
[Cassandra](../../operations-cassandra.md). Per-API sub-bullets below say which
commands apply; run only those for the API under audit.

## Before any command

- Confirm the approved principal, subscription, tenant, Region and exact account.
  Stop on any mismatch; never switch target to make a command work.
- Use a read-only role. Write output to a private temp directory (`umask 077`).
  Never print keys, connection strings or customer data; no key or connection
  string command is part of this audit.
- Control-plane commands only: no data-plane shell, no item, document or row reads.

```bash
set -euo pipefail
: "${EXPECTED_SUBSCRIPTION_ID:?Approved subscription ID}" "${COSMOS_RG:?Approved resource group}" \
  "${COSMOS_ACCOUNT:?Approved Cosmos DB account name}" "${COSMOS_LOCATION:?Approved Region}"
umask 077
audit_dir=$(mktemp -d)
az account show --output json > "$audit_dir/identity.json"   # [COSMOS-E901]
jq -e --arg s "$EXPECTED_SUBSCRIPTION_ID" '.id == $s' "$audit_dir/identity.json" > /dev/null \
  || { echo "Subscription mismatch; stop" >&2; exit 1; }
az cosmosdb show --name "$COSMOS_ACCOUNT" --resource-group "$COSMOS_RG" --output json > "$audit_dir/account.json"   # [COSMOS-E902]
jq -e --arg l "$COSMOS_LOCATION" '(.location|ascii_downcase|gsub(" ";"")) == ($l|ascii_downcase|gsub(" ";""))' \
  "$audit_dir/account.json" > /dev/null || { echo "Region mismatch; stop" >&2; exit 1; }   # [COSMOS-E920]
COSMOS_ACCOUNT_ID=$(jq -r '.id' "$audit_dir/account.json")
```

The Region check compares the account's `location` with `COSMOS_LOCATION`, ignoring
case and spaces, so `West US` and `westus` match.

The account JSON written above is reused by the sections below with local `jq`.
Fields absent from it are UNVERIFIED, not off.

## Backup and restore

- All APIs: inspect the backup policy and restorable-account metadata of the
  account. Which fields the output carries is not enumerated on the CLI page.
- NoSQL, MongoDB, Cassandra: the same account-level commands apply; restore
  options per API are in the operations files.

```bash
jq '.backupPolicy' "$audit_dir/account.json"   # [COSMOS-E902]
az cosmosdb restorable-database-account list --account-name "$COSMOS_ACCOUNT" --output json   # [COSMOS-E903]
```

## Disaster recovery and replication

- All APIs: regions, failover priority, zone redundancy and write-region flags come
  from the account JSON; absent fields are UNVERIFIED.

```bash
jq '{locations, writeLocations, readLocations, enableAutomaticFailover, enableMultipleWriteLocations}' "$audit_dir/account.json"   # [COSMOS-E902]
az cosmosdb list --resource-group "$COSMOS_RG" --output table   # [COSMOS-E902]
```

## Cost

**No live command:** billing and cost data live in Azure Cost Management and the
billing APIs, not in the Cosmos DB resource API; the CLI reference pages read
(`az cosmosdb` and its sql, mongodb and cassandra groups) show no read-only cost
command. Use the throughput commands in Performance for provisioned RU/s only.

## Monitoring

- All APIs: list the metric definitions the account exposes, then read a bounded
  window of aggregated platform metrics. Metric names come from the definitions
  output; the names below are those in the operations reference.

```bash
account_id="$COSMOS_ACCOUNT_ID"
az monitor metrics list-definitions --resource "$account_id" --output table   # [COSMOS-E908]
az monitor metrics list --resource "$account_id" --metrics NormalizedRUConsumption --offset 1d --interval PT1H --aggregation Maximum --output json   # [COSMOS-E907] [COSMOS-E126] [COSMOS-E921]
az monitor metrics list --resource "$account_id" --metrics TotalRequests --offset 1d --interval PT1H --aggregation Count --output json   # [COSMOS-E907] [COSMOS-E921]
```

## Performance

- Throughput and key design per API; `throughput show` fails for a container that
  has no dedicated throughput (it may use shared database throughput).
- NoSQL: set `COSMOS_SQL_DATABASE`, `COSMOS_SQL_CONTAINER`.
- MongoDB: set `COSMOS_MONGO_DATABASE`, `COSMOS_MONGO_COLLECTION`.
- Cassandra: set `COSMOS_CASSANDRA_KEYSPACE`, `COSMOS_CASSANDRA_TABLE`.

**NoSQL:**

```bash
: "${COSMOS_SQL_DATABASE:?Approved database}" "${COSMOS_SQL_CONTAINER:?Approved container}"
az cosmosdb sql database list --account-name "$COSMOS_ACCOUNT" --resource-group "$COSMOS_RG" --output table   # [COSMOS-E909]
az cosmosdb sql container show --account-name "$COSMOS_ACCOUNT" --resource-group "$COSMOS_RG" --database-name "$COSMOS_SQL_DATABASE" --name "$COSMOS_SQL_CONTAINER" --output json   # [COSMOS-E910]
az cosmosdb sql container throughput show --account-name "$COSMOS_ACCOUNT" --resource-group "$COSMOS_RG" --database-name "$COSMOS_SQL_DATABASE" --name "$COSMOS_SQL_CONTAINER" --output json   # [COSMOS-E911]
```

**MongoDB (RU-based):**

```bash
: "${COSMOS_MONGO_DATABASE:?Approved database}" "${COSMOS_MONGO_COLLECTION:?Approved collection}"
az cosmosdb mongodb database list --account-name "$COSMOS_ACCOUNT" --resource-group "$COSMOS_RG" --output table   # [COSMOS-E913]
az cosmosdb mongodb collection show --account-name "$COSMOS_ACCOUNT" --resource-group "$COSMOS_RG" --database-name "$COSMOS_MONGO_DATABASE" --name "$COSMOS_MONGO_COLLECTION" --output json   # [COSMOS-E914]
az cosmosdb mongodb collection throughput show --account-name "$COSMOS_ACCOUNT" --resource-group "$COSMOS_RG" --database-name "$COSMOS_MONGO_DATABASE" --name "$COSMOS_MONGO_COLLECTION" --output json   # [COSMOS-E915]
```

**Cassandra:**

```bash
: "${COSMOS_CASSANDRA_KEYSPACE:?Approved keyspace}" "${COSMOS_CASSANDRA_TABLE:?Approved table}"
az cosmosdb cassandra keyspace list --account-name "$COSMOS_ACCOUNT" --resource-group "$COSMOS_RG" --output table   # [COSMOS-E917]
az cosmosdb cassandra table show --account-name "$COSMOS_ACCOUNT" --resource-group "$COSMOS_RG" --keyspace-name "$COSMOS_CASSANDRA_KEYSPACE" --name "$COSMOS_CASSANDRA_TABLE" --output json   # [COSMOS-E918]
az cosmosdb cassandra table throughput show --account-name "$COSMOS_ACCOUNT" --resource-group "$COSMOS_RG" --keyspace-name "$COSMOS_CASSANDRA_KEYSPACE" --name "$COSMOS_CASSANDRA_TABLE" --output json   # [COSMOS-E919]
```

## IAM

- Account flags such as `disableLocalAuth` and `disableKeyBasedMetadataWriteAccess`
  come from the account JSON; absent fields are UNVERIFIED.
- NoSQL and MongoDB: list data-plane role definitions. Cassandra: the role commands
  found are extension commands; not covered here, so check identity settings from
  the account JSON and the operations file.

**All APIs:**

```bash
jq '{disableLocalAuth, disableKeyBasedMetadataWriteAccess}' "$audit_dir/account.json"   # [COSMOS-E902]
```

**NoSQL:**

```bash
az cosmosdb sql role definition list --account-name "$COSMOS_ACCOUNT" --resource-group "$COSMOS_RG" --output json   # [COSMOS-E912]
```

**MongoDB:**

```bash
az cosmosdb mongodb role definition list --account-name "$COSMOS_ACCOUNT" --resource-group "$COSMOS_RG" --output json   # [COSMOS-E916]
```

## Tagging

- All APIs: tags are on the account resource; inspect the account JSON. Database
  and container tagging is not confirmed in the operations files.

```bash
jq '{tags}' "$audit_dir/account.json"   # [COSMOS-E902]
```

## Encryption

- All APIs: customer-managed key URI and identity come from the account JSON; the
  key itself is never read. Absent fields are UNVERIFIED.

```bash
jq '{keyVaultKeyUri, defaultIdentity, minimalTlsVersion}' "$audit_dir/account.json"   # [COSMOS-E902]
```

## Network isolation

- All APIs: public access, IP rules and virtual network rules from the account
  JSON; list virtual network rules and private endpoint connections.

```bash
jq '{publicNetworkAccess, ipRules, isVirtualNetworkFilterEnabled, virtualNetworkRules}' "$audit_dir/account.json"   # [COSMOS-E902]
az cosmosdb network-rule list --name "$COSMOS_ACCOUNT" --resource-group "$COSMOS_RG" --output json   # [COSMOS-E904]
az network private-endpoint-connection list --id "$COSMOS_ACCOUNT_ID" --output json   # [COSMOS-E905]
```

## Audit logging

- All APIs: list the diagnostic settings on the account and compare the enabled
  categories with the operations file for that API (control plane and API request
  categories).

```bash
az monitor diagnostic-settings list --resource "$COSMOS_ACCOUNT_ID" --output json   # [COSMOS-E906]
```
