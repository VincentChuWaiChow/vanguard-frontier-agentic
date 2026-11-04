---
name: salesforce-integration-review-skill
description: 'Use this skill when a Salesforce integration design must be reviewed for API choice, middleware position, retry and idempotency patterns, error queue design, observability, secret handling, OAuth scope minimization, named credential vs callout patterns, and MuleSoft vs point-to-point architecture. Trigger phrases: "review this Salesforce integration", "is this API design safe", "check OAuth scope for this integration", "review our MuleSoft pattern", "assess this Platform Events design". Do not use when Apex code quality (not integration design) is the focus (use salesforce-apex-lwc-code-review-skill), when a live integration change is being deployed (use salesforce-live-change-approval-protocol), or when marketing data flows are the subject (use salesforce-marketing-consent-review-skill). Works from sanitized integration design documents only; never requests live credentials or direct API access.'
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-05-20"
  category: architecture
  lifecycle: experimental
---

# Salesforce Integration Review Skill

## Purpose
This skill reviews Salesforce integration designs for API choice, middleware
positioning, retry/idempotency, error queue design, observability, secret
handling, OAuth scope minimization, named credential patterns, and
MuleSoft vs point-to-point architecture decisions. It surfaces risks,
anti-patterns, and recommendations. It does not access live systems,
test endpoints, or request credentials.

## When to use
- An integration architecture is being designed or reviewed before build.
- An existing integration is being audited for security or reliability.
- A new API connection is being added and the design must be assessed.
- An incident has occurred and the integration design is under review.

## When not to use
- Apex code quality is the focus — use `salesforce-apex-lwc-code-review-skill`.
- Marketing data flow consent review — use `salesforce-marketing-consent-review-skill`.
- Live integration change deployment — use `salesforce-live-change-approval-protocol`.
- Full org posture — use `salesforce-org-assessment-skill`.

## Minimum payload (required inputs)
- Integration design description: systems involved, data flows, direction, frequency.
- API type(s) used (REST, SOAP, Bulk, Streaming, Platform Events,
  CDC, external services).
- Authentication method (OAuth, session ID, named credential, API key).
- Middleware presence: MuleSoft
,
  other iPaaS, or point-to-point.
- Error handling and retry design (or note that it is undocumented).

## Workflow

### 1. API choice review
- Assess whether the chosen API is appropriate for the use case:
  - REST API: appropriate for record CRUD and metadata; flag if used for
    bulk data (> configurable threshold records per batch) without Bulk API.
  - SOAP API: flag if used for new integrations (REST is preferred);
    acceptable for legacy systems.
  - Bulk API : verify serial vs parallel
    job selection; flag if used for real-time use cases requiring low latency.
  - Streaming API / Platform Events :
    verify subscriber durability; flag if subscriber does not handle replay.
  - Change Data Capture : verify object
    selection; flag if all-object CDC is enabled without consumption capacity planning.
  - External Services : verify OpenAPI
    spec version; flag if used for high-volume synchronous patterns.

### 2. Middleware position
- Assess middleware role: event broker, transformation, routing, error handling.
- Flag: point-to-point connections that bypass a middleware layer for non-trivial
  data transformation.
- Flag: middleware not present for integrations crossing security or compliance
  domains.
- MuleSoft : flag if the Anypoint
  platform is used but policies (rate limiting, IP allowlisting, threat
  protection) are not documented.

### 3. Retry and idempotency
- Verify that retry logic exists for transient failures (network timeouts,
  5xx errors).
- Flag: retry without idempotency key (can create duplicate records on retry).
- Flag: exponential backoff absent (linear retry can cause thundering herd).
- Flag: infinite retry without a dead-letter or poison-message mechanism.

### 4. Error queue design
- Verify that failed messages are routed to an error queue or dead-letter topic.
- Flag: silent failure (errors logged but no queue, no alert, no human review).
- Flag: error queue not monitored or without a defined SLA for remediation.
- Flag: failed records re-inserted without deduplication logic.

### 5. Observability
- Verify that integration events are logged (at minimum: request ID, timestamp,
  direction, status, error code).
- Flag: logging that includes PII or credentials (over-logging).
- Flag: no alerting on error rate threshold breach.
- Flag: no end-to-end tracing that correlates Salesforce event to external system.

### 6. Secret handling
- Flag: API keys, client secrets, or passwords hardcoded in Apex, Flow, or
  integration configuration.
- Verify: Named Credentials  or
  External Credentials used for all outbound callouts.
- Flag: session ID used as an API credential (session IDs expire and are
  not suitable for long-running integrations).
- Flag: client secrets stored in Custom Settings or Custom Metadata without
  encryption.

### 7. OAuth scope minimization
- Review the OAuth scopes requested by connected apps.
- Flag: `full` scope (grants all permissions; should never be used in production).
- Flag: `api` scope without specific object-level restriction where a more
  restricted scope is available.
- Flag: `refresh_token` scope on integrations that do not require long-lived access.
- Recommend: minimum-necessary scope per the integration's function.

### 8. Named credential vs direct callout
- Verify that outbound callouts use Named Credentials
  or External Credentials.
- Flag: callouts using hardcoded URLs or credentials in Apex code.
- Flag: callouts to endpoints not in the org's remote site settings
  (
CSP / remote site settings).

### 9. MuleSoft vs point-to-point assessment
- If MuleSoft  is present: verify
  that it is used for all integration routing; flag bypasses.
- If point-to-point: flag integrations > configurable complexity threshold
  that should be routed through a middleware layer.
- Flag: fan-out from Salesforce to > configurable number of external systems
  without a message broker.

## Evidence requirements
- Sanitized integration design document or description; no credentials, tokens,
  or customer data.
- List of API types, authentication methods, and middleware present.
- Error handling and retry design documentation (or note that it is absent).

## Output format
```
integration_review_findings:
  api_choice:
    - finding: [description]
      severity: Critical | High | Medium | Low
      recommendation: [brief]
  middleware_position: [same structure]
  retry_idempotency: [same structure]
  error_queue_design: [same structure]
  observability: [same structure]
  secret_handling: [same structure]
  oauth_scope: [same structure]
  named_credential_usage: [same structure]
  middleware_vs_point_to_point: [same structure]

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
- If design documents contain real credentials or endpoint secrets, stop and ask for sanitized version.

## Privilege / data handling rules
- Integration review is design-level only; do not carry actual API responses or payload samples.
- Integrations crossing regulated-data domains (HIPAA, PCI) must be flagged for compliance review.

## Handoff rules
- Hands off to: salesforce-apex-lwc-code-review-skill (if callout Apex code needs review),
  salesforce-marketing-consent-review-skill (if integration involves Marketing Cloud data flows),
  salesforce-data-exposure-escalation-protocol (if integration creates data exposure risk).
- If escalation gate fires: salesforce-case-capsule with escalation_required = true.
- Required handoff fields: matter_id, critical_count, escalation_gates_fired,
  secret_handling summary.

## Audit log fields
- matter_id, skill_id, skill_version, invoked_by, input_hash, evidence_quality, output_verdict, escalation_fired, timestamp

## Stop conditions
- Design document contains live credentials, session tokens, or real API keys — stop and ask for sanitized version.
- Integration involves regulated-data cross-org transfer without a documented DPA — fire production-data-exposure gate.
- `full` OAuth scope detected in production — Critical finding; require immediate human review.

## Security notes
- Read-only static design review; never tests endpoints or requests live credentials.
- `full` OAuth scope is always a Critical finding in production integrations.
- Named Credentials are the required pattern for outbound callouts; deviations require documented justification.
- DPA obligations for cross-org data transfers must be verified with legal counsel.
