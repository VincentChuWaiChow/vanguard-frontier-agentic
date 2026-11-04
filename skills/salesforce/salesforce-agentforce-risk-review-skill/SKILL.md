---
name: salesforce-agentforce-risk-review-skill
description: 'Use this skill when an Agentforce or Salesforce AI agent configuration must be reviewed for grounding quality, retrieval scope, action allowlist safety, human handoff design, hallucination containment, prompt injection surface, autonomous action boundary, audit logging, and model-risk controls. Trigger phrases: "review this Agentforce configuration", "assess the risk of this AI agent", "check the action allowlist for this agent", "review grounding and retrieval scope", "is this autonomous AI action safe". Do not use when the subject is a general Salesforce Flow or automation (use salesforce-flow-automation-review-skill), when a live Agentforce deployment is being pushed to production (use salesforce-live-change-approval-protocol), or when the focus is general Apex code quality (use salesforce-apex-lwc-code-review-skill). All Agentforce and Einstein feature names carryannotations; validate current terminology against official Salesforce documentation before use. Works from sanitized configuration exports only; never requests live org access.'
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-05-20"
  category: ai
  lifecycle: experimental
---

# Salesforce Agentforce Risk Review Skill

## Purpose
This skill reviews Salesforce Agentforce
and AI agent configurations for model-risk controls, grounding quality,
retrieval scope, action allowlist safety, human handoff design, hallucination
containment, prompt injection surface, autonomous action boundaries, and audit
logging. It exists because AI agents operating on live Salesforce data and
business processes carry unique risks — autonomous actions, scope creep, and
hallucination-driven errors can cause irreversible data changes or compliance
failures. It does not access live orgs or execute agent actions.

**notice:** All Agentforce, Einstein, Data Cloud, and
related AI feature names in this skill carry a verification requirement.
Salesforce AI product naming, feature availability, and configuration
interfaces evolve rapidly. Validate all product references against the
current official Salesforce documentation before use.

## When to use
- An Agentforce agent configuration is being reviewed before deployment.
- An AI agent's action allowlist has been expanded and must be risk-assessed.
- A human handoff design for an AI agent needs review.
- An AI agent has taken an unexpected action and the configuration must be analyzed.
- Model-risk controls for a Salesforce AI deployment must be documented.

## When not to use
- General Salesforce Flow or automation (not AI) — use `salesforce-flow-automation-review-skill`.
- Live Agentforce deployment to production — use `salesforce-live-change-approval-protocol`.
- Apex code quality (not AI configuration) — use `salesforce-apex-lwc-code-review-skill`.
- Marketing AI features (consent and data flow focus) — use `salesforce-marketing-consent-review-skill`.

## Minimum payload (required inputs)
- Sanitized Agentforce  agent
  configuration: agent name, role description, topic assignments, action
  allowlist, grounding configuration.
- Description of the agent's intended scope and business function.
- Human handoff design: conditions under which the agent escalates to a human.
- Audit logging configuration (or note that it is undocumented).
- Environment: sandbox or production (use placeholder).

## Workflow

### 1. Grounding and retrieval scope review
- Verify that the agent's knowledge base or retrieval source is scoped to
  the minimum necessary data.
- Flag: agents with retrieval access to all records in an object without
  field-level or record-level restriction.
- Flag: agents using Einstein Search
  or Data Cloud  knowledge without
  documented retrieval scope limits.
- Flag: retrieval of data from objects containing PII or regulated data
  without documented access justification.
- Flag: agents grounded on unversioned or unmonitored knowledge sources
  (stale grounding can cause hallucination).

### 2. Action allowlist review
- Review the list of actions the agent is permitted to take (Apex actions,
  Flow invocations, external callouts).
- Flag: actions that mutate production records without a human confirmation step.
- Flag: actions that send external communications (email, SMS) autonomously.
- Flag: actions that access financial, HR, or regulated-data records.
- Flag: actions not explicitly allowlisted (if the agent has a default-permit
  posture, flag as Critical).
- Flag: actions that can invoke other agents or escalate permissions.

### 3. Human handoff design
- Verify that human handoff conditions are defined.
- Flag: no handoff condition (agent runs to completion without any human
  escalation path).
- Flag: handoff conditions that are too narrow (agent can be steered away
  from handoff by adversarial input).
- Flag: handoff destination is not a monitored queue or live human (voicemail,
  unmonitored inbox).
- Flag: handoff does not preserve conversation context (human receives no
  session summary).

### 4. Hallucination containment
- Verify that the agent's responses are grounded in retrieved data, not
  model-generated facts.
- Flag: agent configured to answer questions about policies, regulations, or
  financial data without a retrieval step (pure generation from model).
- Flag: no output validation or confidence threshold before external-facing
  response.
- Flag: agent presents retrieved data as guaranteed fact without a confidence
  indicator.

### 5. Prompt injection surface
- Review how user input is incorporated into agent prompts.
- Flag: user-supplied text inserted directly into a system prompt or
  instruction template without sanitization.
- Flag: agent configured to follow instructions in user-supplied documents
  (uploaded files, email bodies) without an injection guard.
- Flag: agent lacks detection for adversarial instructions embedded in
  retrieved knowledge.

### 6. Autonomous action boundary
- Verify that autonomous action boundaries are explicitly defined and enforced.
- Flag: agent can take actions across multiple records in a single invocation
  without a configurable limit.
- Flag: agent can chain actions in a loop without a termination condition.
- Flag: agent has write access to objects it does not need to read
  (least-privilege violation).
- Flag: sandbox-only actions enabled in the production agent configuration.
- Flag: agent can modify its own configuration or the configurations of other
  agents.

### 7. Audit logging
- Verify that agent actions are logged with: agent ID, action taken, record
  affected (placeholder), timestamp, user or session context, outcome.
- Flag: audit logging disabled or not configured.
- Flag: audit log does not capture failed actions or handoff events.
- Flag: audit log not retained for the minimum required period for the industry
  vertical.
- Flag: audit log accessible to the agent itself (self-modification risk).

### 8. Model-risk controls
- Verify that model-risk controls are documented:
  - Model version pinned or version-change alert in place.
  - Acceptable use policy for the AI feature defined.
  - Human oversight mechanism for high-stakes actions.
  - Incident response plan for AI-driven errors.
- Flag: no model version pinning (silent model update can change agent behavior).
- Flag: no acceptable use policy documented for the AI deployment.
- Flag: no incident response plan for AI-driven errors.

## Evidence requirements
- Sanitized agent configuration export or detailed description; no credentials,
  session tokens, or customer data.
- Action allowlist (complete list of permitted actions).
- Human handoff conditions.
- Audit logging configuration.

## Output format
```
agentforce_risk_review_findings:
  grounding_retrieval:
    - finding: [description]
      severity: Critical | High | Medium | Low
      verify_before_merge: [feature name if applicable]
      recommendation: [brief]
  action_allowlist: [same structure]
  human_handoff: [same structure]
  hallucination_containment: [same structure]
  prompt_injection: [same structure]
  autonomous_action_boundary: [same structure]
  audit_logging: [same structure]
  model_risk_controls: [same structure]

summary:
  total_findings: [count]
  critical_count: [count]
  high_count: [count]
escalation_gates_fired: [from salesforce-risk-taxonomy — autonomous-ai-action gate if applicable]
verify_before_merge_items: [list of product names requiring verification]
assumptions: [list]
missing_evidence: [what would improve the review]
```

## Redaction rules
- Never request secrets, credentials, OAuth tokens, refresh tokens, session IDs, MFA seeds, customer PII.
- Sanitize org IDs, user IDs, and agent session IDs (replace with placeholders) before sharing in outputs.
- Agent conversation logs containing customer data must not be included in review inputs.

## Privilege / data handling rules
- AI agent configurations involving regulated data (health, financial) escalate to compliance review.
- Audit logs are evidence of AI behavior; handle as compliance records.
- Model-risk findings may have regulatory implications in regulated verticals; route to compliance counsel.

## Handoff rules
- Hands off to: salesforce-permission-model-review-skill (if agent permission scope is excessive),
  salesforce-data-exposure-escalation-protocol (if autonomous AI action creates data exposure),
  salesforce-live-change-approval-protocol (if production Agentforce deployment is next),
  salesforce-case-capsule (structured handoff for any Critical finding).
- Required handoff fields: matter_id, critical_count, escalation_gates_fired,
  autonomous_action_boundary summary, verify_before_merge_items.

## Audit log fields
- matter_id, skill_id, skill_version, invoked_by, input_hash, evidence_quality, output_verdict, escalation_fired, timestamp

## Stop conditions
- Agent configuration shows autonomous write access to regulated-data objects without human confirmation — fire autonomous-ai-action gate immediately.
- Action allowlist is effectively unbounded (default-permit) — Critical finding; recommend disabling agent until allowlist is defined.
- Audit logging is disabled in a production agent — Critical finding; escalate to human review before any agent invocation.
- Agent can modify its own configuration — Critical finding; stop and escalate.

## Security notes
- All Agentforce and Einstein feature names require verification against current
  Salesforce documentation before use in production contexts.
- Autonomous action boundary violations are always escalation-grade regardless
  of the action's apparent severity.
- Prompt injection is an active threat surface; agent configurations that
  incorporate uncontrolled user input without sanitization are Critical risks.
- This skill does not execute agent actions, access live agent sessions, or
  retrieve model outputs. Review is configuration-level only.
