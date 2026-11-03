---
name: salesforce-data-exposure-escalation-protocol
description: 'Use this skill when a Salesforce data exposure event has been detected or is strongly suspected. Triggers include: guest-user data exposure via Experience Cloud, cross-org data sync without a Data Processing Agreement, regulated-data sync in Marketing Cloud without a consent map, Experience Cloud sharing-set widening affecting personal data, and Data Cloud cross-org sharing without appropriate controls. Trigger phrases: "guest user can see records they should not", "data syncing across orgs without DPA", "sharing set was widened in production", "marketing data sync without consent", "Data Cloud sharing concern". Do not use for routine permission reviews (use salesforce-permission-model-review-skill), for pre-change risk assessment (use salesforce-live-change-approval-protocol), or for general security questions not involving a suspected exposure event. This skill governs the immediate escalation response path: pause, preserve evidence, name controllers and processors, escalate to privacy counsel and security, and document.'
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-05-20"
  category: security
  lifecycle: experimental
---

# Salesforce Data Exposure Escalation Protocol

## Purpose
This skill defines the immediate escalation response path when a Salesforce
data exposure event has been detected or is strongly suspected. It exists
because exposure events in Salesforce orgs — particularly involving guest
users, Experience Cloud, Data Cloud, or Marketing Cloud — can involve
regulated personal data that triggers legal notification obligations. The
protocol must be followed immediately; it is not advisory after a trigger fires.

## When to use
- Guest-user data exposure: an Experience Cloud site's guest-user profile or
  OWD gives unauthenticated access to records that should be restricted.
- Cross-org data sync without DPA: data is flowing between Salesforce orgs or
  to external systems without a documented Data Processing Agreement.
- Regulated-data Marketing Cloud sync without consent map: personal or
  regulated data is syncing to Marketing Cloud
  without a documented lawful basis and consent map.
- Experience Cloud sharing-set widening: a sharing set or sharing rule change
  has expanded access to personal data beyond the intended scope.
- Data Cloud cross-org sharing: Data Cloud
  is sharing data across orgs without documented controls, purpose limitation,
  or appropriate consent.

## When not to use
- Routine permission review with no suspected exposure — use `salesforce-permission-model-review-skill`.
- Pre-change risk assessment before a deployment — use `salesforce-live-change-approval-protocol`.
- General security audit without a specific exposure event — use `salesforce-org-assessment-skill`.
- The trigger is hypothetical or a design question, not an observed event.

## Minimum payload (required inputs)
- Description of the suspected exposure event (sanitized, no credentials or PII beyond what is strictly necessary to describe the exposure type).
- Trigger type (one or more from the trigger list above).
- Environment: is the exposure in a production org? (If unknown, assume yes.)
- Approximate time of discovery.
- Who discovered it and how.

## Escalation response path

**Step 1 — Pause**
Immediately recommend pausing any ongoing data sync, automation, or
configuration change that is contributing to the exposure. Do not delete
evidence. Do not attempt to silently fix the configuration without escalation.

**Step 2 — Preserve evidence**
Recommend capturing and preserving:
- Sanitized description of the configuration state (sharing rules, sharing
  sets, OWD, guest-user profile permissions, Data Cloud segment definitions,
  Marketing Cloud
data extension scope).
- Approximate time window of exposure.
- System or event logs if accessible (do not request log content containing PII).
- Change history (who changed what, when — from audit trail if available).

Evidence must not be modified or deleted. If litigation hold risk exists,
flag it immediately.

**Step 3 — Name controllers and processors**
Identify (using role/placeholder references, not real names):
- Which Salesforce org is the data controller.
- Which system (org, cloud, middleware) is acting as a data processor.
- Which third parties received or may have received the data.
- Whether a DPA exists between controller and processor.

**Step 4 — Escalate to privacy counsel and security**
Recommend immediate escalation to:
- Privacy counsel or Data Protection Officer (DPO) for assessment of
  notification obligations under applicable law (GDPR, CCPA, HIPAA, or
  other jurisdiction-specific requirements).
- Information security team for technical containment assessment.
- Salesforce Trust (
  https://help.salesforce.com/s/trust) if the exposure may involve a
  Salesforce platform-level issue.

Do not conclude that notification is required or not required — that is a
legal determination for qualified counsel.

**Step 5 — Document**
Produce a structured escalation record (salesforce-case-capsule with
escalation_gate_fired = `production-data-exposure`) containing:
- Trigger type(s) fired.
- Evidence preservation status.
- Controller/processor identification (placeholder references).
- Escalation recipients (roles, not personal identifiers).
- Open questions for privacy counsel.
- Do-not-do list.
- Decision owner (named human).

## Workflow
1. Receive sanitized exposure description.
2. Match against trigger list; identify all triggers that apply.
3. Output ESCALATE immediately — do not defer.
4. Execute steps 1–5 in order.
5. Produce salesforce-case-capsule with escalation_gate_fired = `production-data-exposure`.
6. List open questions for privacy counsel (do not answer them — they require legal determination).
7. Remind invoker: no self-remediation without human authorization.

## Evidence requirements
- Sanitized description of the configuration state at time of discovery.
- Trigger type(s) clearly identified.
- Time of discovery and approximate exposure window.
- Whether regulated data (PII, HIPAA, PCI, financial) is or may be involved.

## Output format
```
escalation_verdict: ESCALATE
triggers_fired: [list]
environment: production | unknown (treat as production)
regulated_data_in_scope: yes | no | unknown
pause_recommendation: [specific actions to pause]
evidence_preservation_checklist: [items to capture]
controller_processor_map: [placeholder references]
escalation_recipients: [roles: privacy counsel, DPO, security team, Salesforce Trust if applicable]
open_questions_for_counsel: [list — do not answer]
do_not_do_list: [explicit prohibitions]
decision_owner: [named human role]
salesforce_case_capsule_required: true
```

## Redaction rules
- Never request secrets, credentials, OAuth tokens, refresh tokens, session IDs, MFA seeds, customer PII.
- Sanitize org IDs, user IDs (replace with placeholders) before sharing in outputs.
- Exposure descriptions must use role and system references, not real names or customer identifiers.

## Privilege / data handling rules
- Escalation records may be subject to legal privilege if prepared in anticipation of litigation.
  Flag this and recommend handling only through or with counsel.
- Do not circulate the escalation record beyond the named escalation recipients.
- The escalation record is not a public incident disclosure; do not draft customer
  communications without qualified counsel involved.

## Handoff rules
- Always hands off to salesforce-case-capsule with escalation_gate_fired = `production-data-exposure`.
- Escalates to privacy counsel (external) and security team (internal) as human recipients.
- If regulated-vertical is in scope, also escalates to compliance lead.
- Required handoff fields: trigger_type, environment, regulated_data_in_scope, evidence_preservation_status, decision_owner.

## Audit log fields
- matter_id, skill_id, skill_version, invoked_by, input_hash, evidence_quality, output_verdict, escalation_fired, timestamp

## Stop conditions
- Invoker provides real PII, credentials, or customer data in the description — stop and ask for sanitized version.
- Invoker requests self-remediation without human involvement — stop and refuse; escalation requires human authorization.
- Notification obligation is asserted or denied without counsel — stop and state that legal determination is required.

## Security notes
- This protocol never determines whether regulatory notification is required.
  That is a legal determination for qualified privacy counsel.
- Pausing is always safer than attempting a silent configuration fix.
- Evidence must be preserved; do not recommend deletion of logs or configuration
  snapshots even if they contain evidence of misconfiguration.
- Salesforce Trust contacts are referenced for platform-level issues only;
  verify current contact information at https://help.salesforce.com/s/trust before use.
