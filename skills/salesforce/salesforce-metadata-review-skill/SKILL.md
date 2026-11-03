---
name: salesforce-metadata-review-skill
description: 'Use this skill when pasted or exported Salesforce metadata must be reviewed for over-customization, unused fields, hardcoded IDs, and deprecated metadata types. Covers: objects, fields, layouts, page layouts, Lightning record pages, profiles, permission sets, and sharing rules. Trigger phrases: "review this Salesforce metadata", "check this object definition", "are there deprecated metadata types here", "find hardcoded IDs in this profile", "review this permission set export". Do not use when you need a full org posture assessment (use salesforce-org-assessment-skill), when you need a permission model deep-dive across topology (use salesforce-permission-model-review-skill), or when you need to review automation logic (use salesforce-flow-automation-review-skill). Works from pasted or exported metadata only; never requests live org access.'
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-05-20"
  category: platform
  lifecycle: experimental
---

# Salesforce Metadata Review Skill

## Purpose
This skill reviews pasted or exported Salesforce metadata for quality,
maintainability, security, and compliance indicators. It flags
over-customization, unused fields, hardcoded IDs, and deprecated metadata
types, and produces a structured findings report. It does not access live
orgs and does not authorize changes.

## When to use
- A metadata export (objects, fields, layouts, profiles, permission sets,
  sharing rules) needs review before a release.
- A code review has surfaced questions about the metadata surface.
- A technical debt assessment requires metadata-level findings.
- An audit requires documentation of deprecated or risky metadata patterns.

## When not to use
- Full org posture assessment — use `salesforce-org-assessment-skill`.
- Permission topology review across profiles and permission set groups — use `salesforce-permission-model-review-skill`.
- Automation logic review (Flow XML, validation rules) — use `salesforce-flow-automation-review-skill`.
- Live production change review — use `salesforce-live-change-approval-protocol`.

## Minimum payload (required inputs)
- One or more sanitized metadata exports: object XML, field definitions,
  layout XML, profile XML, permission set XML, sharing rule XML, or equivalent.
- Metadata type(s) being reviewed.
- Context: release context, org type (production/sandbox placeholder), business use case.

## Workflow
1. **Object and field review**
   - Identify fields with zero or low usage indicators (no referenced automation, no layout placement).
   - Flag fields with hardcoded IDs in default values or formula expressions.
   - Flag objects with excessive custom field counts (> configurable threshold).
   - Identify deprecated field types (Text Area vs Rich Text Area misuse, currency without multi-currency awareness).
   - Flag custom objects that duplicate standard object functionality.

2. **Layout review**
   - Page layouts: flag layouts with > configurable threshold components; flag required fields not aligned with business process.
   - Lightning record pages
: flag components using hardcoded record IDs or hardcoded URLs.
   - Flag layouts assigned to no profiles or permission sets (orphaned).

3. **Profile and permission set review**
   - Flag profiles with excessive object permissions (Create/Edit/Delete/ViewAll/ModifyAll on sensitive objects).
   - Flag permission sets with duplicate coverage of profile permissions (redundant grants).
   - Flag permission sets with no assignees (orphaned).
   - Flag profiles granting API access without IP restrictions.

4. **Sharing rule review**
   - Identify sharing rules that effectively expose all records to a broad group.
   - Flag criteria-based sharing rules with conditions that evaluate to always-true.
   - Flag sharing rules on objects with OWD = Public Read/Write (redundant).

5. **Deprecated metadata types**
   - Flag use of: workflow rules, Process Builder
(deprecated),
     legacy assignment rules (where Flow equivalent exists), Visualforce pages in Lightning context
     without Lightning migration plan.

6. **Hardcoded ID detection**
   - Flag any string matching Salesforce ID patterns (15- or 18-character alphanumeric) in:
     field defaults, formula fields, validation rule expressions, layout XML.

## Evidence requirements
- Sanitized metadata XML or equivalent export; no credentials, session tokens, or customer data.
- If metadata contains what appears to be record-level data (not schema), decline and ask for schema-only.

## Output format
```
metadata_review_findings:
  objects_and_fields:
    - finding: [description]
      severity: Critical | High | Medium | Low
      evidence: [what in the metadata supports this]
      recommendation: [brief]
  layouts:
    - finding: [description]
      severity: [tier]
      evidence: [...]
      recommendation: [...]
  profiles_and_permission_sets:
    - finding: [description]
      severity: [tier]
      evidence: [...]
      recommendation: [...]
  sharing_rules:
    - finding: [description]
      severity: [tier]
      evidence: [...]
      recommendation: [...]
  deprecated_metadata_types:
    - type: [name]
      usage: [where found]
      migration_path: [recommended replacement]
  hardcoded_ids:
    - location: [metadata element]
      pattern: [describe pattern, do not repeat the ID value]
      recommendation: [use Custom Metadata, Custom Setting, or label instead]

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
- Do not repeat hardcoded ID values in output; describe the pattern and location only.

## Privilege / data handling rules
- Metadata review is schema-level only; do not carry record-level data.
- Sharing rule findings involving PII objects escalate to salesforce-permission-model-review-skill.

## Handoff rules
- Hands off to: salesforce-permission-model-review-skill (permission findings),
  salesforce-flow-automation-review-skill (deprecated automation findings),
  salesforce-org-assessment-skill (if full posture context is needed).
- If escalation gate fires: salesforce-case-capsule with escalation_required = true.
- Required handoff fields: matter_id, metadata_review_findings (summary), escalation_gates_fired.

## Audit log fields
- matter_id, skill_id, skill_version, invoked_by, input_hash, evidence_quality, output_verdict, escalation_fired, timestamp

## Stop conditions
- Metadata contains live credentials, record-level PII, or session tokens — stop and ask for sanitized version.
- Metadata is from a regulated-vertical org and compliance scope is unknown — flag and escalate.
- Hardcoded ID count is very high (systemic pattern) — escalate to salesforce-org-assessment-skill for full posture review.

## Security notes
- Read-only static review of exported metadata; never requests live org access.
- Hardcoded ID values must not be repeated in output to avoid accidental leakage.
- Deprecated metadata flags are informational; migration requires human-authorized release planning.
