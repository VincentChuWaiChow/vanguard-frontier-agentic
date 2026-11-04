---
name: salesforce-org-assessment-skill
description: 'Use this skill when a Salesforce org''s overall posture must be assessed from sanitized exports — covering the object model, automation inventory, permission topology, integration map, and technical debt indicators. Produces a risk register and remediation backlog. Trigger phrases: "assess this Salesforce org", "review org health", "generate a risk register for this org", "what is the technical debt in this org", "review this metadata export for posture". Do not use when you need to review a single metadata type in depth (use salesforce-metadata-review-skill), when you need to review only permissions (use salesforce-permission-model-review-skill), or when a live production change is being proposed (use salesforce-live-change-approval-protocol). Works from sanitized exports only; never requests live org credentials or direct API access.'
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-05-20"
  category: platform
  lifecycle: experimental
---

# Salesforce Org Assessment Skill

## Purpose
This skill provides a structured workflow for assessing a Salesforce org's
overall posture from sanitized metadata exports. It produces a risk register
and remediation backlog covering the object model, automation inventory,
permission topology, integration map, and technical debt indicators. It does
not access live orgs, request credentials, or authorize changes.

## When to use
- An org is being evaluated for acquisition, merger, or re-platforming.
- A new technical lead needs a structured health baseline.
- A security or compliance review requires a documented posture assessment.
- A risk register must be produced before a major release or migration.

## When not to use
- Single metadata type deep-dive — use `salesforce-metadata-review-skill`.
- Permission-only review — use `salesforce-permission-model-review-skill`.
- Live production change proposal — use `salesforce-live-change-approval-protocol`.
- Automation-only review — use `salesforce-flow-automation-review-skill`.

## Minimum payload (required inputs)
- Sanitized metadata exports (one or more of): object definitions, field lists,
  automation inventory (Flow list, validation rules), permission set/profile
  summary, installed packages list, integration endpoints (sanitized, no tokens).
- Org environment type (production / sandbox — use placeholder, not real org ID).
- Business context: industry vertical, approximate user count range, primary
  use cases (Sales, Service, Marketing, CPQ, etc.).

## Workflow
1. **Object model review** — Identify custom objects, junction objects, deprecated
   standard objects in use, field count per object (flag objects > configurable
   threshold), hardcoded IDs in default field values, formula complexity.
2. **Automation inventory** — List all automation types present (Flow, validation
   rules, approval processes, triggers, Process Builder
).
   Flag: mixed automation types on same object, Process Builder still active,
   missing fault paths, automation running in system context without sharing.
3. **Permission topology** — Identify profiles, permission sets, permission set
   groups. Flag: toxic combinations (ModifyAllData, ViewAllData, broad API
   Enabled without IP restriction). Summarize permission set proliferation.
4. **Integration map** — List integration patterns present (REST, SOAP, Bulk,
   Streaming, Platform Events
, CDC
, middleware). Flag: point-to-point
   patterns without error queues, missing named credentials, hardcoded endpoints.
5. **Technical debt indicators** — Assess: deprecated metadata types, unused
   fields (> configurable threshold with zero usage), hardcoded IDs, legacy
   APIs still in use, managed package versions behind current release.
6. **Risk register** — For each finding, assign: risk_tier (from
   salesforce-risk-taxonomy), matter_type, remediation_priority, owner_role,
   estimated_effort.
7. **Remediation backlog** — Ordered list of remediation items with priority,
   dependency notes, and recommended approach.

## Evidence requirements
- Sanitized metadata exports; no credentials, no session tokens, no customer data.
- If exports contain field values that appear to be production data samples,
  decline and ask for schema-only exports.
- Object and field counts are acceptable; record values are not.

## Output format
```
org_assessment_summary:
  object_model_findings: [list with risk_tier per finding]
  automation_inventory_findings: [list with risk_tier]
  permission_topology_findings: [list with risk_tier]
  integration_map_findings: [list with risk_tier]
  technical_debt_indicators: [list with risk_tier]

risk_register:
  - finding_id: [generated]
    matter_type: [from salesforce-risk-taxonomy]
    risk_tier: Critical | High | Medium | Low | Unknown
    description: [one sentence]
    evidence_basis: [what in the export supports this]
    owner_role: [placeholder role]
    remediation_priority: P1 | P2 | P3 | P4

remediation_backlog:
  - priority: [P1-P4]
    item: [description]
    dependencies: [other items that must precede this]
    recommended_approach: [brief]

escalation_gates_fired: [from salesforce-risk-taxonomy, or "none"]
missing_evidence: [what would improve the assessment]
assumptions: [explicit list]
```

## Redaction rules
- Never request secrets, credentials, OAuth tokens, refresh tokens, session IDs, MFA seeds, customer PII.
- Sanitize org IDs, user IDs (replace with placeholders) before sharing in outputs.
- Field default values that appear to be real data must be flagged and excluded from output.

## Privilege / data handling rules
- Works from schema-level exports only; does not require record-level data.
- If regulated-vertical indicators are present (Health Cloud, Financial Services
  Cloud
), escalate compliance findings
  to a qualified compliance specialist.
- Assessment output is internal; do not share org topology details externally.

## Handoff rules
- Hands off to: salesforce-permission-model-review-skill (permission topology deep-dive),
  salesforce-flow-automation-review-skill (automation deep-dive),
  salesforce-metadata-review-skill (metadata type deep-dive),
  salesforce-integration-review-skill (integration deep-dive).
- If escalation gate fires: salesforce-case-capsule with escalation_required = true.
- Required handoff fields: matter_id, risk_register (summary), escalation_gates_fired,
  missing_evidence, assumptions.

## Audit log fields
- matter_id, skill_id, skill_version, invoked_by, input_hash, evidence_quality, output_verdict, escalation_fired, timestamp

## Stop conditions
- Export contains live credentials, session tokens, or record-level PII — stop and ask for sanitized version.
- Environment type is production and an escalation gate fires — stop and invoke salesforce-case-capsule before continuing.
- Export is from a regulated-vertical org and jurisdiction is unknown — flag and escalate.

## Security notes
- Read-only static review; never requests live org access or API credentials.
- Sanitized inputs only; any input containing credentials must be refused.
- Risk register is advisory; remediation requires human authorization.
- Regulated-vertical findings must be escalated to qualified compliance counsel.
