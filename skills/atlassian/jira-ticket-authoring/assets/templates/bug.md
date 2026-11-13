# {{observable_failure_in_a_specific_flow}}

<!-- Separate reports, observations, and hypotheses. Use `Unknown — {{question}}` for missing facts, `TBD` for an unconfirmed owner, and `N/A — {{reason}}` for inapplicable fields. Sanitize credentials, personal data, customer payloads, and logs. Keep AC IDs stable; do not invent a reproduction or test result. -->

<!-- Default authoring readiness to Draft until an evidence-backed assessment justifies another applicable verdict. -->

- **Content pattern:** Bug
- **Configured Jira work type:** {{AVAILABLE_PROJECT_WORK_TYPE_OR_UNRESOLVED}} — map this pattern to the target project configuration.
- **Jira workflow status:** {{KNOWN_CURRENT_STATUS_OR_UNSPECIFIED}} — separate from authoring readiness.
- **Authoring readiness (not Jira status):** {{DRAFT_OR_NEEDS_CLARIFICATION_OR_NEEDS_DECOMPOSITION_OR_READY_FOR_TRIAGE_OR_READY_FOR_IMPLEMENTATION}} — {{readiness_reason_and_material_gaps}}
- **Owner:** {{confirmed_owner_or_TBD}}
- **Target context:** {{confirmed_repository_service_revision_environment_or_Unknown}}

## Business outcome and impact

{{which_user_or_business_process_should_recover_and_why}}

- **Confirmed impact:** {{evidence_backed_affected_behavior_population_and_extent_or_Unknown}}
- **Suspected impact:** {{unverified_risk_and_why_it_is_suspected_or_None}}
- **Severity / priority:** {{confirmed_value_and_basis_or_Unknown; do_not_infer_from_urgency_alone}}

## Observed versus expected

- **Reported or observed:** {{failure_with_explicit_reported_or_independently_observed_label}}
- **Expected:** {{source_backed_contract_or_Unknown}}
- **Source anchors:** {{S1_request_or_spec_location; S2_sanitized_log_trace_test_or_artifact_location_and_revision}}

## Sanitized reproduction

- **Reproduction state:** {{Not_attempted_or_Reproduced_or_Not_reproduced_with_evidence}}
- **Preconditions:** {{environment_version_test_identity_tenant_boundary_and_safe_fixture_or_Unknown}}
- **Steps:** {{minimal_ordered_actions_or_Unknown}}
- **Actual result:** {{observed_result_with_evidence_or_NOT_RUN}}
- **Expected result:** {{assertable_result_and_contract_source_or_Unknown}}
- **Evidence gaps:** {{missing_steps_artifacts_access_or_safe_fixture}}

## Diagnosis boundary and unknowns

- **Established evidence:** {{facts_supported_by_the_anchors_above}}
- **Root-cause hypotheses:** {{explicitly_unverified_explanation_and_falsifying_check_or_Unknown}}
- **Material questions:** {{business_rules_tenant_scope_exact_target_and_proof_still_needed; decision_owner_or_TBD}}

## Scope

- **In:** {{specific_failure_path_and_required_regression_coverage}}
- **Out:** {{unrelated_cleanup_new_features_historical_data_repair_or_other_exclusions}}

## Acceptance and verification

| AC ID | Observable pass condition | Verification method and expected evidence | Actual evidence / result |
| --- | --- | --- | --- |
| AC-01 | Given {{confirmed_reproduction_preconditions}}, when {{trigger}}, then {{expected_behavior}}. | {{regression_check_showing_the_original_failure_and_corrected_result}} | NOT RUN |
| AC-02 | {{relevant_retry_concurrency_failure_or_boundary_case_preserves_the_contract}}. | {{method_fixture_and_expected_assertions}} | NOT RUN |
| AC-03 | {{adjacent_supported_behavior_and_tenant_or_permission_boundary_remain_correct}}. | {{compatibility_or_isolation_check_and_expected_artifact}} | NOT RUN |

## Dependencies, handoff, and definition of done

- **Dependencies / blockers:** {{required_decisions_evidence_access_and_order_or_None_identified_with_limits}}
- Preserve an explicitly requested bug-report form. Mark the symptom as reported/unconfirmed and reproduction as Not attempted where applicable. Missing evidence blocks implementation readiness, not creation of the bug report. Record investigation as the next action; propose a separate bounded investigation only when useful, without replacing the requested report.
- Use **Ready for triage** when the report has enough context to route the next investigation. Keep implementation **Draft / Needs clarification** while material business rules, tenant/security boundaries, exact target, or required supporting proof are unresolved. Explain which activity is ready and which remains blocked.
- Hand off to {{executor_or_TBD}} only after readiness is justified. For triage completion, record the clarified symptom, evidence gaps, and next routing action. For implementation completion, provide the artifact/revision, AC-to-result evidence, any residual impact, and any review required by the existing project process (reviewer: {{known_reviewer_or_unassigned_if_required}}).
- The author drafts the ticket; they do not diagnose by changing code or live systems under this template.
