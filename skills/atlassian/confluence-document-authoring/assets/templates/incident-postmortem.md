# Incident postmortem: {{incident_title}}

| Metadata | Value |
|---|---|
| Audience / publication scope | {{business_support_engineering_or_customer_readers}} / {{visibility_and_restrictions_or_unknown}} |
| Incident owner / review facilitator | {{owner_or_unknown}} / {{facilitator_or_unknown}} |
| Document lifecycle / review trigger | draft / {{review_date_new_evidence_or_follow_up_trigger_or_unknown}} |
| Revision / last reviewed | {{revision_or_unknown}} / {{date_or_unknown}} |
| Sources / revisions / dates | {{incident_log_telemetry_case_or_change_sources_revisions_dates_or_unknown}} |
| Product / versions / availability impact | {{product}} / {{affected_versions_or_unknown}} / {{verified_impact_scope_or_unknown}} |
| Incident / Jira / related documentation | {{incident_reference_or_unknown}} / {{jira_links_or_unknown}} / {{runbook_status_or_other_links_or_unknown}} |

Use **observed**, **reported**, **confirmed**, **hypothesis**, and **unknown** accurately. Preserve uncertainty; unknown impact is not zero impact. This page reviews an incident and does not execute response actions or authorize changes. Remove this authoring note from the finished document.

## Summary and current state

{{Plain_language_description_of_what_users_experienced_what_was_done_and_what_is_known_now}}.

- Incident state: {{ongoing_mitigated_recovered_or_unknown_with_evidence}}.
- Severity: {{recorded_severity_authority_and_source_or_unknown}}.
- Start / detection / mitigation / recovery: {{timestamps_and_timezone_with_sources_or_unknown}}.
- Remaining user or business effect: {{known_effect_or_unknown}}.

Do not equate reduced symptoms with full recovery. Name the checks and observation window that support the recorded recovery state.

## Impact

| Dimension | Confirmed or reported impact | Scope / measurement definition | Evidence / uncertainty |
|---|---|---|---|
| Users / customers / regions | {{impact_or_unknown}} | {{population_and_window_or_unknown}} | {{source_and_observed_or_reported_status}} |
| Tasks / requests / availability | {{impact_or_unknown}} | {{metric_denominator_and_window_or_unknown}} | {{source_or_unknown}} |
| Data / financial / contractual effects | {{verified_effect_or_unknown}} | {{scope_or_unknown}} | {{authoritative_evidence_or_unknown}} |

## Timeline

Timezone: {{timezone_and_source_or_unknown}}. Use absolute timestamps where known; mark estimates and event ordering uncertainty. Preserve the source timezone when conversion cannot be verified.

| Timestamp / timezone | Event or observation | Evidence / provenance | Certainty |
|---|---|---|---|
| {{time_or_unknown}} | {{first_known_user_impact}} | {{source_reference}} | {{observed_reported_or_estimated}} |
| {{time_or_unknown}} | {{detection_and_initial_response}} | {{source_reference}} | {{certainty}} |
| {{time_or_unknown}} | {{mitigation_action_and_observed_effect}} | {{source_reference}} | {{certainty}} |
| {{time_or_unknown}} | {{recovery_check_and_result}} | {{source_reference_or_unknown}} | {{certainty_or_unknown}} |

## Detection, mitigation, and recovery

- How it was detected and by whom: {{signal_or_report_and_evidence_or_unknown}}.
- Detection gap: {{supported_gap_or_hypothesis_with_missing_evidence}}.
- Mitigation performed: {{recorded_action_target_executor_and_evidence_or_unknown}}.
- Effect of mitigation: {{observed_change_and_measurement_or_unknown}}.
- Recovery verification: {{checks_results_scope_and_observation_window_or_unknown}}.
- Customer or stakeholder communication: {{sent_message_reference_and_verified_time_or_unknown}}.
- Response procedure reference: {{runbook_or_playbook_and_revision_or_unknown}}.

## Causal analysis

| Statement | Classification | Supporting evidence | Missing / contradictory evidence | Next investigation |
|---|---|---|---|---|
| {{mechanism_that_produced_impact}} | {{confirmed_cause_or_hypothesis}} | {{evidence_link_or_unknown}} | {{gap_or_not_applicable_with_reason}} | {{action_or_unknown}} |
| {{condition_that_increased_likelihood_or_impact}} | {{confirmed_contributing_factor_or_hypothesis}} | {{evidence_link_or_unknown}} | {{gap_or_unknown}} | {{action_or_unknown}} |

Explain the mechanism and the evidence that distinguishes it from alternatives. Timing alone does not prove causation. Describe system conditions, available information, and process gaps without attributing blame or intent to individuals.

## What worked and what did not

| Area | Observation / evidence | Effect on the response | Improvement or retained practice |
|---|---|---|---|
| {{detection_coordination_mitigation_or_recovery}} | {{observation_and_source}} | {{supported_effect_or_unknown}} | {{proposed_action_or_retained_practice}} |

## Follow-up actions

| Action / risk addressed | Owner | Due / review trigger | Completion proof | Jira / state |
|---|---|---|---|---|
| {{specific_prevent_detect_limit_or_recover_action}} | {{owner_or_unknown}} | {{agreed_date_event_or_unknown}} | {{observable_evidence_needed_to_close}} | {{issue_link_or_unknown}} / {{proposed_in_progress_or_verified_complete}} |

Separate proposals from assigned commitments. Closure requires the stated proof, not only a completed issue status. If no action is selected for a known risk, record the rationale and decision authority or **unknown**.

## Open questions and review record

- Material uncertainty: {{question_owner_and_evidence_needed_or_unknown}}.
- Review participants and date: {{participants_and_date_or_unknown}}.
- Review outcome / approval evidence: {{recorded_outcome_and_source_or_pending}}.
- Next review trigger: {{follow_up_event_date_or_unknown}}.
