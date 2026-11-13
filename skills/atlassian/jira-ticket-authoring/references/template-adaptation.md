# Adapt a ticket template without inventing policy

Read this only when selecting or changing a reusable template, importing a team's completion checklist, or reviewing a template-driven ticket.

## Keep three contracts separate

| Contract | Establishes | Does not establish |
| --- | --- | --- |
| Ticket acceptance criteria | The specific behavior or outcome to verify | Team-wide process requirements or observed success |
| Existing Definition of Done | Applicable completion checks from the team's policy and revision | Permission to deploy, or a replacement for acceptance evidence |
| Jira field/workflow configuration | The actual target project's accepted fields, types, and transitions | Semantic readiness or the truth of a completion claim |

If no policy is supplied, record that limit rather than importing generic review, QA, or production-deployment requirements. A known required check with missing policy/evidence remains unresolved. Assess its consequence for the next activity; initial intake can remain ready for triage.

## Adapt the selected content pattern

1. Choose the smallest pattern matching the requested outcome. Use a story for one user-visible outcome; use an epic to coordinate independently scoped children.
2. Retain source anchors, exclusions, stable acceptance IDs, and the distinction between planned checks and actual evidence. Template rows do not impose a minimum number of criteria.
3. Map field requirements only from the actual site/project contract when a Jira write is requested. An unassigned owner, missing sprint, or absent label is not automatically an authoring defect.
4. Treat sample names, dates, issue keys, priorities, and thresholds as examples, never defaults about the user's work. Local child labels remain planning references until Jira returns real identifiers.
5. Keep team policy checks beside their source/revision and evidence state. Do not mark a checkbox complete from the existence of a document, command, or proposed test.

## Change a shared template

Compare the old and proposed structure before rollout. Exercise a small realistic ticket and a sparse/blocked case; confirm that existing acceptance IDs and recorded evidence survive. Identify affected saved templates or issue descriptions and offer a migration diff, not a bulk overwrite.

Existing tickets are historical work records. Do not rewrite them, change workflow status, add fields, or replace a shared template without the corresponding request and resolved target. Version/review the reusable content, explain compatibility changes, and preserve earlier release history.

Structural lint may flag omissions, but a score, checkbox count, or populated assignee/priority/sprint cannot establish behavioral correctness, execution, or readiness. Apply the [evidence-based review](quality-review.md).
