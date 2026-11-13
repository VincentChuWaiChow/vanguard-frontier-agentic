# Document lifecycle

These are documentation conventions for this skill. Follow an established team policy when supplied, and keep recorded facts separate from proposed process.

## Distinguish four kinds of state

| State | Example | Establishes |
| --- | --- | --- |
| Authoring readiness | Draft / review needed / ready for use | Completeness and unresolved issues in the content |
| Decision state | ADR Proposed / Accepted | Whether a particular choice is actually made |
| Native content status | A site's configured page status | The state represented by that specific Confluence feature |
| Publication state | API current / draft / archived | Product storage/publication behavior |

One state does not automatically imply another. A published ADR may still be Proposed; a page labeled “validated” is not evidence a runbook was exercised.

## Architecture decisions

Record one consequential decision per ADR. Keep options, decision drivers, consequences, and acceptance evidence explicit.

| ADR state | Meaning |
| --- | --- |
| Proposed | A decision remains under consideration |
| Accepted | The identified decision authority has accepted the stated choice; record evidence/date |
| Rejected | The proposal was considered and rejected; record why |
| Superseded | A later identified ADR replaces the decision |
| Deprecated | The decision is no longer applicable and has no direct replacement |

An explicit statement from the user can be decision evidence within their stated authority. Do not impose unanimous approval or a new sign-off process. When notes only say “might,” or explicitly say no decision was approved, preserve Proposed even if someone requests cosmetic Accepted wording.

Where a required security or other review is pending, retain Proposed unless an actual authorized decision record explicitly permits conditional acceptance. Record the authority for that exception, permitted activities, remaining restrictions, condition owner, and exit evidence. Conditional acceptance does not imply permission to deploy beyond the recorded scope.

Preserve the substance and rationale of an accepted decision. Make a new ADR when the decision changes, with reciprocal Supersedes/Superseded-by references. Record lifecycle metadata changes and non-substantive corrections transparently. Do not silently rewrite the original rationale to match today's implementation.

This history-preserving convention is supported by [AWS ADR process guidance](https://docs.aws.amazon.com/prescriptive-guidance/latest/architectural-decision-records/adr-process.html). It is not a Confluence platform restriction.

Keep rejected and superseded ADRs discoverable. Archival is a separate retention/organization decision; do not archive automatically just because a decision is superseded.

## Maintained documents

| Type | Maintenance trigger | Evidence to preserve |
| --- | --- | --- |
| PRD | Business rule, user need, outcome, or scope changes | Authoritative requirements, success measures, Jira criteria, unresolved dependencies |
| TDD | Requirement, interface, constraint, or delivery approach changes | Source versions, open decisions, affected tickets |
| API documentation | Published contract or supported version changes | Specification/revision, verified examples, compatibility policy |
| Product explanation / feature overview | User behavior, eligibility, availability, or product scope changes | Verified customer benefit, use instructions, limitations, audience and rollout evidence |
| Support KB / FAQ / how-to | Product behavior, known issue, support route, or workaround changes | Affected versions, reproduced symptoms, verified steps, expected results, escalation limits |
| Runbook/playbook | Service behavior, alerts, access, or recovery procedure changes | Last exercised date/environment, results, unresolved hazards |
| Release notes | Release prepared, deployed, enabled, withdrawn, or corrected | Artifact/version, separate merged/deployed/enabled evidence, affected audience |
| Meeting notes | Minutes corrected or action state updated | Original discussion versus agreed decision, action owner/due evidence |
| Sprint retrospective | Experiment reviewed or action completed | Baseline, intervention, measured result, limitations |
| Postmortem | Incident evidence, causal assessment, or corrective action changes | Timeline, impact evidence, supported causes versus hypotheses, action and validation evidence |

Use dates that actually correspond to the event or verification. “Last validated” remains unknown if no validation evidence exists. Do not replace it with today's authoring date.

Use existing labels and ownership conventions. If none are supplied, propose consistent labels such as `tdd`, `adr`, `api-docs`, `runbook`, `incident-playbook`, `release-notes`, `meeting-notes`, and `retrospective`. An `adr-accepted` label requires a maintained policy and inspection of the underlying decision, not inference from a text match.
