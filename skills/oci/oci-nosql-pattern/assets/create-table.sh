#!/usr/bin/env bash
# Illustrative billable NEW-table operation. Never invoked on skill load.
set -euo pipefail
: "${OCI_PROFILE:?approved existing profile}"
: "${OCI_REGION:?approved region}"
: "${TENANCY_OCID:?approved tenancy OCID}"
: "${COMPARTMENT_OCID:?approved compartment OCID}"
: "${PRINCIPAL_OCID:?externally verified approved principal OCID}"
: "${TABLE_NAME:?approved NEW table name}"
[[ "$TABLE_NAME" =~ ^[A-Za-z][A-Za-z0-9_]{0,63}$ ]] || { echo "Unsafe name" >&2; exit 1; }
asset_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
oci_cmd=(oci --profile "$OCI_PROFILE" --region "$OCI_REGION")
# These metadata reads confirm reachable targets, NOT the current principal identity.
observed_tenancy=$("${oci_cmd[@]}" iam tenancy get --tenancy-id "$TENANCY_OCID" --query data.id --raw-output)
[[ "$observed_tenancy" == "$TENANCY_OCID" ]] || { echo "Tenancy mismatch" >&2; exit 1; }
observed_compartment=$("${oci_cmd[@]}" iam compartment get --compartment-id "$COMPARTMENT_OCID" --query data.id --raw-output)
[[ "$observed_compartment" == "$COMPARTMENT_OCID" ]] || { echo "Compartment mismatch" >&2; exit 1; }
printf 'Declared principal=%s tenancy=%s compartment=%s region=%s NEW table=%s\n' "$PRINCIPAL_OCID" "$TENANCY_OCID" "$COMPARTMENT_OCID" "$OCI_REGION" "$TABLE_NAME"
read -r -p "After independent principal verification, type CREATE ${TABLE_NAME}: " confirmation
[[ "$confirmation" == "CREATE ${TABLE_NAME}" ]] || exit 1
ddl=$(cat "$asset_dir/orders.ddl")
ddl=${ddl/CREATE TABLE orders_v1/CREATE TABLE ${TABLE_NAME}}
# No IF NOT EXISTS, adoption, cleanup or deletion. A conflict is a hard stop.
"${oci_cmd[@]}" nosql table create --compartment-id "$COMPARTMENT_OCID" --name "$TABLE_NAME" \
  --ddl-statement "$ddl" --table-limits "file://${asset_dir}/table-limits.json" \
  --is-auto-reclaimable false --freeform-tags '{"purpose":"orders-example"}' --wait-for-state SUCCEEDED --max-wait-seconds 1200
"${oci_cmd[@]}" nosql index create --compartment-id "$COMPARTMENT_OCID" --table-name-or-id "$TABLE_NAME" \
  --index-name tenant_status_created --keys "file://${asset_dir}/index-keys.json" \
  --is-if-not-exists false --wait-for-state SUCCEEDED --max-wait-seconds 1200
# Readback is not application acceptance, ownership or backup proof.
"${oci_cmd[@]}" nosql table get --compartment-id "$COMPARTMENT_OCID" --table-name-or-id "$TABLE_NAME"
"${oci_cmd[@]}" nosql index get --compartment-id "$COMPARTMENT_OCID" --table-name-or-id "$TABLE_NAME" --index-name tenant_status_created
