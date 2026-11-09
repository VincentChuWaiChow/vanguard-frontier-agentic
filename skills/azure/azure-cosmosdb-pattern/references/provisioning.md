# Provisioning — illustrative alternatives, not deployment permission

Choose [ARM JSON](../assets/account.arm.json) OR [az CLI](../assets/provision.sh),
never run both sequentially against the same owner/resource. Template is complete
for a new NoSQL account/database/container, not a full application landing zone.
Use an existing resource group and separately approved new global account name.
ARM API version is pinned 2025-04-15; no Bicep compiler/download is necessary.

| Invariant | ARM and CLI |
|---|---|
| API/account | GlobalDocumentDB NoSQL, Standard offer, explicit account/location |
| Model | ordersdb/orders, `/tenantId` Hash v2, required string item id |
| Index | consistent automatic root, exclude `_etag` and payload, composite createdAt DESC/id ASC |
| Throughput | dedicated container manual 400 RU/s; DB has no shared offer, no serverless/autoscale |
| Consistency/regions | Session, one non-zone-redundant region, no multi-write/failover |
| Security | local/key auth disabled, public network Disabled, TLS1.2 minimum |
| Encryption | service-managed at rest; no keyVaultKeyUri/--key-uri; not plaintext |
| Backup/TTL | Continuous7Days, container -1 TTL (no default expiry; item opt-in possible) |

400 RU/s and one region are **example assumptions**, not production sizing/HA.
Public access is disabled; without separately approved private endpoint/DNS and
data RBAC, the example intentionally isn't application-accessible. Do not open
public network or enable keys to make a sample succeed. App data roles are not
control-plane Contributor, and no broad role is granted by these assets.

Before any execution confirm tenant/subscription/principal/group/location, new
resource absence and name availability, policy/quotas/cost/network ownership,
exact plan/what-if, preservation, approval and rollback. `create` can update an
existing resource; name-absence check is a prerequisite, not a hidden adoption
fallback. Only a classified NotFound is absence; auth/network/CLI errors stop.

```bash
# Illustrative ARM preview; requires separately authorized Azure reads.
az deployment group what-if --subscription "$AZ_SUBSCRIPTION" \
  --resource-group "$RESOURCE_GROUP" --template-file assets/account.arm.json \
  --parameters accountName="$ACCOUNT_NAME" location="$LOCATION"
# Actual deployment is a separate reviewed operation, not implicit in preview.
```

After approved creation inspect provisioning state/error plus exact resource ID,
kind/API, region, consistency, backup/key/network/auth, container key/index/TTL
and throughput. Then separately validate real app access/tenant denials and
requested recovery/performance; successful provisioning isn't runtime proof.

Optional vector create (only selected when requested): preserve all base security/
backup/capacity assumptions, verify account capability `EnableNoSQLVectorSearch`
and current API/CLI, and create **distinct** `chunks` container using
`--vector-embeddings @assets/vector-policy.json` plus
`--idx @assets/vector-indexing-policy.json`, `/tenantId`, dedicated throughput.
These policies use float32/1536 diskANN; don't silently add a capability to existing
accounts or assume float16 works in this pinned schema. CMK variant is a reviewed
delta documented in [encryption](encryption.md), not hidden Key Vault setup.

Local JSON/resource/CLI equivalence tests don't validate Azure schema acceptance,
authorization, deployment, private connectivity or pricing. No az/Bicep was
installed or provider command executed during package authoring.

Sources: [account ARM schema](https://learn.microsoft.com/azure/templates/microsoft.documentdb/2025-04-15/databaseaccounts),
[container schema](https://learn.microsoft.com/azure/templates/microsoft.documentdb/2025-04-15/databaseaccounts/sqldatabases/containers),
[az create](https://learn.microsoft.com/cli/azure/cosmosdb#az-cosmosdb-create),
[container create](https://learn.microsoft.com/cli/azure/cosmosdb/sql/container#az-cosmosdb-sql-container-create).
