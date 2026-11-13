# {{cross_ticket_business_outcome}}

<!-- An epic coordinates child work; it is not one executable agent assignment. Use `Unknown — {{question}}`, `TBD` for unconfirmed owners, and `N/A — {{reason}}` as appropriate. Use proposed child names until real issue keys exist. Do not invent ownership, releases, dates, or results. Keep AC IDs stable. -->

<!-- Default authoring readiness to Draft until an evidence-backed assessment justifies another applicable verdict. -->

- **Content pattern:** Epic
- **Configured Jira work type:** {{AVAILABLE_PROJECT_WORK_TYPE_OR_UNRESOLVED}} — map this pattern to the target project configuration.
- **Jira workflow status:** {{KNOWN_CURRENT_STATUS_OR_UNSPECIFIED}} — separate from authoring readiness.
- **Authoring readiness (not Jira status):** {{DRAFT_OR_NEEDS_DECOMPOSITION}} — {{decomposition_and_decision_gaps}}
- **Owner:** {{confirmed_epic_owner_or_TBD}}
- **Target context:** {{confirmed_products_repositories_services_revisions_or_Unknown}}

## Business outcome

{{user_or_business_outcome_across_the_children_and_why_it_matters}}

**Success signal:** {{agreed_observable_outcome_and_source_or_Unknown}}

## Evidence, scope, and unknowns

- **Source anchors:** {{S1_spec_or_request_location; other_exact_evidence_anchors}}
- **Confirmed rules:** {{business_compatibility_tenant_and_permission_contracts_with_sources}}
- **In:** {{bounded_capabilities_and_integrations}}
- **Out:** {{excluded_capabilities_migrations_or_operational_actions}}
- **Material unknowns / decision owners:** {{question_required_evidence_and_owner_or_TBD}}

## Child breakdown and dependency order

Each child needs its own ticket, target/revision, owner, AC IDs, and readiness assessment. The labels below are local planning labels, not Jira issue keys.

| Child label / real issue link if supplied | Type and bounded outcome | Depends on | Owner | Readiness / blocker |
| --- | --- | --- | --- | --- |
| C1 — {{child_name}} | {{type_and_deliverable}} | {{None_or_prerequisite}} | {{owner_or_TBD}} | Draft — {{gap_or_evidence}} |
| C2 — {{child_name}} | {{type_and_deliverable}} | {{C1_or_other_confirmed_dependency}} | {{owner_or_TBD}} | Draft — {{gap_or_evidence}} |
| C3 — {{integration_or_validation_child}} | {{type_and_cross_ticket_result}} | {{required_children}} | {{owner_or_TBD}} | Draft — {{gap_or_evidence}} |

## Cross-ticket contracts

| Producer → consumer | Shared interface / invariant | Source or decision needed | Integration proof expected |
| --- | --- | --- | --- |
| {{child_or_component}} → {{child_or_component}} | {{schema_behavior_versioning_tenant_boundary_or_ordering_contract}} | {{anchor_or_Unknown_with_decision_owner}} | {{cross_child_check_and_expected_artifact}} |

**Dependency order and parallel work:** {{which_children_must_finish_first_which_can_overlap_and_why}}

### External dependency gates (when applicable)

| External item / verified reference | Responsible role / confirmed owner | Required input and evidence | Needed by: forecast or commitment / source | Affected children / consequence if blocked |
| --- | --- | --- | --- | --- |
| {{item_or_Unknown}} | {{role_or_TBD}} | {{input_and_observable_gate}} | {{date_type_and_authority_or_Unknown}} | {{child_labels_and_consequence}} |

Use evidence-backed commitments only; a proposed date or owner is not an agreement. Keep this table only when external coordination affects the children.

## Epic acceptance and verification

| AC ID | Observable cross-ticket pass condition | Verification method and expected evidence | Actual evidence / result |
| --- | --- | --- | --- |
| AC-01 | {{end_to_end_business_outcome_is_met_across_named_children}}. | {{integrated_check_and_expected_output}} | NOT RUN |
| AC-02 | {{shared_contract_and_tenant_permission_or_compatibility_boundary_hold}}. | {{cross_component_contract_or_isolation_check}} | NOT RUN |
| AC-03 | {{required_handoffs_and_operational_ownership_are_resolved}}. | {{review_of_child_evidence_and_outstanding_blockers}} | NOT RUN |

## Handoff and definition of done

- **Dependencies / blockers:** {{external_dependencies_and_unresolved_decisions_with_owner_or_TBD}}
- Use **Draft** or **Needs decomposition** for this coordination artifact and report readiness separately for each child; do not present the epic as one implementation assignment.
- Material business rules, tenant/security boundaries, exact targets, or required proof block the affected children from **Ready for implementation**. One child's readiness does not make its dependents ready.
- Completion requires child artifact/revision links, actual cross-ticket AC evidence, resolved or explicitly accepted remaining work, and any review required by the existing project process (reviewer: {{known_reviewer_or_unassigned_if_required}}). A list of closed child tickets alone is not outcome evidence.
- The author decomposes and drafts work; implementation and live operations require separately scoped authorization.
