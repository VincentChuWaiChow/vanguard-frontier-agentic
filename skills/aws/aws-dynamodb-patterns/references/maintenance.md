# Semiannual content review reminder

Review this skill **every June and December**, starting on the first of each
month. Windows are six months apart, aligned to the calendar—not six months after
an arbitrary edit. The initial next window is **1 December 2026**. A missed window
stays overdue until a real full review is completed and recorded.

## Local reminder, not a background schedule

At the start of substantive skill use, run the read-only helper using the user's
current local/client date. For example, from this skill directory:

```bash
python3 scripts/review_due.py --as-of 2026-11-08
```

Replace that example date with the actual date. Output is not_due before the
window, due throughout that June/December, and overdue after an uncompleted
window. In June 2027, an uncompleted December 2026 review is **still overdue**;
merely loading the skill never advances the ledger.

If due/overdue, remind the user and refresh the time-sensitive claims needed for
their current task before relying on them. Propose the broader full review; do
not turn an unrelated bug fix into an unapproved portfolio rewrite.

The helper does not write files, send notifications, register crons/calendar
events, access AWS or schedule an agent. An external notification requires a
separately requested, available scheduling facility and explicit target/time.

## Full review checklist

1. Use exposed AWS documentation MCP and Context7 plus current AWS release notes.
   Record verification dates and primary URLs; distinguish service availability
   from installed SDK/CLI/CloudFormation support.
2. Re-check data types, native vector/index APIs, dimensions/quotas, capacity,
   consistency, filters and unsupported features. Do not assume yesterday's
   “no native vector support” or search-as-new-primitive claim is still correct.
3. Re-check IAM action/resource/condition mappings, resource policies, cross-account
   access, table/stream tagging and ABAC, KMS ownership and backup dependencies.
4. Re-check Streams, DAX clients/cache semantics, PITR/restore, MREC/MRSC/replica
   settings and Region support, CloudWatch dimensions/statistics and alarm behavior.
5. Reconcile conflicting sources explicitly. Validate all affected examples and
   regression/negative cases; inspect diffs against the approved maintenance scope.
   Do not convert simulated calls into live service proof or silently install tools.
6. Update version/changelog and sources for actual content changes. After the full
   audit passes, record last_full_review and the next future June/December first
   in [content-review.json](../assets/content-review.json). For a full review
   completed on 15 December 2026, next_review_due becomes 2027-06-01; one completed
   on 30 June 2027 produces 2027-12-01. A late January review also next targets June.

The initial last_full_review is null deliberately: a targeted capability update
is not proof that every old topic was re-audited. last_targeted_review records
focused work only and does not reset the semiannual deadline. Trigger a targeted
refresh immediately for a relevant AWS announcement/API mismatch or source conflict;
do not wait six months to correct a known defect. Do not update the ledger on an
unrun, partial or failed review.
