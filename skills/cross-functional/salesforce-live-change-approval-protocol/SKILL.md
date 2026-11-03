---
name: salesforce-live-change-approval-protocol
description: 'Use this skill when any proposed mutation to a live Salesforce production org must be evaluated before execution. This is a refusal-by-default gate: if any required precondition is missing, the skill stops and refuses. Required preconditions are target_org_identity, environment_type, user_identity, permission_scope, change_ticket, approval_state, dry_run_or_deployment_preview, backup_rollback_plan, test_evidence, and post_change_verification_plan. Trigger phrases: "approve this Salesforce change", "can we deploy to production", "review this org mutation", "is this change safe to push live", "run this in production". Do not use for sandbox or scratch-org changes that have no production impact, for read-only reviews of exported metadata (use domain review skills), or for classifying matter types (use salesforce-risk-taxonomy). Note: this repo is a markdown marketplace; this protocol governs advisory checklists only, not real org executions. All live mutation decisions require human authorization.'
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-05-20"
  category: delivery
  lifecycle: experimental
---

# Salesforce Live Change Approval Protocol

## Purpose
This skill is a refusal-by-default gate for any proposed mutation to a live
Salesforce production org. It exists because production org changes carry
irreversible risk — data exposure, broken automation, permission widening,
and revenue-logic corruption can all result from unreviewed deployments.
No live-mutation advisory may proceed until all ten required preconditions
are confirmed present and documented.

**Important:** This repository is a markdown marketplace for advisory
workflows. This protocol governs checklist-based advisory review, not real
org executions. Live org mutations require human authorization through your
organization's actual change management process.

## When to use
- Any proposed change to a Salesforce production org is under discussion.
- A deployment is being planned and must be evaluated for approval-readiness.
- A change request has been submitted and needs precondition verification.
- An escalation gate (`live-mutation` from salesforce-risk-taxonomy) has fired.

## When not to use
- The change targets only sandboxes or scratch orgs with no production impact.
- You need read-only review of exported metadata — use the appropriate domain
  review skill (salesforce-metadata-review-skill, salesforce-flow-automation-review-skill, etc.).
- You need to classify matter types — use `salesforce-risk-taxonomy`.
- You need a structured handoff — use `salesforce-case-capsule`.

## Minimum payload (required inputs)
The following ten preconditions must ALL be present. If any is missing, the
skill outputs STOP and lists the missing items.

1. **target_org_identity** — A placeholder identifier for the target org (never a real org ID or credentials). Environment type must be confirmed as production.
2. **environment_type** — Must be `production`. If sandbox or scratch, this protocol does not apply.
3. **user_identity** — The role or placeholder identity of the person authorizing the change (never a real username, email, or SSO ID).
4. **permission_scope** — The permissions held by the deploying identity. Must be documented; "admin" alone is insufficient.
5. **change_ticket** — A reference to an approved change management ticket (e.g., Jira, ServiceNow, Salesforce Cases). Must exist and be in approved state.
6. **approval_state** — The formal approval status. Must be `approved` by a named human owner. `pending` or `draft` → STOP.
7. **dry_run_or_deployment_preview** — Evidence that the change was previewed in a comparable environment (check-only deploy output, sandbox result, or equivalent). Must be present.
8. **backup_rollback_plan** — A documented plan for reversing the change if it fails. Must name the rollback mechanism and estimated recovery time.
9. **test_evidence** — Test results demonstrating the change is safe. Must include test class coverage (Apex) or equivalent automated evidence. Must meet org threshold.
10. **post_change_verification_plan** — Steps to verify the change is working correctly after deployment. Must be documented before deployment begins.

## Workflow
1. Receive the proposed change description (sanitized, no credentials or PII).
2. Check each precondition in order.
3. If ANY precondition is missing or incomplete → output STOP with missing items listed.
4. If all ten preconditions are present → output PROCEED-WITH-HUMAN-AUTHORIZATION, listing each precondition's confirmed state.
5. Check salesforce-risk-taxonomy escalation gates. If any gate fires → output ESCALATE regardless of precondition state.
6. Produce the advisory checklist output.
7. Remind the invoker that human authorization is required before any real deployment.

## Evidence requirements
- All ten preconditions must be documented by the invoker.
- Evidence must be sanitized: no real org IDs, no credentials, no customer PII.
- Dry-run or deployment preview output must be present as text (not a promise to run it later).

## Output format
```
advisory_verdict: STOP | PROCEED-WITH-HUMAN-AUTHORIZATION | ESCALATE
missing_preconditions: [list, or "none"]
escalation_gates_fired: [list from salesforce-risk-taxonomy, or "none"]
precondition_check:
  target_org_identity: confirmed | missing | incomplete
  environment_type: confirmed | missing | incomplete
  user_identity: confirmed | missing | incomplete
  permission_scope: confirmed | missing | incomplete
  change_ticket: confirmed | missing | incomplete
  approval_state: confirmed | missing | incomplete
  dry_run_or_deployment_preview: confirmed | missing | incomplete
  backup_rollback_plan: confirmed | missing | incomplete
  test_evidence: confirmed | missing | incomplete
  post_change_verification_plan: confirmed | missing | incomplete
advisory_notes: [risk observations, not authorizations]
human_authorization_reminder: "All live org mutations require human authorization through your organization's change management process. This advisory checklist does not constitute approval."
```

## Redaction rules
- Never request secrets, credentials, OAuth tokens, refresh tokens, session IDs, MFA seeds, customer PII.
- Sanitize org IDs, user IDs (replace with placeholders) before sharing in outputs.
- If the invoker provides real credentials or org IDs, decline and ask for sanitized placeholders.

## Privilege / data handling rules
- This skill never stores, logs, or repeats credentials or session tokens.
- Production org identifiers must be replaced with placeholders in all outputs.
- Change descriptions must not carry customer data samples.

## Handoff rules
- STOP verdict: handoff to salesforce-case-capsule with missing preconditions as blockers.
- ESCALATE verdict: handoff to salesforce-data-exposure-escalation-protocol if data-exposure gate fired; otherwise to salesforce-case-capsule with escalation_required = true.
- PROCEED-WITH-HUMAN-AUTHORIZATION: advisory output is presented to the human decision owner. No agent takes further action without explicit human authorization.

## Audit log fields
- matter_id, skill_id, skill_version, invoked_by, input_hash, evidence_quality, output_verdict, escalation_fired, timestamp

## Stop conditions
- Any of the ten preconditions is missing or incomplete → output STOP immediately.
- An escalation gate fires → output ESCALATE regardless of precondition state.
- The invoker provides real credentials or production org IDs → stop and refuse.
- The proposed change is described as a live production mutation but environment_type is not confirmed → output STOP.

## Security notes
- Refusal is the default. The burden is on the invoker to supply all ten preconditions.
- This protocol never issues authorization. The output is advisory only.
- Production org IDs, credentials, and session tokens must never appear in any field.
- This repo is a markdown marketplace; no code in this skill executes real Salesforce API calls.
