# Recovery target contract

Record tenant/subscription, source full resource ID **and restorable instance ID**
(names can be reused), NoSQL API, database/container IDs/RIDs, region existence,
policy/tier, resource creation/deletion times, incident time and observed latest
restorable timestamp. Compute bounds dynamically: selected tier AND resource/
region existence AND observed backup readiness, not `now minus 30 days` alone.

Require UTC timezone-aware restore timestamp inside those bounds; source and
destination in the supported same subscription. Bind exact new account name and
existing destination resource group for new-account methods. Verify absence by
authorized lookup: **only a classified ResourceNotFound means absent**. 403,
timeout, CLI failure or wrong subscription must never trigger create/restore.

Deleted-resource mode instead binds source live account, specific Delete event
and original DB/container name, verifies it is currently absent and timestamp
precedes deletion. Never overwrite/reuse a currently existing target. Name-race
or drift is a hard stop, not delete/retry.

Confirm caller's restore/enumeration/control permissions separately from data
roles; preserve source key versions, Key Vault access/default identity and restore
identity continuity. Fix TTL treatment explicitly to avoid deleting recovered
data immediately. No automatic network opening, role broadening or key cleanup.

After async completion independently read back source/time/destination/createMode,
API, region, indexes/partition/TTL, consistency/throughput and key identity. Restore
network/private endpoints/data RBAC/regions/server-side functions as required;
validate application reads/writes/tenant denials, counts and domain invariants,
replay/deduplication, dependencies and measured RPO/RTO. Only then propose cutover;
retain source/rollback until approved. No deletion in this workflow.
