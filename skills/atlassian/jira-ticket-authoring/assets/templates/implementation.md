# {{SUMMARY}}

<!-- Fill this template from evidence. Remove comments, unused optional sections, and unresolved placeholders from the final ticket. Keep unresolved decisions explicit; do not turn a proposal into a requirement. A small change may need only the core sections below. -->

**Content pattern:** Implementation — feature, maintenance, or refactor.
**Configured Jira work type:** {{AVAILABLE_PROJECT_WORK_TYPE_OR_UNRESOLVED}} — use a type configured in the target project.
**Authoring readiness:** {{DRAFT_OR_NEEDS_CLARIFICATION_OR_NEEDS_DECOMPOSITION_OR_READY_FOR_IMPLEMENTATION}}.
**Jira workflow status:** {{KNOWN_CURRENT_STATUS_OR_UNSPECIFIED}} — separate from authoring readiness.

## Outcome and reason

Deliver {{OBSERVABLE_CHANGE}} for {{USER_OR_SYSTEM}} so that {{VALUE_OR_PROBLEM_RESOLVED}}.

Current behavior: {{OBSERVED_OR_REPORTED_BASELINE_WITH_SOURCE}}.
Desired behavior: {{SPECIFIC_RESULT_AND_RELEVANT_BOUNDARIES}}.

## Facts and open decisions

<!-- A supplied brief establishes requested intent, not proof of deployed behavior. Keep a source location and what it supports beside each fact. -->

- **Verified source facts:** {{DIRECTLY_CHECKED_FACTS_WITH_SOURCE_LOCATIONS_OR_NONE_CHECKED}}.
- **Reported facts:** {{CLAIMS_FROM_THE_BRIEF_OR_REPORTER_NOT_INDEPENDENTLY_VERIFIED_OR_NONE}}.
- **Assumptions:** {{EACH_ASSUMPTION_ITS_REASON_AND_EFFECT_IF_FALSE_OR_NONE}}.
- **Questions:** {{UNRESOLVED_QUESTION_DECISION_OWNER_AND_BLOCKING_EFFECT_OR_NONE}}.

## Scope and constraints

- **In scope:** {{BOUNDED_BEHAVIORS_OR_ARTIFACTS_TO_CHANGE}}.
- **Out of scope:** {{ADJACENT_WORK_EXCLUDED_FROM_THIS_CHANGE}}.
- **Constraints and invariants:** {{REQUIRED_COMPATIBILITY_OR_BEHAVIOR_TO_PRESERVE}}.
- **Implementation context:** {{VERIFIED_REPOSITORY_GUIDANCE_INTERFACES_OR_RELEVANT_LOCATIONS_OR_NOT_YET_INSPECTED}}. Distinguish a suggested approach from a required constraint.

## Acceptance criteria

1. **AC-01 — {{PRIMARY_BEHAVIOR_LABEL}}:** Given {{PRECONDITION}}, when {{ACTION_OR_EVENT}}, then {{OBSERVABLE_EXPECTED_RESULT}}.
2. **AC-02 — {{BOUNDARY_OR_NEGATIVE_CASE_LABEL}}:** Given {{EDGE_OR_DISALLOWED_CONDITION}}, when {{ACTION_OR_EVENT}}, then {{EXPECTED_RESULT_AND_ANY_PROHIBITED_SIDE_EFFECT}}.

<!-- Add criteria for distinct required outcomes. Remove an irrelevant sample criterion instead of inventing a requirement. Avoid “works correctly,” “fully tested,” and unmeasured performance claims. -->

## Verification plan

| Criterion | Method, environment, and fixture | Pass condition | Evidence to retain | Evidence status |
|---|---|---|---|---|
| AC-01 | {{VERIFIED_CHECK_OR_PLANNED_TEST_CASE_AND_CONTEXT}} | {{OBSERVATION_THAT_SATISFIES_AC_01}} | {{RESULT_OR_ARTIFACT_TO_CAPTURE}} | Not run |
| AC-02 | {{BOUNDARY_CHECK_AND_CONTEXT}} | {{OBSERVATION_THAT_SATISFIES_AC_02}} | {{RESULT_OR_ARTIFACT_TO_CAPTURE}} | Not run |

<!-- Name commands only when supplied or verified. A proposed test case is a plan, not evidence of an existing test or a passing result. -->

## Dependencies and handoff

{{DEPENDENCIES_OR_EXPLICIT_NONE_OR_NOT_YET_CHECKED}}. For each dependency, state the exact known item or external input, the relationship as a sentence, the condition needed before starting, and the owner if known. Keep parent hierarchy separate from blocking links.

Ready for implementation requires a bounded outcome, usable context, observable acceptance, a feasible verification method, and resolution of decisions that block this work. If those conditions are unmet, record the specific gap in authoring readiness.

## Completion evidence

Keep behavior-specific acceptance criteria separate from the team's existing Definition of Done. Include only applicable checks from a supplied policy; do not invent mandatory sign-offs or deployment gates. A known mandatory policy that is unavailable is an implementation-handoff gap, not automatically a triage blocker.

| Applicable team completion check | Policy / revision | Evidence required | Actual result |
| --- | --- | --- | --- |
| {{CHECK_OR_NO_POLICY_SUPPLIED}} | {{SOURCE_OR_UNKNOWN}} | {{REQUIRED_EVIDENCE_OR_UNKNOWN}} | Not verified |

This checklist does not authorize deployment or execution of a live change.

- **Changes delivered:** {{ACTUAL_CHANGE_AND_ARTIFACT_REFERENCES_TO_RECORD_AFTER_WORK}}.
- **Verification results:** Not run — replace with observed results mapped to the acceptance criteria.
- **Remaining gaps or deviations:** {{UNFINISHED_CRITERIA_FAILURES_OR_AGREED_SCOPE_CHANGES_TO_RECORD_AFTER_WORK}}.

<!-- OPTIONAL MODULE: retain only for maintenance or refactoring where externally visible behavior must stay stable. -->
## Optional — Maintenance or refactor

- **Behavior to preserve:** {{OBSERVABLE_COMPATIBILITY_INVARIANTS}}.
- **Reason for the internal change:** {{CONCRETE_MAINTAINABILITY_OR_DEFECT_RISK}}.
- **Permitted structural change:** {{BOUNDARY_OF_REORGANIZATION_AND_EXISTING_CONVENTIONS}}.
- **Equivalence evidence:** {{HOW_OLD_AND_NEW_BEHAVIOR_WILL_BE_COMPARED}}.

<!-- OPTIONAL MODULE: retain for changes to a public API, event, or shared interface. -->
## Optional — Interface contract

- **Inputs, outputs, and errors:** {{EXACT_CONTRACT_OR_VERIFIED_REFERENCE}}.
- **Compatibility and consumers:** {{AFFECTED_CONSUMERS_AND_REQUIRED_TRANSITION_BEHAVIOR}}.
- **Shared ownership:** {{CONTRACT_OWNER_AND_DEPENDENT_WORK}}.

<!-- OPTIONAL MODULE: retain for a user-facing interface when these requirements apply. -->
## Optional — User interface

- **Relevant states and copy:** {{LOADING_EMPTY_ERROR_SUCCESS_OR_OTHER_REQUIRED_STATES}}.
- **Interaction and accessibility requirements:** {{EVIDENCED_KEYBOARD_FOCUS_LABEL_OR_ASSISTIVE_TECH_BEHAVIOR}}.
- **Design reference:** {{SUPPLIED_DESIGN_AND_WHICH_DETAILS_ARE_BINDING_OR_UNRESOLVED}}.

<!-- OPTIONAL MODULE: retain when persistence, deployment, or a measurable runtime property is affected. Use the operational-change template for a separately executed live change. -->
## Optional — Data and runtime effects

- **Data or migration effects:** {{AFFECTED_DATA_COMPATIBILITY_AND_RECOVERY_REQUIREMENTS}}.
- **Rollout and observation:** {{REQUIRED_RELEASE_MECHANISM_SUCCESS_SIGNAL_AND_OBSERVATION_PERIOD}}.
- **Quantitative constraint:** {{SUPPLIED_THRESHOLD_WORKLOAD_AND_MEASUREMENT_METHOD_OR_EXPLICITLY_UNDECIDED}}.
