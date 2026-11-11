# Access — control plane isn't data access

| Layer | Native mechanism | Boundary |
|---|---|---|
| Azure resource management | Azure RBAC actions on Microsoft.DocumentDB account resources | Contributor/Operator controls account configuration, not automatically NoSQL item access |
| NoSQL data plane | Cosmos SQL role definitions/assignments with `dataActions`, Entra token | Scope account `/`, database `/dbs/<db>` or container `/dbs/<db>/colls/<container>` as required |
| Account keys | Read/write or read-only keys; broad account access when local auth enabled | Not scoped per tenant; don't list/print/embed keys or expose to browsers |
| Application tenant | Server-authenticated subject/tenant and trusted route | Partition key/query predicate isn't data RBAC row-level isolation |
| CMK | Cosmos service/default identity plus Key Vault key permissions | Data reader role doesn't authorize key administration or restore |

Prefer managed identity/Entra with existing credential flow. Data Reader built-in
ID ends `00000000-0000-0000-0000-000000000001`; Data Contributor ends `...0002`.
Use the account-specific role-definition resource ID and narrow scope. Query/change
feed access needs appropriate query/change-feed/readMetadata data actions; don't
copy a role name and assume effective permissions. Custom roles list exact
supported actions. ARM tags govern metadata/cost, not per-item authorization.

Approved creation example only (do not execute on load):

```bash
az cosmosdb sql role assignment create --subscription "$AZ_SUBSCRIPTION" \
  --resource-group "$RESOURCE_GROUP" --account-name "$ACCOUNT_NAME" \
  --principal-id "$APP_PRINCIPAL_OBJECT_ID" \
  --role-definition-id "$DATA_READER_ROLE_DEFINITION_ID" \
  --scope '/dbs/ordersdb/colls/orders'
```

Principal OBJECT ID isn't app/client ID. Before disabling local auth on an existing
account, prove all legitimate clients and operations support Entra, data roles
propagate and break-glass/recovery posture is reviewed; don't lock applications out.
The new-account example disables local auth deliberately but assigns no broad role.
Network ingress/private endpoint/DNS/firewall are separate from token authorization.

Restore requires management-plane enumerate/restore permissions such as
`Microsoft.DocumentDB/locations/restorableDatabaseAccounts/read` and applicable
restore actions, plus permission to create destination resources. Grant the exact
documented role/scope after reviewing current permission docs; a Data Contributor
assignment alone can't restore an account. Read-only audit never grants roles.

Sources: [Entra and data/control RBAC](https://learn.microsoft.com/azure/cosmos-db/how-to-setup-rbac),
[native data RBAC](https://learn.microsoft.com/azure/cosmos-db/how-to-connect-role-based-access-control),
[restore permissions](https://learn.microsoft.com/azure/cosmos-db/continuous-backup-restore-permissions).
