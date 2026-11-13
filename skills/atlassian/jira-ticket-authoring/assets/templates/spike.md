# {{research_question_or_decision_to_resolve}}

<!-- A spike produces evidence and a decision recommendation. It does not include the eventual implementation. Use `Unknown — {{question}}` for missing facts, `TBD` for an unconfirmed owner, and `N/A — {{reason}}` when appropriate. Do not invent a timebox, deadline, source, or result. Keep AC IDs stable. -->

<!-- Default authoring readiness to Draft until an evidence-backed assessment justifies another applicable verdict. -->

- **Content pattern:** Investigation / spike
- **Configured Jira work type:** {{AVAILABLE_PROJECT_WORK_TYPE_OR_UNRESOLVED}} — map this pattern to the target project configuration.
- **Jira workflow status:** {{KNOWN_CURRENT_STATUS_OR_UNSPECIFIED}} — separate from authoring readiness.
- **Authoring readiness (not Jira status):** {{DRAFT_OR_NEEDS_CLARIFICATION_OR_READY_FOR_INVESTIGATION}} — {{readiness_reason_and_material_gaps}}
- **Owner:** {{confirmed_investigator_or_TBD}}
- **Target context / evidence boundary:** {{confirmed_system_revision_environment_and_allowed_sources_or_Unknown}}

## Business outcome and research question

{{business_decision_this_investigation_enables_and_the_cost_of_leaving_it_unresolved}}

**Question:** {{one_bounded_answerable_question}}

**Decision recipient:** {{confirmed_decision_owner_or_TBD}}

## Evidence and unknowns

| Anchor | Source and exact location | Known fact / limitation |
| --- | --- | --- |
| S1 | {{supplied_report_spec_or_repository_anchor}} | {{what_is_established_and_what_needs_investigation}} |

- **Hypotheses / options:** {{explicitly_unverified_candidates_or_Unknown}}
- **Material unknowns:** {{business_rules_tenant_boundaries_targets_or_proof_needed}}

## Bounded investigation

- **In:** {{permitted_questions_sources_and_safe_methods}}
- **Out:** Implementation, live changes, and {{other_exclusions}}.
- **Budget:** {{supplied_effort_cap_with_source_or_clearly_labeled_proposed_bound_or_Unknown}}. No deadline or approved budget is implied.
- **Stopping rule:** Stop when {{evidence_or_decision_threshold}} is met or an applicable agreed budget is exhausted. Report unresolved questions and the next decision; do not silently extend an agreed budget.
- **Dependencies / blockers:** {{target_access_evidence_decisions_or_budget_needed_with_owner_or_TBD}}

## Acceptance and verification

| AC ID | Observable deliverable condition | Verification method and expected evidence | Actual evidence / result |
| --- | --- | --- | --- |
| AC-01 | The investigation record answers {{question}} or explains why the bounded work was inconclusive. | {{review_of_sourced_findings_methods_and_limitations}} | NOT RUN |
| AC-02 | The decision brief compares {{relevant_options}} against {{agreed_constraints}} and distinguishes facts, hypotheses, and unknowns. | {{review_of_options_evidence_tradeoffs_and_recommendation}} | NOT RUN |
| AC-03 | The handoff lists the decision needed, its owner or TBD, blockers, and bounded follow-up tickets; no unresolved implementation is labeled ready. | {{review_of_decision_record_and_follow_up_readiness}} | NOT RUN |

## Handoff and definition of done

- Assess **Ready for investigation** only when the question, available inputs, methods, deliverable, and stopping condition are sufficient to begin. Resolve a proposed budget before execution if it materially constrains authorization or feasibility. This is never **Ready for implementation**.
- Deliver {{decision_artifact_location_or_Unknown}} with source anchors, investigation results, consumed budget if known, limitations, and the recommendation to {{reviewer_or_TBD}}.
- A documented inconclusive outcome can complete the spike; it cannot resolve an implementation blocker without the necessary evidence or decision.
- Open or update implementation work only as a separate draft with its own target, scope, AC IDs, dependencies, and readiness assessment. The ticket author does not run the investigation while drafting.
