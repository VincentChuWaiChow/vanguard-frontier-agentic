# Establish the result breakdown and missing-result cause for payments-run-as-non-root on invoice-api

- **Local ID:** KYV-DEC-0001 (no Jira key yet). Rationale page: Confluence product explainer, local ID KYV-DEC-0001 (no page URL yet).
- **Content pattern:** Investigation / spike
- **Configured Jira work type:** Unresolved — map this pattern to the target project configuration at publish time.
- **Jira workflow status:** Unspecified — not yet created.
- **Authoring readiness (not Jira status):** Draft — the question is bounded, but the owner and timebox are unconfirmed and the evidence needed for AC-01 has not been supplied.
- **Owner:** Assurance analyst (role; named person TBD); owns the result breakdown. The cause of the 20 is owned by the platform security engineer (role).
- **Target context / evidence boundary:** Service invoice-api (synthetic, revenue-critical), namespace payments, control payments-run-as-non-root (ValidatingPolicy), evidence snapshot as_of 2026-11-20T00:00:00Z with a 7 day window. Allowed sources: the supplied sanitized inventory and PolicyReport extracts only. Kyverno controller version: v1.20.0 (supplied); Kubernetes version: Unknown — not supplied.

## Business outcome and research question

Platform security and the invoice-api service owner need to know whether the run-as-non-root control can be described as covering the service. Today only 100 of 120 in-scope workload objects have a fresh result, and no compliance percentage is supportable. Leaving this open means the control is either overstated to leadership or the 20 objects without results stay unexamined.

**Question:** What is the pass, fail, warn and error breakdown of the 92 observed non-excepted objects, and what is the cause of the 20 in-scope objects with no fresh result?

**Decision recipient:** Platform security control owner (role; named person TBD)

## Evidence and unknowns

| Anchor | Source and exact location | Known fact / limitation |
| --- | --- | --- |
| EV-1 | population-report: inventory of in-scope workload objects, as_of 2026-11-20T00:00:00Z, window 7 days | 120 in-scope workload objects. Omits objects created and deleted inside the window. |
| EV-2 | population-report: PolicyReport extract, same as_of and window | 100 objects have fresh results. Per-result breakdown is not supplied. |
| EV-3 | installed-config: sanitized PolicyExceptions referencing payments-run-as-non-root | 8 excepted objects, which report skip by default [KYV-E13]. A skip from an exception is not demonstrated prevention. Approval records are not supplied. |
| EV-4 | runtime-metric: kyverno_admission_requests_total with request_allowed=false, payments pods CREATE, 7 days [KYV-E12] | 6 denied pod creates in payments, not attributable to this policy from the counter labels. These are requests, not existing objects, and are excluded from the object denominator. |

- **Hypotheses / options:** Unknown — no hypothesis for the 20 missing results is supported by supplied evidence. Candidate causes (policy matching, background evaluation disabled, controller health, stale reports) are unverified.
- **Material unknowns:**
  - Unknown — pass/fail/warn/error counts for the other 92 observed objects (the 100 fresh minus the 8 excepted). The report extract would settle it.
  - Unknown — why 20 objects have no fresh result. A per-object listing with matching and last-scan information would settle it.
  - Unknown — whether the 20 are excluded by scope, unscanned or erroring.

## Bounded investigation

- **In:** Reading the supplied inventory and report extracts; classifying each of the 92 by result; classifying each of the 20 by candidate cause using supplied evidence only.
- **Out:** Implementation, live changes, collecting data from the cluster, changing the policy or exceptions, and any compliance percentage claim.
- **Budget:** Unknown — no effort cap supplied. TBD by the requester; no deadline or approved budget is implied.
- **Stopping rule:** Stop when the 92 are fully classified and each of the 20 has a stated cause or an explicit unknown with the evidence that would settle it, or when an agreed budget is exhausted. Report unresolved questions and the next decision; do not silently extend an agreed budget.
- **Dependencies / blockers:** Requester supplies the per-result extract (owner: assurance analyst, role) and the per-object listing with the cause of the 20 (owner: platform security engineer, role). Control owner confirms scope.

## Acceptance and verification

| AC ID | Observable deliverable condition | Verification method and expected evidence | Actual evidence / result |
| --- | --- | --- | --- |
| AC-01 | The investigation record gives pass, fail, warn and error counts for the 92 observed objects, with their sum equal to 92, or explains why the bounded work was inconclusive. | Review of the report extract against EV-2 and EV-3; counts reconcile to 92 and the method is stated. | NOT RUN |
| AC-02 | The record states a cause or an explicit unknown for each of the 20 objects without a fresh result, and keeps the 6 blocked creates out of the object denominator. | Review of the per-object listing against EV-1 and the denominator rule; EV-4 cited only as request evidence. | NOT RUN |
| AC-03 | The handoff states the supportable coverage claim (for example 100 of 120 with fresh results, with the 8 skips labeled as exceptions), names the decision owner, lists blockers and proposes bounded follow-up tickets; no unresolved item is labeled ready. | Review of the decision brief for fact, hypothesis and unknown separation, and for absence of a compliance percentage. | NOT RUN |

## Handoff and definition of done

- Assess **Ready for investigation** only when the extracts in the dependencies are supplied and the owner is confirmed. This is never **Ready for implementation**.
- Deliver the decision brief (location: Unknown — set at publish time) with source anchors, results, consumed budget if known, limitations, and the recommendation to the platform security control owner.
- A documented inconclusive outcome can complete the spike; it cannot support a compliance claim without the breakdown.
- Open implementation or remediation work only as a separate draft with its own target, scope, AC IDs and dependencies.
- Board outcome for KYV-DEC-0001: conditional; repository verdict: needs-review.
