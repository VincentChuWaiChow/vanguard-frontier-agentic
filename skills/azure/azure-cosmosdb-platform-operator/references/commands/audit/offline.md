# audit --mode offline

Inputs are the repository (Bicep, ARM, Terraform, application and client config)
and sanitized exports supplied by the user. No Azure CLI, network call, credential
or customer data. Commands are local only (`rg`, `grep`, `jq`, `find`, `git`,
`terraform show` on a saved plan). Property names follow the ARM
`Microsoft.DocumentDB/databaseAccounts` template reference; patterns are
case-insensitive so camelCase and snake_case spellings both match. Compare
findings with the operations reference for the API in scope:
[NoSQL](../../operations.md), [MongoDB](../../operations-mongodb.md),
[Cassandra](../../operations-cassandra.md).

```bash
: "${REPO:?Approved repository path}"
cd "$REPO"
rg -n -i 'Microsoft\.DocumentDB/databaseAccounts' --glob '*.bicep' --glob '*.json' --glob '*.tf'
```

## Backup and restore

- Check `backupPolicy`: `type` (Periodic or Continuous), `continuousModeProperties`
  tier, and for periodic `backupIntervalInMinutes`, `backupRetentionIntervalInHours`
  and `backupStorageRedundancy`.
- NoSQL: a continuous account cannot return to periodic, so flag any template that
  flips the type. MongoDB and Cassandra: confirm which backup modes the operations
  file lists for that API before judging.
- Look for restore settings (`createMode`, `restoreParameters`, `restoreMode`) left
  in a template that is re-applied routinely.
- Firewall, virtual network, private endpoint and data-plane role settings are not
  restored; look for IaC that recreates them.

```bash
rg -n -i 'backup.?policy|continuous.?mode|periodic.?mode|backup.?interval|backup.?retention|backup.?storage.?redundancy|create.?mode|restore.?(parameters|mode)' --glob '*.bicep' --glob '*.json' --glob '*.tf'
```

## Disaster recovery and replication

- Check `locations` entries for `failoverPriority` and `isZoneRedundant` per region,
  `enableAutomaticFailover` and `enableMultipleWriteLocations`.
- NoSQL: serverless accounts are single-region. MongoDB and Cassandra: check the
  operations file for the multi-region behavior of that API.
- Check `consistencyPolicy` against the stated RPO needs; Strong is not available
  with multiple write regions in the NoSQL reference.
- Client code: look for preferred or excluded region settings and retry policy.

```bash
rg -n -i 'locations|failover.?priority|is.?zone.?redundant|enable.?automatic.?failover|enable.?multiple.?write.?locations|consistency.?policy|default.?consistency|preferred.?regions|excluded.?regions' --glob '*.bicep' --glob '*.json' --glob '*.tf' --glob '*.cs' --glob '*.java' --glob '*.py' --glob '*.ts'
```

## Cost

- Throughput mode: provisioned, autoscale (maximum throughput) or serverless;
  look for oversized fixed RU/s and for `capacity.totalThroughputLimit`.
- Features that add billing dimensions per the operations file: extra regions,
  `enableMultipleWriteLocations`, analytical storage, zone redundancy, continuous
  backup tiers, diagnostic export to Log Analytics.
- Do not write prices or amounts; record dimensions only.
- NoSQL, MongoDB and Cassandra each list their own meters; use the file for the API
  in scope.

```bash
rg -n -i 'autoscale|max.?throughput|throughput|total.?throughput.?limit|enable.?analytical.?storage|serverless' --glob '*.bicep' --glob '*.json' --glob '*.tf'
```

## Monitoring

- Look for alert rules or metric alerts on the account resource (for example on
  `NormalizedRUConsumption` and `TotalRequests`) and who owns them.
- Check client code and logs for handling of HTTP 429 and retry settings, so
  throttling is visible and not only masked by SDK retries.
- MongoDB: look for request-charge and 16500 handling; Cassandra: look for
  `OverloadedException` handling. Confirm the exact codes in the operations file.

```bash
rg -n -i 'NormalizedRUConsumption|TotalRequests|metric.?alert|429|too.?many.?requests|max.?retry|retry.?(policy|options)' --glob '*.bicep' --glob '*.json' --glob '*.tf' --glob '*.cs' --glob '*.java' --glob '*.py' --glob '*.ts'
```

## Performance

- Partition key choice in container or collection definitions (`partitionKey`,
  shard key, Cassandra partition key); flag keys with low cardinality or skew.
- Indexing policy (`indexingPolicy`) that indexes everything on write-heavy paths,
  or composite indexes missing for sorted queries (NoSQL).
- Client singletons and connection mode; cross-partition or unbounded queries in
  hot paths.
- MongoDB and Cassandra: use the performance section of the operations file for
  API-specific limits.

```bash
rg -n -i 'partition.?key|shard.?key|indexing.?policy|composite.?indexes|included.?paths|excluded.?paths|cross.?partition|enable.?cross.?partition' --glob '*.bicep' --glob '*.json' --glob '*.tf' --glob '*.cs' --glob '*.java' --glob '*.py' --glob '*.ts'
```

## IAM

- `disableLocalAuth` set to true where Microsoft Entra ID is the intended data
  access path; flag connection strings or account keys in repository config.
- `disableKeyBasedMetadataWriteAccess` for control-plane changes.
- NoSQL data-plane RBAC role definitions and assignments are separate resources
  from Azure RBAC; check their scope and that built-in roles are preferred.
- MongoDB: native role-based access (`EnableMongoRoleBasedAccessControl` capability
  per the operations file). Cassandra: confirm identity options in its file.

```bash
rg -n -i 'disable.?local.?auth|disable.?key.?based.?metadata.?write.?access|sqlRoleAssignments|sqlRoleDefinitions|role.?definition.?id|principal.?id|EnableMongoRoleBasedAccessControl|AccountEndpoint=|AccountKey=|mongodb(\+srv)?://[^ ]*@' --glob '!node_modules' .
```

## Tagging

- Check the `tags` object on the account resource for the owner, environment and
  data-classification keys the organization requires.
- Container and database tagging is not confirmed in the operations file; do not
  report missing tags there as a defect without evidence.
- Look for Azure Policy definitions or assignments that require tags.

```bash
rg -n -i '\btags\b' --glob '*.bicep' --glob '*.json' --glob '*.tf'
```

## Encryption

- Customer-managed key: `keyVaultKeyUri` on the account and `defaultIdentity`;
  the operations file lists the Key Vault prerequisites.
- `minimalTlsVersion` and client-side TLS settings; flag TLS below the documented
  minimum or certificate validation disabled in client code.
- Confirm the identity type used for key access per the operations file.

```bash
rg -n -i 'key.?vault.?key.?uri|default.?identity|minimal.?tls.?version|tls.?version|ssl.?invalid|verify.?cert|trust.?all|insecure.?skip' --glob '*.bicep' --glob '*.json' --glob '*.tf' --glob '*.cs' --glob '*.java' --glob '*.py' --glob '*.ts'
```

## Network isolation

- `publicNetworkAccess`, `ipRules`, `isVirtualNetworkFilterEnabled`,
  `virtualNetworkRules` and `networkAclBypass`; flag open or wide ranges.
- Private endpoint resources (`Microsoft.Network/privateEndpoints`) and the group
  id each uses for the API in scope.
- Private DNS zones linked for the account endpoints; a private endpoint without
  DNS resolution does not isolate clients.

```bash
rg -n -i 'public.?network.?access|ip.?rules|is.?virtual.?network.?filter.?enabled|virtual.?network.?rules|network.?acl.?bypass|privateEndpoints|private.?link|privatelink\.' --glob '*.bicep' --glob '*.json' --glob '*.tf'
```

## Audit logging

- A `Microsoft.Insights/diagnosticSettings` resource scoped to the account with the
  log categories the operations file names (control plane and, per API, data-plane
  request categories).
- Destination (Log Analytics, Storage, Event Hubs) and retention owner. For
  Cassandra, full-text diagnostics can log query parameters (PII); check the
  warning in its operations file.
- Disabling key-based metadata writes keeps control-plane changes on the audited
  path.

```bash
rg -n -i 'diagnosticSettings|ControlPlaneRequests|DataPlaneRequests|MongoRequests|CassandraRequests|QueryRuntimeStatistics|log.?analytics|workspace.?id' --glob '*.bicep' --glob '*.json' --glob '*.tf'
```
