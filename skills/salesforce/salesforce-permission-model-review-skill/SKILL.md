---
name: salesforce-permission-model-review-skill
description: 'Use this skill when Salesforce profiles, permission sets, permission set groups, permission set licenses, muting permission sets, sharing rules, OWD, role hierarchy, IP restrictions, or session policies must be reviewed for toxic combinations and over-privilege. Flags: ModifyAllData with broad assignment, ViewAllData on PII objects, API Enabled without IP restriction, Customize Application outside admin profiles, and sharing-rule widening on regulated-data objects. Trigger phrases: "review this permission model", "check for toxic permission combinations", "is this permission set safe", "review sharing rules on this object", "assess our OWD and role hierarchy". Do not use when you need a full org posture review (use salesforce-org-assessment-skill), when metadata quality is the focus (use salesforce-metadata-review-skill), or when a live permission change is being proposed (use salesforce-live-change-approval-protocol). Works from sanitized exports only; never requests live org access.'
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-05-20"
  category: security
  lifecycle: experimental
---

# Salesforce Permission Model Review Skill

## Purpose
This skill reviews the Salesforce permission model — profiles, permission sets,
permission set groups, permission set licenses, muting permission sets, sharing
rules, OWD, role hierarchy, IP restrictions, and session policies — for
over-privilege, toxic combinations, and sharing design risk. It flags
combinations that create security or compliance exposure and produces a
structured findings report. It does not access live orgs or authorize changes.

## When to use
- A permission audit is required before a compliance review or certification.
- A new permission set or profile is being designed and needs adversarial review.
- Guest-user access patterns or sharing rule changes need security review.
- A toxic-permission alert has been raised and must be investigated.

## When not to use
- Full org posture assessment — use `salesforce-org-assessment-skill`.
- Metadata quality review (fields, layouts) — use `salesforce-metadata-review-skill`.
- Live permission change proposal — use `salesforce-live-change-approval-protocol`.
- Data exposure event response — use `salesforce-data-exposure-escalation-protocol`.

## Minimum payload (required inputs)
- Sanitized permission exports: profile XML, permission set XML, permission set
  group definitions, sharing rule definitions, OWD settings, role hierarchy
  summary, IP restriction settings, session policy settings.
- Context: approximate user population, key regulated-data objects, industry vertical.

## Workflow

### 1. Profile review
- List all profiles and their base permissions.
- Flag: `ModifyAllData` assigned to any non-system-admin profile.
- Flag: `ViewAllData` on profiles with access to PII-classified objects.
- Flag: `API Enabled` on profiles without corresponding IP restriction.
- Flag: `Customize Application` on profiles outside the designated admin group.
- Flag: `Manage Users` outside HR/IT admin profiles.
- Flag: profiles with direct object-level Create/Edit/Delete on financial or
  regulated-data objects without documented business justification.

### 2. Permission set review
- List all permission sets and their grants.
- Flag: permission sets granting `ModifyAllData` or `ViewAllData` that are
  assigned broadly (> configurable user threshold).
- Flag: permission sets duplicating profile permissions (redundant, adds attack surface).
- Flag: permission sets with no current assignees (orphaned — attack surface if reassigned).
- Flag: `Field Service`
or
  `Experience Cloud`
permission sets
  granting object access beyond their intended scope.

### 3. Permission set groups and muting
- Review permission set group composition.
- Flag: muting permission sets that silently narrow permissions without clear documentation.
- Flag: permission set groups that combine permissions creating toxic combinations.

### 4. Sharing model review
- Review OWD (Organization-Wide Defaults) per object.
- Flag: OWD = Public Read/Write on objects containing PII or financial data.
- Flag: OWD = Public Read Only on regulated-data objects where stricter control is expected.
- Review role hierarchy: flag roles that grant data access beyond job function.
- Review sharing rules:
  - Flag: criteria-based rules that are effectively always-true.
  - Flag: sharing rules on objects classified as regulated data.
  - Flag: sharing rules granting Edit access where Read is sufficient.

### 5. Guest user review
- Review guest-user profile permissions (Experience Cloud
).
- Flag: guest-user profile with any object-level Read access to records containing PII.
- Flag: sharing sets that grant guest users access to records via lookup relationships
  that could expose unintended records.
- Flag: public-site access (Sites
) with
  Apex REST endpoints lacking CSRF protection.

### 6. IP restrictions and session policies
- Flag: named credentials or connected apps without IP allowlisting in production.
- Flag: session timeout > configurable threshold for privileged user groups.
- Flag: `Lock sessions to the IP address from which they originated` disabled
  for profiles with sensitive permissions.

## Toxic combination registry
These specific combinations always produce a Critical or High finding:

| Combination | Risk | Rating |
|---|---|---|
| `ModifyAllData` + broad assignment (> threshold users) | Mass data destruction or exfiltration | Critical |
| `ViewAllData` + PII object access + no IP restriction | PII exposure | Critical |
| `API Enabled` + no IP restriction + sensitive object access | API-based data exfiltration | High |
| `Customize Application` + non-admin profile | Privilege escalation via metadata change | High |
| `Manage Users` + no IP restriction | Account takeover / privilege escalation | High |
| Guest user + sharing set + PII object | Unauthenticated PII exposure | Critical |
| `ModifyAllData` + `Manage Users` on same profile | Full org compromise posture | Critical |

## Evidence requirements
- Sanitized permission exports; no credentials, session tokens, or customer data.
- OWD settings and role hierarchy summary are required for sharing model review.
- User assignment counts (not names) are required for broad-assignment checks.

## Output format
```
permission_model_findings:
  profile_findings:
    - finding: [description]
      severity: Critical | High | Medium | Low
      combination: [which toxic combination, if applicable]
      evidence: [what in the export supports this]
      recommendation: [brief]
  permission_set_findings: [same structure]
  sharing_model_findings: [same structure]
  guest_user_findings: [same structure]
  ip_session_findings: [same structure]

toxic_combinations_detected: [list from registry]
escalation_gates_fired: [from salesforce-risk-taxonomy, or "none"]
summary:
  critical_count: [count]
  high_count: [count]
assumptions: [list]
missing_evidence: [what would improve the review]
```

## Redaction rules
- Never request secrets, credentials, OAuth tokens, refresh tokens, session IDs, MFA seeds, customer PII.
- Sanitize org IDs, user IDs (replace with placeholders) before sharing in outputs.
- Do not include actual user names; use role and count references only.

## Privilege / data handling rules
- Permission review findings involving regulated data must be flagged for compliance specialist review.
- Guest-user findings involving PII must trigger salesforce-data-exposure-escalation-protocol.

## Handoff rules
- Hands off to: salesforce-org-assessment-skill (full posture context),
  salesforce-data-exposure-escalation-protocol (if guest-user PII exposure confirmed),
  salesforce-case-capsule (structured handoff for any Critical finding).
- Required handoff fields: matter_id, toxic_combinations_detected, escalation_gates_fired,
  critical_count, assumptions.

## Audit log fields
- matter_id, skill_id, skill_version, invoked_by, input_hash, evidence_quality, output_verdict, escalation_fired, timestamp

## Stop conditions
- Export contains live credentials or session tokens — stop and ask for sanitized version.
- Guest-user PII exposure is confirmed — stop, output ESCALATE, invoke salesforce-data-exposure-escalation-protocol.
- Critical toxic combination detected in a production org — stop and require human review before continuing.

## Security notes
- Read-only static review; never requests live org access or runs SOQL queries.
- Sanitized inputs only; any input containing credentials must be refused.
- Toxic combinations are objective findings; remediation requires human-authorized change management.
- Guest-user exposure is always escalation-grade regardless of record count.
