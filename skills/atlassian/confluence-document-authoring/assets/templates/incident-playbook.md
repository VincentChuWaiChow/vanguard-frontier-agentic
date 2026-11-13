# Incident Playbook: {{SYMPTOM_OR_FAILURE_MODE}}

<!-- This is a response guide, not an incident execution record. Use verified service knowledge; leave unverified diagnostics, thresholds, and actions explicit. Remove comments and unused optional sections from the filled document. -->

| Metadata | Value |
| --- | --- |
| Service / applicable environments | {{SERVICE_AND_ENVIRONMENTS}} |
| Playbook owner / version | {{OWNER}} / {{VERSION}} |
| Status / last verified | {{DRAFT_VERIFIED_OR_RETIRED}} / {{DATE_OR_NOT_VERIFIED}} |
| Incident commander role | {{VERIFIED_ROLE_AND_ROTA_REFERENCE}} |
| Communications owner role | {{ROLE_AND_VERIFIED_CONTACT_PATH}} |
| Jira incident / related work | {{VERIFIED_REFERENCE_OR_NOT_YET_CREATED}} |
| Related runbooks | {{VERIFIED_REFERENCES_OR_NONE}} |

## Recognize the incident

{{DESCRIBE_USER_VISIBLE_SYMPTOMS_ALERT_SIGNATURES_AND_SCOPE_WITH_SOURCE_IDS}}

| Signal | Where observed | Known interpretation | Uncertainty / false positive |
| --- | --- | --- | --- |
| {{SYMPTOM_OR_ALERT}} | {{VERIFIED_OBSERVABILITY_SOURCE}} | {{SUPPORTED_INTERPRETATION}} | {{LIMIT_OR_ALTERNATIVE}} |

## First response and ownership

1. {{VERIFY_TARGET_AND_CAPTURE_INCIDENT_START_TIME_WITH_TIMEZONE}}
2. {{ASSESS_IMPACT_USING_THE_VERIFIED_SEVERITY_POLICY}}
3. {{IDENTIFY_COMMANDER_TECHNICAL_LEAD_AND_COMMUNICATIONS_OWNER}}
4. {{RECORD_OBSERVATIONS_AND_PRESERVE_RELEVANT_EVIDENCE}}

{{LINK_THE_ACTUAL_INCIDENT_RECORD_AND_SEPARATE_OBSERVATIONS_FROM_HYPOTHESES}}

## Triage questions

| Question / read-only check | Observation to collect | What the result supports | Owner |
| --- | --- | --- | --- |
| {{CHECK}} | {{OBSERVABLE_RESULT}} | {{BOUNDED_INFERENCE_NOT_PROOF}} | {{ROLE}} |

## Decision branches

| Condition supported by evidence | Next action / runbook | Required authority | Stop / escalation condition |
| --- | --- | --- | --- |
| {{CONDITION_A}} | {{VERIFIED_DIAGNOSTIC_OR_RESPONSE_REFERENCE}} | {{ROLE_OR_EXISTING_AUTHORIZATION}} | {{STOP_RULE}} |
| {{CONDITION_B}} | {{VERIFIED_DIAGNOSTIC_OR_RESPONSE_REFERENCE}} | {{ROLE_OR_EXISTING_AUTHORIZATION}} | {{STOP_RULE}} |
| Cause or target remains uncertain | {{SAFE_EVIDENCE_COLLECTION_AND_ESCALATION}} | {{INCIDENT_LEAD}} | {{DO_NOT_PROCEED_WITH_UNBOUNDED_CHANGE}} |

## Containment and mitigation

| Option / trigger | Intended effect | Side effects / reversibility | Evidence / authorization needed | Validation |
| --- | --- | --- | --- | --- |
| {{VERIFIED_OPTION}} | {{BOUNDED_EFFECT}} | {{KNOWN_COST_AND_RECOVERY_LIMIT}} | {{SOURCE_AND_AUTHORITY}} | {{OBSERVABLE_RESULT_AND_WINDOW}} |

{{STATE_WHEN_TO_STOP_AN_INEFFECTIVE_MITIGATION_AND_WHO_DECIDES_THE_NEXT_ACTION}}

## Escalation

| Trigger / elapsed condition | Responsible role / contact reference | Information to provide |
| --- | --- | --- |
| {{VERIFIED_ESCALATION_TRIGGER}} | {{VERIFIED_ROTA_OR_CONTACT_REFERENCE}} | {{IMPACT_TIMELINE_EVIDENCE_AND_ACTIONS}} |

## Communications plan

| Audience / channel reference | Owner | Trigger / cadence | Required message facts |
| --- | --- | --- | --- |
| {{AUDIENCE_AND_VERIFIED_CHANNEL}} | {{COMMUNICATIONS_OWNER}} | {{POLICY_OR_INCIDENT_AGREEMENT}} | {{IMPACT_CONFIRMED_ACTIONS_UNKNOWNS_NEXT_UPDATE}} |

{{RECORD_WHICH_MESSAGES_ARE_DRAFTED_APPROVED_OR_ACTUALLY_SENT_WITH_EVIDENCE}}

## Recovery and return to service

| Recovery gate | Evidence / observation window | Decision owner | Failure response |
| --- | --- | --- | --- |
| {{SERVICE_AND_USER_PATH_RECOVERED}} | {{VERIFIED_HEALTH_SIGNALS_AND_WINDOW}} | {{OWNER}} | {{REENTER_TRIAGE_OR_ESCALATE}} |
| {{TEMPORARY_MITIGATION_REMOVAL}} | {{SAFE_REMOVAL_CRITERION_OR_DEFERRED}} | {{OWNER}} | {{RETAIN_AND_TRACK_FOLLOW_UP}} |

## Follow-up and verification history

| Follow-up / review | Owner / due | Jira / evidence |
| --- | --- | --- |
| {{POST_INCIDENT_REVIEW_OR_PLAYBOOK_CORRECTION}} | {{OWNER_AND_DATE}} | {{VERIFIED_REFERENCE_OR_PENDING}} |

| Version / verified date | Method / verifier | Finding / next review trigger |
| --- | --- | --- |
| {{VERSION_AND_DATE_OR_NOT_VERIFIED}} | {{TABLETOP_TEST_OR_ACTUAL_USE_AND_VERIFIER}} | {{LIMITATION_OR_TRIGGER}} |

## Sources and unknowns

| Source / revision | Supported fact or step | Uncertainty / owner to resolve |
| --- | --- | --- |
| {{VERIFIED_REFERENCE_OR_SOURCE_ID}} | {{SUPPORTED_CONTENT}} | {{UNKNOWN_ASSUMPTION_OR_LIMIT_AND_OWNER}} |
