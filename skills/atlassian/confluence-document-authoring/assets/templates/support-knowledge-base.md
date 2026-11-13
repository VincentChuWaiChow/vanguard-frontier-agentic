# {{customer_visible_symptom_or_task}}: troubleshooting and help

| Metadata | Value |
|---|---|
| Audience / publication scope | {{customer_or_internal_readers}} / {{intended_visibility_and_actual_restrictions_or_unknown}} |
| Accountable owner / reviewer | {{support_owner_or_unknown}} / {{reviewer_or_unknown}} |
| Document lifecycle / review trigger | draft / {{fix_release_behavior_change_or_review_date_or_unknown}} |
| Revision / last reviewed | {{revision_or_unknown}} / {{date_or_unknown}} |
| Sources / revisions / dates | {{case_test_issue_or_document_sources_revisions_dates_or_unknown}} |
| Product / affected versions / availability | {{product}} / {{affected_and_unaffected_scope_or_unknown}} / {{feature_availability_or_unknown}} |
| Jira / related documentation | {{issue_links_or_unknown}} / {{how_to_release_notes_or_other_links_or_unknown}} |

Replace placeholders with sourced facts or **unknown**. Preserve customer reports as reported; a matching symptom alone does not confirm a known defect. Remove this note before publication.

## Symptom and scope

- What the reader sees: {{symptom_and_exact_sanitized_error_text_or_unknown}}.
- When it occurs: {{trigger_and_relevant_environment_or_unknown}}.
- Expected behavior: {{expected_behavior_and_source_or_unknown}}.
- Affected versions, roles, configuration, or regions: {{verified_scope_and_source_or_unknown}}.
- Issue status: {{confirmed_known_issue_with_evidence_or_suspected_issue_or_cause_unknown}}.
- Fix / resolution state: {{verified_fix_and_available_version_or_unknown}}.

## Before you start

{{Required_access_and_safe_preconditions_or_unknown}}. Preserve unsaved work and existing evidence where relevant. Do not request passwords, tokens, full customer datasets, or unredacted secrets.

## Customer-safe checks

Use non-destructive checks appropriate for the stated audience. Each numbered check must have an expected observation and a next branch; include only sourced product labels and instructions.

| Check | Action | Expected observation | If expected | If different |
|---|---|---|---|---|
| 1 | {{safe_check_of_product_version_or_scope}} | {{matching_affected_scope}} | Continue to check 2. | {{correct_article_or_escalation_path}} |
| 2 | {{safe_check_of_precondition_or_visible_status}} | {{expected_status}} | {{next_check_or_resolution_check}} | {{documented_recovery_or_escalation}} |
| 3 | {{safe_check_or_supported_repeat_attempt}} | {{observable_result}} | {{completion_check}} | {{escalation_route}} |

Stop condition: {{condition_that_makes_further_checks_inappropriate_and_where_to_get_help_or_unknown}}.

## Workaround

| Workaround | Validation state / evidence | Applies to | Limits / side effects / recovery |
|---|---|---|---|
| {{documented_workaround_or_no_validated_workaround_known}} | {{validated_version_environment_date_and_result_source_or_NOT_VALIDATED}} | {{verified_scope_or_unknown}} | {{known_limits_or_unknown}} |

Present a workaround as validated only with supporting evidence for the relevant scope. Keep an untested idea in a separate restricted investigation record; do not present it as a customer instruction. If no validated workaround is known, state that explicitly.

## Confirm resolution

{{Observable_check_that_the_original_task_now_succeeds_and_the_error_is_absent}}. If the symptom returns or the check fails: {{support_or_escalation_reference_or_unknown}}.

## Escalate with minimal sanitized evidence

Send through {{approved_support_route_or_unknown}}:

- Product / version / environment and relevant feature availability, or **unknown**.
- Time and timezone; frequency; impact on the requested task.
- Sanitized exact error, minimal reproduction steps, expected result, and actual result.
- Checks already completed and their observations; workaround attempted and result.
- A non-secret request or correlation identifier, if supported, and a redacted screenshot or minimal log excerpt only when needed.

Exclude passwords, access tokens, private keys, session cookies, unnecessary personal data, and unrelated customer records. Use the approved protected intake route for evidence that cannot be safely included in the article.

## Frequently asked questions

| Question | Answer / evidence |
|---|---|
| Is this a known issue? | {{confirmed_issue_reference_or_suspected_status_and_evidence_gap}} |
| Is a fix available for my version? | {{verified_answer_and_availability_source_or_unknown}} |
| Can I continue using the product? | {{supported_scope_and_limitations_or_unknown}} |

## Internal-only notes — restricted publication required

A heading is not access control. Before including internal investigation details, apply and verify actual page or space access restrictions for the intended internal audience. For a customer-visible article, keep these details on a separate restricted page and omit this section from the customer copy. If restrictions cannot be verified, do not include restricted content in the publication.

- Restricted investigation reference / verified access scope: {{reference_and_restriction_verification_or_unknown}}.
- Suspected causes, confidence, and missing evidence: {{hypotheses_and_evidence_gaps_or_unknown}}.
- Engineering escalation / linked issue: {{route_and_link_or_unknown}}.
- Publication review outcome and evidence: {{reviewer_scope_outcome_and_source_or_pending}}.
