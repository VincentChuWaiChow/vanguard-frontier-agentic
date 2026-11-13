# Product requirements: {{capability}}

| Metadata | Value |
|---|---|
| Document lifecycle | draft |
| Accountable owner | {{owner_or_unknown}} |
| Reviewer / decision authority | {{reviewer_and_authority_or_unknown}} |
| Revision / last edited | {{revision_or_unknown}} / {{date_or_unknown}} |
| Sources and their revisions | {{source_links_versions_and_dates_or_unknown}} |
| Jira / related documentation | {{jira_links_or_unknown}} / {{related_links_or_unknown}} |
| Review trigger | {{date_or_event_that_requires_review}} |

Replace `{{placeholders}}` with evidence-backed values. Write **unknown** when evidence is missing; use **not applicable — {{reason}}** only with a reason. The lifecycle is `draft`, `in-review`, or `approved`; it does not describe Confluence publication, delivery, or deployment. Record actual approval evidence before using `approved`.

## Problem and intended outcome

- Affected users and context: {{users_context_and_source_or_unknown}}
- Current problem and observed impact: {{problem_evidence_or_unknown}}
- Intended outcome: {{outcome}}
- Why now: {{priority_and_source_or_unknown}}
- Decision this document supports: {{decision_and_authority_or_unknown}}

| Success measure | Current baseline / source | Target / authority | Measurement method | Evidence state |
|---|---|---|---|---|
| {{measure}} | {{baseline_and_source_or_unknown}} | {{target_and_authority_or_unknown}} | {{method_or_unknown}} | NOT MEASURED |

## Scope

| Included | Excluded | Constraint and source |
|---|---|---|
| {{included_user_outcome}} | {{excluded_outcome}} | {{constraint_and_source_or_unknown}} |

## User journey and requirements

Describe the starting condition, user action, result, and important failure or exception paths: {{journey_and_exceptions}}.

Every material requirement must trace to a business rule or stated need. Link a specific Jira acceptance criterion, not only an epic title. A proposed criterion is not an agreed requirement.

| Requirement ID | Business rule / authoritative source and revision | Jira acceptance criterion: link + identifier | Verification method / evidence | State |
|---|---|---|---|---|
| BR-01 | {{rule_and_source_or_unknown}} | {{jira_ac_or_unknown}} | {{verification_method}}; NOT RUN | {{proposed_agreed_or_blocked}} |

### Acceptance criterion detail

- Criterion: {{ac_identifier_or_unknown}}
- Given: {{initial_conditions}}
- When: {{user_or_system_action}}
- Then: {{observable_result}}
- Relevant exception / boundary: {{exception_or_boundary_or_unknown}}
- Authority and agreement evidence: {{authority_and_evidence_or_unknown}}

## Dependencies and material unknowns

| Dependency / unresolved fact | Authoritative source and version | Owner | Effect on requirements | Resolution / gate |
|---|---|---|---|---|
| {{external_schema_or_dependency}} | {{contract_source_and_version_or_unknown}} | {{owner_or_unknown}} | {{affected_requirement_ids}} | {{required_validation_before_commitment}} |

If a material external schema is unavailable, state **unknown** and mark dependent requirements conditional or blocked. Do not invent field names, payloads, limits, or compatibility promises. Identify the contract owner and the evidence needed to unblock the requirement.

## Options and tradeoffs

| Option | User value / evidence | Cost or limitation | Risk / unresolved assumption | Disposition and reason |
|---|---|---|---|---|
| {{option_a}} | {{value_and_source_or_unknown}} | {{cost_or_unknown}} | {{risk_or_unknown}} | {{proposed_disposition}} |
| {{option_b_or_status_quo}} | {{value_and_source_or_unknown}} | {{cost_or_unknown}} | {{risk_or_unknown}} | {{proposed_disposition}} |

## Delivery and operational acceptance

- Readiness dependencies: {{dependencies_and_owners_or_unknown}}
- Accessibility, privacy, security, and support requirements: {{applicable_requirements_and_sources_or_unknown}}
- Rollout audience and success/stop criteria: {{audience_and_criteria_or_unknown}}
- Training, documentation, or support changes: {{changes_and_owners_or_unknown}}
- Validation evidence needed before declaring the outcome achieved: {{evidence_required}}

## Open questions and review record

| Question / decision | Blocks | Owner | Due / trigger | Resolution evidence |
|---|---|---|---|---|
| {{question}} | {{requirement_or_milestone}} | {{owner_or_unknown}} | {{date_or_event_or_unknown}} | unknown |

| Review date | Reviewer / authorized role | Scope reviewed | Outcome and conditions | Evidence |
|---|---|---|---|---|
| {{date_or_unknown}} | {{reviewer_or_unknown}} | {{scope}} | {{pending_or_recorded_outcome}} | {{evidence_link_or_unknown}} |
