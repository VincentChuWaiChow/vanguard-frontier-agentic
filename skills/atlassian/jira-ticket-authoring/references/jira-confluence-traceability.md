# Jira–Confluence traceability

Keep intent and evidence connected without copying every document into every ticket.

| Artifact | Owns | Link carried in the ticket |
|---|---|---|
| PRD / requirements | Business problem, outcomes, material business rules | Exact section and revision/date |
| Technical design | Interfaces, architecture, implementation constraints | Relevant design and unresolved decisions |
| ADR | Decision, authority, rationale, consequences | Decision status and source evidence |
| Jira ticket | One bounded delivery result and acceptance | AC identifiers and dependencies |
| Repository / tests | Implementation and observed verification | Revision, relevant paths, test output |
| Runbook / release notes | Operational procedure and delivered change | Relevant version and validation |

## Minimum trace table

| Source rule or decision | Source anchor/revision | Ticket criterion | Verification evidence | Documentation impact |
|---|---|---|---|---|
| BR-01 or decision title | Actual source or unknown | AC-01 | NOT RUN until observed | Page/section to update or N/A with reason |

Rule identifiers may be local to the draft; say so. Never fabricate Confluence page IDs, Jira issue keys, repository revisions, or URLs. A reported fact without an accessible source stays reported.

## Link pattern

Use a descriptive ordinary hyperlink in the portable draft. On publication, use the tool's supported link representation and verify it resolves. A Markdown link is not a guarantee of a Smart Link card. A source page can be visible to the author and inaccessible to the delivery agent; record that limitation rather than assuming access.

If publishing both artifacts is authorized, link both ways after creation returns real identifiers. On a partial failure, report which artifact exists and which cross-link remains pending. Do not retry both creates.

## Changes and drift

When a source rule changes, compare its affected ACs and verification evidence. Record the change and obtain the relevant decision when intended behavior is disputed. Preserve accepted decision history; use a new or superseding ADR when needed. Do not rewrite historical rationale to match today's implementation.

For a new finding outside the ticket, capture the observation, evidence, impact confidence, proposed follow-up and relation to the current work. Draft a follow-up issue; publish it only when authorized. This preserves useful discoveries while keeping the active task bounded.
