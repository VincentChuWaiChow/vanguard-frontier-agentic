---
name: salesforce-marketing-consent-review-skill
description: 'Use this skill when marketing data flows must be reviewed for consent capture, lawful basis, purpose limitation, preference center coverage, suppression list integrity, subscriber-key collision risk, deliverability authentication (SPF, DKIM, DMARC), and unsubscribe link integrity. Covers Marketing Cloud, Account Engagement (formerly Pardot), Data Cloud, and related marketing orchestration tools. Trigger phrases: "review our marketing consent model", "check this Marketing Cloud data flow for consent", "is our preference center compliant", "review subscriber key design", "check SPF and DKIM for our sending domain". Do not use when the subject is a general Salesforce integration not involving marketing data (use salesforce-integration-review-skill), when a live Marketing Cloud change is being deployed (use salesforce-live-change-approval-protocol), or when a data exposure event has occurred (use salesforce-data-exposure-escalation-protocol). Works from sanitized design documents and configuration exports only; never requests live credentials or accesses marketing accounts.'
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-05-20"
  category: compliance
  lifecycle: experimental
---

# Salesforce Marketing Consent Review Skill

## Purpose
This skill reviews marketing data flows for consent capture, lawful basis,
purpose limitation, preference center coverage, suppression list integrity,
subscriber-key collision risk, deliverability authentication, and unsubscribe
link integrity. It exists because marketing data flows — particularly in
Marketing Cloud , Account Engagement
, and Data Cloud
 — frequently cross jurisdictions and
involve regulated personal data subject to GDPR, CCPA, CASL, and other
consent-based frameworks. It does not access live marketing accounts or
authorize changes.

## When to use
- A marketing data flow is being designed or reviewed for compliance.
- A consent model is being assessed for a new campaign or channel.
- A deliverability issue may be related to authentication configuration.
- A preference center redesign needs consent-model review.
- A subscriber-key migration or Data Cloud integration is planned.

## When not to use
- General Salesforce integration (no marketing data) — use `salesforce-integration-review-skill`.
- Live Marketing Cloud change deployment — use `salesforce-live-change-approval-protocol`.
- Data exposure event response — use `salesforce-data-exposure-escalation-protocol`.
- Full org assessment — use `salesforce-org-assessment-skill`.

## Minimum payload (required inputs)
- Description of the marketing data flow: source systems, destination (Marketing
  Cloud, Account Engagement, Data Cloud), data categories involved.
- Jurisdiction(s) where subscribers are located (or note that it is unknown).
- Consent model description: how consent is captured, what lawful basis is claimed.
- Preference center description (or note that it is absent).
- Sending domain(s) (sanitized, no live API keys or credentials).

## Workflow

### 1. Consent capture review
- Identify where consent is captured: web form, API, import, point of sale.
- Flag: consent captured without a clear affirmative action (pre-ticked boxes,
  bundled consent).
- Flag: consent captured without a timestamp and source record (not auditable).
- Flag: consent captured via a third-party list import without documented
  lawful basis for the jurisdiction.
- Flag: re-permission campaigns not used when consent records are > configurable
  age threshold.

### 2. Lawful basis assessment
- Identify the claimed lawful basis per jurisdiction:
  - GDPR (EU/EEA): consent, legitimate interest (with documented LIA),
    contract, legal obligation.
  - CCPA (California): right to opt-out of sale; identify whether the
    flow constitutes a "sale" under CCPA.
  - CASL (Canada): express vs implied consent; flag implied consent without
    an expiry tracking mechanism.
  - Other jurisdictions: flag if jurisdiction is identified but lawful basis
    is not documented.
- Flag: single lawful basis asserted globally when multi-jurisdiction subscribers
  require jurisdiction-specific bases.
- Flag: legitimate interest asserted without a documented Legitimate Interest
  Assessment (LIA).

### 3. Purpose limitation
- Verify that data collected for one purpose is not used for a materially
  different marketing purpose without a separate consent.
- Flag: contact data collected in a Service Cloud context being synced to
  Marketing Cloud for promotional campaigns without a separate consent capture.
- Flag: Data Cloud  segments built
  from data collected under a different purpose than marketing.

### 4. Preference center coverage
- Verify that a preference center exists and covers all active channels
  (email, SMS, push, direct mail).
- Flag: preference center that does not honor opt-outs within a documented
  processing time (e.g., 10 business days).
- Flag: preference center that does not propagate suppression to all active
  sending platforms (Marketing Cloud, Account Engagement, and any third-party
  senders).
- Flag: preference center that requires an account login to opt out (barrier to
  opt-out is a compliance risk).

### 5. Suppression list integrity
- Verify that suppression lists (global unsubscribes, do-not-contact lists,
  hard bounces) are applied across all sending platforms.
- Flag: suppression list sync with > configurable delay (stale suppression
  can result in sending to opted-out subscribers).
- Flag: suppression list managed manually without automated sync to all platforms.
- Flag: hard bounces not suppressed (can damage sender reputation and may
  violate CAN-SPAM/CASL).

### 6. Subscriber-key design and collision risk
- Review the subscriber key design in Marketing Cloud
  .
- Flag: subscriber keys using email addresses as the key (email changes cause
  key collisions and duplicate subscriber records).
- Flag: subscriber key not synchronized with the CRM contact ID (leads to
  orphaned subscriber records).
- Flag: subscriber key strategy not defined before Data Cloud integration
  (can cause identity resolution failures).

### 7. Deliverability authentication
- Review the sending domain configuration:
  - SPF: verify that an SPF record exists for the sending domain and includes
    Marketing Cloud sending IPs.
  - DKIM: verify that DKIM signing is configured for the sending domain.
  - DMARC: verify that a DMARC policy exists; flag if policy is `p=none`
    (monitoring only, no enforcement).
- Flag: sending from a shared IP pool without dedicated IP warm-up plan.
- Flag: DMARC policy `p=reject` or `p=quarantine` without monitoring in place
  (can result in false positives if misconfigured).

### 8. Unsubscribe link integrity
- Verify that all email sends include a functional unsubscribe link.
- Flag: unsubscribe link that requires authentication or account creation.
- Flag: unsubscribe that routes to a preference center with a long opt-out
  process (more than 2 clicks).
- Flag: transactional email templates where an unsubscribe link is absent but
  the email contains promotional content.
- Flag: one-click unsubscribe not implemented where List-Unsubscribe header
  is expected by receiving mail providers.

## Evidence requirements
- Sanitized marketing data flow description; no credentials, API keys, or
  customer data.
- Jurisdiction(s) identified or noted as unknown.
- Consent model description; if absent, flag as a high-risk unknown.

## Output format
```
marketing_consent_review_findings:
  consent_capture:
    - finding: [description]
      severity: Critical | High | Medium | Low
      jurisdiction: [applicable, or "all"]
      recommendation: [brief]
  lawful_basis: [same structure]
  purpose_limitation: [same structure]
  preference_center: [same structure]
  suppression_list: [same structure]
  subscriber_key_design: [same structure]
  deliverability_authentication: [same structure]
  unsubscribe_integrity: [same structure]

summary:
  total_findings: [count]
  critical_count: [count]
  high_count: [count]
escalation_gates_fired: [from salesforce-risk-taxonomy, or "none"]
open_questions_for_counsel: [list — do not answer; require legal determination]
assumptions: [list]
missing_evidence: [what would improve the review]
```

## Redaction rules
- Never request secrets, credentials, OAuth tokens, refresh tokens, session IDs, MFA seeds, customer PII.
- Sanitize org IDs, user IDs (replace with placeholders) before sharing in outputs.
- Subscriber data, email addresses, and contact records must not appear in outputs.

## Privilege / data handling rules
- This review may surface findings that have regulatory notification implications.
  Flag and route to privacy counsel before any public statement.
- Lawful basis assessments are not legal conclusions; all findings require
  verification by qualified privacy counsel.
- Regulated-data marketing flows (health, financial) escalate to compliance review.

## Handoff rules
- Hands off to: salesforce-data-exposure-escalation-protocol (if consent violation
  constitutes a data exposure event), salesforce-integration-review-skill (if
  integration design gaps underlie consent failures), salesforce-case-capsule
  (structured handoff for any Critical finding requiring escalation).
- Required handoff fields: matter_id, critical_count, escalation_gates_fired,
  open_questions_for_counsel.

## Audit log fields
- matter_id, skill_id, skill_version, invoked_by, input_hash, evidence_quality, output_verdict, escalation_fired, timestamp

## Stop conditions
- Data flow involves regulated health or financial data sent for marketing purposes
  without documented consent — fire production-data-exposure gate and escalate.
- Suppression list is confirmed as stale and sends are ongoing — Critical finding;
  recommend immediate send pause for human review.
- Lawful basis is absent for GDPR-jurisdiction subscribers — Critical finding;
  escalate to privacy counsel before any further sends.

## Security notes
- Read-only static review; never accesses live Marketing Cloud accounts or APIs.
- Lawful basis findings are not legal conclusions; they require verification by
  qualified privacy counsel.
- Unsubscribe failures are a regulatory risk in most jurisdictions; treat as High
  even when severity appears low.
- SPF, DKIM, and DMARC findings should be verified against current DNS records
  by the sending organization's technical team.
