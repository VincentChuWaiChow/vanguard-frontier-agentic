# Selective feature change plans

```text
/aws-dynamodb-patterns feature-plan --features pitr,streams --action enable
```

```bash
python3 scripts/select_mode.py feature-plan --features encryption --action configure
```

Read this command only; audit and performance siblings are not prerequisites.
Split incompatible feature/actions into separate plans; never apply a partial set.

Load only for feature-plan mode. The selector records **intent**, not AWS state.
Use only the selected feature guides; their existing CloudFormation/CLI examples
remain lazy. Do not add another deployment system or silently adopt resources.

## Required plan

1. Record requested feature/action and exact target/account/role/Region/table/index,
   owner and existing configuration. Prefer repo evidence; live inspection requires
   approved metadata scope. Unknown actual state blocks an execution-ready delta.
2. Explain the workload need, supported API/IaC change and prerequisites. Show the
   smallest desired-state diff and generated commands **without executing them**.
   A mode option is not an AWS API or proof a boolean toggle exists.
3. Check item/data preservation, index/replica dependencies, key/backup availability,
   consumer disruption, asynchronous transitions and cost. Preserve unrelated state.
4. Define preview/change-set evidence through the existing owner, rollback limits,
   completion/readback checks and application-level verification. A successful
   update request is not readiness, correctness or recovery proof.
5. Obtain exact change authorization before execution. Reconfirm current state and
   stop on drift. Use an existing authorized execution route; never auto-apply or
   run destructive cleanup from this planning mode.

## Do not treat every feature as on/off

- TTL, PITR and Streams accept enable/disable/configure **planning intents**. Still
  verify service-specific transition rules. Disabling changes retention/change
  capture behavior; it is not cost-free rollback. TTL is not a precise expiry gate.
- Encryption configuration selects a key owner/key, not an unencrypted table.
  Key transitions, grants and old-backup dependencies require the encryption guide.
- Enabling indexes/vectors means proposing a new/index lifecycle operation, not
  mutating immutable dimensions/schema. LSI creation is a new-table decision.
- DAX/replicas require topology, ownership and dependency plans, not a simple flag.
  Generic disable is intentionally rejected for these and index/vector removal;
  use a separately scoped removal review with preservation/rollback evidence.
- IAM, tags/resource policies and embedding/model changes use configure. Model
  migration needs re-embedding, isolated generations and measured cutover.

Split incompatible feature/action combinations into separate plans. Never execute
the supported subset of a rejected combination. An approved plan still does not
authorize a different account, table, new spend or irreversible operation.

Return the requested versus observed state, proposed diff/commands, dependencies,
cost/rollback uncertainty, verification and exact pending approval. Use UNVERIFIED
for missing live or runtime evidence; do not report an enabled feature from intent.
