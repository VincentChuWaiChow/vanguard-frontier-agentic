# Document quality review

Review the selected document's usefulness, not the number of filled headings. Empty template markers, invented precision, and unsupported approval/deployment claims are blockers.

## Common checks

- The reader, purpose, scope, owner or unresolved ownership, relevant status, and source context are clear.
- Material claims have a source locator/version or are explicitly reported, assumed, proposed, or unknown.
- Dates, owners, numbers, interfaces, commands, links, and completion claims match available evidence.
- Source disagreements are visible and their practical consequence is explained.
- Instructions and `{{PLACEHOLDERS}}` are removed from the filled result; unresolved facts remain explicit.
- The chosen artifact answers the user's need without unnecessary sections.
- Jira links, requirement IDs, decision references, and evidence mappings are meaningful.
- A portable draft does not claim native rendering, publication, installation, or bidirectional links without verification.
- Customer and business explanations lead with the user's task, impact, and verified availability; expected benefits are distinguished from measured outcomes. Technical detail is kept where its audience needs it.

## Type-specific review

| Type | Decisive question |
| --- | --- |
| PRD | Do user needs and business rules trace to specific acceptance criteria and observable outcomes without invented external contracts? |
| TDD | Can a reviewer assess the solution, alternatives, interface/data implications, delivery/testing, and recovery limits? |
| ADR | Is one actual decision separated from proposals, with rationale, consequences, authority/evidence, and lifecycle links? |
| API | Can a consumer use the documented contract and handle specified errors without invented endpoints or guarantees? |
| Product explanation / feature overview | Can the intended reader understand what it does, why it matters, who can use it, and how to start without unsupported benefit or availability claims? |
| Support KB / FAQ / how-to | Are symptoms or questions scoped to supported versions, with verified steps, expected results, limits, and a usable escalation route? |
| Runbook | Does each relevant step identify a precondition, action, expected observation, and stop/recovery path? |
| Incident playbook | Can an operator triage and select a supported action without assuming a root cause or running unverified remediation? |
| Release notes | Are planned, merged, deployed, and enabled changes separated, with verified version/audience evidence, customer impact, feature-use guidance, and migration actions? |
| Meeting notes | Are discussion, agreed decisions, and action items distinguishable, without fabricated attendees, owners, or deadlines? |
| Retrospective | Do observations lead to owned, measurable experiments rather than blame or invented statistics? |
| Postmortem | Are impact, timeline, confirmed causes, unresolved hypotheses, and corrective actions supported by distinct evidence? |
| Project charter | Are mandate, stakeholder decision rights, funding estimates, and approval gates explicit without treating a draft as authorization? |
| Project plan | Are delivery sequence, capacity, dependencies, forecasts versus commitments, and completion evidence clear? |
| Project status | Is the assessment a dated, sourced snapshot rather than an inference of delivery from ticket closure? |
| Roadmap | Are directional horizons, actual commitments, and audience disclosure boundaries distinct? |
| OKRs | Are baseline, target, method, actual observation, and dated confidence assessments separate? |
| Risk assessment | Are proposed responses, tested control effectiveness, and authorized residual-risk acceptance distinct? |
| Experiment | Are protocol, run observations, deviations, and interpretation separate, with NOT RUN retained when no data exists? |
| Customer-impact assessment | Are anticipated effects, existing commitments, actual impact, and communication/change authority distinct? |
| Team working agreement | Are proposed practices, confirmed agreements, organizational policy, and unnecessary personal data handled appropriately? |

## Pressure cases

Treat these as reusable review questions, not claims of automated test coverage.

- An ADR is requested as Accepted but the supplied discussion has no decision: keep the proposal and explain the absent decision evidence.
- Required security review is pending: conditional acceptance needs explicit authority and conditions; it is not inferred from a request to proceed.
- An SFTP playbook reports login success and AccessDenied: investigate authorization paths; do not declare a particular policy the cause without evidence.
- A release uses a live Jira macro: preserve a dated release record and actual deployment evidence.
- A feature is merged or deployed but cohort enablement is unknown: describe that limit rather than announcing availability to all customers.
- An API example is generated from memory: verify it against the supplied specification or mark it as illustrative and unverified.
- A concurrent page edit arrives: merge the intended change against the new version.
- A native template operation is unavailable: complete the content, identify the missing installation capability, and report the actual outcome.
- A saved template changes: preserve older filled records and their evidence, offer a migration diff, and do not claim installed-template compatibility from local Markdown checks alone; use [template maintenance](template-maintenance.md).
- An executive or customer roadmap hides internal columns: filtering is not an access-control boundary; verify disclosure and sharing scope.
