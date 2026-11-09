# Illustrative provisioning and core OCI CLI

Choose Terraform OR CLI for a NEW table; neither auto-imports/adopts an existing
table. Default path: Oracle-managed encryption, provisioned 50 RU/50 WU/1 GB,
no auto-reclaim, 30-day TTL, composite tenant/status/time index. Read the actual
[DDL](../assets/orders.ddl), [Terraform](../assets/orders.tf), [limits](../assets/table-limits.json)
and [index keys](../assets/index-keys.json) only when requested. `tenantId` shard
locality is not row-level IAM. Capacity is illustrative, not production sizing.

Terraform: place the DDL beside the module, supply approved region/compartment/new
name through normal variable inputs, pin an approved OCI provider and retain its
lockfile. Credentials/principal setup is deliberately external. Obtain an exact
non-destructive plan before apply; `prevent_destroy` is not a backup or import
permission. Do not run init/download/deployment as part of loading this skill.
No per-table CMEK option is supplied. Terraform CLI validation was unavailable in
the authoring sandbox; see sources. Local fixture tests are not provider validation.

The guarded [create-table.sh](../assets/create-table.sh) checks reachable tenancy/
compartment IDs and explicit typed confirmation, then creates table/index and reads
metadata back. It declares, but does not attest, the active principal. Independently
verify principal and compartment ancestry before execution. On errors/timeouts stop
and inspect exact work requests; never adopt/delete/retry uncertain writes blindly.

Core cookbook below is illustrative, not run. Export existing approved `OCI_PROFILE`,
`OCI_REGION`, `COMPARTMENT_OCID`, `TABLE_NAME` first. Quote every variable.

```bash
oci --profile "$OCI_PROFILE" --region "$OCI_REGION" nosql table get --compartment-id "$COMPARTMENT_OCID" --table-name-or-id "$TABLE_NAME"
oci --profile "$OCI_PROFILE" --region "$OCI_REGION" nosql row get --compartment-id "$COMPARTMENT_OCID" --table-name-or-id "$TABLE_NAME" --key 'tenantId:tenant-demo' --key 'orderId:order-demo' --consistency ABSOLUTE
# Mutation: separate approval; absent condition, not an unconditional upsert.
oci --profile "$OCI_PROFILE" --region "$OCI_REGION" nosql row update --compartment-id "$COMPARTMENT_OCID" --table-name-or-id "$TABLE_NAME" --value file://assets/row.json --option IF_ABSENT
# Prepare the reviewed SELECT; replace compartment + Base64 statement in query.json.
oci --profile "$OCI_PROFILE" --region "$OCI_REGION" nosql query prepare --compartment-id "$COMPARTMENT_OCID" --statement "$(cat assets/query.sql)" --query data.statement --raw-output
oci --profile "$OCI_PROFILE" --region "$OCI_REGION" nosql query execute --from-json file://assets/query.json
```

REST/CLI query pagination uses returned `opc-next-page` with `--page`, NOT SDK
`continuationKey`; an empty result with continuation still needs another request.
Neither pagination path proves snapshot consistency. For conditional updates/deletes
retain the actual service row version/ETag and correct `--if-match` contract; don't
substitute a timestamp or table ETag. [Portable runtime contract](runtime-contract.md).

[OCI CLI NoSQL](https://docs.oracle.com/en-us/iaas/tools/oci-cli/latest/oci_cli_docs/cmdref/nosql.html),
[table provider](https://docs.oracle.com/en-us/iaas/tools/terraform-provider-oci/latest/docs/r/nosql_table.html), checked 2026-11-08.
