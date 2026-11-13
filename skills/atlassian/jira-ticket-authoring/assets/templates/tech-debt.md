# {{bounded_debt_reduction_and_result}}

> Establish why the debt matters before proposing cleanup. Use `Unknown — {{question}}`, `TBD` for unconfirmed owners, and `N/A — {{reason}}` where appropriate. Do not invent cost savings, performance targets, file paths, deadlines, or test results. Preserve AC IDs.

- **Type:** Technical debt
- **Readiness assessment (not Jira status):** Draft — {{readiness_reason_and_material_gaps}}
- **Owner:** {{confirmed_owner_or_TBD}}
- **Target context:** {{confirmed_repository_component_revision_environment_or_Unknown}}

## Business outcome and current cost

{{which_delivery_operational_or_user_problem_is_reduced_and_why_now}}

- **Observed debt:** {{specific_existing_condition_with_source}}
- **Confirmed cost or risk:** {{measured_or_observed_effect_with_evidence_or_Unknown}}
- **Suspected benefit:** {{clearly_labeled_hypothesis_and_how_to_check_it_or_None}}
- **Success signal:** {{observable_improvement_and_existing_measure_or_Unknown; no_arbitrary_target}}

## Evidence and decisions

| Anchor | Source and exact location | Established fact / limitation |
| --- | --- | --- |
| S1 | {{repository_revision_incident_or_delivery_evidence_anchor}} | {{what_the_source_demonstrates}} |

- **Behavior to preserve:** {{public_contract_business_rules_tenant_and_permission_boundaries_with_sources}}
- **Unknowns / decisions:** {{question_required_evidence_and_decision_owner_or_TBD}}
- **Proposed approach:** {{optional_approach_and_rationale_separate_from_required_constraints}}

## Scope

- **In:** {{small_coherent_debt_reduction_with_explicit_boundaries}}
- **Out:** {{feature_changes_adjacent_refactors_and_other_cleanup}}
- **Dependencies / blockers:** {{known_prerequisites_and_order_or_Unknown}}

## Acceptance and verification

| AC ID | Observable pass condition | Verification method and expected evidence | Actual evidence / result |
| --- | --- | --- | --- |
| AC-01 | {{named_debt_condition_is_removed_or_reduced_to_the_agreed_state}}. | {{before_and_after_inspection_or_measurement_using_the_same_method}} | NOT RUN |
| AC-02 | {{supported_external_behavior_and_tenant_permission_contracts_are_preserved}}. | {{focused_regression_or_contract_checks_and_expected_results}} | NOT RUN |
| AC-03 | {{migration_compatibility_or_operability_requirement_is_met_or_NA_with_reason}}. | {{relevant_verification_and_expected_artifact_or_NA_with_reason}} | NOT RUN |

## Handoff and definition of done

- Keep **Draft / Needs discovery** while material business rules, tenant/security boundaries, exact target, or required baseline proof are unresolved. Assess **Ready for implementation** only once the bounded improvement and verification plan are supported.
- Hand off to {{executor_or_TBD}} with the baseline, preserved contracts, exact target/revision, and dependencies.
- Completion requires an artifact/revision, actual evidence against every applicable AC, an honest account of observed improvement, and review by {{reviewer_or_TBD}}. Do not claim savings or reliability gains that were not measured.
- The ticket author prepares the scope and handoff; they do not perform the refactor while drafting.
