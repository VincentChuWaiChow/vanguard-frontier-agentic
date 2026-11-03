---
name: salesforce-infrastructure-audit-skill
description: 'Use this skill when Salesforce infrastructure security posture must be audited — covering network security policies, trusted IP ranges, login IP ranges per profile, CSP Trusted Sites configuration, session security settings, sandbox isolation strategy, sandbox data-sensitivity alignment, Hyperforce deployment controls, and data residency commitments. Trigger phrases: "audit Salesforce infrastructure security", "review network policies", "check IP allowlist config", "assess sandbox isolation", "review session settings", "check Hyperforce security posture". Do not use when identity or IAM permissioning is the focus (use salesforce-permission-model-review-skill), when a live production change is being made (use salesforce-live-change-approval-protocol), or when zero-trust maturity scoring across NIST pillars is needed (use salesforce-zero-trust-maturity-skill). Works from sanitized config exports only; never requests live org access.'
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-05-21"
  category: security
  lifecycle: experimental
---

# Salesforce Infrastructure Audit Skill

## Purpose
This skill conducts a structured security audit of Salesforce infrastructure controls — network access policies, session security settings, sandbox isolation, Hyperforce deployment configuration, and CSP Trusted Sites. It produces a tiered risk register of findings without accessing live orgs or executing API calls. It is the shared workflow called by all infrastructure security agents in the Salesforce agent catalog.

## When to use
- A compliance review or certification requires evidence of infrastructure security controls.
- Sandbox environments handle production-equivalent data and isolation must be verified.
- Network access policy, IP allowlisting, or CSP configuration is being assessed before audit.
- Hyperforce deployment region selection or data residency commitments need review.
- Session security settings (timeout, clickjack, HTTPS enforcement) are under scrutiny.

## When not to use
- Identity or IAM review (profiles, permission sets, sharing rules) — use `salesforce-permission-model-review-skill`.
- Live production configuration changes — use `salesforce-live-change-approval-protocol`.
- Zero-trust maturity scoring across NIST ZTA pillars — use `salesforce-zero-trust-maturity-skill`.
- Full org posture assessment combining all domains — use `salesforce-org-assessment-skill`.

## Minimum payload (required inputs)
- Sanitized network access exports: trusted IP ranges, login IP ranges per profile.
- CSP Trusted Sites list (URLs and directive scope).
- Session settings export: timeout values, clickjack protection level, HTTPS enforcement flag, high-assurance session trigger configuration.
- Sandbox inventory: sandbox name, type (Developer, Developer Pro, Partial, Full), last refresh date, data masking configuration.
- Hyperforce configuration summary: deployment region, data residency scope, Infrastructure Access audit log status.
- Context: industry vertical, regulatory framework (e.g., HIPAA, PCI-DSS, GDPR), approximate user population.

## Workflow

### 1. Network policy inventory
- List all configured trusted IP ranges (Network Access).
- List login IP ranges applied per profile.
- Flag: no IP restriction on profiles with `API Enabled` or `ModifyAllData`.
- Flag: overly broad CIDR ranges (e.g., /8 or wider) applied to sensitive profiles.
- Flag: CSP Trusted Sites entries using wildcard (`*`) domains or non-HTTPS origins.
- Flag: CSP Trusted Sites entries pointing to third-party origins not documented in vendor inventory.
- Record total trusted IP range count and CIDR surface area.

### 2. Session security review
- Review session timeout values per security zone (standard users, admins, privileged service accounts).
- Flag: session timeout > 2 hours for profiles with `Modify All Data`, `Manage Users`, or `View All Data`.
- Flag: `Lock sessions to the IP address from which they originated` disabled for privileged profiles.
- Flag: clickjack protection set below `Allow framing by same-origin pages only` in production.
- Flag: HTTPS-only enforcement not enabled at org level.
- Flag: high-assurance session triggers absent for sensitive operations (e.g., report export, data export).
- Record clickjack protection level and HTTPS enforcement status.

### 3. Sandbox isolation check
- Map each sandbox to its type and the data sensitivity of workloads it hosts.
- Flag: Full or Partial sandbox refreshed from production without data masking configuration present.
- Flag: Full sandbox type used for external vendor or contractor access without network restriction.
- Flag: sandbox refresh policy absent or refresh interval > 90 days for sandboxes handling regulated data.
- Flag: Developer sandbox used to test integrations that require production-equivalent PII volume.
- Record sandbox count by type, masking coverage, and refresh cadence.

### 4. Hyperforce posture review
- Confirm deployment region matches documented data residency commitments.
- Flag: Hyperforce region selection inconsistent with regulatory data-localization requirements.
- Flag: Infrastructure Access
not configured or audit log not enabled.
- Flag: data residency scope documentation absent or not reviewed within the last 12 months.
- Record region, residency commitment document reference, and Infrastructure Access status.

### 5. Risk register assembly
- Consolidate findings from steps 1–4.
- Assign risk_tier per finding: Critical | High | Medium | Low.
- Map each finding to its relevant control domain (network, session, sandbox, Hyperforce, CSP).
- Identify findings that meet escalation gates from salesforce-risk-taxonomy.

## Evidence requirements
- Sanitized exports only; no credentials, session tokens, or customer records.
- Sandbox inventory must include type and data masking status to produce sandbox findings.
- Hyperforce region and data residency documentation are required for step 4.
- Session settings export is required for step 2; absence produces an "insufficient evidence" note.

## Output format
```
infrastructure_audit_findings:
  network_policy_findings:
    - finding: [description]
      severity: Critical | High | Medium | Low
      control_domain: network | session | sandbox | hyperforce | csp
      evidence: [what in the export supports this]
      recommendation: [brief]
  session_security_findings: [same structure]
  sandbox_isolation_findings: [same structure]
  hyperforce_findings: [same structure]
  csp_findings: [same structure]

escalation_gates_fired: [from salesforce-risk-taxonomy, or "none"]
summary:
  critical_count: [count]
  high_count: [count]
  medium_count: [count]
  low_count: [count]
assumptions: [list]
missing_evidence: [what would improve the review]
```

## Redaction rules
- Never request secrets, credentials, OAuth tokens, refresh tokens, session IDs, MFA seeds, customer PII.
- Sanitize org IDs, user IDs before sharing in outputs.
- Do not include actual IP addresses in outputs; use range notation and CIDR width only.

## Privilege / data handling rules
- Works from schema-level exports and sanitized configs only.
- Sandbox findings involving regulated-data exposure must be flagged for compliance specialist review.
- Hyperforce data residency gaps in regulated industries (healthcare, finance) must trigger escalation review.

## Handoff rules
- Hands off to: salesforce-permission-model-review-skill (if IP restriction gaps require permission-level review), salesforce-zero-trust-maturity-skill (if session or network findings indicate broader ZTA gaps), salesforce-case-capsule (for any Critical finding requiring human authorization).
- Required handoff fields: matter_id, risk_register (summary), escalation_gates_fired, missing_evidence, assumptions.

## Audit log fields
- matter_id, skill_id, skill_version, invoked_by, input_hash, evidence_quality, output_verdict, escalation_fired, timestamp

## Stop conditions
- Export contains live credentials, session tokens, or unredacted customer PII — stop and request sanitized version.
- Hyperforce data residency gap confirmed in a regulated industry with active data-localization obligation — stop, output ESCALATE, require compliance specialist review before continuing.
- Sandbox confirmed to contain unmasked production PII with external-vendor access — stop and require human review.

## Security notes
- Read-only static review; never requests live org access or API credentials.
- Sanitized inputs only; any input containing credentials or session tokens must be refused.
- Risk register is advisory; remediation requires human-authorized change management.
- CSP Trusted Sites wildcard entries are always at least High severity regardless of context.
