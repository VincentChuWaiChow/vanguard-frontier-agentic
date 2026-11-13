# Route product, engineering and support work

Choose the artifact by the next decision or activity, not by who reports it. These are content conventions; discover the tenant's configured Jira/Jira Service Management request types and workflow before publishing.

| Incoming request | First template | Engineering follow-up when justified | Useful Confluence companion |
|---|---|---|---|
| “Could we add scheduled exports?” | Feature request | Story/epic after the product decision | PRD, feature overview |
| “Explain what this product does for finance” | Confluence document, not an unnecessary ticket | Task only if documentation work needs tracking | Product explainer |
| “Export failed for my account” | Support case | Bug if evidence establishes incorrect behavior; spike if uncertain | Troubleshooting KB |
| “Expected contract says A; observed behavior is B” | Bug | Bounded fix with reproduction and regression | Known issue, release notes |
| “Please grant access / change configuration” | Service request | Task/change when implementation is needed | How-to, entitlement/permission reference |
| “The service is unavailable now” | Incident | Mitigation/change, later bug/debt as evidenced | Runbook/playbook; postmortem afterward |
| “Why did last week's incident happen?” | Postmortem document | Separate preventive tasks | Incident postmortem |

If the user explicitly requests a bug report, preserve that requested form and mark the defect/reproduction as reported or unconfirmed. Recommend a linked support case where useful; do not silently replace the requested artifact or require a confirmed root cause before recording a bug report. The table is a routing default, not an override of explicit user intent.

## Route without inventing certainty

A support report can be valid without a known cause or repository. Record the customer symptom and business impact, then gather the minimum evidence needed to distinguish a defect, configuration issue, access issue, how-to question, unsupported capability, or unresolved condition. A feature gap is not automatically a defect.

Create a separate linked implementation issue only when the work boundary warrants it. Preserve the customer case and its communication history rather than replacing it with a developer ticket. Do not create every possible child as a live ticket without authorization.

An incident tracks active impact and recovery. A bug tracks incorrect behavior and its fix. A problem/cause investigation can outlive the incident. Restored service does not prove the underlying cause has been fixed.

## Audience pattern

Start with who is affected, what they cannot do, why it matters and the desired result. Put implementation evidence and proposed design in a technical handoff section. Explain acronyms on first use. Preserve constraints when simplifying: an acknowledged request is not necessarily completed work, and a possible workaround is not a verified resolution.

Distinguish **severity** (observed impact), **urgency** (time sensitivity) and **priority** (delivery order under the team's rules). Record the team's actual classification and rationale; do not assign a severe level solely because a user writes “urgent.” SLA values, escalation contacts, due dates and customer commitments require source evidence.

## Readiness by activity

| Assessment | What must be sufficiently clear |
|---|---|
| Ready for triage | Request, reporter/context to the extent provided, impact, open evidence and next classification step |
| Ready for investigation | Bounded question, accessible evidence or resolved access prerequisite, investigation limit and decision deliverable |
| Ready for fulfillment | Exact requested service action, target, eligibility/authorization, validation and completion criteria |
| Ready for implementation | Agreed behavior, relevant technical context, bounded scope, dependencies and verification |

These labels do not imply the activity occurred or was authorized for the agent to execute. Ticket authoring stops at the artifact unless the user separately requested implementation or operational action. For support, closure needs the applicable resolution evidence and policy/customer-confirmation requirement; do not claim customer confirmation without it.

## Evidence worth preserving

Keep relevant product/version/environment, time with timezone, sanitized request/correlation IDs, reproducibility and error text, scope of confirmed impact, verified workaround and limits, linked known issue, next owner and expected evidence. Minimize sensitive data. A customer-safe update is a draft until sending is explicitly requested and authorized.
