# Cohort matrix worksheet

Records, per cohort, which controls it needs, what evidence exists for each operation, the canary window and the abort and rollback limits, so a migration decision is made cohort by cohort. The migration lead fills it; the independent challenger fills the challenge column. Use it with the offline test harness and cohort rollout references.

## How to use

- Fill from supplied evidence only; mark `Unknown` with the smallest item that would settle it.
- For UPDATE record offline and real-pair evidence separately; an offline UPDATE row alone is not parity for a control that compares `oldObject`.
- Never paste secrets, tokens or customer identifiers; keep the decision ID and name cohorts by role or namespace.
- State the abort criterion before the canary starts, with its owner.
- Rollback limits say what a rollback does not restore.

## Cohorts

| Cohort | Required controls | CREATE | UPDATE (offline / real pairs) | DELETE | Exceptions | Canary window | Challenge result | Decision | Abort criterion | Rollback limit |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `<namespace or role group>` | `<control IDs>` | `<offline result>` | `<offline result / real-pair result>` | `<result or not applicable>` | `<exception IDs and expiry>` | `<start, end, owner>` | `<challenger and finding>` | `<supported / conditional / blocked / indeterminate>` | `<signal, threshold, owner>` | `<what rollback does not restore>` |
| `payments` invoice services | `example-require-run-as-non-root` | pass / fail as expected, 2 rows | Offline pass / none yet | Not applicable: control reads `object` | None | Unknown: owner to set | Pending: challenger not yet assigned | `conditional` | Unexpected denial of `legitimate-workload` pattern; threshold set by service owner | Pods created or blocked during the canary are not undone |

## Evidence status

| Item | Type | Locator | Captured at | Omits |
| --- | --- | --- | --- | --- |
| `<item>` | `<type>` | `<locator>` | `<timestamp>` | `<what it cannot show>` |
| parity-corpus-starter output | `offline-test` | `<run ID>` | `<timestamp>` | Admission effect, webhook, populations |
