# Workflow and output contract (repository adapter)

This file exists so this repository's AWS skill contract can find the workflow and
output rules in a fixed place. It adds no new procedure. The skill keeps
the substance in route-specific files; load only the one the request selects.

## Which workflow to load

- One command (`design`, `refactor`, `backup`, `restore`, `audit`,
  `performance-test`, `feature-plan`, `select`, `help`): load that command's guide
  from the command table in `SKILL.md`, then only the method or variant it names.
- A broad design or implementation task with no command: load
  [the end-to-end workflow](workflow.md), then only the domain files it selects.
- A focused feature question: load the single task-route file from `SKILL.md`.

Never chain commands automatically, and never preload siblings, examples or evals.

## Output contract

Return proportionately; a small bug fix does not need every heading.

1. Selected route and scope, including what was deliberately not loaded.
2. Evidence level: repository-local, sanitized user evidence, approved live
   metadata, or documentation only. Cite rows from
   [the evidence ledger](evidence-ledger.md) for material AWS claims, or label the
   statement `inference`, `assumption` or `unverified`.
3. Findings or design, with alternatives that were rejected and why.
4. Proposed code, IaC or CLI as changes for the user to apply, each with its
   approval gate, blast radius and rollback.
5. PASS / MISS / UNVERIFIED per checked item. Never claim a design is bulletproof
   from examples or local tests.

The full response contract for broad work is the "Response contract" section of
[the end-to-end workflow](workflow.md).
