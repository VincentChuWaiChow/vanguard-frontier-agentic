# Technical Design: {{FEATURE_OR_CHANGE}}

<!-- Fill from supplied evidence. Mark missing facts Unknown; do not invent approval or measurements. Remove all comments and unused optional sections from the filled document. -->

| Metadata | Value |
| --- | --- |
| Author | {{AUTHOR_OR_UNKNOWN}} |
| Document owner | {{OWNER_AND_TEAM}} |
| Status | {{DRAFT_IN_REVIEW_APPROVED_OR_SUPERSEDED}} |
| Document version / updated | {{VERSION}} / {{DATE}} |
| Reviewers / decision authority | {{REVIEWERS}} / {{AUTHORITY_OR_UNKNOWN}} |
| Jira delivery reference | {{VERIFIED_ISSUE_KEY_OR_URL_OR_NOT_LINKED}} |
| Related ADRs / design | {{VERIFIED_REFERENCES_OR_NONE}} |
| Target environment / release | {{ENVIRONMENT_AND_RELEASE_OR_UNKNOWN}} |

## Summary

{{STATE_THE_PROBLEM_PROPOSED_CHANGE_AND_EXPECTED_OUTCOME_IN_THREE_SENTENCES}}

## Business problem and evidence

{{DESCRIBE_WHO_IS_AFFECTED_CURRENT_BEHAVIOR_AND_WHY_CHANGE_IS_NEEDED}}

| Evidence / source ID | Observation | Impact and limits |
| --- | --- | --- |
| {{SOURCE_ID}} | {{OBSERVED_FACT}} | {{SUPPORTED_IMPACT_AND_UNCERTAINTY}} |

## Goals and boundaries

| Type | Requirement | Acceptance measure / evidence |
| --- | --- | --- |
| Goal | {{GOAL}} | {{MEASURE_AND_HOW_TO_VERIFY}} |
| Non-goal | {{EXCLUDED_OUTCOME}} | {{REASON_OR_FUTURE_REFERENCE}} |

## Constraints and options

{{LIST_FIXED_CONSTRAINTS_AND_DISTINGUISH_THEM_FROM_ASSUMPTIONS}}

| Option | Benefits | Costs / failure modes | Evidence / unresolved question |
| --- | --- | --- | --- |
| {{OPTION_A}} | {{BENEFITS}} | {{COSTS_AND_RISKS}} | {{SOURCE_OR_UNKNOWN}} |
| {{OPTION_B_OR_NO_CHANGE}} | {{BENEFITS}} | {{COSTS_AND_RISKS}} | {{SOURCE_OR_UNKNOWN}} |

## Proposed design and decision

{{EXPLAIN_COMPONENTS_RESPONSIBILITIES_AND_END_TO_END_BEHAVIOR}}

{{STATE_RECOMMENDATION_AND_TRADEOFFS_OR_LINK_THE_RECORDED_DECISION}}

| Review / decision | Authority | Evidence / date | Remaining condition |
| --- | --- | --- | --- |
| {{PROPOSED_OR_RECORDED_OUTCOME}} | {{PERSON_OR_ROLE}} | {{SOURCE_OR_NOT_RECORDED}} | {{CONDITION_OR_NONE}} |

## Interfaces, data, and business rules

| Interface / data object | Producer → consumer | Contract / rule | Compatibility / failure behavior |
| --- | --- | --- | --- |
| {{NAME}} | {{PRODUCER_AND_CONSUMER}} | {{SCHEMA_VALIDATION_AND_RULE}} | {{VERSIONING_AND_ERROR_BEHAVIOR}} |

<!-- Optional: retain the following paragraph for changes affecting persistent data, security, privacy, or capacity. -->
{{DESCRIBE_DATA_LIFECYCLE_MIGRATION_ACCESS_CONTROLS_LOAD_AND_OBSERVABILITY_WITH_VERIFIED_LIMITS}}

## Implementation and dependencies

| Deliverable | Owner | Dependency / Jira reference | Completion evidence |
| --- | --- | --- | --- |
| {{DELIVERABLE}} | {{OWNER}} | {{DEPENDENCY_OR_VERIFIED_KEY}} | {{EXPECTED_ARTIFACT_OR_CHECK}} |

## Test and acceptance strategy

| Requirement / risk | Test scenario | Expected result | Evidence / owner |
| --- | --- | --- | --- |
| {{REQUIREMENT_OR_RISK}} | {{NORMAL_EDGE_OR_FAILURE_CASE}} | {{MEASURABLE_EXPECTATION}} | {{PLANNED_OR_ACTUAL_EVIDENCE_AND_OWNER}} |

## Rollout, monitoring, and recovery

| Stage | Entry condition / owner | Action | Success signal / stop condition | Recovery |
| --- | --- | --- | --- | --- |
| {{STAGE}} | {{REQUIRED_EVIDENCE_AND_OWNER}} | {{BOUNDED_ACTION}} | {{THRESHOLD_WINDOW_AND_STOP_RULE}} | {{VERIFIED_RECOVERY_PATH}} |

{{STATE_DATA_REVERSIBILITY_LIMITS_AND_WHO_DECIDES_TO_CONTINUE_OR_RECOVER}}

## Sources, assumptions, and open questions

| Source ID | Reference / revision | Supports | Checked / limitation |
| --- | --- | --- | --- |
| {{SOURCE_ID}} | {{VERIFIED_REFERENCE_AND_REVISION}} | {{CLAIM_OR_SECTION}} | {{DATE_AND_LIMITATION}} |

| Unknown / assumption | Consequence if wrong | Owner | Resolution / due date |
| --- | --- | --- | --- |
| {{UNKNOWN_OR_LABELED_ASSUMPTION}} | {{CONSEQUENCE}} | {{OWNER_OR_UNASSIGNED}} | {{CHECK_AND_DATE_OR_UNKNOWN}} |

## Change history

| Version / date | Change | Author / evidence |
| --- | --- | --- |
| {{VERSION_AND_DATE}} | {{MATERIAL_CHANGE}} | {{AUTHOR_AND_REFERENCE}} |
