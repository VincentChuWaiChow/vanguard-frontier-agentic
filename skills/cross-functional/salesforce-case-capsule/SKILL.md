---
name: salesforce-case-capsule
description: 'Use this skill when a Salesforce specialist agent must hand a matter to another agent and the context, uncertainty, evidence quality, privilege posture, and privacy posture must survive the handoff intact. Defines the shared salesforce-case-capsule — a controlled, auditable exchange record with redacted identifiers, risk labels, privilege and privacy labels, a decision-owner field, and an explicit do-not-do list. Trigger phrases: "create a handoff capsule for this Salesforce matter", "structure this Salesforce handoff", "prepare a Salesforce case capsule". Do not use when you only need to classify a matter (use salesforce-routing-protocol), when you need to assess live-mutation risk (use salesforce-live-change-approval-protocol), or when you need to escalate a data exposure event (use salesforce-data-exposure-escalation-protocol). Does not give Salesforce or business advice and does not authorize any action.'
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-05-20"
  category: platform
  lifecycle: experimental
---

# Salesforce Case Capsule

## Purpose
This skill defines the **salesforce-case-capsule** — the single structured
record that Salesforce specialist agents exchange when a matter crosses an agent
boundary. The capsule exists so that no agent works in a silo, no context is
lost in a handoff, and every cross-domain exchange is auditable. It is a data
contract, not a decision: a capsule never approves, denies, or directs an
action. It carries facts, uncertainty, risk posture, and an explicit decision
owner.

## When to use
- A maestro agent routes a Salesforce matter to one or more specialist agents.
- A specialist agent escalates a matter that has crossed into another domain.
- Any matter touches multiple Salesforce risk domains and must be reviewed
  in parallel.
- A matter must be paused and escalated to a human owner.

## When not to use
- You only need to classify a matter — use `salesforce-routing-protocol`.
- A live org mutation is being proposed — use `salesforce-live-change-approval-protocol`.
- A data exposure event has been triggered — use `salesforce-data-exposure-escalation-protocol`.
- The handoff is within the same agent session without domain crossing.

## Minimum payload (required inputs)
- Description of the Salesforce matter in plain language (sanitized, no credentials or PII).
- Matter type (from salesforce-risk-taxonomy).
- Risk tier (Critical / High / Medium / Low / Unknown).
- Originating agent or human invoker.
- Intended receiving agent(s).

## Core rules
- Every cross-agent handoff MUST be expressed as a capsule. No free-form
  agent-to-agent chatter.
- Every capsule MUST name exactly one `decision_owner` (an accountable human)
  and exactly one `primary_agent`.
- Every capsule MUST carry a `do_not_do_list`. An empty list is not acceptable;
  if nothing is prohibited, the capsule is not ready to send.
- Every capsule MUST label `privilege_sensitivity` and `privacy_sensitivity`.
- Identifiers MUST be redacted to the minimum necessary. Replace org IDs, user
  IDs, customer names, and tenant IDs with placeholders.
- A capsule records uncertainty honestly: `assumptions`, `inferences`, and
  `missing_evidence` are mandatory fields, not optional.
- A capsule never states "this is compliant", "this is approved", or "this
  configuration is safe". `risk_rating` uses risk language only.
- High-risk cross-domain capsules MUST set `escalation_required: true` and
  `recommended_next_action` to a pause-and-escalate posture unless documented
  controls already exist.

## Capsule field set

### Identity and routing
- `matter_id` — unique identifier (generated, never a real org ID)
- `matter_type` — from salesforce-risk-taxonomy matter type list
- `source_agent` — agent or human invoker creating the capsule
- `receiving_agent` — primary receiving agent
- `secondary_agents` — list of parallel-review agents if any
- `primary_agent` — agent with primary accountability
- `decision_owner` — named accountable human
- `org_environment_type` — sandbox / scratch / staging / production (placeholder, not real ID)
- `business_unit` — redacted business unit reference
- `timeline` — relevant dates and deadlines (no PII)

### Evidence discipline
- `facts` — confirmed facts from sanitized inputs
- `assumptions` — inferences the agent made without direct evidence
- `inferences` — derived conclusions labeled as such
- `missing_evidence` — what is unknown and needed
- `evidence_quality` — High / Medium / Low / Insufficient

### Risk posture
- `risk_rating` — Critical / High / Medium / Low / Unknown per salesforce-risk-taxonomy
- `matter_type_flags` — relevant escalation gates from salesforce-risk-taxonomy
- `privilege_sensitivity` — None / Advisory / Legal-Privilege
- `privacy_sensitivity` — None / Internal / PII / Regulated-Data
- `data_exposure_risk` — boolean + description if true
- `guest_user_risk` — boolean
- `autonomous_ai_risk` — boolean
- `finance_revenue_risk` — boolean
- `irreversible_change_risk` — boolean

### Ownership and action
- `human_approval_required` — boolean
- `escalation_required` — boolean
- `escalation_gate_fired` — name of gate from salesforce-risk-taxonomy or null
- `recommended_next_action` — pause-and-escalate / proceed-with-approval / proceed-with-review / blocked
- `do_not_do_list` — explicit list of actions the receiving agent must not take
- `open_questions` — questions that must be answered before the matter can proceed
- `blockers` — conditions that prevent forward progress
- `approval_state` — draft / pending-human-review / approved / rejected
- `audit_log_summary` — one-line summary of routing decision and rationale

## Workflow
1. Receive matter description and classify using salesforce-risk-taxonomy.
2. Assign matter_id (generated placeholder), matter_type, risk_rating.
3. Identify decision_owner, primary_agent, receiving_agent.
4. Populate facts, assumptions, inferences, missing_evidence honestly.
5. Assess risk posture flags (data exposure, guest user, AI, finance, irreversible).
6. Set privilege_sensitivity and privacy_sensitivity.
7. Populate do_not_do_list — must not be empty.
8. Set escalation_required and escalation_gate_fired if applicable.
9. Set recommended_next_action.
10. Output the complete capsule.

## Evidence requirements
- Sanitized matter description (no credentials, tokens, PII, org IDs).
- Matter type classification with basis stated.
- Risk rating with evidence basis stated.
- Named decision owner.

## Output format
Produce a structured capsule with all fields from the field set above.
Label each section clearly. Every `assumptions` entry must state its basis.
Every `do_not_do_list` entry must state the reason.

## Redaction rules
- Never request secrets, credentials, OAuth tokens, refresh tokens, session IDs, MFA seeds, customer PII.
- Sanitize org IDs, user IDs (replace with placeholders such as `[ORG-ID]`, `[USER-ID]`) before including in capsule.
- Never carry privileged email text, legal hold communications, or medical data.

## Privilege / data handling rules
- The capsule is a minimum-necessary record. Do not widen it to carry full
  configuration exports, debug logs with PII, or production data samples.
- A capsule that would extend circulation of privileged or investigation material
  must be narrowed before it is sent.
- Privacy_sensitivity = Regulated-Data requires escalation_required = true.

## Handoff rules
- Capsule hands off to: salesforce-org-assessment-skill, salesforce-metadata-review-skill,
  salesforce-permission-model-review-skill, salesforce-flow-automation-review-skill,
  salesforce-apex-lwc-code-review-skill, salesforce-release-readiness-skill,
  salesforce-integration-review-skill, salesforce-marketing-consent-review-skill,
  salesforce-agentforce-risk-review-skill (per matter_type routing).
- Required handoff fields: matter_id, matter_type, risk_rating, decision_owner,
  do_not_do_list, escalation_required, recommended_next_action.

## Audit log fields
- matter_id, skill_id, skill_version, invoked_by, input_hash, evidence_quality, output_verdict, escalation_fired, timestamp

## Stop conditions
- do_not_do_list cannot be populated — capsule is not ready; gather more context first.
- decision_owner is unknown — stop and ask the invoker to name an accountable human.
- Matter involves regulated personal data and privacy_sensitivity cannot be determined — stop and escalate.
- Capsule would require carrying production credentials or PII — stop and refuse.

## Security notes
- The capsule is a minimum-necessary record. Never authorize action. Any field
  that reads as a directive to deploy, configure, or modify production is a
  defect — rewrite as a recommendation with a named human owner.
- Org IDs and user IDs are always replaced with placeholders before inclusion.
- Capsules with escalation_gate_fired set must be reviewed by a human before
  any downstream agent acts.
