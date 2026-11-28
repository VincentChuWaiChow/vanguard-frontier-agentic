# Referral worksheet

Records each referral from a lead agent to a specialist so the specialist receives only what its one question needs, duplicates collapse and no referral loops back. The lead fills it; the challenger and the board secretary read it. The decision itself is recorded in the decision-record template (`assets/templates/decision-record.template.json`), which passes `scripts/validate_board_decision.py` as an `indeterminate` skeleton; this worksheet feeds that record's referrals and does not replace it.

## How to use

- One referral, one return question; send scoped evidence IDs, never whole manifests or tickets.
- Write why the answer could change the result; if it could not, do not refer.
- Mark an identical question as a duplicate and point to the first referral; share its answer under both decision IDs.
- Never refer back to the lead that sent the referral; keep sensitive content out and keep the decision ID.
- Status values: open, answered, duplicate, withdrawn.

## Referrals

| Decision ID | From (lead) | To | Scoped evidence IDs | One return question | Why the answer could change the result | Status | Answer | Duplicate-of |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `<KYV-DEC-nnnn>` | `<lead agent>` | `<specialist agent>` | `<EV-n, EV-m>` | `<one question>` | `<how the answer moves the outcome>` | `<status>` | `<answer or pending>` | `<referral ID or none>` |
| `KYV-DEC-0003` | `kyverno-migration-readiness-agent` | `kyverno-admission-reliability-agent` | `EV-4, EV-5` | What is the p99 admission latency for Pod CREATE in `payments` during the canary window? | A p99 above the owner's budget moves the cohort from `conditional` to `blocked` | open | Pending | none |

## Evidence status

| Item | Type | Locator | Captured at | Omits |
| --- | --- | --- | --- | --- |
| `<item>` | `<type>` | `<locator>` | `<timestamp>` | `<what it cannot show>` |
| Admission latency export | `runtime-metric` | `<export ID>` | `<timestamp>` | Registry latency under degradation |
