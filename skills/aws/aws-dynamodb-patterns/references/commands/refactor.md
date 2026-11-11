# refactor

Improve an existing DynamoDB model, repository access layer, table configuration
or item-update path. Read this command and selected feature references only.
Default to local code/templates/sanitized snapshots; no implicit live discovery,
backfill, feature update or deployment. The selector emits paths, not a migration.

```text
/aws-dynamodb-patterns refactor --features core,indexes
```

```bash
python3 scripts/select_mode.py refactor --features core,indexes
```

Provide existing schema/IaC, application read/write paths and the desired change in
the surrounding request. Unknown current state blocks an execution-ready delta,
not a clearly labeled proposal. Live evidence may be obtained through a separately
scoped authorized audit; that audit is not an automatic prerequisite or side effect.

## Classify the update before writing a patch

| Change | Correct treatment |
|---|---|
| SDK/repository/query refactor | Preserve persisted data and API/cursor/consistency/error contracts; add characterization and negative tests before surgical code changes |
| Ordinary item attribute/update expression | Generate conditional/versioned UpdateItem or a justified transaction; key attributes are not mutable fields; distinguish missing vs NULL, partial batches and ambiguous outcomes |
| Capacity/encryption/TTL/PITR/Streams settings | Use the actual service operation and current IaC owner; inspect permissions, transitions and retention/consumer/key dependencies; feature-plan may generate a separate reviewed delta |
| GSI addition/replacement | Add the new path and inspect limits/backfill/throttling/readiness; service GSI backfill indexes existing qualifying attributes, not newly invented application-derived keys |
| Base table key schema (attribute names/types) or new LSI | Treat as new-table/data migration, not an in-place UpdateTable key edit; preserve the old table/read path until verified cutover |
| Vector schema/dimensions or embedding model | New index/generation and possibly re-embedding; equal dimensions do not align models; evaluate authorized recall/freshness before switching |

UpdateItem can create a missing item: require attribute_exists/version conditions
when the intent is update-only. Moving an item's primary-key value needs a new
identity plus deliberate retirement of the old one, not an UpdateExpression.
Changing only key-value encoding may use a same-table migration if the table key
schema stays unchanged; define old/new identity/version coexistence explicitly.

Do not trust a CloudFormation summary's interruption label as proof that a key or
LSI layout can be edited in place. Inspect the actual change set and service
constraints. Replacement of a custom-named table needs a distinct name; never
delete the original to make that name available. CloudFormation completion does
not establish that an online-added GSI is ACTIVE and ready for application reads.

## Preserve data through the change

| Phase | Gate / evidence |
|---|---|
| Baseline | Current owner/schema/indexes, access-pattern matrix, code/SDK versions, data shape/volume/skew, cursors and observed invariants; distinguish snapshot from live evidence |
| Compatibility | Old/new reader and writer contracts, additive attributes/versioning, canonical source of truth, rollback points and measurable acceptance criteria |
| Proposed delta | Small code/IaC diff plus exact generated commands; preview/change-set evidence, replacement/cost analysis and separately approved target/actions |
| Synchronization | Application-derived keys need bounded, checkpointed, idempotent backfill; protect newer writes with conditions/versions and reconcile races; define duplicate/out-of-order/delete handling |
| Write continuity | Explicitly choose and review write quiescence or a synchronization/dual-write strategy; independent writes are not atomic, so handle partial success/unknown outcomes and reconciliation |
| Verification | Compare authorized shadow/parity reads and representative samples; counts alone are insufficient; check expected non-indexed items/type violations, index readiness, lag and consumer behavior |
| Cutover | Stage readers/writers with explicit routing/feature flags, cursor/schema compatibility, observability and stop thresholds; state when rollback requires reverse synchronization rather than a flag flip |
| Retirement | Separate approval after retention/recovery/runtime evidence; no automatic table/index/item/key deletion, reset, destructive cleanup or old-path removal |

Create-if-absent alone cannot prevent backfill from resurrecting a deleted item.
Account for tombstones/authoritative versions, deletes and restart/replay horizons
before publishing the new representation.

Test stale versions, timeout-after-commit, retries/partial batches, concurrent
updates/deletes during backfill, missing/invalid index keys, expired sessions,
wrong-tenant/cursor reuse, index lag and cutover/rollback failures. A fixture load
or matching row count is not a proof of complete compatibility or migration safety.
Retain recovery posture and verify restore procedures separately; backups alone
do not reverse a cutover with new writes.

Return the classified change, baseline evidence, smallest patch/generated commands,
compatibility/backfill/cutover/rollback plan and criterion-level tests/unknowns.
For code-only changes, do not invent a migration. For data/model changes, do not
hide required migration work behind a successful compile. No patch or planning
command authorizes applying it to AWS.

Load [runtime/migration guidance](../runtime-contract.md), [indexes](../lsi-vs-gsi.md)
or [provisioning](../provisioning.md) only for the selected operation. Refresh current
service/CFN/SDK behavior through primary docs/Context7 before generating a real delta.

Primary contracts checked 2026-11-08:
[UpdateItem](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_UpdateItem.html),
[LSI lifecycle](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/LCICli.html),
[online GSI operations](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/GSI.OnlineOps.html),
[CloudFormation table constraints](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-dynamodb-table.md).
