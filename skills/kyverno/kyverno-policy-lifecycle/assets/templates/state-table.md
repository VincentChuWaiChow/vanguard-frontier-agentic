# State table worksheet

Records, per object, what a lifecycle policy changed and what survives a Git revert, so a rollback claim is judged on state and not on intent. The policy owner fills it; the reviewer checks each row's evidence. Use it with the reversal and restoration reference.

## How to use

- Fill from supplied evidence only; mark `Unknown` and name the smallest item that would settle it.
- Fill "After Git revert" only from cited behavior or a `controlled-test`; otherwise write `unverified`.
- Keep a reversal plan (policy and reconciler) apart from a restoration plan (objects and fields).
- Never paste secrets, object contents or customer identifiers; keep the decision ID.
- A second-run result records whether a repeat run changed the object again.

## Object state

| Object | Owner / reconciler | Before change | After change | After Git revert | After restoration plan | Evidence | Second-run result |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `<kind/namespace/name>` | `<team or controller>` | `<state>` | `<state>` | `<state or unverified>` | `<state or none>` | `<evidence ID>` | `<changed again or unchanged>` |
| ConfigMap `payments/invoice-settings` | Payments platform team | No `cost-center` label | Label `cost-center: payments` | Unverified: persistence assumed until downstream state is shown | None needed if the label is acceptable | EV-1 (offline-test), EV-2 pending | Unchanged: the matchCondition skips labelled objects |
| ConfigMap `tenant-a/invoice-limits` | Kyverno generator (GitOps reconciler unknown) | Absent | Present with `maxRetries: "3"` | Retained, because orphaning is enabled; confirm by inspection | None | EV-3 (offline-test) | Unknown: needs a controlled-test |

## Evidence status

| Item | Type | Locator | Captured at | Omits |
| --- | --- | --- | --- | --- |
| `<item>` | `<type>` | `<locator>` | `<timestamp>` | `<what it cannot show>` |
| Mutating kit test output | `offline-test` | `<run ID>` | `<timestamp>` | Background application and reconciler behavior |
