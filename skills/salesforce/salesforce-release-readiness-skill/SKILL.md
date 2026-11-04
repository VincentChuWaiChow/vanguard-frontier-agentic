---
name: salesforce-release-readiness-skill
description: 'Use this skill when a Salesforce release must be evaluated for deployment readiness. Covers: sandbox refresh strategy, source tracking state, package version diff, destructiveChanges.xml review, test coverage threshold verification, post-deploy steps, rollback plan, comms plan, and approval matrix. Trigger phrases: "is this release ready to deploy", "review the destructive changes for this release", "check our sandbox strategy", "validate the release checklist", "pre-deploy readiness review". Do not use when you need to approve a live production mutation at execution time (use salesforce-live-change-approval-protocol), when code quality is the focus (use salesforce-apex-lwc-code-review-skill), or when automation logic needs review (use salesforce-flow-automation-review-skill). Works from release artifacts and plans only; never requests live org access or executes deployments.'
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-05-20"
  category: delivery
  lifecycle: experimental
---

# Salesforce Release Readiness Skill

## Purpose
This skill evaluates a Salesforce release for deployment readiness. It produces
a structured readiness assessment covering sandbox strategy, source tracking,
package diff, destructive changes, test coverage, post-deploy steps, rollback
plan, communications plan, and approval matrix. It does not execute deployments,
access live orgs, or authorize production changes.

## When to use
- A Salesforce release is being prepared and readiness must be assessed.
- A change set or DX package deployment is planned and the checklist must be verified.
- A destructiveChanges.xml must be reviewed before a production deploy.
- A release retrospective revealed a gap in the pre-deploy process.

## When not to use
- Live production mutation approval at execution time — use `salesforce-live-change-approval-protocol`.
- Code quality review — use `salesforce-apex-lwc-code-review-skill`.
- Automation logic review — use `salesforce-flow-automation-review-skill`.
- Metadata quality review — use `salesforce-metadata-review-skill`.

## Minimum payload (required inputs)
- Release description: what is being deployed (features, fixes, metadata types).
- Target environment: sandbox → sandbox, sandbox → production (use placeholder, not real org ID).
- Deployment method: change set, SFDX/SF CLI deploy, CI/CD pipeline, ISV package install.
- Available artifacts: package diff, destructiveChanges.xml (if applicable), test run output.
- Rollback plan (or note that it is missing).

## Workflow

### 1. Sandbox refresh strategy
- Verify that testing occurred in a sandbox refreshed from production within
  a defined window (configurable threshold, e.g., 30 days).
- Flag: testing only in a developer sandbox without a full-copy or partial-copy
  refresh for complex data-dependent features.
- Flag: sandbox refresh done after feature development began (data divergence risk).

### 2. Source tracking state
- Review source tracking status: are there untracked local changes?
- Flag: uncommitted changes in tracked source that are not part of the release.
- Flag: metadata retrieved outside source tracking that may overwrite tracked changes.
- Verify: scratch org (
Salesforce DX)
  or sandbox state matches the deployment package.

### 3. Package version diff
- Review the list of metadata components in the deployment package.
- Flag: metadata types that are unexpectedly included (scope creep).
- Flag: metadata components that are expected but missing from the package.
- Flag: large diff (> configurable threshold components) without a phased
  deployment plan.

### 4. Destructive changes review
- If destructiveChanges.xml is present:
  - List all components marked for deletion.
  - Flag: deletion of custom fields on objects with active sharing rules or
    automation referencing those fields.
  - Flag: deletion of Apex classes referenced by other classes not in the package.
  - Flag: deletion without a data migration plan for fields containing data.
  - Flag: deletion of permission sets that are actively assigned.
- If destructiveChanges.xml is absent but deletions are expected: flag missing
  destructive changes manifest.

### 5. Test coverage threshold
- Verify that Apex test coverage meets the org threshold (default 75%; flag if
  org has a higher custom threshold).
- Flag: test classes with `SeeAllData=true` in the test run.
- Flag: test run results showing failures (zero failures required for production deploy).
- Flag: test coverage calculated only on deployed classes, not org-wide (insufficient).
- Recommend: run all tests (`RunAllTestsInOrg`) for production deployments.

### 6. Post-deploy steps
- Verify that a post-deploy step list exists and is documented.
- Flag: manual configuration steps not captured in the package (e.g., custom
  metadata values, permission set assignments, named credential setup).
- Flag: post-deploy steps that require production data access without a
  documented data-access control.

### 7. Rollback plan
- Verify that a rollback plan exists and is actionable.
- Flag: rollback plan that is "redeploy the previous package" without a
  version reference.
- Flag: rollback plan for destructive changes (field/object deletion is
  irreversible without data backup).
- Recommend: document rollback estimated time and who authorizes rollback.

### 8. Communications plan
- Verify that a user communications plan exists for user-visible changes.
- Flag: changes to page layouts, record types, or permissions without a
  comms plan.
- Flag: scheduled deployments during business hours without maintenance window
  notification.

### 9. Approval matrix
- Verify that the approval matrix is documented: who approved the release,
  at what level (developer, tech lead, architect, change advisory board).
- Flag: production deployments without documented architecture or change board
  approval for significant changes.
- Flag: emergency deployments without a documented fast-track approval path.

## Evidence requirements
- Release description, deployment method, and target environment.
- Package component list or diff.
- Test run output (or note that it is unavailable).
- Rollback plan (or note that it is missing).

## Output format
```
release_readiness_assessment:
  sandbox_refresh:
    status: ready | at-risk | not-ready
    findings: [list]
  source_tracking:
    status: ready | at-risk | not-ready
    findings: [list]
  package_version_diff:
    status: ready | at-risk | not-ready
    findings: [list]
  destructive_changes:
    status: ready | at-risk | not-ready | not-applicable
    findings: [list]
  test_coverage:
    status: ready | at-risk | not-ready
    findings: [list]
  post_deploy_steps:
    status: ready | at-risk | not-ready
    findings: [list]
  rollback_plan:
    status: ready | at-risk | not-ready
    findings: [list]
  communications_plan:
    status: ready | at-risk | not-ready
    findings: [list]
  approval_matrix:
    status: ready | at-risk | not-ready
    findings: [list]

overall_readiness: ready | at-risk | not-ready
blocking_issues: [list — items that must be resolved before deploy]
non_blocking_issues: [list — recommendations for improvement]
escalation_gates_fired: [from salesforce-risk-taxonomy, or "none"]
assumptions: [list]
missing_evidence: [what would improve the assessment]
```

## Redaction rules
- Never request secrets, credentials, OAuth tokens, refresh tokens, session IDs, MFA seeds, customer PII.
- Sanitize org IDs, user IDs (replace with placeholders) before sharing in outputs.
- Do not include real usernames or approver identities; use role references.

## Privilege / data handling rules
- Release artifacts must not contain production data samples.
- Destructive changes review involving regulated-data fields escalates to compliance review.

## Handoff rules
- Hands off to: salesforce-live-change-approval-protocol (if overall_readiness = ready and
  production deploy is next), salesforce-apex-lwc-code-review-skill (if test coverage gaps
  need code-level review), salesforce-case-capsule (if escalation_gates_fired).
- Required handoff fields: matter_id, overall_readiness, blocking_issues,
  escalation_gates_fired.

## Audit log fields
- matter_id, skill_id, skill_version, invoked_by, input_hash, evidence_quality, output_verdict, escalation_fired, timestamp

## Stop conditions
- Destructive changes include field deletion on objects with unreviewed data — stop and flag irreversible-deploy gate.
- Test run shows failures — overall_readiness = not-ready; do not proceed.
- Rollback plan is absent and deployment includes irreversible changes — stop and require rollback plan before assessment can complete.

## Security notes
- Read-only advisory assessment; never executes deployments or requests live org access.
- Destructive changes are irreversible; always require human-authorized rollback plan documentation.
- Approval matrix verification is a governance control; its absence is a high-severity finding.
