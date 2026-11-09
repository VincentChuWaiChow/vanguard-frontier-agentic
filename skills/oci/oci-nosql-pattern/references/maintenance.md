# June / December content review

Next review starts **2026-12-01**. Run the read-only helper with the actual
client-local date: `python3 scripts/review_due.py --as-of YYYY-MM-DD`.
It never advances [ledger](../assets/content-review.json), writes files, creates
cron/calendar/memory tasks or sends notifications. Missed windows stay overdue,
even across years. A targeted source change is not a complete audit.

Full review: verify current NDCS(Cloud, not SQL/on-prem) sources; keys/transactions/
consistency/TTL; native vector boundary and model/store constraints; IAM/CMEK/hosted
eligibility; CLI/provider schema; actual customer recovery capabilities/limits;
SDK languages/signatures; metrics and runtime guards; routes/links/evals and human
trigger tables. Record lookup failures/uncertainties explicitly. Only after a
completed documented full review may an explicitly authorized maintainer update
last_full_review and choose the next June/December 1 ledger date. No automatic roll.
