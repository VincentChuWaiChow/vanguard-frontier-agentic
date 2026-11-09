# June / December review — in-session reminder only

Run `python3 scripts/review_due.py --as-of YYYY-MM-DD` with the actual client-local
date at substantive use, not help. It reads assets/content-review.json and prints
due status; never writes, schedules, advances dates or contacts any provider.
Initial next full review is **2026-12-01**. After that date it remains overdue even
if June/December windows pass. Invalid/missing dates fail, not default to server UTC.

Full review checklist: scope/API/SDK support, runtime/partition/transaction limits,
index/vector constraints/lifecycle, data/control roles/CMK identities and recovery,
backup tiers/modes/windows/CLI/ARM schema, metric deprecations, all selected routes,
provisioning equivalence, negative/fakeCLI/runtime tests and adversarial prompts.
Refresh primary docs via Context7 and MicrosoftDocs/Azure documentation tools;
don't inspect accounts, install integrations or run examples to check freshness.

A targeted fix/source refresh is **not a full audit** and does not clear the due
date. Only an explicitly completed full review with dated evidence and approval
may update the ledger and advance to the next June 1 / December 1. No helper
performs that update. Reminder isn't a calendar/external notification or memory.
