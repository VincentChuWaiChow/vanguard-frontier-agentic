---
name: sap-testing-quality-gate-review
description: "Review SAP testing strategy and quality gate completeness for S/4HANA and SAP BTP programs: test scope and risk-based prioritization, SAP Cloud ALM test management configuration, automation coverage with Tricentis/CBTA, regression strategy for upgrades and transports, test data availability and masking, defect management workflow, and entry and exit criteria per test phase. Advisory only — does not execute tests, trigger automation runs, or mutate test management configuration."
allowed-tools: Read Grep Glob WebSearch WebFetch
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-06-19"
  category: delivery
  lifecycle: experimental
---

# SAP Testing and Quality Gate Review

## Purpose

Assess the completeness and risk coverage of the SAP testing strategy for S/4HANA transformation and SAP BTP programs. Review test scope and risk-based test prioritization: which business processes are in scope, how risk tiers are assigned, and whether high-risk or integration-intensive processes have adequate test coverage. Evaluate SAP Cloud ALM test management configuration: test plan structure, test case library completeness, defect record integration, and test execution tracking. Assess automation coverage with Tricentis Test Automation for SAP (formerly CBTA): which regression suites are automated, which processes remain manual, and whether automation coverage is sufficient for regression cycles on upgrade or transport-heavy releases. Review the regression strategy for system upgrades, support packages, and change transport cycles: how regression scope is defined, how often regression is executed, and whether regression results gate transport imports into production. Assess test data availability and data masking: whether representative test data exists for all test tiers, whether production data used in test environments is masked or anonymized, and whether test data refresh cadence supports the test schedule. Evaluate defect management workflow: severity and priority classification, defect-to-transport traceability, defect backlog governance, and exit criteria enforcement at defect closure gates. Validate that entry and exit criteria are formally defined and measurable for each test phase (unit, integration, UAT, regression, performance). This is an advisory quality gate review. It does not execute tests, trigger automation runs, import transports, or mutate SAP Cloud ALM test management configuration.

## When to use

Use this skill when the user asks to:

- review the overall testing strategy for an SAP S/4HANA transformation or BTP program, including which test types are planned (unit, string, integration, regression, UAT, performance) and how coverage is allocated across business processes,
- assess risk-based test prioritization: whether high-risk processes (financial close, order-to-cash, payroll, integration-intensive flows) have adequate test case coverage and whether the risk tier assignment rationale is documented,
- evaluate SAP Cloud ALM test management configuration: whether test plans are structured per phase, whether test cases are linked to business processes and requirements, whether defect records are created from failed test cases, and whether test execution progress is tracked and reported,
- assess automation coverage with Tricentis Test Automation for SAP (formerly CBTA): what percentage of regression test cases are automated, whether automation scripts are maintained and current for the target release, and whether automated regression is executed as part of the transport release cycle,
- review the regression strategy for upgrades and change transport cycles: how regression scope is selected, how baseline regression is maintained, and whether regression execution gates transport imports into the production environment,
- assess test data availability and masking: whether representative test data exists at the volume and variety required for each test phase, whether production data used in lower environments is anonymized or masked, and whether test data refresh procedures are documented and followed,
- evaluate defect management workflow: whether defect severity and priority classifications are consistently applied, whether defects are traceable to the transport responsible for the change, whether the defect backlog is actively governed, and whether exit criteria require resolution of all open defects above a defined severity threshold before phase sign-off,
- review entry and exit criteria for each test phase: whether criteria are formally documented, measurable (not subjective), and enforced by named sign-off authority.

## When not to use

- When the user wants to execute test cases, trigger Tricentis automation runs, or operate SAP Cloud ALM test plans — this skill does not assist with test execution. A separate guarded execution agent would be required.
- When the user needs a data migration readiness assessment — use `sap-data-migration-cutover-readiness`.
- When the user needs a transport or change management collision review — use `sap-release-change-collision-review`.
- When the user needs a go-live or hypercare incident command review — use `sap-hypercare-incident-commander-review`.
- When the user needs live SAP Cloud ALM operations monitoring or incident governance review rather than test management — use `sap-cloud-alm-sre-incident-review`.

## Advisory-only boundary — explicitly stated

**This skill does not execute tests, trigger automation runs, or mutate any SAP Cloud ALM test management configuration.** It does not:

- connect to SAP Cloud ALM tenants, Tricentis platforms, CBTA test runners, or any SAP system,
- trigger test case execution, automation script runs, or regression suite invocations,
- create, update, or close defect records, test plans, or test cases in any test management tool,
- approve, authorize, or imply authorization for phase exit or go/no-go sign-off decisions,
- access production test data, personally identifiable data, or business records from any SAP environment.

All quality gate assessment is based on testing strategy documentation, test plan descriptions, defect status summaries, automation coverage reports, and test data management documentation supplied by the user. Test execution is a risk-relevant activity that requires a separate explicitly scoped execution skill.

## Lean operating rules

- Risk-based prioritization must be evidence-grounded. A testing strategy that applies equal effort across all business processes regardless of risk profile is not a risk-based strategy. Risk tiers must be assigned with documented rationale and must drive test case count and automation investment allocation.
- Automation coverage must be quantified. A claim that "automation is in place" is not sufficient. The automation coverage percentage for regression test cases must be stated, and the maintenance status of automation scripts must be confirmed against the target SAP release.
- Entry and exit criteria must be measurable. Subjective exit criteria such as "UAT is substantially complete" or "quality is acceptable" are not enforceable. Criteria must include specific numeric thresholds: defect counts by severity, test case execution percentage, pass rate minimum.
- Test data masking is non-negotiable for production data in test environments. Any use of unmasked production data containing personally identifiable information or confidential business data in a lower environment is a compliance finding regardless of the testing benefit.
- Defect backlog governance is required before phase exit. An unreviewed defect backlog at phase exit boundary is a quality gate failure. All open defects must be reviewed, classified, and assigned a resolution or accepted-risk status before exit criteria can be evaluated.
- Do not fabricate defect counts, test execution rates, or automation coverage percentages. Only classify findings the user has provided from their actual test management reports or automation coverage outputs.
- Evidence from official SAP Cloud ALM documentation and Tricentis documentation takes precedence over memory or training data.

## Evidence rules

Label all claims with one of:

- `documentation-based` — grounded in SAP Cloud ALM test management documentation, Tricentis Test Automation for SAP documentation, SAP Activate testing methodology guidance, or SAP Help Portal
- `user-provided evidence` — test strategy documents, test plans, defect status reports, automation coverage reports, test data management documentation, or phase exit criteria documents provided by the user
- `inference` — derived reasoning not directly confirmed by official docs or user evidence; must always be labeled as such

## Live-environment rules

**This skill does not touch live systems.** There is no SAP Cloud ALM API call, Tricentis platform connection, test case execution, automation script run, or SAP system access in this skill's execution path. Users must supply testing strategy documentation, test plan descriptions, defect status summaries, automation coverage reports, and test data management documentation for this skill to review.

## References

Load only when needed:

- [Workflow and output contract](references/workflow-and-output.md) — test phase taxonomy, quality gate assessment dimensions, risk tier classification, defect governance criteria, output format.
- [Safety checklist](references/safety-checklist.md) — non-negotiables, common testing strategy review mistakes, advisory boundary enforcement, when to push back.
- [Official sources](references/official-sources.md) — SAP Cloud ALM test management, Tricentis Test Automation for SAP, SAP Activate testing methodology, test data management guidance.

## Response minimum

Return, at minimum:

- **Problem classification**: testing program scope (transformation phase, system types in scope, test types planned), automation maturity (no automation / partial / regression-ready), and quality gate posture (entry and exit criteria defined / partially defined / absent).
- **Evidence used**: documentation-based / user-provided evidence / inference.
- **Risk level**: quality-gate-blocking / high / medium / low per assessment dimension.
- **Recommended action**: specific quality gate control per finding (define measurable exit criteria, add test cases for high-risk process, configure Tricentis automation for regression suite, implement test data masking, create defect governance checkpoint, etc.), grounded in SAP Cloud ALM documentation or SAP Activate testing methodology.
- **Refusal / escalation triggers**: refuse to assess phase readiness if no test execution results or defect status data have been provided; do not provide test execution guidance under any circumstances; escalate to compliance review if unmasked production data is confirmed in a lower environment.
- **Business impact**: go-live date risk from insufficient test coverage, defect leakage into production, regression risk from insufficient automation, data privacy compliance exposure from unmasked test data.
- **Next verification step**: which test phase evidence or defect status data must be provided before the quality gate can be assessed as met.
