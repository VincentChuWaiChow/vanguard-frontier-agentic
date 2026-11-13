# {{customer_problem_in_plain_language}}

> Keep the customer's report separate from confirmed findings. Use `Unknown — {{question}}`, `TBD` for unconfirmed owners, and `N/A — {{reason}}` where appropriate. Sanitize customer identifiers, payloads, and diagnostics. Do not infer a product defect, promised response time, or resolution. Preserve AC IDs.

- **Type:** Support case
- **Readiness assessment (not Jira status):** Draft — {{triage_or_investigation_readiness_and_gaps}}
- **Owner:** {{confirmed_support_owner_or_TBD}}
- **Customer / case anchor:** {{permitted_sanitized_reference_or_Unknown}}
- **Product / target context:** {{known_product_version_environment_and_tenant_boundary_or_Unknown}}
- **Service SLA / response commitment:** {{supplied_policy_or_commitment_with_source_or_Unknown}}

## Customer need and business impact

- **Requested outcome:** {{what_the_customer_needs_to_accomplish}}
- **Reported symptom:** {{customer_report_with_source}}
- **Confirmed symptom:** {{independently_supported_finding_with_evidence_or_Not_yet_confirmed}}
- **Expected result:** {{customer_expectation_and_product_contract_sources; identify_any_difference}}
- **Impact:** {{confirmed_affected_work_users_or_data; reported_or_suspected_extent_separately}}
- **Urgency:** {{customer_deadline_or_time_sensitivity_with_source_or_Unknown}}
- **Priority / severity:** {{confirmed_policy_based_assessment_or_Unknown; do_not_equate_urgency_with_blast_radius}}

## Evidence and safe reproduction

| Anchor | Sanitized source and exact location | Finding / limitation |
| --- | --- | --- |
| S1 | {{customer_report_screenshot_diagnostic_or_contract_anchor}} | {{what_it_establishes_and_cannot_establish}} |

- **Reproduction state:** {{Not_attempted_Reproduced_or_Not_reproduced_with_evidence}}
- **Safe preconditions and steps:** {{confirmed_test_context_minimal_actions_and_synthetic_inputs_or_Unknown}}
- **Expected versus actual:** {{source_backed_expectation_and_observation_or_NOT_RUN}}
- **Additional diagnostics needed:** {{minimal_sanitized_evidence_and_purpose_or_None; do_not_request_secrets}}

## Classification, workaround, and escalation

- **Working classification:** {{Product_defect_Configuration_Access_How_to_Feature_gap_or_Unresolved}} — {{evidence_and_confidence}}.
- **Workaround:** {{known_option_or_None_known}}. **Evidence / limits:** {{tested_context_and_result_or_Proposed_unverified_NOT_RUN}}.
- **Material unknowns / blockers:** {{business_rules_tenant_boundary_target_or_missing_proof_with_owner_or_TBD}}
- **Escalation decision:** {{not_needed_proposed_or_confirmed_route_with_reason_evidence_and_next_owner}}
- **Linked follow-up:** {{existing_bug_feature_service_request_or_incident_link_or_None; proposed_title_if_not_created}}. Create a bug draft only when the evidence warrants investigating a product defect; a support complaint alone is not defect proof.

## Customer update — DRAFT, NOT SENT

{{plain_language_update_stating_what_is_known_what_remains_uncertain_and_the_next_useful_step; include_timing_only_if_supported_by_an_actual_commitment}}

## Resolution acceptance and verification

| AC ID | Observable customer outcome | Verification method and expected evidence | Actual evidence / result |
| --- | --- | --- | --- |
| AC-01 | {{customer_can_complete_the_agreed_workflow_or_receives_the_agreed_answer}}. | {{safe_check_or_customer_validation_and_required_evidence}} | NOT RUN |
| AC-02 | {{relevant_data_tenant_permission_and_existing_behavior_boundaries_hold}}. | {{focused_check_or_review_with_sanitized_result}} | NOT RUN |
| AC-03 | {{customer_confirms_resolution_or_an_applicable_documented_closure_rule_is_met}}. | {{customer_confirmation_or_exact_policy_and_evidence_of_its_conditions}} | NOT RUN |

## Handoff and closure

- Assess **Ready for triage** when the report, available evidence, and requested outcome are understandable. Assess **Ready for investigation** only with a bounded target, permitted methods, and an evidence plan. Neither is **Ready for implementation**; an engineering follow-up needs its own readiness assessment.
- **Next action / owner:** {{bounded_action_and_confirmed_owner_or_TBD}}
- **Resolution evidence / customer confirmation:** {{actual_evidence_or_Pending}}
- Routing, sending an update, offering an untested workaround, or opening a bug does not by itself resolve the case. Keep it unresolved when required outcome evidence or confirmation is missing; cite any applicable closure exception instead of inventing one.
- The author drafts the case and communication; they do not send messages, perform live changes, or claim checks were run while writing it.
