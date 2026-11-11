# audit --mode live

Read only for a requested live audit; do not load the offline sibling. Commands
are examples for an operator holding read-only authorization; the skill does not
run them. DDS operation names for `hcloud` were not found on an allowlisted
official reference page in this pass, so the resource sections below record
`**No live command:**` rather than guess an operation name.

## Before any command

- Confirm the approved principal, Huawei Cloud account, IAM project, Region and
  exact instance. Stop on any mismatch; never switch target to make a command work.
- Use a read-only role. Write output to a private directory (`umask 077`). Never
  print keys, connection strings or customer data.
- KooCLI invokes API operations as `hcloud [options] <service> <operation>` [HW-DDS-E901].
  The local profile check below shows only profile name, mode and Region, not
  credentials [HW-DDS-E902]. It does not prove the remote principal: confirm the
  account and IAM user manually in the console and stop on mismatch.

```bash
set -euo pipefail
: "${HCLOUD_PROFILE:?Approved KooCLI profile name}"
: "${EXPECTED_REGION:?Approved Region}"
umask 077
audit_dir=$(mktemp -d)
hcloud configure list --cli-output=json --cli-query="profiles[].{Name:name,Mode:mode,Region:region}" > "$audit_dir/profiles.json" # [HW-DDS-E902]
jq -e --arg p "$HCLOUD_PROFILE" --arg r "$EXPECTED_REGION" 'map(select(.Name==$p and .Region==$r))|length==1' "$audit_dir/profiles.json"
```

## Backup and restore

**No live command:** `hcloud DDS` operation names are not stated on any allowlisted official page I fetched. The API reference documents `GET /v3/{project_id}/backups` (Querying the Backup List) and `GET /v3/{project_id}/instances/{instance_id}/backups/policy` (Querying an Automated Backup Policy), but no page maps them to an `hcloud` operation name. Use the console Backups view meanwhile.

## Disaster recovery and replication

**No live command:** no CLI command documented; searched the DDS API reference and KooCLI user guide. Review mode, AZ layout and read replicas in the console (instance details are `GET /v3/{project_id}/instances` in the API reference).

## Cost

**No live command:** billing lives in the Billing Center and BSS, not the DDS resource API; no read-only BSS operation was fetched and cited.

## Monitoring

**No live command:** Cloud Eye alarm and metric reads were not fetched and cited in this pass; review alarms in the Cloud Eye console.

## Performance

**No live command:** no control-plane CLI command documented that was fetched and cited; use console monitoring and slow-log views.

## IAM

**No live command:** identity reads are secret-adjacent or were not fetched; review policy attachments for the audit principal in the IAM console.

## Tagging

**No live command:** no tag-listing operation was fetched and cited; read instance tags in the console.

## Encryption

**No live command:** no documented CLI command was fetched and cited; check disk encryption, SSL and the DEW key in the console.

## Network isolation

**No live command:** no documented CLI command was fetched and cited; check security group, EIP and public gateway bindings in the console.

## Audit logging

**No live command:** the API reference documents `GET /v3/{project_id}/instances/{instance_id}/auditlog-policy` and `.../auditlog`, but no `hcloud DDS` operation name was verified on an allowlisted page, and the audit log list is data-plane-adjacent. Review the audit policy in the console; read CTS traces in the CTS console.
