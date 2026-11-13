# {{outcome_focused_title}}

> Fill every field from supplied evidence. Use `Unknown — {{question}}` for missing facts, `TBD` for an unconfirmed owner, and `N/A — {{reason}}` only when a field does not apply. Do not invent issue keys, files, metrics, dates, or results. Preserve AC IDs when revising the ticket.

- **Type:** Story
- **Readiness assessment (not Jira status):** Draft — {{readiness_reason_and_material_gaps}}
- **Owner:** {{confirmed_owner_or_TBD}}
- **Target context:** {{confirmed_product_repository_service_revision_environment_or_Unknown}}

## Business outcome

{{who_needs_which_capability_and_why_it_matters}}

**Success signal:** {{observable_user_outcome_and_existing_measure_or_Unknown}}

## Context, evidence, and decisions

| Anchor | Source and exact location | What it establishes / limits |
| --- | --- | --- |
| S1 | {{supplied_spec_link_section_or_user_statement}} | {{supported_requirement_and_limitations}} |

- **Confirmed behavior and rules:** {{source_backed_rules_including_tenant_or_permission_boundaries}}
- **Proposals requiring a decision:** {{proposed_behavior_with_rationale_or_None}}
- **Unknowns:** {{material_questions_their_decision_owner_or_TBD_and_required_evidence}}

## Scope

- **In:** {{specific_user_flow_and_boundaries}}
- **Out:** {{adjacent_flows_or_changes_excluded}}
- **Compatibility / existing behavior:** {{behavior_that_must_be_preserved_and_source_or_Unknown}}

## Acceptance and verification

Keep criteria observable and independent of a preferred implementation. Add boundary, negative, and tenant/permission cases where relevant. An unknown verification command stays unknown until the target is inspected.

| AC ID | Observable pass condition | Verification method and expected evidence | Actual evidence / result |
| --- | --- | --- | --- |
| AC-01 | Given {{starting_state}}, when {{action}}, then {{observable_outcome}}. | {{test_or_review_method_and_required_output}} | NOT RUN |
| AC-02 | Given {{boundary_or_denied_case}}, when {{action}}, then {{expected_result_without_unwanted_effect}}. | {{test_or_review_method_and_required_output}} | NOT RUN |
| AC-03 | {{existing_behavior_or_cross_tenant_isolation_remains_true}}. | {{regression_or_isolation_check_and_required_output}} | NOT RUN |

## Dependencies and blockers

{{confirmed_dependencies_and_order_or_None_identified_with_limits; unresolved_decisions_and_owners_or_TBD}}

## Handoff and definition of done

Acceptance criteria describe this story's behavior. A team's Definition of Done describes its existing completion policy; neither replaces the other. Include only applicable, source-backed policy checks, not a default production deployment or invented sign-off.

| Applicable team completion check | Policy / revision | Required evidence | Actual result |
| --- | --- | --- | --- |
| {{check_or_no_policy_supplied}} | {{source_or_Unknown}} | {{evidence_required_or_Unknown}} | NOT VERIFIED |

A missing mandatory policy can block implementation handoff; it need not block initial triage. A check or policy reference does not authorize deployment or another live action.

- Keep **Draft / Needs discovery** while material business rules, tenant/security boundaries, exact target context, or required supporting proof are unresolved. Assess **Ready for implementation** only with those gaps resolved and a feasible verification plan; this does not imply that tests have run.
- Hand off the agreed scope, source anchors, stable AC IDs, and confirmed target/revision to {{executor_or_TBD}}.
- Completion requires an artifact/revision, AC-to-result evidence, recorded limitations, and review by {{reviewer_or_TBD}}. A failed or unrun required check remains open.
- Ticket authors prepare the work; this ticket alone does not authorize code execution or live changes.
