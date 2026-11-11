# audit --mode live

Commands are examples for an operator holding read-only authorization; the skill
does not run them. Expectations come from [the operations reference](../../operations.md).
Only control-plane metadata is read; no table data.

## Before any command

- Confirm the approved principal, account, Region and exact Tablestore instance.
  Stop on any mismatch; never switch target to make a command work.
- Use a read-only RAM role. Run `umask 077` and write output to a private temp
  directory. Never print AccessKeys, tokens or customer data.

```bash
set -euo pipefail
: "${TS_EXPECTED_ACCOUNT_ID:?Approved account ID}" "${TS_REGION:?Approved Region}" \
  "${TS_INSTANCE:?Approved instance name}"
umask 077
audit_dir=$(mktemp -d)
aliyun sts get-caller-identity --region "$TS_REGION" > "$audit_dir/identity.json" # [TS-E901] [TS-E914]
jq -e --arg a "$TS_EXPECTED_ACCOUNT_ID" '.AccountId == $a' "$audit_dir/identity.json"
```

Commands use the plugin-based CLI (3.3.0 or later), which names commands and flags
in kebab-case derived from the API action and parameter names [TS-E913]. Flags below
follow that rule from the cited API pages; confirm each with
`aliyun <product> <command> --help` before running. Every command passes the global
`--region "$TS_REGION"` option, which overrides the profile Region [TS-E914]; the
instance read below also stops unless the instance's `RegionId` equals `$TS_REGION`.

## Backup and restore

```bash
: "${TS_REGION:?Approved Region}" "${audit_dir:?Private temp dir}"
aliyun hbr describe-vaults --region "$TS_REGION" --vault-type OTS_BACKUP > "$audit_dir/hbr-vaults.json" # [TS-E904] [TS-E905] [TS-E914]
aliyun hbr describe-backup-plans --region "$TS_REGION" > "$audit_dir/hbr-plans.json" # [TS-E904] [TS-E914]
```

## Disaster recovery and replication

```bash
: "${TS_INSTANCE:?Approved instance name}" "${audit_dir:?Private temp dir}" "${TS_REGION:?Approved Region}"
aliyun tablestore get-instance --region "$TS_REGION" --instance-name "$TS_INSTANCE" > "$audit_dir/instance.json" # [TS-E902] [TS-E903] [TS-E914]
jq -e --arg r "$TS_REGION" '.RegionId == $r' "$audit_dir/instance.json" >/dev/null \
  || { echo "Instance Region mismatch; stop" >&2; exit 1; } # RegionId [TS-E903]
jq '{InstanceName, RegionId, IsMultiAZ}' "$audit_dir/instance.json"
```

Global Table configuration: **No live command:** the Tablestore CLI command list
([TS-E902]) was searched and shows no Global Table read command; check it in the console.

## Cost

**No live command:** billing data lives in the billing console and BSS API, not the
Tablestore resource API; no cited read-only billing command was found. Use
`InstanceSpecification` and `PaymentType` from the instance output above as inputs.

## Monitoring

```bash
: "${audit_dir:?Private temp dir}" "${TS_INSTANCE:?Approved instance name}" "${TS_REGION:?Approved Region}"
aliyun cms describe-metric-rule-list --region "$TS_REGION" --namespace acs_ots_new > "$audit_dir/cms-rules.json" # [TS-E908] [TS-E910] [TS-E914]
aliyun cms describe-metric-list --region "$TS_REGION" --namespace acs_ots_new --metric-name InstanceCU \
  --dimensions "[{\"instanceName\":\"$TS_INSTANCE\"}]" > "$audit_dir/cms-instancecu.json" # [TS-E908] [TS-E909] [TS-E914]
```

## Performance

**No live command:** per-partition throughput and hotspots are not exposed by a
documented control-plane command (searched the Tablestore CLI command list [TS-E902]).
Use the CloudMonitor output from the Monitoring section and client error logs.

## IAM

```bash
: "${TS_INSTANCE:?Approved instance name}" "${audit_dir:?Private temp dir}"
jq '{Policy, PolicyVersion}' "$audit_dir/instance.json" # [TS-E903]
: "${RAM_POLICY_NAME:?Approved custom RAM policy name}" "${TS_REGION:?Approved Region}"
aliyun ram get-policy --region "$TS_REGION" --policy-type Custom --policy-name "$RAM_POLICY_NAME" > "$audit_dir/ram-policy.json" # [TS-E911] [TS-E912] [TS-E914]
```

## Tagging

```bash
: "${audit_dir:?Private temp dir}"
jq '{Tags, ResourceGroupId}' "$audit_dir/instance.json" # [TS-E903]
```

## Encryption

**No live command:** encryption at rest is a per-table setting returned by the
data-plane table description, not by a control-plane command; none appears in the
Tablestore CLI command list [TS-E902]. Use the offline audit of table definitions.

## Network isolation

```bash
: "${TS_INSTANCE:?Approved instance name}" "${audit_dir:?Private temp dir}" "${TS_REGION:?Approved Region}"
jq '{Network, NetworkTypeACL, NetworkSourceACL}' "$audit_dir/instance.json" # [TS-E903]
aliyun tablestore list-vpc-info-by-instance --region "$TS_REGION" --instance-name "$TS_INSTANCE" > "$audit_dir/vpc.json" # [TS-E902] [TS-E914]
```

## Audit logging

```bash
: "${audit_dir:?Private temp dir}" "${TS_START:?Start time, UTC ISO 8601}" "${TS_END:?End time, UTC ISO 8601}" \
  "${TS_REGION:?Approved Region}"
aliyun actiontrail describe-trails --region "$TS_REGION" > "$audit_dir/trails.json" # [TS-E906] [TS-E914]
aliyun actiontrail lookup-events --region "$TS_REGION" --start-time "$TS_START" --end-time "$TS_END" --max-results 20 > "$audit_dir/events.json" # [TS-E906] [TS-E907] [TS-E914]
```
