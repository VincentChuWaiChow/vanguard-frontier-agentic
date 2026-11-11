# audit --mode live

Read only for a requested live audit; do not load the offline sibling. Commands
are examples for an operator holding read-only authorization against
[the operations reference](../../operations.md); the skill does not run them.
They read control-plane metadata only. Never read rows or table contents.

## Before any command

- Confirm the approved principal, tenancy, region, compartment and exact table.
  Stop on any mismatch; never switch target to make a command work.
- Use a read-only role. Run `umask 077` and write output to a private temporary
  directory. Never print keys, fingerprints, OCIDs beyond what the report needs,
  or customer data.

```bash
set -euo pipefail
: "${OCI_CLI_PROFILE:?Approved profile}" "${OCI_REGION:?Approved region}" \
  "${TENANCY_ID:?Approved tenancy OCID}" "${COMPARTMENT_ID:?Approved compartment OCID}" \
  "${TABLE_NAME:?Approved table name}" "${START_TIME:?Approved window start, RFC 3339}" \
  "${END_TIME:?Approved window end, RFC 3339}"
umask 077
audit_dir=$(mktemp -d)
oci_target=(oci --profile "$OCI_CLI_PROFILE" --region "$OCI_REGION" --output json)
"${oci_target[@]}" iam tenancy get --tenancy-id "$TENANCY_ID" > "$audit_dir/tenancy.json" # [OCI-NOSQL-E901]
jq -e --arg t "$TENANCY_ID" '.data.id == $t' "$audit_dir/tenancy.json" >/dev/null \
  || { echo "Tenancy mismatch; stop" >&2; exit 1; } # [OCI-NOSQL-E915] [OCI-NOSQL-E916]
```

The tenancy read confirms the profile reaches the approved tenancy. The OCI CLI has
no documented command that returns the calling principal; confirm the profile's user
or instance principal before running.

## Backup and restore

No native backup is documented; backup is a customer-run migrator export. The
only metadata read is the object listing of the export bucket (names, sizes and
creation times, never contents).

```bash
: "${BACKUP_NAMESPACE:?Approved Object Storage namespace}" "${BACKUP_BUCKET:?Approved backup bucket}"
"${oci_target[@]}" os object list --namespace-name "$BACKUP_NAMESPACE" --bucket-name "$BACKUP_BUCKET" \
  --fields name,size,timeCreated --all > "$audit_dir/backup-objects.json" # [OCI-NOSQL-E913]
```

A listing shows files exist; it does not prove a restore works. Report restore as
UNVERIFIED without a separately approved test into a scratch table.

## Disaster recovery and replication

```bash
"${oci_target[@]}" iam region-subscription list --tenancy-id "$TENANCY_ID" > "$audit_dir/regions.json" # [OCI-NOSQL-E902]
"${oci_target[@]}" nosql table get --table-name-or-id "$TABLE_NAME" --compartment-id "$COMPARTMENT_ID" > "$audit_dir/table.json" # [OCI-NOSQL-E903]
```

Read the table document for replication state as returned; repeat in each replica
region. Replica Lag is a Monitoring metric (see Monitoring). No RPO or RTO figure
is documented.

## Cost

**No live command:** billing and cost data live in the Usage API, whose CLI page
(`oci usage-api usage-summary`) offers only `request-summarized-usages`, a request
verb that fails this audit's read-verb rule, so no cost command is recorded. Use
the Cost Analysis console for the tenancy. Capacity mode and limits come from the
table read in Disaster recovery, and usage records from Performance.

## Monitoring

```bash
"${oci_target[@]}" monitoring alarm list --compartment-id "$COMPARTMENT_ID" --all > "$audit_dir/alarms.json" # [OCI-NOSQL-E908]
"${oci_target[@]}" monitoring metric list --compartment-id "$COMPARTMENT_ID" --namespace oci_nosql --all > "$audit_dir/metrics.json" # [OCI-NOSQL-E911]
```

Compare alarm queries with the metric names in [OCI-NOSQL-E110] (shard size,
throttle counts, Replica Lag) and check notification ownership. Metric values
are not read: the aggregation command `summarize-metrics-data` has no read verb
under this audit's rule, so read data points in the Console metrics explorer.

## Performance

```bash
"${oci_target[@]}" nosql table list-table-usage --table-name-or-id "$TABLE_NAME" --compartment-id "$COMPARTMENT_ID" \
  --time-start "$START_TIME" --time-end "$END_TIME" > "$audit_dir/usage.json" # [OCI-NOSQL-E906]
"${oci_target[@]}" nosql index list --table-name-or-id "$TABLE_NAME" --compartment-id "$COMPARTMENT_ID" > "$audit_dir/indexes.json" # [OCI-NOSQL-E905]
"${oci_target[@]}" nosql table list --compartment-id "$COMPARTMENT_ID" --all > "$audit_dir/tables.json" # [OCI-NOSQL-E904]
```

Compare index count and table count with the documented limits in the operations
reference. Usage records do not prove client latency.

## IAM

```bash
"${oci_target[@]}" iam policy list --compartment-id "$COMPARTMENT_ID" --all > "$audit_dir/policies.json" # [OCI-NOSQL-E910]
jq -r '.data[].statements[]' "$audit_dir/policies.json" > "$audit_dir/policy-statements.txt"
grep -i 'nosql' "$audit_dir/policy-statements.txt" || echo "No policy statement mentions nosql"
```

The page notes there is no automatic way to learn which policies apply to a group;
review statements by hand. Policies in the tenancy root or parent compartments
need a separate approved read.

## Tagging

**No live command:** the tag namespace list command (`oci iam tag-namespace`)
carries the word tag, which this audit's mutating-verb rule rejects, and the
`oci nosql table get` page does not document table tags in its output. Review
tags offline from IaC and in the Console, and report live tags as UNVERIFIED.

## Encryption

**No live command:** searched the nosql table and index command pages; none
documents an encryption or CMEK setting, and the customer-managed key exists only
in a dedicated environment. Encryption state is not readable with a documented
NoSQL CLI command. Use the operations reference and key audit events instead.

## Network isolation

```bash
"${oci_target[@]}" network service-gateway list --compartment-id "$COMPARTMENT_ID" --all > "$audit_dir/service-gateways.json" # [OCI-NOSQL-E912]
```

This shows whether a service gateway exists. It does not show routes, subnet use,
or any NDCS private endpoint, none of which is documented.

## Audit logging

```bash
"${oci_target[@]}" audit event list --compartment-id "$COMPARTMENT_ID" \
  --start-time "$START_TIME" --end-time "$END_TIME" --all > "$audit_dir/audit-events.json" # [OCI-NOSQL-E909]
"${oci_target[@]}" nosql work-request list --compartment-id "$COMPARTMENT_ID" --all > "$audit_dir/work-requests.json" # [OCI-NOSQL-E907]
```

Audit events cover control-plane calls. Row operations are not documented as
audited; do not claim row-level audit coverage. Times must have seconds set to 0.
