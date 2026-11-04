---
name: salesforce-flow-automation-review-skill
description: 'Use this skill when Salesforce Flow XML, validation rules, approval processes, or record-triggered automation must be reviewed for correctness, safety, and maintainability. Flags recursion, ungoverned bypass flags, brittle null handling, missing fault paths, hardcoded recipients, before-save vs after-save misuse, and mixed Process Builder plus Flow plus Apex on same object. Trigger phrases: "review this Flow", "check this automation for recursion", "review this approval process", "is this validation rule safe", "review this record-triggered automation". Do not use when Apex code is the primary subject (use salesforce-apex-lwc-code-review-skill), when metadata quality (not logic) is the focus (use salesforce-metadata-review-skill), or when a live automation activation is proposed (use salesforce-live-change-approval-protocol). Works from pasted or exported automation definitions only; never requests live org access.'
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-05-20"
  category: platform
  lifecycle: experimental
---

# Salesforce Flow Automation Review Skill

## Purpose
This skill reviews Salesforce Flow XML, validation rules, approval processes,
and record-triggered automation for correctness, safety, and maintainability.
It flags patterns that cause data corruption, runaway governor-limit
consumption, incorrect execution order, and logic errors. It does not access
live orgs and does not authorize automation activation.

## When to use
- A Flow is being reviewed before activation in production.
- A record-triggered automation has caused unexpected behavior and must be analyzed.
- An automation inventory contains mixed automation types on the same object.
- An approval process or validation rule needs pre-release safety review.

## When not to use
- Apex code is the primary subject — use `salesforce-apex-lwc-code-review-skill`.
- Metadata quality (field definitions, layouts) — use `salesforce-metadata-review-skill`.
- Live automation activation proposal — use `salesforce-live-change-approval-protocol`.
- Full org automation inventory — use `salesforce-org-assessment-skill`.

## Minimum payload (required inputs)
- Flow XML (or equivalent exported Flow definition) or automation description.
- Automation type: record-triggered, screen, scheduled, autolaunched, or subflow.
- Object(s) the automation acts on.
- Whether other automation types (Apex triggers, Process Builder
,
  validation rules) are active on the same object.

## Workflow

### 1. Recursion detection
- Identify flows that update the same object that triggered them.
- Check for recursion prevention mechanisms: `$Flow.CurrentStage`, custom boolean
  field as recursion guard, or explicit entry condition excluding re-triggered records.
- Flag flows that could recursively fire without a guard.

### 2. Bypass flags
- Flag flows that check custom bypass permissions or hierarchy settings to skip
  execution (e.g., `Bypass_All_Flows__c` custom field or permission set bypass).
- Determine whether bypass is documented, scoped, and auditable.
- Flag ungoverned bypass: bypass that has no associated approval process or
  documentation.

### 3. Null handling
- Review Decision elements for null-check branches.
- Flag formulas or conditions that will throw errors on null values.
- Flag flows that access lookup fields without a null guard on the lookup itself.
- Flag `Get Records` elements whose output is used without a null check after
  the element.

### 4. Fault paths
- Verify that every element that can fail (DML, invocable actions, callouts,
  platform events) has a connected Fault path.
- Flag flows where a Fault path is missing entirely.
- Flag Fault paths that silently swallow errors (no logging, no notification).

### 5. Before-save vs after-save misuse
- Record-triggered flows operating before-save: flag if they make DML operations
  (not allowed in before-save context).
- Record-triggered flows operating after-save: flag if they could be moved to
  before-save for efficiency (field updates only, no related-record operations).
- Flag after-save flows that cause cascading DML triggering Apex triggers.

### 6. Hardcoded recipients and IDs
- Flag hardcoded email addresses in Send Email elements.
- Flag hardcoded user IDs or group IDs in assignment or notification elements.
- Recommend: Custom Metadata, Custom Labels, or hierarchy settings instead.

### 7. Mixed automation on same object
- Identify if Process Builder
(deprecated),
  Apex triggers, and Flows are all active on the same object.
- Flag execution order risks: trigger order between Apex and Flows is not always
  deterministic; flag if both Fire at the same DML moment.
- Recommend: migration plan to consolidate to a single automation layer.

### 8. Approval process review
- Flag approval processes without email templates (hardcoded notification text).
- Flag approval processes where the final approver is a specific user ID.
- Flag approval processes with no rejection branch behavior defined.
- Flag approval processes on objects also governed by Flow with duplicate logic.

### 9. Validation rule review
- Flag validation rules with no descriptive error message.
- Flag validation rules that reference fields not on the current object without
  a VLOOKUP — this can cause silent failures.
- Flag validation rules that fire on all events when they should be
  conditioned on specific record types or entry points.

## Evidence requirements
- Exported Flow XML or detailed automation description; no credentials, no session tokens.
- Object name(s) and list of other active automation on the same object.
- If the automation accesses external systems (callouts), note the endpoint category.

## Output format
```
flow_automation_findings:
  recursion_risks:
    - finding: [description]
      severity: Critical | High | Medium | Low
      evidence: [element name or condition]
      recommendation: [brief]
  bypass_flag_risks: [same structure]
  null_handling_risks: [same structure]
  fault_path_gaps: [same structure]
  before_after_save_issues: [same structure]
  hardcoded_values: [same structure]
  mixed_automation_risks: [same structure]
  approval_process_findings: [same structure]
  validation_rule_findings: [same structure]

summary:
  total_findings: [count]
  critical_count: [count]
  high_count: [count]
escalation_gates_fired: [from salesforce-risk-taxonomy, or "none"]
assumptions: [list]
missing_evidence: [what would improve the review]
```

## Redaction rules
- Never request secrets, credentials, OAuth tokens, refresh tokens, session IDs, MFA seeds, customer PII.
- Sanitize org IDs, user IDs (replace with placeholders) before sharing in outputs.
- Do not repeat hardcoded email addresses or user IDs from Flow XML in output; describe pattern and location.

## Privilege / data handling rules
- Automation review is logic-level only; do not carry record data from Flow outputs.
- Findings involving data exposure patterns escalate to salesforce-permission-model-review-skill.

## Handoff rules
- Hands off to: salesforce-apex-lwc-code-review-skill (if Apex triggers are involved),
  salesforce-permission-model-review-skill (if permission bypass is found),
  salesforce-org-assessment-skill (if mixed automation is systemic).
- If escalation gate fires: salesforce-case-capsule with escalation_required = true.
- Required handoff fields: matter_id, automation_type, object_name, escalation_gates_fired,
  critical_count.

## Audit log fields
- matter_id, skill_id, skill_version, invoked_by, input_hash, evidence_quality, output_verdict, escalation_fired, timestamp

## Stop conditions
- Flow XML contains live credentials, callout secrets, or PII — stop and ask for sanitized version.
- Recursion without guard detected in a production-active flow — output Critical finding and recommend immediate deactivation review with human authorization.
- Ungoverned bypass flag grants admin-level access to non-admin users — escalate to salesforce-permission-model-review-skill.

## Security notes
- Read-only static review of exported automation definitions; never requests live org access.
- Hardcoded recipient values must not be repeated in output.
- Automation activation or deactivation decisions require human-authorized change management.
- Process Builder deprecation status should be verified against current Salesforce release notes.
