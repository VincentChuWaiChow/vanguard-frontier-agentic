# {{CONCISE_OBSERVED_FINDING}}

<!-- Remove comments, unused optional sections, and unresolved placeholders from the final ticket. Capturing a finding preserves evidence; it does not expand the originating task or authorize a fix. Keep a suspected problem distinct from a demonstrated defect. -->

**Content pattern:** Finding / technical debt capture.
**Configured Jira work type:** {{AVAILABLE_PROJECT_WORK_TYPE_OR_UNRESOLVED}}.
**Authoring readiness:** {{DRAFT_OR_NEEDS_CLARIFICATION_OR_READY_FOR_INVESTIGATION}}.
**Jira workflow status:** {{KNOWN_CURRENT_STATUS_OR_UNSPECIFIED}} — separate from authoring readiness.
**Originating work:** {{KNOWN_WORK_ITEM_OR_SESSION_CONTEXT}}.

## Finding and reason to retain it

During {{BOUNDED_ORIGINATING_ACTIVITY}}, {{OBSERVATION}} was found at {{VERIFIED_LOCATION_OR_REPORTED_LOCATION_LABELLED_AS_SUCH}}.

Retain this finding because {{KNOWN_OR_POSSIBLE_IMPACT_DISTINGUISHED_EXPLICITLY}}. The capture outcome is an evidence-backed record and a proposed next step, not an unrequested repair.

## Facts and open decisions

- **Verified source facts:** {{DIRECT_OBSERVATIONS_WITH_LOCATIONS_AND_RELEVANT_CONTEXT_OR_NONE_CHECKED}}.
- **Reported facts:** {{CLAIMS_NOT_INDEPENDENTLY_CHECKED_AND_THEIR_SOURCES_OR_NONE}}.
- **Assumptions:** {{POSSIBLE_EXPLANATIONS_OR_IMPACT_ASSUMPTIONS_AND_EFFECT_IF_FALSE_OR_NONE}}.
- **Questions:** {{WHAT_REMAINS_UNKNOWN_OWNER_IF_KNOWN_AND_EFFECT_ON_NEXT_WORK}}.

## Evidence and impact

- **Evidence:** {{SANITIZED_OBSERVATION_LOG_SNIPPET_ARTIFACT_OR_REPRODUCTION_AND_WHAT_IT_PROVES}}.
- **Conditions:** {{KNOWN_ENVIRONMENT_VERSION_INPUT_OR_STATE_OR_UNKNOWN}}.
- **Reproduction status:** {{REPRODUCED_REPORTED_ONLY_NOT_ATTEMPTED_OR_NOT_APPLICABLE_WITH_REASON}}.
- **Demonstrated impact:** {{OBSERVED_EFFECT_OR_NOT_YET_ESTABLISHED}}.
- **Potential impact:** {{EXPLICITLY_LABELLED_RISK_OR_HYPOTHESIS_WITH_BASIS_OR_NONE}}.
- **Related or duplicate record:** {{VERIFIED_EXISTING_KEY_OR_NOT_CHECKED_OR_SEARCH_SCOPE_WITH_NO_MATCH}}.

## Scope and relationship to current work

- **In scope for this capture:** {{FACTS_AND_CONTEXT_TO_PRESERVE}}.
- **Out of scope:** {{REPAIR_REDESIGN_OR_OTHER_ACTION_NOT_INCLUDED_IN_THE_ORIGINATING_TASK}}.
- **Effect on originating work:** {{BLOCKS_WITH_REASON_OR_CAN_CONTINUE_WITH_REASON_OR_UNRESOLVED}}.
- **Proposed next action:** {{BOUNDED_TRIAGE_INVESTIGATION_OR_REPAIR_PROPOSAL}} — label it as proposed until its scope is established.

## Capture acceptance criteria

1. **AC-01 — Evidence is recoverable:** The record includes {{OBSERVATION_AND_LOCATION}} with {{CONDITIONS_AND_SOURCE_DETAIL}} sufficient for another person or agent to find the evidence, and labels anything not independently checked.
2. **AC-02 — Disposition is explicit:** The record states {{KNOWN_IMPACT_OR_UNCERTAINTY}}, its relationship to {{ORIGINATING_WORK}}, and {{BOUNDED_NEXT_ACTION_OR_OPEN_TRIAGE_QUESTION}} without claiming an unperformed fix.

## Verification plan

| Criterion | Review or check | Pass condition | Evidence to retain | Evidence status |
|---|---|---|---|---|
| AC-01 | {{CHECK_RECORD_AGAINST_AVAILABLE_SOURCE_EVIDENCE}} | {{OBSERVATION_LOCATION_AND_UNCERTAINTY_ARE_TRACEABLE}} | {{SOURCE_REFERENCE_AND_REVIEW_NOTE}} | Not run |
| AC-02 | {{CHECK_SCOPE_RELATIONSHIP_AND_PROPOSED_DISPOSITION}} | {{IMPACT_LIMITS_AND_NEXT_ACTION_ARE_EXPLICIT}} | {{DISPOSITION_AND_ANY_VERIFIED_RELATED_RECORD}} | Not run |

## Dependencies and readiness

{{DEPENDENCIES_OR_EXPLICIT_NONE_OR_NOT_YET_CHECKED}}. State any evidence, access, owner decision, or known issue needed for the proposed next step. Keep the originating task, parent hierarchy, and blocking relationship distinct.

**Readiness reason:** {{SPECIFIC_GAP_OR_EVIDENCE_THAT_A_BOUNDED_INVESTIGATION_CAN_BEGIN}}. Ready for investigation does not mean a repair is scoped or complete.

## Completion evidence for capture

- **Evidence preserved:** {{ACTUAL_RECORD_AND_SOURCE_REFERENCES_TO_ADD_AFTER_CAPTURE}}.
- **Verification results:** Not run — replace with observed checks of AC-01 and AC-02.
- **Disposition:** {{ACTUAL_TRIAGE_DECISION_OR_EXPLICITLY_UNDECIDED_PROPOSAL}}.
- **Follow-up:** {{VERIFIED_FOLLOWUP_KEY_IF_CREATED_OR_UNCREATED_PROPOSAL}}.

<!-- OPTIONAL: retain if a potential fix is useful to triage; it remains a hypothesis until investigated. -->
## Optional — Possible remediation

{{PROPOSED_CHANGE_WITH_SUPPORTING_REASON_AND_UNKNOWNS}}. Identify evidence needed before selecting this approach and any compatibility risks it might introduce.
