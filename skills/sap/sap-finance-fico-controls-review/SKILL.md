---
name: sap-finance-fico-controls-review
description: "Review SAP S/4HANA Finance (FI-CO) internal controls: document posting controls, account assignment validations and substitutions, period-end close governance, Financial Close Cockpit task management, segregation of duties in financial postings, parallel ledger configuration, and intercompany reconciliation controls. Flags control gaps, mis-configured validation rules, uncontrolled period management, and SoD exposures in financial processes. Does not post financial documents or mutate any live system."
allowed-tools: Read Grep Glob WebSearch WebFetch
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-06-19"
  category: finance
  lifecycle: experimental
---

# SAP Finance FI-CO Controls Review

## Purpose

Assess the internal control posture of SAP S/4HANA Finance (FI-CO) configurations and financial close processes. Evaluate document posting controls including tolerance groups, posting keys, and document type assignments. Review account assignment validation rules and substitution logic for correctness, completeness, and circumvention risk. Assess period-end close governance including fiscal year variant configuration, posting period management, and period open/close authorization controls. Evaluate Financial Close Cockpit (FCC) task list design, task dependencies, and governance workflow. Identify SoD exposures in financial posting processes including combinations of document creation, approval, payment release, and reversal authority. Assess parallel ledger design (Leading Ledger, Reconciliation Ledger, Extension Ledger) for completeness and reconciliation control coverage. Review intercompany process controls including clearing accounts, intercompany reconciliation configuration, and elimination governance. Does not connect to or mutate any live SAP system. Never posts, reverses, or modifies financial documents.

## When to use

Use this skill when the user asks to:

- review SAP S/4HANA FI document posting controls: document types, posting keys, tolerance groups, field status groups, and account assignment validation rules,
- assess SAP FI validations and substitutions configured in transaction OB28 / OBB1 — evaluating logic correctness, callout dependencies, and whether any substitution creates a bypass risk for key control fields (cost center, profit center, business area, tax code),
- evaluate period-end close governance: fiscal year variant design, posting period variant assignment to company codes, and the authorization model for opening and closing posting periods (transaction OB52, role for S_PERIOD_OPEN or equivalent),
- review Financial Close Cockpit (FCC) or SAP S/4HANA Financial Close Cockpit task list design — task sequencing, dependency modeling, responsible user or group assignments, and escalation or blocking logic,
- identify SoD exposures in financial posting processes: users or roles with combined authority to create, approve, pay, and reverse financial documents within the same company code or across company codes,
- assess parallel ledger configuration: number and purpose of ledgers in the ledger group, whether extension ledgers are used correctly, reconciliation controls between the leading and non-leading ledger, and whether reporting currency differences are explained,
- review intercompany accounting controls: intercompany clearing account assignment, matching logic in the intercompany reconciliation hub, elimination rule configuration, and whether intercompany postings can be created without a corresponding offsetting entry,
- evaluate accrual and deferral control: recurring entry configurations, accrual engine use, and whether manual accrual postings are subject to approval and reversal controls,
- assess journal entry controls in the context of SOX ITGC or internal audit: who can post freely to G/L, whether manual journal entries above a materiality threshold require approval, and whether posting restrictions by document type are enforced.

## When not to use

- When the user needs live inspection of SAP S/4HANA Finance configuration, journal entry lists, or posting period tables — this skill accepts only user-provided configuration exports, screen captures, role lists, validation/substitution descriptions, or written descriptions of the FI-CO landscape.
- When the request is about SAP Controlling (CO) planning, internal order settlement, or product cost analysis without a financial controls angle — this skill focuses on financial controls, not CO planning methodology.
- When the request concerns consolidation in SAP S/4HANA Group Reporting — use a dedicated Group Reporting skill; intercompany controls at the entity level are in scope here, but full group-level elimination and consolidation processes are not.
- When the request is about SAP GRC Access Control ruleset design for FI transaction codes — use `sap-security-iam-grc-sod-review` for GRC ruleset assessment; this skill identifies SoD exposures in FI-CO process terms rather than reviewing the GRC system itself.
- When the request is specifically about SAP MDG master data quality for FI master data (G/L accounts, cost centers) — use `sap-mdg-master-data-quality-review`.

## Does not touch live systems

This skill operates on user-provided configuration descriptions, validation/substitution rule exports, role and authorization object lists, Financial Close Cockpit task list exports, posting period variant summaries, parallel ledger configuration descriptions, or written descriptions of the FI-CO control landscape. It does not connect to any SAP S/4HANA system, Fiori launchpad, SAP GUI session, or backend RFC. It does not post, reverse, park, or modify any financial document. All live inspection is out of scope.

**This skill never posts financial documents.** No document creation (FB01, F-02, FB50, MIRO, or equivalent), no reversal (FB08, MR8M), no payment run (F110), and no clearing (F-44, F-32) is performed or recommended as a direct action in this skill's execution path. All remediation recommendations describe configuration changes to be implemented and tested in a non-production environment.

## Lean operating rules

- Classify control findings before recommending. Every finding must be assigned to a control domain (document posting controls / validations and substitutions / period management / Financial Close Cockpit / SoD in FI / parallel ledgers / intercompany) before a remediation path is proposed.
- SoD in financial postings is a first-order risk. Any user or role with combined authority to create financial documents, approve payment runs, and reverse postings within the same company code is a `critical` finding. Escalate before other remediation.
- Validation and substitution bypass risk must be assessed. A substitution that silently overwrites a key control field (cost center, profit center, tax code, business area) without user notification or approval is a `high` control risk regardless of the business justification.
- Period management authorization is a key segregation control. The authority to open posting periods (S_PERIOD_OPEN or equivalent in S/4HANA) must be separated from the authority to post documents. Shared authority over both is a `high` control gap.
- Financial Close Cockpit task lists must have owners and dependencies. A task list with no responsible user or group, with tasks that can be completed in any order without dependency enforcement, or without an escalation path for blocked tasks is a governance gap.
- Parallel ledger reconciliation gaps require explanation. Any unexplained difference between the leading ledger and a non-leading ledger (local GAAP, IFRS, tax ledger) that is not covered by a documented reconciliation control is a `high` finding.
- Intercompany postings must require a matching offsetting entry. Intercompany G/L postings that can be created without a corresponding offsetting entry in the partner company code — or without routing through the intercompany reconciliation hub — are a `high` process control gap.
- Evidence from user-provided artifacts or official SAP S/4HANA Finance documentation takes precedence over inference.
- Load only the reference needed for the FI-CO control domain under review.

## Evidence rules

Label all claims with one of:

- `documentation-based` — grounded in SAP S/4HANA Finance, SAP Help Portal FI-CO documentation, SAP Financial Closing Cockpit, or SAP audit and compliance guidance
- `user-provided evidence` — validation/substitution exports, role lists, posting period variant descriptions, FCC task list exports, intercompany configuration summaries, or written descriptions provided by the user
- `inference` — derived reasoning not directly confirmed by official docs or user evidence

## Live-environment rules

**This skill does not touch live systems.** There is no RFC call, Fiori OData API invocation, SAP GUI session, BAPI execution, or direct database query in this skill's execution path. Users must supply configuration exports, validation/substitution rule descriptions, period management summaries, Financial Close Cockpit task list exports, parallel ledger configuration descriptions, or written accounts of their FI-CO control landscape for this skill to review.

**This skill never posts financial documents.** Recommendations describing remediation always apply to configuration, authorization, or process design — not to direct financial document creation or modification.

## References

Load only when needed:

- [Workflow and output contract](references/workflow-and-output.md) — FI-CO control finding taxonomy, severity assignment, output format.
- [Safety checklist](references/safety-checklist.md) — non-negotiables, common FI-CO control review mistakes, when to push back.
- [Official sources](references/official-sources.md) — SAP S/4HANA Finance, FI-CO controls, Financial Close Cockpit, parallel ledgers, and intercompany documentation.

## Response minimum

Return, at minimum:

- **Problem classification**: FI-CO control domain(s) affected (document posting controls / validations and substitutions / period management / Financial Close Cockpit / SoD in FI / parallel ledgers / intercompany) and specific finding(s) per domain.
- **Evidence used**: documentation-based / user-provided evidence / inference.
- **Risk level**: critical (SoD in financial postings enabling fraud, uncontrolled period management, financial statement misstatement risk) / high (validation bypass, unsupported parallel ledger difference, intercompany imbalance) / medium (FCC governance gap, missing accrual reversal control, incomplete field status) / low (best practice deviation).
- **Recommended action**: specific configuration or process remediation per finding (validation rule correction, tolerance group tightening, period management role separation, FCC task dependency addition, parallel ledger reconciliation control, intercompany hub routing enforcement, etc.).
- **Refusal / escalation triggers**: if unmitigated critical SoD conflicts in financial postings are found, escalate to the audit team and GRC team before any further role or access change is authorized. If a finding requires live SAP system inspection, state that clearly and ask the user to supply the relevant export or description.
- **Business impact**: financial statement integrity risk, SOX ITGC compliance gap, internal audit finding risk, fraud enablement, or period-close efficiency impact.
- **Next verification step**: confirm recommended configuration changes against the user's current FI-CO setup in a non-production system before promoting to production.
