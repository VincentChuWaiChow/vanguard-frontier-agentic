# Write verification

Mechanics for each write (field contracts, body formats, version handling) live in `../../jira-ticket-authoring/references/jira-integration.md` and `../../confluence-document-authoring/references/publishing.md`. This reference covers only what the guard adds: proof, retry rules, and undo.

## Before the write

Record in the capsule: target locator, current value or page version, the exact intended change, and the idempotency handle (the key or ID the system returns, or a search predicate that proves the item exists).

## Read-after-write

After every write, read the target back and compare to the intended change.

- **Pass:** the read-back matches; record the returned key or ID, version, and time in `verification_evidence`.
- **Mismatch:** report observed versus intended, set `handoff_status` to `failed`, do not retry automatically.
- **Cannot read back** (permission or outage): report `UNVERIFIED`; the write is not confirmed.

`completed` requires `verification_evidence`.

## Timeout or unknown outcome

1. Do not retry yet.
2. Read back: search for the item by the idempotency predicate, or read the page version.
3. Exists as intended: treat as done and record it. Exists differently: report. Absent: retry within `max_retries`, then stop.

A blind retry of a create can produce duplicates and duplicate notifications.

## Compensation plan per write type

| Write | Compensation | Limit |
| --- | --- | --- |
| Edit of an issue or page | Restore the recorded previous value or previous page version | The restore is itself a write and needs authorization; earlier watcher notices remain sent |
| Create | Cannot un-notify. Mark or close per the owner's choice; deletion is prohibited | Owner decides the final state |
| Link | Remove the link | Requires authorization like any write |
| Transition | Transition back only if the workflow allows it | Automation triggered by the first transition may already have run |
| Archive or move (T4) | Reverse move or restore, per owner decision | Requires T4 approval again, two approvers |

## Non-atomic outcomes

Jira and Confluence writes in one request are separate operations with no transaction across them. Examples: the page is published but the Jira link failed; three of five issues were created before approval expired. Report each step as done, failed, or not started, with read-back evidence. Compensation is per step and each compensating write needs authorization. Never present a partly applied set as complete.

## Notifications

Watchers, assignees, mentions, and customer-facing messages fire at write time and cannot be recalled. The go decision must name who is likely to be notified, or state that this is unknown.

## Failure criteria for the guard itself

The guard failed if it issued go without items 2 to 7 of the checklist, accepted a comment as approval, allowed a retry before read-back, or reported completion without read-back evidence.
