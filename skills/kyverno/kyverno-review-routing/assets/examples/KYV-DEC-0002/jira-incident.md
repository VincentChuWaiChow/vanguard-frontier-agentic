# invoice-api pods fail to start during release R-synthetic-42 while image verification gates pod CREATE

- **Local ID:** KYV-DEC-0002 (no Jira key yet). Procedure page: Confluence runbook, local ID KYV-DEC-0002 (no page URL yet).
- **Type:** Incident
- **Readiness assessment (not Jira status):** Draft — the symptom and gating policy are known; the cause, timeline and severity are not.
- **Operational state:** Reported active
- **Incident owner / review recipient:** Payments on-call SRE (role; named person TBD)
- **Affected context:** Service invoice-api (synthetic, revenue-critical), namespace payments, release R-synthetic-42, an ImageValidatingPolicy with failurePolicy Fail and timeoutSeconds 15 (EV-3, installed config) gating pod CREATE; Kyverno controller v1.20.0 (supplied), Kubernetes version Unknown.
- **Severity / urgency:** Unknown — no severity policy source supplied.

## User and business impact

- **Reported degradation:** invoice-api pods fail to start during release R-synthetic-42. Source: reporter statement, not independently confirmed.
- **Confirmed impact:** Unknown — the affected workflow, population and extent are not evidenced.
- **Estimated / suspected impact:** Release blocked; invoices queue until pods start (suspected from the supplied observations, extent not measured).
- **Impact window:** Start Unknown — timestamp with timezone not supplied; end Unknown.
- **Expected service behavior:** Unknown — no service contract or target supplied.

## Evidence and timeline

| Time and timezone | Event / observation | Reported, confirmed, or proposed | Source / limitation |
| --- | --- | --- | --- |
| Unknown | Incident timeline is Unknown — an admission response or event log with timestamps would settle it | Unknown | None supplied |

- **Established findings:** Supplied evidence: three failed pod CREATE responses without a policy message (EV-1), elevated registry latency in the release window (EV-2), and the installed policy with failurePolicy Fail and timeoutSeconds 15 (EV-3). The ImageValidatingPolicy has failurePolicy (Ignore or Fail) and a timeout of 1 to 30 seconds, and its CRD states no failurePolicy default [KYV-E36]; the value here comes from the installed config. For a ValidatingPolicy, failurePolicy governs evaluation errors, not validations that evaluate to false [KYV-E04]; that statement is not established for an ImageValidatingPolicy here. Verification functions return counts [KYV-E18]; which count the policy treats as a failure depends on its expression, which was not supplied.
- **Working hypotheses:** (1) Policy denial (trust rejection): the response carries the policy message; none of the three supplied responses does, but a suppressed message is not excluded. (2) Webhook call error or timeout: an error naming the webhook and no policy message; after the 15 second timeout the request fails or ignores the result per failurePolicy [KYV-E36]. (3) Controller unavailable: the same error shape plus controller availability evidence. (4) Registry dependency error: no policy message; how it surfaces is unverified and needs an admission-trace. Cheapest discriminator: verify the release image digest's signature against the policy attestors; the denial text and its source in the pod CREATE response is next. Dissent from `kyverno-policy-review-agent`: a trust rejection is not excluded until that signature is checked.
- **Unknowns / blockers:** Which hypothesis holds; whether a private registry secret is configured for verification (ImageValidatingPolicy does not use the pod's imagePullSecrets [KYV-E19]); webhook timeout in effect (timeoutSeconds 15 per EV-3 [KYV-E36]). Owner: payments on-call SRE.

## Mitigation and recovery record

| Action | State | Exact target / authorization source | Evidence of effect / limits |
| --- | --- | --- | --- |
| Capture a pod CREATE admission response for invoice-api | Proposed | Read-only; target Unknown — namespace payments, authorization from on-call | NOT RUN |

- **Recovery condition:** A subsequent invoice-api pod CREATE is admitted with the policy evaluated, and not admitted because the policy was bypassed via Ignore.
- **Recovery verification:** Admission-trace of a new pod CREATE showing evaluation; actual result NOT RUN.
- **Residual impact / recovery work:** Unknown — pods already failed to start are not accounted for.
- **Risks / reversal or recovery plan:** Switching failurePolicy to Ignore would admit pods unevaluated and is not recovery; it needs a live-guard handoff and a human approval. Reversal limits Unknown.
- **Dependencies / next owner:** Payments on-call SRE (restore owner); platform security for any policy change (TBD).

## Communication — DRAFT, NOT SENT

invoice-api pods are reported as failing to start during release R-synthetic-42. The cause is not yet known and the impact is not yet confirmed. The next update time is not committed.

**Audience / approved communication route:** Unknown — to be confirmed by the incident owner.

## Incident acceptance and verification

These criteria cover the incident record and recovery assessment. A permanent fix or postmortem has separate scope.

| AC ID | Observable condition | Verification method and expected evidence | Actual evidence / result |
| --- | --- | --- | --- |
| AC-01 | The impact scope and window are supported, or the remaining limits are explicit. | Review of a timezoned timeline against sources and estimate labels. | NOT RUN |
| AC-02 | New invoice-api pod CREATE requests are admitted with the policy evaluated, and residual impact is accounted for. | Admission-trace of a post-recovery pod CREATE showing the policy evaluated, plus pod start confirmation, with actual outputs. | NOT RUN |
| AC-03 | The cause is classified as policy denial, webhook call error or timeout, controller unavailable, or registry dependency error, and follow-up work has a recorded owner. | Review of the denial text and admission-trace; review of follow-up links. | NOT RUN |

## Handoff and follow-up

- Assess **Ready for triage** now. **Ready for investigation** requires a bounded target and permitted methods; neither means permission to act on live systems.
- **Human recovery / closure assessment:** Pending. A mitigation attempt or quieter alert is not recovery proof.
- **Separate follow-ups:** postmortem, registry resilience review and any failurePolicy change, each with owner TBD.
- The author records evidence and drafts handoffs. Writing this ticket does not authorize mitigation, deployment, data repair, external communication, or other live response actions.
