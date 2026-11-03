---
name: sap-procurement-ariba-value-leakage-review
description: "Review SAP Ariba and SAP S/4HANA source-to-pay processes for value leakage: maverick and off-contract spend, contract compliance rates, supplier risk exposure, three-way match exceptions (invoice/PO/GR), early payment discount capture, and guided buying adoption. Flags leakage patterns, control gaps in procurement compliance, supplier risk blind spots, and discount capture failures. Does not create purchase orders, approve invoices, or mutate any live procurement system."
allowed-tools: Read Grep Glob WebSearch WebFetch
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-06-19"
  category: finance
  lifecycle: experimental
---

# SAP Procurement Ariba Value Leakage Review

## Purpose

Assess the source-to-pay process and configuration quality of SAP Ariba and SAP S/4HANA Procurement to identify value leakage patterns and control gaps. Evaluate maverick and off-contract spend by reviewing purchasing channel discipline, preferred supplier utilization rates, and guided buying configuration in SAP Ariba Buying and Invoicing. Assess contract compliance by reviewing contract consumption rates, contract-to-PO linkage configuration, contract leakage thresholds, and sourcing workflow design in SAP Ariba Sourcing. Review supplier risk exposure by analyzing supplier qualification status, supplier risk scoring configuration in SAP Ariba Supplier Risk, and diversity and sustainability compliance gaps. Evaluate three-way match discipline by examining invoice tolerance configuration, PO/GR/invoice matching rules, hold and exception management workflows, and leakage from tolerance overrides. Assess early payment discount capture rates by reviewing payment term configuration in SAP S/4HANA Accounts Payable, discount monitoring dashboard coverage, and dynamic discounting setup in SAP Ariba Discount Management. Review guided buying adoption and catalog governance for completeness of approved supplier content and user adoption barriers. Does not connect to or mutate any live SAP Ariba or SAP S/4HANA Procurement system. Never creates purchase orders, approves invoices, releases payments, or modifies supplier master data.

## When to use

Use this skill when the user asks to:

- review SAP Ariba Buying and Invoicing or SAP S/4HANA Procurement configuration for maverick spend patterns: off-contract purchasing channels, absence of guided buying enforcement, catalog coverage gaps, free-text requisition rates, and whether procurement policy directs spend to preferred suppliers and negotiated contracts,
- assess contract compliance in SAP Ariba Contracts: contract consumption rates versus negotiated volume commitments, contract-to-PO linkage configuration, whether purchase orders can be created without referencing an active contract for in-scope spend categories, and contract leakage threshold configuration in sourcing workflow,
- evaluate supplier risk posture using SAP Ariba Supplier Risk or equivalent supplier qualification processes: supplier onboarding completeness, risk score configuration and threshold alerts, supplier financial health monitoring, geographic and concentration risk, regulatory and sanctions screening, and ESG/diversity qualification gaps,
- review three-way match control discipline: invoice/PO/GR matching rule configuration (quantity tolerance, price tolerance, delivery cost tolerance), hold and exception workflow routing for match failures, whether tolerance overrides require approval, frequency and financial impact of tolerance-based invoice releases, and whether the accounts payable team can bypass match requirements without documented escalation,
- assess early payment discount capture: payment term configuration and discount window setup in SAP S/4HANA Accounts Payable, SAP Ariba Discount Management or dynamic discounting program setup, discount capture rate reporting, and whether the AP process is structured to surface and capture available discounts before the discount window closes,
- evaluate guided buying adoption and catalog governance: SAP Ariba catalog configuration (punchout, static, dynamic), catalog content completeness for high-spend categories, supplier catalog update frequency, guided buying policy enforcement rules, and adoption metrics indicating whether users bypass guided buying channels,
- review source-to-pay cycle time and process efficiency: requisition-to-order cycle time, sourcing event duration, invoice processing time, and whether process bottlenecks create value leakage through delayed payments (penalty risk), delayed sourcing (spend without contract coverage), or slow exception resolution (aged invoice holds),
- assess spend analytics visibility: whether SAP Ariba Spend Analysis or equivalent reporting covers all spend channels (PO-based, non-PO, procurement card), category hierarchy configuration, supplier normalization quality, and whether spend analysis data is sufficient to identify leakage patterns in specific categories.

## When not to use

- When the user needs live inspection of SAP Ariba purchase orders, contracts, invoices, or supplier records — this skill accepts only user-provided configuration summaries, process descriptions, spend analysis reports, contract compliance metrics, exception reports, or written descriptions of the source-to-pay landscape.
- When the request is about SAP S/4HANA Finance Accounts Payable period-end closing controls, G/L posting controls, or payment run governance without a procurement value leakage angle — use `sap-finance-fico-controls-review` for AP financial control assessment.
- When the request is about SAP Ariba technical integration architecture, API connectivity between SAP Ariba and SAP S/4HANA, or integration suite design — this skill focuses on procurement process and value leakage, not integration platform configuration.
- When the request concerns SAP Fieldglass contingent workforce or external services procurement — Fieldglass-specific processes are a distinct scope from source-to-pay goods and services procurement.
- When the request is about SAP Ariba Network trading partner onboarding at a technical level (EDI, cXML configuration) rather than a procurement compliance and value leakage assessment.

## Does not touch live systems

This skill operates on user-provided configuration descriptions, spend analysis exports, contract compliance reports, three-way match exception reports, supplier risk dashboard summaries, discount capture rate reports, guided buying adoption metrics, or written descriptions of the source-to-pay landscape. It does not connect to any SAP Ariba tenant, SAP Ariba Network, SAP S/4HANA Procurement system, Fiori launchpad, or Ariba APIs. It does not create, approve, or modify purchase requisitions, purchase orders, contracts, invoices, supplier records, or payment runs. All live inspection is out of scope.

**This skill never creates or approves procurement documents.** No purchase order creation, no invoice approval, no payment release, no contract activation, and no supplier record modification is performed or recommended as a direct action in this skill's execution path. All remediation recommendations describe configuration and process changes to be implemented and tested in a non-production environment.

## Lean operating rules

- Classify value leakage findings before recommending. Every finding must be assigned to a leakage domain (maverick/off-contract spend / contract compliance / supplier risk / three-way match / discount capture / guided buying / spend analytics visibility) before a remediation path is proposed.
- Maverick spend without contract coverage is a first-order value leakage signal. Any spend category with material volume and no active contract coverage, or with active contracts that are consistently bypassed, is a `high` leakage finding requiring spend channel enforcement action.
- Three-way match tolerance overrides that bypass approval are a critical control gap. Tolerances are a managed control parameter — not an uncontrolled bypass valve. Any configuration allowing invoice release on match failure without documented approver authorization is a `critical` or `high` finding depending on the materiality threshold.
- Contract compliance leakage must be quantified when data is available. If the user provides contract consumption rates, the finding must reference the gap between committed and actual volume. Do not treat low contract consumption as a purely operational metric — it represents direct savings leakage.
- Supplier risk gaps with active spend exposure are `high`. A supplier with significant spend volume, no completed qualification, unresolved financial health flags, or active sanctions screening alerts represents both financial and reputational risk.
- Discount capture failures have a measurable cost. An early payment discount window that expires without capture is a direct cost — express this in terms of the annualized cost of missed discounts when discount rate and spend data are provided.
- Guided buying bypass rate above 20% for in-scope categories is a `medium` leakage indicator requiring adoption analysis and policy enforcement review.
- Evidence from user-provided artifacts or official SAP Ariba and SAP S/4HANA Procurement documentation takes precedence over inference.
- Load only the reference needed for the procurement leakage domain under review.

## Evidence rules

Label all claims with one of:

- `documentation-based` — grounded in SAP Ariba, SAP S/4HANA Procurement, or SAP Help Portal source-to-pay documentation, SAP Ariba best practice guides, or SAP procurement compliance guidance
- `user-provided evidence` — spend analysis exports, contract compliance reports, three-way match exception reports, supplier risk summaries, discount capture rate data, guided buying adoption metrics, or written descriptions provided by the user
- `inference` — derived reasoning not directly confirmed by official docs or user evidence

## Live-environment rules

**This skill does not touch live systems.** There is no SAP Ariba API call, Ariba Network transaction, SAP S/4HANA RFC invocation, Fiori OData request, or direct database query in this skill's execution path. Users must supply configuration exports, spend analysis reports, contract compliance metrics, exception reports, supplier risk dashboard summaries, or written descriptions of their source-to-pay landscape for this skill to review.

**This skill never creates or approves procurement documents.** Recommendations describing remediation always apply to configuration, policy, or process design — not to direct purchase order creation, invoice approval, payment release, or supplier record modification.

## References

Load only when needed:

- [Workflow and output contract](references/workflow-and-output.md) — procurement value leakage finding taxonomy, severity assignment, output format.
- [Safety checklist](references/safety-checklist.md) — non-negotiables, common procurement review mistakes, when to push back.
- [Official sources](references/official-sources.md) — SAP Ariba Buying and Invoicing, SAP Ariba Contracts, SAP Ariba Supplier Risk, SAP S/4HANA Procurement, and source-to-pay documentation.

## Response minimum

Return, at minimum:

- **Problem classification**: value leakage domain(s) affected (maverick/off-contract spend / contract compliance / supplier risk / three-way match / discount capture / guided buying / spend analytics visibility) and specific finding(s) per domain.
- **Evidence used**: documentation-based / user-provided evidence / inference.
- **Risk level**: critical (three-way match bypass without approval enabling payment fraud; supplier payments to sanctioned or disqualified parties) / high (maverick spend above materiality threshold, contract compliance leakage in core categories, supplier risk with active spend exposure, tolerance override without approval, missed discount capture at material scale) / medium (guided buying bypass above 20%, catalog coverage gap, spend analytics blind spot, process cycle time creating compliance risk) / low (best practice deviation in sourcing workflow design or catalog governance).
- **Recommended action**: specific configuration or process remediation per finding (guided buying enforcement rule, contract-to-PO linkage activation, tolerance approval workflow, supplier risk threshold alert, discount capture monitoring setup, catalog content update, spend analysis category hierarchy correction, etc.).
- **Refusal / escalation triggers**: if three-way match bypass without approval or supplier payments to potentially sanctioned parties are identified, escalate to the compliance and audit team before any further procurement or payment action is taken. If a finding requires live SAP Ariba or S/4HANA system inspection, state that clearly and ask the user to supply the relevant export or description.
- **Business impact**: direct savings leakage (quantified when data is available), regulatory or sanctions compliance risk, supplier concentration or financial risk, audit finding risk, working capital impact from uncaptured discounts or delayed AP processing.
- **Next verification step**: confirm recommended configuration changes against the user's current SAP Ariba and SAP S/4HANA Procurement setup in a non-production environment before promoting to production.
