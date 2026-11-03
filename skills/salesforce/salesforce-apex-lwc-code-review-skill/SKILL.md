---
name: salesforce-apex-lwc-code-review-skill
description: 'Use this skill when Salesforce Apex classes, triggers, Lightning Web Components (LWC), or async jobs (Queueable, Batch, Future, Schedulable) must be reviewed for security vulnerabilities and governor-limit risk. Flags: SOQL and DML inside loops, missing test coverage patterns, WITH SECURITY_ENFORCED or WITH USER_MODE or stripInaccessible usage, sharing keyword omission, governor-limit risk, LWC XSS surface, and Locker Service issues. Trigger phrases: "review this Apex class", "check this trigger for SOQL in loops", "is this LWC safe", "review this batch job", "check sharing enforcement in this code". Do not use when Flow automation (not code) is the primary subject (use salesforce-flow-automation-review-skill), when metadata quality is the focus (use salesforce-metadata-review-skill), or when a live code deployment is proposed (use salesforce-live-change-approval-protocol). Works from pasted code only; never requests live org access or executes code.'
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-05-20"
  category: security
  lifecycle: experimental
---

# Salesforce Apex and LWC Code Review Skill

## Purpose
This skill reviews Salesforce Apex classes, triggers, Lightning Web Components
(
LWC), and async jobs for security
vulnerabilities, governor-limit risk, test coverage patterns, and sharing
enforcement. It flags patterns that cause data exposure, runaway resource
consumption, or privilege escalation. It does not execute code, access live
orgs, or authorize deployments.

## When to use
- Apex or LWC code must be reviewed before a release.
- A code review has raised security or governor-limit concerns.
- A trigger is suspected of causing SOQL or DML governor limit errors.
- LWC components are being reviewed for XSS or Locker Service issues.
- Async job design (Batch, Queueable, Future, Schedulable) needs safety review.

## When not to use
- Flow automation (not code) is the primary subject — use `salesforce-flow-automation-review-skill`.
- Metadata quality (not code logic) — use `salesforce-metadata-review-skill`.
- Live code deployment proposal — use `salesforce-live-change-approval-protocol`.
- Full org posture assessment — use `salesforce-org-assessment-skill`.

## Minimum payload (required inputs)
- Pasted Apex code (class, trigger, or test class) or LWC component files
  (JS controller, HTML template, or Apex controller).
- Context: what the code does, what object(s) it operates on, execution context
  (trigger, batch, scheduled, invocable, REST endpoint, LWC component).
- Whether the code is deployed in production, sandbox, or scratch org.

## Workflow

### 1. SOQL inside loops
- Scan for SOQL queries (`[SELECT ... FROM ...]`) inside `for` loops,
  `while` loops, or recursive method calls.
- Flag every SOQL-in-loop occurrence as a governor-limit risk.
- Recommend: bulkify by collecting IDs outside the loop and running a single
  SOQL query.

### 2. DML inside loops
- Scan for DML statements (`insert`, `update`, `delete`, `upsert`, `merge`,
  `undelete`) inside loops.
- Flag every DML-in-loop occurrence.
- Recommend: collect records in a list and perform a single DML outside the loop.

### 3. Sharing enforcement
- Check every Apex class declaration for the `sharing` keyword:
  - `with sharing` — enforces the running user's sharing rules (recommended default).
  - `without sharing` — bypasses sharing; flag all usages and require documented justification.
  - `inherited sharing` — flag if used in entry-point classes where explicit control is expected.
  - No keyword — implicitly `without sharing` in most contexts; flag all missing keywords.
- Flag any class that handles PII or financial data and uses `without sharing`
  or omits the keyword.

### 4. Field-level security enforcement
- Check SOQL queries for `WITH SECURITY_ENFORCED` or `WITH USER_MODE` clause.
- Absence on queries returning PII or sensitive fields → flag as data exposure risk.
- Check for `Security.stripInaccessible` usage before accessing returned field values.
- Flag: stripping only on output but not on input (incomplete FLS enforcement).

### 5. Governor limit risk
- SOQL: flag if total distinct SOQL calls per transaction could exceed 100.
- DML: flag if total DML statements per transaction could exceed 150.
- Heap: flag large in-memory collections (Lists, Maps, Sets) that grow with record volume.
- CPU: flag complex nested loops or recursive methods without break conditions.
- Callouts: flag synchronous callouts from triggers (not allowed); flag callouts
  without timeout handling.

### 6. Test coverage patterns
- Check for test classes covering the reviewed code.
- Flag: test classes with `SeeAllData=true` (
deprecated pattern).
- Flag: test methods with no assertions (`System.assertEquals`, `System.assertNotEquals`,
  `System.assert`).
- Flag: test data that is hardcoded with real org IDs or email addresses.
- Flag: missing negative test cases for validation or error paths.

### 7. LWC security surface
- HTML template: flag dynamic `<lightning-formatted-rich-text>` or
  `innerHTML` bindings with unsanitized values (XSS risk).
- JavaScript: flag `eval`, `Function` constructor, or `dangerouslySetInnerHTML`
  equivalents.
- Wire adapters: flag wire adapters returning PII fields exposed in template
  without FLS enforcement in the backing Apex.
- Locker Service
: flag cross-namespace
  DOM access patterns that depend on bypassing Locker Service.

### 8. Async job design
- Batch Apex: flag `Database.Batchable` implementations that query without
  scope limitation; flag missing `finish` method implementations.
- Queueable: flag Queueable chains > configured depth without a termination
  condition (runaway chaining).
- Future: flag `@future(callout=true)` methods called from loops.
- Schedulable: flag scheduled classes that chain into Batch without a
  concurrency guard.

## Evidence requirements
- Pasted code; no credentials, session tokens, or customer data in code comments.
- If code contains hardcoded credentials or tokens, stop and ask for sanitized version.
- Context about the object and field sensitivity helps calibrate sharing and FLS findings.

## Output format
```
apex_lwc_code_review_findings:
  soql_in_loops:
    - location: [class/method or line range]
      severity: High
      recommendation: [brief]
  dml_in_loops:
    - location: [...]
      severity: High
      recommendation: [...]
  sharing_enforcement:
    - class: [name]
      keyword: with sharing | without sharing | inherited sharing | missing
      severity: Critical | High | Medium | Low
      justification_required: true | false
      recommendation: [...]
  fls_enforcement:
    - query_location: [...]
      with_security_enforced: present | absent
      strip_inaccessible: present | partial | absent
      severity: [...]
      recommendation: [...]
  governor_limit_risks:
    - pattern: [description]
      severity: [...]
      recommendation: [...]
  test_coverage_patterns:
    - finding: [description]
      severity: [...]
      recommendation: [...]
  lwc_security_findings:
    - finding: [description]
      severity: [...]
      recommendation: [...]
  async_job_findings:
    - finding: [description]
      severity: [...]
      recommendation: [...]

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
- If pasted code contains hardcoded credentials or tokens, stop and ask for sanitized version.

## Privilege / data handling rules
- Code review is logic-level only; do not carry record data from code samples.
- `without sharing` classes handling PII must be flagged for compliance review.
- Test code containing real org IDs or customer email addresses must be refused.

## Handoff rules
- Hands off to: salesforce-flow-automation-review-skill (if automation logic is intertwined),
  salesforce-permission-model-review-skill (if sharing or FLS findings are systemic),
  salesforce-release-readiness-skill (for deployment readiness after review).
- If escalation gate fires: salesforce-case-capsule with escalation_required = true.
- Required handoff fields: matter_id, critical_count, high_count, escalation_gates_fired,
  sharing_enforcement summary.

## Audit log fields
- matter_id, skill_id, skill_version, invoked_by, input_hash, evidence_quality, output_verdict, escalation_fired, timestamp

## Stop conditions
- Pasted code contains live credentials, session tokens, or customer PII in comments — stop and ask for sanitized version.
- Critical `without sharing` on PII-handling class in production — flag immediately and require human review.
- SOQL-in-loop count is very high (systemic) — escalate to salesforce-org-assessment-skill for full automation review.

## Security notes
- Read-only static code review; never executes code or requests live org access.
- `without sharing` is a security-relevant decision; every usage must have documented justification.
- LWC XSS patterns are particularly high risk in Experience Cloud guest-user context.
- Test class quality is a proxy for deployment safety; missing assertions are a reliability risk.
