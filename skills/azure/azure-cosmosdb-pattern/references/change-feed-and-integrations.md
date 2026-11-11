# Change feed / Azure consumers

| Mode | Contract |
|---|---|
| Latest version (default) | Creates/updates, possibly coalesced to latest version; no deletes/TTL-deletion history |
| All versions and deletes | NoSQL-only, requires continuous backup; history bounded by configured continuous retention. Verify selected SDK/mode/region/preview support and compatible consumer before enabling; not every Functions binding/processor has parity. |

Use existing SDK change-feed iterator/processor or Azure Functions binding for
the chosen mode. Lease container ownership/throughput, checkpoint persistence,
scale/rebalance, at-least-once/idempotent processing, per-partition ordering and
retry/poison-message handling are explicit. Don't promise global total ordering
or exactly-once effects. Preserve lease/checkpoint state through deployment and
test restart/rebalance/lag. Consumer data/lease RBAC is distinct from app read RBAC.

Exception: same-account deleted-resource restore creates a **new resource lifetime**.
Invalidate cached session/continuation tokens, restart clients/queries, and restart
feed listeners from the beginning for that restored lifetime. Old leases/checkpoints
are evidence, not valid resumption state; pre-deletion feed history isn't restored.
Idempotent replay/rebuild and business/tombstone/TTL-gap reconciliation are required;
restarting alone doesn't recover missing history. Load the selected same-account
restore guide only when this incident/workflow is actually selected.

For projections/search/embedding pipelines, stable source version/operation IDs
deduplicate and prevent stale overwrites. Latest-version mode needs explicit
tombstones or other deletion reconciliation; a TTL flag followed by eventual TTL
deletion must allow consumers time to observe the marker. All-versions retention
isn't an unlimited audit archive or a replacement for backup/restore.

Sources: [feed modes](https://learn.microsoft.com/azure/cosmos-db/change-feed-modes),
[same-account restore lifetime](https://learn.microsoft.com/azure/cosmos-db/restore-in-account-continuous-backup-introduction),
[processor](https://learn.microsoft.com/azure/cosmos-db/change-feed-processor).
