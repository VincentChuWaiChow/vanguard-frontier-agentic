# {{BOUNDED_OPERATIONAL_CHANGE}}

<!-- Remove comments, unused optional sections, and unresolved placeholders from the final ticket. This template specifies a change; completing the template does not execute it. Use verified runbook steps or clearly unresolved plans, never invented runnable commands. -->

**Content pattern:** Operational change.
**Configured Jira work type:** {{AVAILABLE_PROJECT_WORK_TYPE_OR_UNRESOLVED}}.
**Authoring readiness:** {{DRAFT_OR_NEEDS_CLARIFICATION_OR_NEEDS_DECOMPOSITION_OR_READY_FOR_IMPLEMENTATION}}.
**Jira workflow status:** {{KNOWN_CURRENT_STATUS_OR_UNSPECIFIED}} — separate from authoring readiness.

## Outcome and reason

Change {{PRECISE_TARGET}} from {{CURRENT_STATE}} to {{INTENDED_STATE}} because {{PROBLEM_OR_OPERATIONAL_NEED}}.

- **Environment and target identity:** {{ENVIRONMENT_ACCOUNT_CLUSTER_SERVICE_RESOURCE_OR_OTHER_UNAMBIGUOUS_IDENTIFIER}}.
- **Execution boundary:** {{AUTHORIZED_ACTIONS_TARGETS_AND_LIMITS_OR_UNCONFIRMED_SCOPE}}.
- **Expected effect and affected users:** {{EXPECTED_BEHAVIOR_AND_KNOWN_IMPACT}}.
- **Timing constraints:** {{USER_SUPPLIED_WINDOW_OR_DEPENDENCY_CONSTRAINT_OR_NONE_KNOWN}}.

## Facts and open decisions

- **Verified source facts:** {{OBSERVED_BASELINE_WITH_TIME_AND_SOURCE_OR_NONE_CHECKED}}.
- **Reported facts:** {{UNVERIFIED_STATE_OR_REQUIREMENT_CLAIMS_AND_SOURCES_OR_NONE}}.
- **Assumptions:** {{ASSUMPTIONS_AND_EFFECT_IF_FALSE_OR_NONE}}.
- **Questions:** {{UNRESOLVED_DECISIONS_OWNERS_AND_BLOCKING_EFFECT_OR_NONE}}.

## Scope and preconditions

- **In scope:** {{EXACT_RESOURCES_ACTIONS_AND_CHANGE_DELTA}}.
- **Out of scope:** {{RESOURCES_ACTIONS_OR_FOLLOWUP_WORK_EXCLUDED}}.
- **Preconditions:** {{REQUIRED_BASELINE_ACCESS_CAPACITY_BACKUP_OR_DEPENDENCY_STATE_AND_HOW_TO_CHECK}}.
- **Preserved state:** {{INVARIANTS_AND_UNAFFECTED_BEHAVIOR_THAT_MUST_HOLD}}.
- **Execution and response owners:** {{KNOWN_OPERATOR_AND_PERSON_RESPONSIBLE_IF_A_TRIGGER_FIRES_OR_UNRESOLVED}}.

## Change procedure

| Stage | Verified action or runbook step | Expected intermediate state | Check before continuing |
|---|---|---|---|
| {{STAGE_1}} | {{BOUNDED_ACTION_OR_VERIFIED_RUNBOOK_REFERENCE}} | {{EXPECTED_STATE}} | {{SPECIFIC_OBSERVATION_AND_LIMIT}} |
| {{STAGE_2_IF_NEEDED}} | {{BOUNDED_ACTION_OR_VERIFIED_RUNBOOK_REFERENCE}} | {{EXPECTED_STATE}} | {{SPECIFIC_OBSERVATION_AND_LIMIT}} |

<!-- Remove unused stages. Identify a proposal as unverified until the target and procedure have been checked. -->

## Success, abort, and recovery

**Success signal:** {{METRIC_OR_OBSERVABLE_STATE}} must satisfy {{SPECIFIC_EXPECTED_VALUE_OR_THRESHOLD}} over {{DEFINED_OBSERVATION_PERIOD_OR_DISCRETE_STATE_CHECK_WITH_REASON}}, measured at {{VERIFIED_SOURCE}}.

**Abort trigger:** If {{SPECIFIC_FAILURE_CONDITION_OR_THRESHOLD}} occurs during {{APPLICABLE_STAGE_OR_PERIOD}}, {{RESPONSIBLE_OWNER}} will {{IMMEDIATE_STOP_OR_CONTAINMENT_ACTION}}.

**Rollback trigger:** {{CONDITION_REQUIRING_ROLLBACK_AND_RELATION_TO_ABORT}}.
**Rollback target:** {{EXACT_PREVIOUS_VERSION_CONFIGURATION_OR_RESTORABLE_STATE}}.
**Rollback procedure:** {{VERIFIED_RESTORE_STEPS_OR_RUNBOOK_WITH_REQUIRED_INPUTS}}.
**Rollback limit:** {{LAST_SAFE_POINT_TIME_LIMIT_OR_IRREVERSIBLE_EFFECT_AND_REASON}}.
**Recovery validation:** Confirm {{SPECIFIC_RESTORED_BEHAVIOR_OR_THRESHOLD}} at {{EVIDENCE_SOURCE}} using {{OBSERVATION_PERIOD_OR_DISCRETE_STATE_CHECK_WITH_REASON}}.
**If rollback fails or is impossible:** {{CONCRETE_CONTAINMENT_AND_RECOVERY_PATH_WITH_RESPONSIBLE_OWNER}}.

<!-- Applicable success, abort, and recovery specifics are required before Ready for implementation. Do not substitute vague “monitor” or “rollback if needed.” Use N/A — reason for an individual field that genuinely does not apply. A discrete observable state check is sufficient when a time-based observation is unnecessary; never invent a duration. If reversal is impossible, explain the irreversible effects and identify concrete containment or forward recovery where applicable. Missing material recovery decisions still block readiness. -->

## Acceptance criteria

1. **AC-01 — Target and effect:** After the change, {{TARGET_STATE_AND_REQUIRED_BEHAVIOR}} satisfy the success signal under the defined observation period or discrete state check.
2. **AC-02 — Bounds and abort handling:** {{REQUIRED_PRESERVED_STATE}} remains within {{DEFINED_LIMITS}}; if the abort trigger occurs, execution stops or is contained as specified and the observed state is recorded.
3. **AC-03 — Recovery is usable:** Before execution, {{ROLLBACK_INPUTS_PROCEDURE_AND_RECOVERY_CHECK}} are verified as available for the target; if recovery is exercised, its actual result is recorded against the recovery validation condition.

## Verification plan

| Criterion | Method and stage | Pass condition | Evidence to retain | Evidence status |
|---|---|---|---|---|
| AC-01 | {{BASELINE_AND_POST_CHANGE_OBSERVATION_METHOD}} | {{EXACT_SUCCESS_CONDITION_AND_APPLICABLE_OBSERVATION}} | {{TIMESTAMPED_BEFORE_AND_AFTER_SIGNALS}} | Not run |
| AC-02 | {{BOUNDARY_AND_TRIGGER_CHECK_METHOD}} | {{LIMITS_HELD_OR_SPECIFIED_RESPONSE_PERFORMED}} | {{OBSERVATIONS_AND_ANY_ABORT_ACTION}} | Not run |
| AC-03 | {{RECOVERY_READINESS_CHECK_AND_POST_RECOVERY_CHECK_IF_USED}} | {{INPUTS_USABLE_AND_ACTUAL_RECOVERY_RESULT_RECORDED_IF_USED}} | {{PROCEDURE_CHECK_OR_RECOVERY_EVIDENCE}} | Not run |

## Dependencies and readiness

{{DEPENDENCIES_OR_EXPLICIT_NONE_OR_NOT_YET_CHECKED}}. Identify exact known dependencies, their required state, relationship, and owner. Preserve existing authorization context; record a missing execution boundary as unresolved rather than inventing one.

**Readiness reason:** {{EVIDENCE_THAT_TARGET_DELTA_PRECONDITIONS_PROCEDURE_SUCCESS_ABORT_AND_RECOVERY_ARE_SPECIFIC_OR_EXACT_GAPS}}.

## Completion evidence

- **Executed delta and stages:** {{ACTUAL_ACTIONS_TARGETS_AND_TIMES_TO_RECORD_AFTER_WORK}}.
- **Verification results:** Not run — replace with observed results per criterion and applicable observation.
- **Abort or recovery actions:** {{ACTUAL_ACTIONS_AND_RESULTS_OR_NOT_NEEDED_WITH_OBSERVED_BASIS}}.
- **Final observed state:** {{STATE_EVIDENCE_AND_OBSERVATION_TIME}}.
- **Remaining work or uncertainty:** {{KNOWN_GAPS_AND_REQUIRED_FOLLOWUP}}.
