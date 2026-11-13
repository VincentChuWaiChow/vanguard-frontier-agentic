# Audience and product/support patterns

Choose the reader's task before choosing detail. This is a writing and routing convention, not a new product taxonomy or a guarantee about any system.

| Audience / purpose | Lead with | Keep accessible later |
|---|---|---|
| Business reader understanding a product | Problem, users, workflow, intended benefit, limits | Plain-language terms and optional technical appendix |
| Product team deciding a feature | Evidence of need, alternatives, behavior, measurable outcome | Feasibility, dependencies and design decisions |
| Customer learning a feature | What changed, who has access, how to use it, limitations | Troubleshooting and support route |
| Support diagnosing a report | Symptom, affected scope/version, safe checks, next branch | Sanitized diagnostics, known issue and escalation |
| Engineer implementing | Business rules, interfaces, invariants, verification | Full design and decision evidence |
| Operator restoring service | Current impact, exact target, safe observations, procedure | Recovery, escalation, incident history |

## Explain a product in business terms

Use this order: what it helps someone do; who uses it; the problem today; the intended improved workflow; a concrete example; capabilities and limitations; prerequisites or commercial facts only when sourced; relevant terms; support/next step. Label desired benefits as intended until measurements establish them.

Translate jargon by preserving the constraint that affects the user:

| Technical wording | Business-oriented explanation pattern | Constraint to retain |
|---|---|---|
| Asynchronous processing | “The request is accepted first; the work finishes later.” | How users observe completion/failure must come from the contract |
| Idempotency | “Retrying the same identified request should not repeat its business effect.” | Identity, scope, retention and supported operations must be specified |
| Read-only access | “You can view information but cannot change it through this access.” | Actual allowed data and actions require verified permissions |
| Recovery target | “The service aims to restore the agreed work within the stated target.” | A design target is not measured recovery or a contractual guarantee |

Use examples as illustrations, not proof of capabilities. Do not add pricing, time savings, reliability promises, ROI, integration support or compliance claims because they sound persuasive. If a term is uncertain, ask or mark it unknown.

## New feature: proposal, design, enablement and release

A **feature request** asks whether to invest. A **PRD** establishes intended outcome and rules. A **TDD** explains implementation. A **feature overview/how-to** teaches the user. **Release notes** describe verified delivery and impact. Link these artifacts where useful; do not require all of them for a tiny feature.

State the exact known availability: proposed, planned, under development, deployed to a named environment, or enabled for a specified group. These are descriptions, not assumed workflow statuses. A merged change, release tag, or completed Jira issue alone does not establish customer availability. Avoid invented dates or roadmap commitments.

## Support knowledge base

Separate the report, confirmed behavior, hypothesis, workaround and permanent resolution. Provide checks with expected observations and branches. Identify who may perform a step. Mark untested steps unverified and exclude unsafe copy-paste commands with unresolved targets.

Write FAQs from known questions and answers, keeping unknown answers explicit. Keep the customer article free of private logs, credentials, internal-only URLs and implementation discussion that cannot help the reader decide or act. If internal notes are necessary, keep them in a separately restricted artifact or verified restricted location; an “Internal” heading does not enforce access control.

## Cross-artifact example

A user reports an export timeout. Start with a support case. If evidence shows a defect, link a bug. A live wider outage can also need an incident. Record a validated workaround in a KB article, the permanent delivered change in release notes, and material causes/prevention in a postmortem. Each artifact serves its reader and preserves links to the relevant evidence.
