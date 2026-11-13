# Handoff: {{ticket_title_or_confirmed_issue_key}}

> Record what exists, what was checked, and what remains. Use `Unknown — {{question}}` when evidence is missing, `TBD` for an unconfirmed owner, and `N/A — {{reason}}` when appropriate. Copy AC IDs exactly from the source ticket. Do not convert a plan into claimed implementation or test results.

- **Prepared by:** {{confirmed_human_or_agent_identifier_or_Unknown}}
- **Source ticket / agreed scope:** {{confirmed_link_or_source_anchor_and_revision_or_Unknown}}
- **Handoff assessment (not Jira status):** {{Draft_or_Ready_for_review_or_Blocked_with_reason}}
- **Next owner / reviewer:** {{confirmed_recipient_or_TBD}}

## Intended outcome and delivered scope

- **Intended outcome:** {{agreed_business_or_user_result}}
- **Delivered:** {{actual_artifacts_or_work_completed_with_evidence_or_None}}
- **Scope deviations:** {{what_changed_from_the_agreement_why_and_decision_source_or_None}}

## Artifact and revision

| Artifact | Exact location and revision | What it establishes / limits |
| --- | --- | --- |
| {{code_document_config_or_investigation_record}} | {{confirmed_path_link_revision_or_Unknown}} | {{actual_content_and_environment_limitations}} |

## Acceptance criteria → verification → results

Use the actual command or review method only when known. Record the target/revision and environment with the evidence. Use `NOT RUN`, `PASS`, `FAIL`, `BLOCKED`, or `N/A — reason`; a command's existence is not proof that it ran or passed.

| AC ID from source | Test / review method | Expected evidence / result | Actual evidence location and result |
| --- | --- | --- | --- |
| AC-01 | {{actual_method_or_Unknown}} | {{agreed_assertion_and_output}} | NOT RUN |
| AC-02 | {{actual_method_or_Unknown}} | {{agreed_assertion_and_output}} | NOT RUN |
| AC-03 | {{actual_method_or_Unknown}} | {{agreed_assertion_and_output}} | NOT RUN |

## Unresolved work and blockers

| Remaining work / unknown | Effect on acceptance or readiness | Next action | Owner |
| --- | --- | --- | --- |
| {{unmet_AC_missing_proof_or_material_question}} | {{blocking_or_nonblocking_with_reason}} | {{bounded_next_action}} | {{confirmed_owner_or_TBD}} |

**Risk / recovery information relevant to the handoff:** {{evidence_backed_limitations_or_NA_with_reason}}

## Unrelated findings

{{separate_observations_outside_the_agreed_scope_with_evidence_and_proposed_follow_up_or_None; do_not_silently_expand_this_ticket}}

## Human review and completion decision

- **Review requested:** {{specific_decisions_or_checks_for_confirmed_reviewer_or_TBD}}
- **Review outcome / evidence:** PENDING — {{record_only_after_review}}
- **Completion assessment:** {{incomplete_ready_for_review_or_complete_with_evidence}}. Required failed, blocked, or unrun checks and material unresolved rules/tenant boundaries/targets/proof prevent a completion claim.
- **Execution boundary:** {{state_whether_this_is_only_a_ticket_or_plan_handoff_or_an_evidenced_implementation_handoff}}. A ticket-authoring handoff must state that no implementation or live change was performed.
