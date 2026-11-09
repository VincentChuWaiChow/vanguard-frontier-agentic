# Workflow and output contract

## Workflow

1. Confirm the question is about Amazon DocumentDB (with MongoDB compatibility), not a different AWS database. If it is
   about DynamoDB, hand off to `aws-dynamodb-patterns`.
2. Load [the operations reference](operations.md) and use only the sections the
   question needs.
3. Treat each section's status as binding: `UNVERIFIED` or `PARTIAL` parts are
   reported as unconfirmed, never as fact.
4. For anything that would change a live resource, run
   [the safety checklist](safety-checklist.md) first.

## Output contract

1. Verdict: READY / READY WITH RISKS / NOT READY / NEEDS EVIDENCE.
2. Evidence level: documentation only, sanitized user evidence, or approved
   read-only metadata.
3. Findings, each citing an evidence ID or labelled inference, assumption or
   unverified.
4. Safe next actions, each with its approval gate and rollback.
5. Unverified facts and open questions.
