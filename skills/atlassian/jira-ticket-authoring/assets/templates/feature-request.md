# {{user_problem_or_desired_capability}}

> Describe the problem before choosing a solution. Use `Unknown — {{question}}` for missing facts, `TBD` for an unconfirmed owner, and `N/A — {{reason}}` only when appropriate. Label suggestions as proposals. Do not invent customer demand, metrics, priority, commitments, or test results. Keep AC IDs stable.

- **Type:** Feature request
- **Readiness assessment (not Jira status):** Draft — {{triage_readiness_and_material_gaps}}
- **Owner / decision recipient:** {{confirmed_owner_or_TBD}}
- **Product / user group:** {{known_product_and_affected_users_or_Unknown}}
- **Priority proposal and rationale:** {{evidence_backed_proposal_or_Unknown; urgency_alone_is_not_severity}}

## Problem, user, and business value

- **User problem:** {{what_the_user_cannot_do_today_and_in_which_situation}}
- **Current approach:** {{current_workaround_or_process_and_its_evidenced_limits_or_Unknown}}
- **Value of solving it:** {{observable_customer_or_business_benefit_in_plain_language}}
- **Demand and impact:** {{confirmed_feedback_or_usage_and_its_limits; distinguish_reported_from_measured_impact}}

## Evidence and customer feedback

| Anchor | Source and exact location | What it supports / limitations |
| --- | --- | --- |
| S1 | {{sanitized_feedback_request_research_or_usage_anchor}} | {{observed_need_and_limits_of_the_sample}} |

**Material unknowns:** {{questions_about_users_business_rules_tenant_permissions_or_success_with_decision_owner_or_TBD}}

## Desired behavior and options

- **Desired user experience:** When {{user_and_situation}}, the user can {{action}} and observes {{result}}. {{State_which_parts_are_requested_versus_agreed}}.
- **Business rules and boundaries:** {{confirmed_rules_eligibility_tenant_permissions_and_compatibility_constraints_or_Unknown}}
- **Options:** {{requested_option; viable_alternative_or_improving_current_process; no_change_option_where_relevant}}
- **Tradeoffs / dependencies:** {{evidenced_benefits_costs_dependencies_and_open_decisions; no_invented_estimates}}
- **Non-goals:** {{related_needs_or_implementation_work_excluded_from_this_request}}

## Success measure

| Measure | Baseline and source | Target and decision source | How it would be checked |
| --- | --- | --- | --- |
| {{observable_user_or_business_outcome}} | {{evidenced_baseline_or_Unknown}} | {{agreed_target_or_Unknown}} | {{measurement_method_or_Unknown}} |

An unknown baseline or target stays unknown; do not manufacture a percentage to make the request look complete.

## Triage acceptance and verification

These criteria accept a decision about the request. They do not claim that a feature has shipped. Put implementation acceptance criteria in the resulting story or task.

| AC ID | Observable decision deliverable | Verification method and expected evidence | Actual evidence / result |
| --- | --- | --- | --- |
| AC-01 | {{problem_user_and_value_are_supported_or_evidence_gaps_are_explicit}}. | {{review_of_source_linked_problem_statement_and_unknowns}} | NOT RUN |
| AC-02 | {{relevant_options_are_compared_against_confirmed_rules_and_business_value}}. | {{review_of_options_tradeoffs_and_open_decisions}} | NOT RUN |
| AC-03 | {{accept_defer_decline_or_discover_decision_has_a_rationale_and_next_owner}}. | {{decision_record_and_follow_up_links_or_reason_no_follow_up_is_needed}} | NOT RUN |

## Decision and handoff

- **Decision / source:** {{Pending_or_confirmed_decision_with_rationale_and_anchor}}
- **Dependencies / blockers:** {{needed_evidence_decisions_or_access_with_owner_or_TBD}}
- **Follow-up:** {{confirmed_PRD_epic_story_or_discovery_link_or_proposed_title; no_fabricated_issue_key}}
- Assess **Ready for triage** when the need, available evidence, and decision sought are clear. Use **Ready for investigation** only for a bounded agreed discovery scope. Neither means **Ready for implementation**.
- Implementation requires its own agreed behavior, exact target, business and tenant/security rules, required supporting proof, bounded scope, and feasible AC verification. Keep unresolved material questions visible in that follow-up.
- Complete this request's triage only when a human decision and disposition are recorded. The author prepares the request and handoff; no implementation, delivery commitment, or live action occurs by drafting it.
