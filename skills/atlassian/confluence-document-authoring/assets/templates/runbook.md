# Runbook: {{BOUNDED_OPERATION}}

<!-- Write procedures from supplied, verified service knowledge. Do not invent commands, access rights, thresholds, or recovery guarantees. Remove comments and unused optional sections from the filled document. -->

| Metadata | Value |
| --- | --- |
| Service / environment | {{SERVICE}} / {{ENVIRONMENT}} |
| Owning team | {{TEAM_OR_UNKNOWN}} |
| Procedure owner / backup | {{OWNER}} / {{BACKUP_OR_UNKNOWN}} |
| Status / version | {{DRAFT_VERIFIED_OR_RETIRED}} / {{VERSION}} |
| Last verified | {{DATE_ENVIRONMENT_AND_VERIFIER_OR_NOT_VERIFIED}} |
| Jira change / maintenance reference | {{VERIFIED_REFERENCE_OR_NOT_LINKED}} |
| Related incident playbook | {{VERIFIED_REFERENCE_OR_NONE}} |
| On-call / escalation reference | {{VERIFIED_CONTACT_OR_ROTA_REFERENCE}} |

## Purpose and use conditions

{{STATE_THE_OUTCOME_WHEN_TO_USE_THIS_PROCEDURE_AND_WHEN_TO_STOP_AND_ESCALATE}}

## Scope and prerequisites

| Requirement | Verification / source | Required before starting |
| --- | --- | --- |
| Target identity | {{HOW_TO_VERIFY_SERVICE_ENVIRONMENT_AND_SCOPE}} | {{REQUIRED_MATCH}} |
| Access / authorization | {{REQUIRED_ROLE_AND_CHANGE_AUTHORITY}} | {{EVIDENCE_OR_STOP}} |
| Dependency / tooling | {{VERIFIED_VERSIONS_AND_HEALTH_CHECK}} | {{PASS_CONDITION}} |
| Recovery readiness | {{BACKUP_CHECKPOINT_OR_KNOWN_RECOVERY_PATH}} | {{EVIDENCE_OR_STOP}} |

{{STATE_EXPECTED_IMPACT_DURATION_AND_MAINTENANCE_CONSTRAINTS_OR_MARK_UNKNOWN}}

## Baseline and preflight

| Check | Expected observation | Evidence to capture | Stop condition |
| --- | --- | --- | --- |
| {{READ_ONLY_CHECK}} | {{EXPECTED_BASELINE}} | {{ARTIFACT_OR_RECORD}} | {{UNSAFE_OR_AMBIGUOUS_RESULT}} |

## Ordered procedure

Complete steps in order. Recheck the target before any action that changes state.

| Step | Action / verified instruction reference | Expected output / observation | Stop condition | Recovery / next step |
| --- | --- | --- | --- | --- |
| 1 | {{BOUNDED_ACTION}} | {{OBSERVABLE_RESULT}} | {{WHEN_TO_STOP}} | {{RECOVERY_OR_STEP_2}} |
| 2 | {{BOUNDED_ACTION}} | {{OBSERVABLE_RESULT}} | {{WHEN_TO_STOP}} | {{RECOVERY_OR_STEP_3}} |
| 3 | {{BOUNDED_ACTION}} | {{OBSERVABLE_RESULT}} | {{WHEN_TO_STOP}} | {{RECOVERY_OR_VALIDATION}} |

<!-- Optional: include exact commands only when their source, target placeholders, effects, and recovery have been verified. No command is required to complete this template. -->
{{OPTIONAL_VERIFIED_COMMAND_REFERENCE_AND_REQUIRED_PARAMETERS}}

## Completion verification

| Check | Success criterion / observation window | Evidence / owner |
| --- | --- | --- |
| {{FUNCTIONAL_OR_HEALTH_CHECK}} | {{MEASURABLE_CRITERION_AND_WINDOW}} | {{CAPTURE_AND_OWNER}} |
| {{DEPENDENCY_OR_USER_PATH_CHECK}} | {{CRITERION}} | {{CAPTURE_AND_OWNER}} |

{{STATE_WHEN_THE_OPERATION_CAN_BE_DECLARED_COMPLETE_AND_WHO_CONFIRMS_IT}}

## Recovery and escalation

| Trigger | Recovery action / source | Limit / irreversible effect | Escalation owner |
| --- | --- | --- | --- |
| {{TRIGGER}} | {{VERIFIED_RECOVERY_REFERENCE_OR_STOP}} | {{KNOWN_LIMIT_OR_UNKNOWN}} | {{VERIFIED_ROLE_OR_CONTACT}} |

{{DESCRIBE_HOW_TO_VERIFY_RECOVERY_AND_WHEN_TO_SWITCH_TO_INCIDENT_RESPONSE}}

## Execution record

| Execution date / operator | Target / change reference | Outcome / evidence | Follow-up |
| --- | --- | --- | --- |
| {{DATE_AND_OPERATOR}} | {{VERIFIED_TARGET_AND_REFERENCE}} | {{OBSERVED_RESULT_OR_NOT_RUN}} | {{ACTION_OWNER_AND_DUE}} |

## Verification history

| Procedure version | Verified by / date | Method / environment | Findings / expiry trigger |
| --- | --- | --- | --- |
| {{VERSION}} | {{VERIFIER_AND_DATE_OR_NOT_VERIFIED}} | {{WALKTHROUGH_TEST_OR_ACTUAL_USE}} | {{LIMITS_AND_RECHECK_TRIGGER}} |

## Sources and unknowns

| Source | Revision / checked | Supports / limitation |
| --- | --- | --- |
| {{VERIFIED_SERVICE_DOC_OR_EVIDENCE}} | {{REVISION_AND_DATE}} | {{STEP_OR_CLAIM_AND_LIMIT}} |

| Unknown / assumption | Operational consequence | Owner / resolution |
| --- | --- | --- |
| {{UNKNOWN_OR_ASSUMPTION}} | {{WHY_IT_MATTERS}} | {{OWNER_AND_REQUIRED_CHECK}} |
