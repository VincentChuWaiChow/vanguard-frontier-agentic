---
name: salesforce-devsecops-pipeline-skill
description: 'Use this skill when Salesforce development pipelines must be reviewed for DevSecOps compliance — covering Salesforce Code Analyzer (SCA) finding triage and false positive review, sandbox data governance and PII masking strategy, change impact analysis across metadata dependencies and downstream automation, CI/CD security gate requirements, and DevOps Center deployment governance. Trigger phrases: "review SCA findings", "triage code analyzer results", "assess sandbox data masking", "review change impact", "audit DevSecOps pipeline", "check CI/CD security gates", "review Salesforce DevOps". Do not use when Apex or LWC code patterns are the focus (use salesforce-apex-lwc-code-review-skill), when release readiness approval is needed (use salesforce-release-readiness-skill), when a live deployment is being executed (use salesforce-live-change-approval-protocol), or when permission model changes are under review (use salesforce-permission-model-review-skill). Works from sanitized exports and tool outputs only; never requests live org access.'
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-05-21"
  category: delivery
  lifecycle: experimental
---

# Salesforce DevSecOps Pipeline Skill

## Purpose
This skill conducts a structured DevSecOps review of Salesforce development pipelines — including Salesforce Code Analyzer (SCA) finding triage, sandbox data governance and PII masking adequacy, change impact assessment across metadata dependencies and downstream automation, CI/CD security gate compliance, and DevOps Center deployment governance. It produces a prioritized remediation register without accessing live orgs or executing deployments. It is the shared workflow called by DevSecOps and pipeline governance agents in the Salesforce agent catalog.

## When to use
- SCA or PMD/ESLint/RetireJS findings from a pipeline run require structured triage and severity classification.
- Sandbox refresh governance and PII masking adequacy must be assessed before data is loaded.
- A change impact analysis is needed to identify metadata dependencies, downstream automation risk, and destructive change exposure.
- CI/CD pipeline security gate configuration must be reviewed against a minimum control baseline.
- DevOps Center deployment governance — work item traceability, approval gates, promotion rules — needs audit.

## When not to use
- Apex or LWC code pattern review (logic-level code review) — use `salesforce-apex-lwc-code-review-skill`.
- Release readiness approval gate with sign-off workflow — use `salesforce-release-readiness-skill`.
- Live deployment execution or change approval — use `salesforce-live-change-approval-protocol`.
- Permission model or sharing rule changes — use `salesforce-permission-model-review-skill`.
- Full org posture assessment combining all domains — use `salesforce-org-assessment-skill`.

## Minimum payload (required inputs)
- SCA output: PMD rule violations, ESLint issues, RetireJS CVEs, tool version, scan scope (full org vs. delta).
- Sandbox inventory relevant to the pipeline: sandbox type, data masking configuration, last refresh date.
- Change inventory: metadata types being deployed, destructive change manifest (if any), field type changes.
- CI/CD pipeline configuration summary: gate names, pass/fail criteria, tool integrations.
- Context: deployment target environment (sandbox, staging, production), release timeline, regulatory framework.

## Workflow

### 1. SCA findings triage
- Review all PMD rule violations from the SCA output.
- Classify findings by severity tier:
  - P1 Critical: security vulnerabilities (SOQL injection, XSS, credential exposure, `without sharing` on PII classes).
  - P2 High: governor-limit risks, missing FLS enforcement, unsafe async patterns.
  - P3 Medium: code quality issues, missing documentation, test coverage gaps.
  - P4 Advisory: style suggestions, minor refactor recommendations.
- Flag: any P1 Critical finding — these must block deployment until resolved.
- Review ESLint issues for LWC components: flag any security-relevant rules (no-eval, no-inner-html, no-unsanitized).
- Review RetireJS CVE list: flag CVEs with CVSS score ≥ 7.0 as P1; flag CVSS 4.0–6.9 as P2.
- Record total finding count by severity tier and tool source (PMD / ESLint / RetireJS).

### 2. False positive review
- For each finding flagged for false positive consideration, require:
  - The rule ID and description.
  - The specific code pattern triggering the finding.
  - A documented rationale for why the pattern is not a risk in this context.
- Flag: false positive claims on security-critical rules (SOQL injection, XSS, credential exposure) — these require security specialist sign-off, not self-service suppression.
- Flag: blanket suppression annotations (`@SuppressWarnings('all')` or equivalent) without scoped justification.
- Record false positive claims with rule IDs and rationale for audit trail.

### 3. Sandbox governance check
- Review sandbox type against the intended test data classification:
  - Developer or Developer Pro sandbox: must not receive masked-production data without explicit governance approval.
  - Partial sandbox: confirm masking rules cover all regulated field types before refresh.
  - Full sandbox: confirm masking configuration is applied before every refresh from production.
- Flag: Full or Partial sandbox refreshed from production without a data masking configuration present.
- Flag: sandbox with external vendor or contractor access that lacks network access restrictions.
- Flag: sandbox refresh policy absent or refresh interval undocumented for sandboxes used in the pipeline.
- Flag: pipeline configured to deploy directly to Full sandbox without a preceding security gate pass.
- Record sandbox type, masking status, last refresh date, and external-access flag.

### 4. Change impact assessment
- Review the metadata deployment inventory for dependency risk:
  - Flag: custom field type changes (e.g., Text → Picklist, Number → Text) — data conversion risk.
  - Flag: destructive changes in the deployment manifest (field deletions, object deletions, class removals) — confirm rollback plan exists.
  - Flag: Apex class changes that affect trigger handlers, batch jobs, or scheduled classes without test coverage update.
  - Flag: Flow or Process Builder metadata changes that affect production-active automations without deactivation plan.
  - Flag: permission set or profile changes included in the deployment — require permission model review handoff.
- Map downstream automation impact: identify Flows, Process Builders, triggers, or scheduled jobs that reference changed metadata.
- Flag: changed metadata referenced by active production automations with no stated deactivation or coordination plan.
- Flag: API version downgrade in any Apex class or LWC component included in the deployment.
- Record total metadata component count, destructive change count, and downstream automation impact count.

### 5. CI/CD security gate compliance
- Review the pipeline configuration for the following required security gates:
  - SCA scan with fail threshold on P1 findings (required).
  - Test execution with minimum 75% code coverage gate (required; 85% recommended for regulated orgs).
  - Static analysis for dependency CVEs (RetireJS or equivalent) (required).
  - Destructive change review gate — manual approval required before any destructive change deployment (required).
  - Sandbox deployment verification before production promotion (required).
- Flag: any required gate absent from the pipeline configuration.
- Flag: SCA gate configured as advisory-only (no fail threshold) — this does not constitute a security gate.
- Flag: test coverage gate absent or set below 75%.
- Flag: production promotion path that bypasses sandbox deployment verification.
- Record gate presence matrix: gate name → present | absent | advisory-only.

### 6. Risk register assembly
- Consolidate findings from steps 1–5.
- Assign risk_tier per finding: Critical | High | Medium | Low.
- Map each finding to its pipeline domain: sca | false_positive | sandbox | change_impact | cicd_gate.
- Assign remediation priority: P1 must block deployment; P2 should resolve before promotion; P3 resolve within sprint; P4 advisory.
- Identify findings that meet escalation gates from salesforce-risk-taxonomy.

## Evidence requirements
- SCA output with rule IDs and finding counts is required for step 1; absence produces "insufficient evidence — assume all gates unverified".
- Sandbox inventory with masking configuration is required for step 3.
- Metadata deployment manifest is required for step 4; absence produces "change impact assessment not possible".
- CI/CD pipeline configuration or gate summary is required for step 5.
- Absence of any required input produces an "insufficient evidence" note with conservative (worst-case) rating for that domain.

## Output format
```
devsecops_pipeline_findings:
  sca_findings:
    - tool: PMD | ESLint | RetireJS
      rule_id: [rule identifier]
      severity_tier: P1 | P2 | P3 | P4
      description: [brief]
      false_positive_claim: true | false
      false_positive_rationale: [if applicable]
      recommendation: [brief]
  sandbox_governance_findings:
    - sandbox_name: [sanitized name or type label]
      sandbox_type: Developer | Developer Pro | Partial | Full
      masking_configured: true | false | unknown
      finding: [description]
      severity: Critical | High | Medium | Low
      recommendation: [brief]
  change_impact_findings:
    - metadata_type: [e.g., CustomField, ApexClass, Flow]
      component: [sanitized name]
      finding: [description]
      severity: Critical | High | Medium | Low
      downstream_automation_impact: [list or "none identified"]
      recommendation: [brief]
  cicd_gate_findings:
    - gate: [gate name]
      status: present | absent | advisory-only
      severity: Critical | High | Medium | Low
      recommendation: [brief]

sca_summary:
  p1_critical: [count]
  p2_high: [count]
  p3_medium: [count]
  p4_advisory: [count]
  false_positive_claims: [count]

cicd_gate_matrix:
  sca_fail_threshold: present | absent | advisory-only
  test_coverage_gate: present | absent | below-minimum
  cve_scan: present | absent | advisory-only
  destructive_change_approval: present | absent
  sandbox_verification: present | absent

escalation_gates_fired: [from salesforce-risk-taxonomy, or "none"]
summary:
  total_findings: [count]
  critical_count: [count]
  high_count: [count]
  deployment_recommendation: block | conditional | proceed
assumptions: [list]
missing_evidence: [what would improve the review]
```

## Redaction rules
- Never request secrets, credentials, OAuth tokens, refresh tokens, session IDs, MFA seeds, customer PII.
- Sanitize org IDs, user IDs, sandbox names before sharing in outputs.
- SCA output must not include hardcoded credentials or API keys found in code — stop and request sanitized version.

## Privilege / data handling rules
- Works from SCA tool output, sanitized pipeline configs, and metadata manifests only.
- Sandbox findings involving unmasked production data must be flagged for compliance specialist review.
- P1 SCA findings in Apex classes handling PII or financial data must trigger escalation review.
- Destructive change findings must be escalated to a change advisory board or equivalent human review.

## Handoff rules
- Hands off to: salesforce-apex-lwc-code-review-skill (if P1 SCA findings require deeper code-level review), salesforce-release-readiness-skill (after pipeline findings are remediated and deployment approval is needed), salesforce-permission-model-review-skill (if permission set or profile changes are in the deployment), salesforce-case-capsule (for any P1 Critical finding or missing required CI/CD gate).
- Required handoff fields: matter_id, risk_register (summary), escalation_gates_fired, missing_evidence, assumptions.

## Audit log fields
- matter_id, skill_id, skill_version, invoked_by, input_hash, evidence_quality, output_verdict, escalation_fired, timestamp

## Stop conditions
- SCA output or pipeline config contains live credentials, session tokens, or unredacted customer PII — stop and request sanitized version.
- P1 Critical SCA finding confirmed on a production-bound deployment with no remediation plan — stop, output ESCALATE, require human review before continuing.
- Destructive change manifest confirmed for a production deployment without a documented rollback plan — stop and require change advisory board review.
- Full sandbox confirmed to contain unmasked production PII feeding a pipeline with no masking gate — stop and require compliance specialist review.

## Security notes
- Read-only static review; never requests live org access, executes deployments, or triggers pipeline runs.
- Sanitized inputs only; SCA outputs containing hardcoded credentials must be refused.
- Risk register is advisory; P1 blocking decisions and destructive change approvals require human authorization.
- RetireJS CVE findings reference public CVE records; applicability to the specific Salesforce runtime context requires qualified assessor confirmation.
