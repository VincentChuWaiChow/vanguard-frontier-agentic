# {{requested_access_configuration_or_information_and_purpose}}

> A request is not an approval or proof of fulfillment. Use `Unknown — {{question}}`, `TBD` for unconfirmed owners, and `N/A — {{reason}}` where appropriate. Do not invent authority, privileges, service commitments, or completed actions. Sanitize identity and resource details. Preserve AC IDs.

- **Type:** Service request
- **Category:** {{Access_Configuration_Information_or_Unresolved}}
- **Readiness assessment (not Jira status):** {{Draft_or_Needs_clarification_or_Ready_for_triage_or_Ready_for_fulfillment}} — {{activity_assessed_and_evidence_or_gaps}}
- **Owner:** {{confirmed_service_owner_or_TBD}}
- **Requester / beneficiary:** {{confirmed_references_and_whether_they_are_the_same_person_or_Unknown}}
- **Service SLA / requested timing:** {{supplied_policy_or_requested_time_with_source_or_Unknown; distinguish_request_from_commitment}}

## Business purpose and requested outcome

{{what_the_requester_needs_to_accomplish_and_why_the_request_is_needed}}

- **Requested service:** {{specific_access_config_change_or_information}}
- **Exact target and boundary:** {{confirmed_resource_environment_account_tenant_scope_and_revision_if_relevant_or_Unknown}}
- **Current state:** {{evidenced_baseline_or_Unknown}}
- **Requested state:** {{precise_role_permissions_config_delta_or_information_scope; requested_duration_if_applicable_or_Unknown}}
- **In / out of scope:** {{bounded_fulfillment_and_explicit_exclusions}}

## Sources, authority, and unknowns

| Anchor | Source and exact location | What it establishes / limits |
| --- | --- | --- |
| S1 | {{request_baseline_or_service_policy_anchor}} | {{facts_supported_and_limits}} |

| Requested action / information | Required approver or applicable policy | Authorization evidence and exact scope | State |
| --- | --- | --- | --- |
| {{bounded_action}} | {{verified_approver_or_policy_or_Unknown}} | {{approval_anchor_scope_conditions_and_validity_or_Unknown}} | {{Unknown_Pending_Confirmed_or_NA_with_policy_reason}} |

Record the request separately from authorization. A requester naming a role, an agent's proposed assignment, or an existing ticket does not establish approval. Do not assume privileged access is authorized. Where no approval is required, cite the applicable service rule or explain the confirmed basis; do not invent a requirement either.

- **Material unknowns / blockers:** {{business_tenant_or_permission_rules_exact_target_authorization_or_missing_proof_with_owner_or_TBD}}
- **Dependencies / ordering:** {{known_prerequisites_or_Unknown}}

## Fulfillment plan and evidence

- **Planned method:** {{known_bounded_method_or_Unknown; proposal_only_until_executed_with_authority}}
- **Actual fulfillment:** NOT PERFORMED — {{replace_only_with_supported_action_record}}
- **Artifact / revision / effective state:** {{actual_evidence_or_Unknown}}
- **Reversal / expiry / recovery:** {{applicable_plan_and_owner_or_NA_with_reason}}
- **Requester update:** DRAFT, NOT SENT — {{plain_language_state_next_step_and_known_timing_only}}

## Acceptance and verification

| AC ID | Observable fulfilled outcome | Verification method and expected evidence | Actual evidence / result |
| --- | --- | --- | --- |
| AC-01 | {{the_requested_outcome_is_fulfilled_for_the_confirmed_subject_and_target_within_authorized_scope}}. | {{actual_state_or_delivery_record_compared_with_exact_request_and_authorization}} | NOT RUN |
| AC-02 | {{unrequested_privileges_other_tenants_and_out_of_scope_configuration_remain_unaffected}}. | {{focused_scope_review_or_safe_validation_with_sanitized_evidence}} | NOT RUN |
| AC-03 | {{requester_can_use_the_result_or_receives_and_accepts_the_agreed_information}}. | {{requester_validation_or_documented_service_acceptance_method}} | NOT RUN |

## Handoff and completion

- Assess **Ready for triage** when the requested outcome and known context can be routed. Assess **Ready for fulfillment** only when the exact target, material rules, relevant authorization, dependencies, and verification are resolved. A readiness label does not grant authority.
- If engineering changes are needed, create a separately scoped draft and assess **Ready for implementation** there; neither request submission nor triage resolves its technical blockers.
- **Executor / reviewer:** {{confirmed_people_or_roles_or_TBD}}
- **Completion / requester confirmation:** {{Pending_or_actual_evidence_and_review_decision}}. Approval without fulfillment evidence remains unfulfilled; action without outcome verification remains unverified.
- The author drafts the request and handoff. They do not grant access, change configuration, release information, send messages, or claim fulfillment while writing it.
