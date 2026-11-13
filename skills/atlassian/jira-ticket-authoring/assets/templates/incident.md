# {{active_or_reported_degradation_and_affected_user_outcome}}

> Record facts as they become available; label estimates and hypotheses. Use `Unknown — {{question}}`, `TBD` for unconfirmed owners, and `N/A — {{reason}}` as appropriate. Every timestamp needs a timezone. Do not invent severity, impact duration, recovery, service targets, or response actions. Preserve AC IDs.

- **Type:** Incident
- **Readiness assessment (not Jira status):** Draft — {{triage_or_investigation_readiness_and_gaps}}
- **Operational state:** {{Unknown_Reported_active_Confirmed_active_Mitigated_Recovery_pending_validation_or_Recovered_with_evidence}}
- **Incident owner / review recipient:** {{confirmed_owner_or_TBD}}
- **Affected context:** {{known_services_environments_revisions_and_tenant_scope_or_Unknown}}
- **Severity / urgency:** {{confirmed_assessment_with_policy_source_or_Unknown}}

## User and business impact

- **Reported degradation:** {{what_users_cannot_do_or_observe_with_source}}
- **Confirmed impact:** {{evidenced_affected_workflow_population_and_extent_or_Unknown}}
- **Estimated / suspected impact:** {{estimate_method_assumptions_and_confidence_or_None}}
- **Impact window:** Start {{timestamp_with_timezone_or_Unknown}}; end {{timestamp_with_timezone_Ongoing_or_Unknown}}. {{Label_each_boundary_confirmed_or_estimated_and_cite_its_source}}.
- **Expected service behavior:** {{existing_contract_or_service_target_with_source_or_Unknown; no_invented_threshold}}

## Evidence and timeline

| Time and timezone | Event / observation | Reported, confirmed, or proposed | Source / limitation |
| --- | --- | --- | --- |
| {{timestamp_with_timezone_or_Unknown}} | {{detection_user_report_diagnostic_mitigation_or_recovery_event}} | {{evidence_state}} | {{exact_sanitized_anchor_and_limit}} |

- **Established findings:** {{source_backed_observations}}
- **Working hypotheses:** {{unverified_explanations_and_discriminating_checks_or_Unknown}}
- **Unknowns / blockers:** {{target_scope_business_tenant_rules_missing_proof_or_access_with_owner_or_TBD}}

## Mitigation and recovery record

| Action | State | Exact target / authorization source | Evidence of effect / limits |
| --- | --- | --- | --- |
| {{candidate_or_reported_action}} | {{Proposed_Not_performed_or_Performed_with_source}} | {{confirmed_target_and_existing_authorization_or_Unknown}} | {{actual_observation_or_NOT_RUN}} |

- **Recovery condition:** {{observable_restored_behavior_and_required_check_or_observation_period_from_existing_policy_or_Unknown}}
- **Recovery verification:** {{method_target_expected_evidence_and_actual_result_or_NOT_RUN}}
- **Residual impact / recovery work:** {{remaining_affected_users_data_backlog_or_Unknown}}
- **Risks / reversal or recovery plan:** {{evidenced_limits_and_relevant_plan_or_NA_with_reason}}
- **Dependencies / next owner:** {{required_decisions_access_or_teams_with_confirmed_owner_or_TBD}}

## Communication — DRAFT, NOT SENT

{{plain_language_summary_of_confirmed_impact_current_state_and_next_step; clearly_label_uncertainty; include_next_update_time_only_if_committed_with_source}}

**Audience / approved communication route:** {{confirmed_audience_and_route_or_Unknown}}

## Incident acceptance and verification

These criteria cover the incident record and recovery assessment. A permanent fix or postmortem has separate scope.

| AC ID | Observable condition | Verification method and expected evidence | Actual evidence / result |
| --- | --- | --- | --- |
| AC-01 | {{impact_scope_and_window_are_supported_or_remaining_limits_are_explicit}}. | {{review_of_timezoned_timeline_sources_and_estimate_labels}} | NOT RUN |
| AC-02 | {{agreed_recovery_conditions_are_met_and_residual_impact_is_accounted_for}}. | {{service_and_user_outcome_checks_with_target_time_and_actual_outputs}} | NOT RUN |
| AC-03 | {{unresolved_causes_and_follow_up_work_have_a_recorded_disposition_and_owner}}. | {{review_of_follow_up_links_or_proposed_tickets_and_human_recovery_assessment}} | NOT RUN |

## Handoff and follow-up

- Assess **Ready for triage** when responders can understand the reported degradation and known evidence. **Ready for investigation** requires a bounded target and permitted methods; neither assessment means **Ready for implementation** or permission to act on live systems.
- **Human recovery / closure assessment:** {{Pending_or_confirmed_decision_with_source}}. A mitigation attempt, quieter alert, or untested hypothesis is not recovery proof.
- **Separate follow-ups:** {{postmortem_permanent_fix_data_repair_or_observability_ticket_links_or_proposed_titles_with_owner_or_TBD}}. Do not silently expand incident response into those projects.
- The author records evidence and drafts handoffs. Writing this ticket does not authorize mitigation, deployment, data repair, external communication, or other live response actions.
