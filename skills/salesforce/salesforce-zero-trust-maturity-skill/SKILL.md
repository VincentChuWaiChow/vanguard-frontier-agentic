---
name: salesforce-zero-trust-maturity-skill
description: 'Use this skill when Salesforce deployment zero-trust readiness must be evaluated against NIST SP 800-207 ZTA principles — covering continuous verification configuration, MFA and Always-On MFA status, OAuth token lifetimes, session re-authentication triggers, certificate lifecycle health, CA-signed vs. self-signed certificate posture, mTLS enforcement for external integrations via Named Credentials, contextual risk scoring via Transaction Security Policies, and Event Monitoring behavioral baselines. Trigger phrases: "assess zero-trust maturity", "review Salesforce zero trust", "evaluate continuous verification", "check certificate lifecycle", "audit mTLS configuration", "review adaptive authentication". Do not use when network or infrastructure policies are the focus (use salesforce-infrastructure-audit-skill), when identity and IAM permissioning is the focus (use salesforce-permission-model-review-skill), or when a live deployment or change approval is needed (use salesforce-live-change-approval-protocol). Works from sanitized config exports only; never requests live org access.'
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-05-21"
  category: security
  lifecycle: experimental
---

# Salesforce Zero Trust Maturity Skill

## Purpose
This skill evaluates Salesforce deployment zero-trust readiness using NIST SP 800-207 ZTA principles across five pillars: Device, Network, Application, User, and Automation. It reviews continuous verification configuration, certificate lifecycle health, mTLS enforcement for external integrations, adaptive authentication triggers via Transaction Security Policies, and behavioral baseline coverage via Event Monitoring. It produces a pillar-scored ZTA maturity assessment with a prioritized gap register. It does not access live orgs or execute API calls.

## When to use
- Zero-trust architecture compliance is required for a regulatory audit or certification.
- Certificate expiry or rotation schedule gaps need to be identified before a security review.
- mTLS configuration for Named Credentials or external service integrations must be assessed.
- Adaptive authentication and Transaction Security Policy coverage is being reviewed.
- Event Monitoring behavioral baseline presence must be confirmed or documented.
- An overall NIST ZTA pillar score is needed to benchmark Salesforce deployment maturity.

## When not to use
- Network access policies, IP allowlisting, sandbox isolation, or Hyperforce controls — use `salesforce-infrastructure-audit-skill`.
- Identity and IAM permission model review (profiles, permission sets, sharing) — use `salesforce-permission-model-review-skill`.
- Live deployment or change approval workflow — use `salesforce-live-change-approval-protocol`.
- Full org posture assessment combining all domains — use `salesforce-org-assessment-skill`.

## Minimum payload (required inputs)
- MFA configuration export: MFA enforcement status per profile, Always-On MFA status.
- OAuth token configuration: access token lifetime, refresh token lifetime, connected app policies.
- Session re-authentication configuration: high-assurance session triggers, re-auth timeout settings.
- Certificate inventory: certificate names, expiry dates, issuing CA (CA-signed vs. self-signed), rotation schedule.
- Named Credential configuration: auth protocol per credential (OAuth 2.0, certificate, password), mTLS flag.
- Transaction Security Policies: policy names, event types monitored, actions configured.
- Event Monitoring configuration: enabled log types, retention period, SIEM or alerting integration.
- Context: industry vertical, regulatory framework, approximate integration count.

## Workflow

### 1. Continuous verification inventory
- Review MFA enforcement status: confirm MFA is enforced for all internal users, not optional or waivable.
- Flag: MFA not enforced for any profile with `API Enabled`, `Modify All Data`, or `Manage Users`.
- Review Always-On MFA
configuration status.
- Flag: Always-On MFA not enabled in production for orgs with regulated-data classifications.
- Review OAuth access token lifetime: flag lifetimes > 2 hours for integrations with sensitive object access.
- Review OAuth refresh token lifetime: flag never-expiring refresh tokens on connected apps used in production.
- Review session re-authentication triggers: confirm high-assurance sessions are required for sensitive operations (e.g., report export, data download, user management).
- Flag: high-assurance session triggers absent or not mapped to sensitive operation types.

### 2. Certificate lifecycle health
- List all certificates in the certificate inventory.
- Flag: any certificate expiring within 90 days (High) or within 30 days (Critical).
- Flag: self-signed certificates used for external-facing integrations or Named Credentials.
- Flag: certificate rotation schedule absent or not documented.
- Flag: certificates issued by untrusted or internal-only CAs used in regulated integration paths.
- Record expiry date distribution, CA-signed vs. self-signed ratio, and rotation schedule coverage.

### 3. mTLS review for external integrations
- Review Named Credential auth protocols.
- Flag: Named Credentials using username-password auth for integrations that support certificate or OAuth 2.0.
- Flag: Named Credentials accessing regulated-data endpoints without mTLS configured.
- Review external service integrations (REST, SOAP, platform events) for mTLS enforcement.
- Flag: integrations with external payment, identity, or healthcare endpoints lacking mutual TLS.
- Record mTLS coverage rate across Named Credentials and external service definitions.

### 4. Adaptive access controls
- Review Transaction Security Policy inventory.
- Flag: no Transaction Security Policies configured (absence is a gap regardless of org size).
- Flag: Transaction Security Policies present but configured to notify-only without block or MFA-prompt action for Critical event types (e.g., ExportLeads, ReportExport, ApiAnomalousUsage).
- Review Event Monitoring configuration.
- Flag: Event Monitoring not enabled or licensed but not configured.
- Flag: Event Monitoring enabled but no SIEM, alerting integration, or retention > 30 days configured.
- Flag: anomaly detection for login, API usage, or report access not configured.
- Record Transaction Security Policy count, action types, and Event Monitoring log coverage.

### 5. NIST ZTA pillar scoring
Score each pillar 0–4 using the scale: 0 = Not Present, 1 = Initial, 2 = Developing, 3 = Defined, 4 = Optimized.

| Pillar | Key Controls Assessed |
|---|---|
| User (U) | MFA enforcement, Always-On MFA, session re-auth, OAuth token lifetimes |
| Device (D) | Certificate lifecycle, CA-signed posture, mTLS on Named Credentials |
| Network (N) | mTLS for external integrations, session IP locking (from infra audit if available) |
| Application (A) | Transaction Security Policies, Event Monitoring, anomaly detection |
| Automation (Au) | OAuth refresh token policy, connected app IP restrictions, named credential auth protocols |

- Produce a D/N/A/U/Au score tuple.
- Flag: any pillar scoring 0 or 1 as a maturity gap requiring remediation roadmap.

### 6. Risk register assembly
- Consolidate findings from steps 1–5.
- Assign risk_tier per finding: Critical | High | Medium | Low.
- Map each finding to its ZTA pillar and relevant NIST SP 800-207 control reference.
- Identify findings that meet escalation gates from salesforce-risk-taxonomy.

## Evidence requirements
- Certificate inventory with expiry dates and CA details is required for step 2.
- Named Credential auth protocol list is required for step 3.
- Transaction Security Policy and Event Monitoring configuration are required for step 4.
- Absence of any required input produces an "insufficient evidence" note with assumed worst-case scoring for that pillar.

## Output format
```
zero_trust_maturity_findings:
  continuous_verification_findings:
    - finding: [description]
      severity: Critical | High | Medium | Low
      zta_pillar: User | Device | Network | Application | Automation
      nist_reference: [NIST SP 800-207 section, if applicable]
      evidence: [what in the export supports this]
      recommendation: [brief]
  certificate_findings: [same structure]
  mtls_findings: [same structure]
  adaptive_access_findings: [same structure]

zta_pillar_scores:
  user: [0-4]
  device: [0-4]
  network: [0-4]
  application: [0-4]
  automation: [0-4]
  overall_maturity_tier: Initial | Developing | Defined | Optimized

escalation_gates_fired: [from salesforce-risk-taxonomy, or "none"]
summary:
  critical_count: [count]
  high_count: [count]
  pillars_at_initial_or_below: [list]
assumptions: [list]
missing_evidence: [what would improve the review]
```

## Redaction rules
- Never request secrets, credentials, OAuth tokens, refresh tokens, session IDs, MFA seeds, customer PII.
- Sanitize org IDs, user IDs before sharing in outputs.
- Do not include actual certificate private key material or thumbprints in outputs; use expiry dates and CA references only.

## Privilege / data handling rules
- Works from schema-level exports and sanitized configs only.
- Certificate expiry findings in regulated industries must be flagged for immediate remediation review.
- Event Monitoring absence in orgs handling regulated data must trigger compliance specialist notification.

## Handoff rules
- Hands off to: salesforce-infrastructure-audit-skill (if network or session controls need infrastructure-layer review), salesforce-permission-model-review-skill (if MFA gaps require permission-level remediation), salesforce-case-capsule (for any Critical finding or pillar score of 0).
- Required handoff fields: matter_id, risk_register (summary), escalation_gates_fired, missing_evidence, assumptions.

## Audit log fields
- matter_id, skill_id, skill_version, invoked_by, input_hash, evidence_quality, output_verdict, escalation_fired, timestamp

## Stop conditions
- Export contains live credentials, session tokens, certificate private keys, or unredacted customer PII — stop and request sanitized version.
- Certificate expiry confirmed within 7 days in a production integration path — stop, output ESCALATE, require immediate human action.
- Never-expiring OAuth refresh tokens confirmed on a production connected app with sensitive object access — stop and require human review before continuing.

## Security notes
- Read-only static review; never requests live org access or API credentials.
- Sanitized inputs only; any input containing credentials or private key material must be refused.
- ZTA pillar scores are advisory maturity indicators; remediation requires human-authorized change management.
- NIST SP 800-207 references are informational; compliance determination requires qualified assessor review.
