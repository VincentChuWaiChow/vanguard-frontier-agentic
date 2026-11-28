# Product explainer: payments-run-as-non-root coverage for invoice-api

| Metadata | Value |
| --- | --- |
| Local ID | KYV-DEC-0001 (no page URL yet) |
| Audience / purpose | Service teams in payments / what the 100 of 120 number means and does not mean |
| Accountable owner / reviewer | Platform security control owner (role) / assurance analyst (role); cause of the 20 owned by platform security engineer (role), breakdown owned by assurance analyst (role) |
| Document lifecycle / review trigger | draft / when the result breakdown for the 92 observed objects is supplied |
| Revision / last reviewed | Unknown — not yet published / Unknown — not yet reviewed |
| Sources / revisions / dates | Evidence EV-1 to EV-4, as_of 2026-11-20T00:00:00Z, 7 day window; revisions Unknown — locators come from the evidence pack |
| Product / version / availability | payments-run-as-non-root (ValidatingPolicy) on invoice-api / Kyverno controller v1.20.0 (supplied); Kubernetes version Unknown — not supplied / Unknown — availability not verified |
| Jira / related documentation | Jira spike, local ID KYV-DEC-0001 (no Jira key yet) / none |

Decision state: conditional (board outcome), needs-review (repository verdict). This page explains the number. The spike owns the work to complete it. Document approval does not establish that the control is effective.

## What it does

The payments-run-as-non-root control checks that workloads in the payments namespace are configured to run as a non-root user. Reports describe objects that exist, and a blocked create never becomes an object [KYV-E09].

## Who it helps and why

- Intended users and situation: invoice-api service teams reading a coverage figure on a dashboard or in a review.
- Problem and evidence: the figure 100 of 120 can be misread as 83 percent compliant. Evidence EV-1 and EV-2 support only that 100 of 120 in-scope objects have a fresh result.
- Why the problem matters: a misread figure can overstate protection for a revenue-critical service. Unknown — no business impact has been measured.

| Before | With the product | Evidence and status |
| --- | --- | --- |
| Teams read 100/120 as a compliance percentage | Teams read it as fresh-result coverage and ask for the breakdown | EV-1, EV-2; observed |

| Benefit | Desired outcome | Measured outcome / method / period |
| --- | --- | --- |
| Accurate reading of coverage | Teams do not cite a compliance percentage without its population and window | not measured |

## Key terms in business language

| Term | Meaning for the reader | Concrete example | Source / qualification |
| --- | --- | --- | --- |
| Fresh result | A report result recent enough to count in the window | 100 of the 120 objects have one | EV-2 |
| pass / fail / warn / error | The result values include these four; warn can mean a policy is not scored, and error means evaluation failed [KYV-E10] | Illustrative: an object with a fail needs a fix | Definition from the Kyverno reports guide |
| skip | An excepted object reports skip by default [KYV-E13]; it is not a pass | 8 excepted objects report skip | EV-3 (sanitized PolicyExceptions, installed-config) |
| Blocked create | A request refused at admission; no object exists afterwards [KYV-E09] | 6 denied pod creates in payments, not attributable to this policy from the counter labels | EV-4 |

## Common workflows

| Step | What the person does | What happens next | Exception / help |
| --- | --- | --- | --- |
| 1 | Read the coverage figure with its population and window | Understand 100 of 120 means fresh results, as_of 2026-11-20T00:00:00Z, 7 days | If population or window is missing, ask the assurance analyst |
| 2 | Read skip versus fail for your objects | An excepted object reports skip by default [KYV-E13] (other causes of skip are not established here); a fail is a violation to fix | Ask the control owner if a skip is unexpected |
| 3 | Look for blocked creates in request metrics and Events, not in reports [KYV-E09] [KYV-E12] | The 6 denied pod creates in payments appear in the request counter, not attributable to this policy from the counter labels | Unknown — dashboards or queries to use are not supplied |

Detailed instructions: Unknown — no verified how-to exists yet.

## Capabilities and boundaries

| Need | Supported capability / state | Limitation or excluded case | Evidence / version |
| --- | --- | --- | --- |
| See which objects have results | Supported for the 100 with fresh results | 20 objects have no fresh result, cause Unknown — a per-object listing would settle it | EV-1, EV-2 |
| State a compliance percentage | Not supported today | Breakdown of the other 92 observed objects is Unknown — the report extract would settle it | EV-2, EV-3 |
| Show prevention | Not supported by exceptions | The 8 exceptions are not demonstrated prevention | EV-3 |

## Availability, prerequisites, and commercial terms

| Reader question | Answer | Evidence / applicable scope |
| --- | --- | --- |
| Who can use it now? | Unknown — no availability statement supplied | Unknown — owner to confirm |
| What is required first? | Unknown — access to reports is not described | Unknown |
| What does it cost? | Not applicable — internal control | Not applicable |
| What commitments apply? | Unknown — no service commitment supplied | Unknown |

## Questions and help

- Common question: Is invoice-api 83 percent compliant? Answer: that cannot be said. The 6 denied pod creates in payments are not objects, are not attributable to this policy from the counter labels, and are excluded from the denominator, and the other 92 observed objects have no supplied breakdown.
- Support / feedback route: Unknown — assurance analyst contact to be confirmed.
- Open question affecting this explanation: why 20 objects have no fresh result; owner platform security engineer (the result breakdown for the 92 is owned by the assurance analyst); resolves when the Jira spike (local ID KYV-DEC-0001, no Jira key yet) answers it.

## Optional technical appendix

- Architecture, integration, or data-flow reference: Unknown — not supplied.
- Relevant technical constraints and their user impact: exceptions report as skip by default [KYV-E13]; background results describe existing objects only [KYV-E09].
- Implementation state / validation evidence: observed counts only, from EV-1 to EV-4; no controlled test has been run.
