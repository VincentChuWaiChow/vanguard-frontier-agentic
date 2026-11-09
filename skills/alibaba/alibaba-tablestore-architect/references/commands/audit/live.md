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
aliyun sts get-caller-identity > "$audit_dir/identity.json" # [TS-E901]
jq -e --arg a "$TS_EXPECTED_ACCOUNT_ID" '.AccountId == $a' "$audit_dir/identity.json"
```

Commands use the plugin-based CLI (3.3.0 or later), which names commands and flags
in kebab-case derived from the API action and parameter names [TS-E913]. Flags below
follow that rule from the cited API pages; confirm each with
`aliyun <product> <command> --help` before running. Run every command with the CLI
profile whose Region equals `$TS_REGION`; stop if it differs.

## Backup and restore

```bash
: "${TS_REGION:?Approved Region}" "${audit_dir:?Private temp dir}"
aliyun hbr describe-vaults --vault-type OTS_BACKUP > "$audit_dir/hbr-vaults.json" # [TS-E904] [TS-E905]
aliyun hbr describe-backup-plans > "$audit_dir/hbr-plans.json" # [TS-E904]
```

## Disaster recovery and replication

```bash
: "${TS_INSTANCE:?Approved instance name}" "${audit_dir:?Private temp dir}"
aliyun tablestore get-instance --instance-name "$TS_INSTANCE" > "$audit_dir/instance.json" # [TS-E902] [TS-E903]
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
: "${audit_dir:?Private temp dir}" "${TS_INSTANCE:?Approved instance name}"
aliyun cms describe-metric-rule-list --namespace acs_ots_new > "$audit_dir/cms-rules.json" # [TS-E908] [TS-E910]
aliyun cms describe-metric-list --namespace acs_ots_new --metric-name InstanceCU \
  --dimensions "[{\"instanceName\":\"$TS_INSTANCE\"}]" > "$audit_dir/cms-instancecu.json" # [TS-E908] [TS-E909]
```

## Performance

**No live command:** per-partition throughput and hotspots are not exposed by a
documented control-plane command (searched the Tablestore CLI command list [TS-E902]).
Use the CloudMonitor output from the Monitoring section and client error logs.

## IAM

```bash
: "${TS_INSTANCE:?Approved instance name}" "${audit_dir:?Private temp dir}"
jq '{Policy, PolicyVersion}' "$audit_dir/instance.json" # [TS-E903]
: "${RAM_POLICY_NAME:?Approved custom RAM policy name}"
aliyun ram get-policy --policy-type Custom --policy-name "$RAM_POLICY_NAME" > "$audit_dir/ram-policy.json" # [TS-E911] [TS-E912]
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
: "${TS_INSTANCE:?Approved instance name}" "${audit_dir:?Private temp dir}"
jq '{Network, NetworkTypeACL, NetworkSourceACL}' "$audit_dir/instance.json" # [TS-E903]
aliyun tablestore list-vpc-info-by-instance --instance-name "$TS_INSTANCE" > "$audit_dir/vpc.json" # [TS-E902]
```

## Audit logging

```bash
: "${audit_dir:?Private temp dir}" "${TS_START:?Start time, UTC ISO 8601}" "${TS_END:?End time, UTC ISO 8601}"
aliyun actiontrail describe-trails > "$audit_dir/trails.json" # [TS-E906]
aliyun actiontrail lookup-events --start-time "$TS_START" --end-time "$TS_END" --max-results 20 > "$audit_dir/events.json" # [TS-E906] [TS-E907]
```
