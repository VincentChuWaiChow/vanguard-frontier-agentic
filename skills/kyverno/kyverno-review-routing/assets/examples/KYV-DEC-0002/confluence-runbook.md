# Runbook: Distinguish a policy denial from a webhook, controller or registry dependency failure for invoice-api pod CREATE

| Metadata | Value |
| --- | --- |
| Local ID | KYV-DEC-0002 (no page URL yet) |
| Service / environment | invoice-api (synthetic) / Unknown — environment not supplied; Kyverno controller v1.20.0 (supplied), Kubernetes version Unknown |
| Owning team | Payments |
| Procedure owner / backup | Payments on-call SRE (role) / Unknown |
| Status / version | DRAFT / 0.1 |
| Last verified | NOT_VERIFIED |
| Jira change / maintenance reference | Jira incident, local ID KYV-DEC-0002 (no Jira key yet) |
| Related incident playbook | NONE |
| On-call / escalation reference | Unknown — rota reference not supplied |

## Purpose and use conditions

Use when invoice-api pods fail to start and an ImageValidatingPolicy with failurePolicy Fail gates pod CREATE, for example during release R-synthetic-42. Supplied evidence: three failed CREATE responses with no policy message (EV-1), elevated registry latency (EV-2), and an installed policy with failurePolicy Fail and timeoutSeconds 15 (EV-3) [KYV-E36]. The outcome is to classify the cause as one of four branches (policy denial, webhook call error or timeout, controller unavailable, registry dependency error) and to verify recovery. Stop and escalate to platform security if the cause fits none, or if recovery needs a policy or failurePolicy change, which requires a live-guard handoff and human approval.

## Scope and prerequisites

| Requirement | Verification / source | Required before starting |
| --- | --- | --- |
| Target identity | Confirm service invoice-api and namespace payments in the denial response | Match, otherwise stop |
| Access / authorization | Read access to admission responses or events; no change authority needed for classification | Evidence of access, otherwise stop |
| Dependency / tooling | Unknown — tooling and versions not supplied | Owner to confirm |
| Recovery readiness | Unknown — a known recovery path is not documented | Evidence, otherwise stop |

Expected impact and duration: Unknown. This procedure is read-only.

## Baseline and preflight

| Check | Expected observation | Evidence to capture | Stop condition |
| --- | --- | --- | --- |
| Read the failed pod CREATE response for invoice-api | A denial or an error is visible | The response text, request time with timezone | No response available |
| Confirm the policy kind, failurePolicy and timeout | ImageValidatingPolicy with failurePolicy Fail and timeoutSeconds 15 as supplied in EV-3; the CRD states no failurePolicy default, so the installed value is the evidence [KYV-E36] | Policy name and revision | Different policy, failurePolicy or timeout |

## Ordered procedure

Complete steps in order. Recheck the target before any action that changes state.

| Step | Action / verified instruction reference | Expected output / observation | Stop condition | Recovery / next step |
| --- | --- | --- | --- | --- |
| 1 | Cheapest discriminator first: verify the release image digest's signature against the policy attestors, using the supply chain owner's verification record | A verification result for the release image digest; no result is supplied today | No record obtainable | Continue to step 2 and record the gap |
| 2 | Read the denial text from the pod CREATE response | Text carrying the policy message, or an error with no policy message | No text captured | Escalate to payments on-call SRE |
| 3 | Branch 1, policy denial: the policy message is present. The validation evaluated false and the denial carries the policy message | The message names the policy | Message absent | Go to step 4 |
| 4 | Branch 2, webhook call error or timeout: an API error naming the webhook call and no policy message. Compare the webhook latency with timeoutSeconds 15; after expiry the request fails or ignores the result per failurePolicy [KYV-E36] | An error naming the webhook, no policy message | Cannot decide | Go to step 5 |
| 5 | Branch 3, controller unavailable: the same error shape as branch 2 plus controller pod availability evidence for the same minute | Controller pods not ready at the failure time | No availability evidence | Capture it and escalate to payments on-call SRE |
| 6 | Branch 4, registry dependency error: no policy message and no controller fault, with elevated registry latency (EV-2). How a registry error surfaces in the admission response is unverified; needs an admission-trace | An error without a policy message | Cannot decide | Capture an admission-trace and escalate |
| 7 | Check whether credentials for the private registry are listed in the policy, since the pod's imagePullSecrets are not used [KYV-E19] | Credentials present or absent | Unknown | Record as an unknown with owner |

Dissent recorded: `kyverno-policy-review-agent` holds that a trust rejection is not excluded until the release image signature is checked, so a suppressed policy message cannot be ruled out from the three responses alone. The incident timeline stays Unknown.

Exact commands: Unknown — no verified command reference supplied.

## Completion verification

| Check | Success criterion / observation window | Evidence / owner |
| --- | --- | --- |
| A subsequent invoice-api pod CREATE is admitted with the policy evaluated, not via Ignore | An admission-trace showing evaluation and admission; window Unknown | Trace captured; payments on-call SRE |
| Pods start and serve | Unknown — user path check not supplied | Owner to confirm |

Recovery is declared complete only when both checks pass and the incident owner confirms. A pod admitted because the policy was set to Ignore is not recovery.

## Recovery and escalation

| Trigger | Recovery action / source | Limit / irreversible effect | Escalation owner |
| --- | --- | --- | --- |
| Policy denial (trust rejection) | Fix the artifact signature or attestation through the supply chain owner; no webhook change | Unknown | Platform security |
| Webhook call error or timeout, or controller unavailable | Restore the controller or webhook path; do not switch to Ignore without approval | After a timeout the request fails or ignores the result per failurePolicy, so Ignore can admit pods unevaluated [KYV-E36] | Payments on-call SRE, then platform security |
| Registry dependency error | Restore the registry dependency; do not switch to Ignore without approval | Same Ignore limit [KYV-E36] | Payments on-call SRE, then platform security |

Switch to incident response when impact is confirmed beyond a single release.

## Execution record

| Execution date / operator | Target / change reference | Outcome / evidence | Follow-up |
| --- | --- | --- | --- |
| Unknown | KYV-DEC-0002 | NOT RUN | Owner TBD |

## Verification history

| Procedure version | Verified by / date | Method / environment | Findings / expiry trigger |
| --- | --- | --- | --- |
| 0.1 | NOT_VERIFIED | None | Recheck after the first real use |

## Sources and unknowns

| Source | Revision / checked | Supports / limitation |
| --- | --- | --- |
| Board decision KYV-DEC-0002 | Unknown | Classification logic and the failurePolicy and timeout evidence [KYV-E36]; incident timeline is Unknown — an admission-trace log would settle it |

| Unknown / assumption | Operational consequence | Owner / resolution |
| --- | --- | --- |
| Exact surfacing of a registry dependency error | The step 6 classification stays suspected | Platform security; admission-trace |
| Incident timeline | Impact window cannot be stated | Payments on-call SRE; timestamped events |
