# Workflow and output contract

## Workflow

1. Confirm the question is about OVHcloud Public Cloud Databases for MongoDB, not another database engine or a
   self-managed MongoDB deployment.
2. Load [the operations reference](operations.md) and use only the sections the
   question needs.
3. Treat each section's status as binding: `UNVERIFIED` or `PARTIAL` parts are
   reported as unconfirmed, never as fact.
4. For anything that would change a live resource, run
   [the safety checklist](safety-checklist.md) first.

## Output contract

1. Verdict: READY / READY WITH RISKS / NOT READY / NEEDS EVIDENCE.
2. Evidence level: documentation only, sanitized user evidence, or approved
   read-only metadata from the OVHcloud API or Control Panel.
3. Findings, each citing an evidence ID or labelled inference, assumption or
   unverified.
4. Safe next actions, each with its approval gate and rollback.
5. Unverified facts and open questions.
