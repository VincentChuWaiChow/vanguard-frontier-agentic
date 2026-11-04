---
name: sap-order-to-cash-review
description: "Review SAP S/4HANA Order-to-Cash (OTC) processes: order management configuration, pricing procedure design, credit management controls, billing and revenue recognition, order and delivery blocks and holds, fulfillment exception management, Days Sales Outstanding (DSO) drivers, and cash application. Flags control gaps, revenue leakage patterns, pricing errors, unresolved order blocks, credit management weaknesses, and DSO drivers. Does not create sales orders, release order blocks, post billing documents, or mutate any live OTC system."
allowed-tools: Read Grep Glob WebSearch WebFetch
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-06-19"
  category: finance
  lifecycle: experimental
---

# SAP Order-to-Cash Review

## Purpose

Assess the process design quality, configuration posture, and control coverage of SAP S/4HANA Order-to-Cash (OTC) processes. Evaluate order management configuration by reviewing sales order document type design, partner function and organizational assignment, order entry completeness controls, and whether order incompletion procedures are configured to prevent downstream fulfillment failures. Assess pricing procedure design by reviewing condition type configuration, pricing procedure determination, pricing control parameters (mandatory condition types, statistical conditions, calculation types), and whether pricing errors or gaps can result in orders processed at incorrect prices. Evaluate credit management controls by reviewing credit control area configuration, credit limit assignment governance, automatic credit check configuration (static vs. dynamic), blocked order management, and whether the credit management process allows unauthorized credit limit overrides. Assess billing and revenue recognition by reviewing billing document type configuration, billing plan design for milestone or periodic billing, SD–FI integration for revenue posting, and whether billing schedule adherence is monitored. Review order and delivery blocks and holds by examining standard block reasons (credit block, incompletion block, listing block, delivery block), block release authorization governance, aged block management, and whether unresolved blocks represent revenue and customer experience risk. Evaluate fulfillment exception management by reviewing availability check configuration, transfer of requirements (TOR) design, delivery split logic, goods issue (GI) posting governance, and whether fulfillment exceptions are tracked and resolved within target cycle times. Assess Days Sales Outstanding (DSO) drivers by reviewing billing cycle timing, cash application process design, dispute management configuration in SAP Dispute Management (FSCM), dunning procedure governance, and deductions management. Does not connect to or mutate any live SAP S/4HANA Sales (SD) system or Finance (FI) system. Never creates, modifies, or releases sales orders, delivery documents, billing documents, or customer payments.

## When to use

Use this skill when the user asks to:

- review SAP S/4HANA SD sales order management configuration: sales order document type design, partner function determination, organizational assignment (sales area, plant, shipping point), order incompletion procedure configuration, order entry validation, and pricing determination completeness controls,
- assess SAP S/4HANA pricing procedure design: pricing procedure assignment (VKOA and condition type determination), mandatory condition type configuration, pricing control parameters (ERL and ERB account determination), manual price override authorization controls, pricing error detection, free goods and discount condition type governance, and whether pricing gaps allow orders to be processed at zero or incorrect prices,
- evaluate SAP S/4HANA credit management configuration: credit control area assignment, credit limit governance (who can set, modify, or override credit limits), automatic credit check type (static check, dynamic check, maximum document value), credit block reason design, credit block release authorization, and whether the credit management process provides adequate controls over the credit risk exposure in the order book,
- review billing and revenue recognition process design: billing document type configuration (F1, F2, credit memo, debit memo), billing plan types (milestone billing, periodic billing, down payment), SD–FI account determination for revenue and deferred revenue posting, billing schedule completion rate monitoring, and whether billing cycle timing aligns with contract terms and IFRS 15 or ASC 606 revenue recognition requirements,
- assess order and delivery block management: standard block reason configuration (credit block A, listing and exclusion block B, incompletion block C, delivery block), block aging patterns and whether aged blocks represent systemic fulfillment or commercial policy issues, block release authorization segregation, and whether unresolved blocks in the order book represent revenue at risk,
- evaluate fulfillment exception management: availability check configuration (ATP checking rule, checking scope, replenishment lead time), transfer of requirements (TOR) design and accuracy, delivery split logic and split criteria, partial delivery tolerance configuration, goods issue (GI) posting timing controls, and whether fulfillment exception resolution cycle time meets customer service level requirements,
- assess Days Sales Outstanding (DSO) drivers: billing cycle timing relative to delivery completion, cash application process design and remittance matching efficiency, SAP Dispute Management (FSCM) configuration for invoice dispute tracking and resolution, dunning procedure configuration (dunning levels, dunning amounts, dunning escalation), deductions management process design, and whether the collections process is supported by adequate receivables analytics,
- review revenue leakage patterns in the OTC process: unauthorized pricing overrides, unbilled deliveries (delivered goods not yet invoiced), billing plan milestone slippage, unapplied cash sitting in the clearing account, unresolved credit memos that should have been applied against outstanding receivables, and whether revenue completeness controls are in place across the OTC cycle.

## When not to use

- When the user needs live inspection of SAP S/4HANA sales orders, deliveries, billing documents, or customer accounts — this skill accepts only user-provided configuration summaries, block aging reports, billing schedule adherence reports, DSO analytics, dispute management summaries, or written descriptions of the OTC landscape.
- When the request is about SAP S/4HANA Finance Accounts Receivable (FI-AR) period-end close controls or G/L posting controls without an OTC revenue or DSO angle — use `sap-finance-fico-controls-review` for FI-AR financial control assessment.
- When the request concerns SAP CRM or SAP Sales Cloud (formerly C/4HANA Sales) customer relationship management processes rather than SAP S/4HANA SD order fulfillment and billing — those are distinct front-office platforms.
- When the request is about SAP Revenue Accounting and Reporting (RAR) for multi-element arrangement accounting under IFRS 15 or ASC 606 without an S/4HANA SD billing configuration angle — SAP RAR is a distinct module with its own configuration scope.
- When the request concerns SAP Transportation Management (TM) route planning or freight settlement rather than OTC fulfillment exception management at the delivery level.

## Does not touch live systems

This skill operates on user-provided configuration descriptions, order block aging reports, billing schedule adherence reports, pricing procedure documentation, credit management configuration summaries, DSO analytics, dispute management reports, dunning procedure documentation, or written descriptions of the OTC landscape. It does not connect to any SAP S/4HANA Sales (SD) system, SAP S/4HANA Finance (FI) system, Fiori launchpad, SAP GUI session, or backend RFC. It does not create, modify, release, or cancel sales orders, delivery documents, billing documents, customer credit limits, or customer payments. All live inspection is out of scope.

**This skill never creates or releases OTC documents.** No sales order creation, no delivery release, no goods issue posting, no billing document creation, no credit limit modification, and no customer payment posting is performed or recommended as a direct action in this skill's execution path. All remediation recommendations describe configuration and process design changes to be implemented and tested in a non-production environment.

## Lean operating rules

- Classify OTC control findings before recommending. Every finding must be assigned to an OTC process domain (order management / pricing / credit management / billing and revenue / order blocks / fulfillment exceptions / DSO drivers / revenue completeness) before a remediation path is proposed.
- Pricing overrides without authorization control are a first-order revenue leakage risk. Any configuration allowing users to override pricing condition types to zero or below standard without an approval workflow is a `high` revenue integrity finding — it enables unauthorized discounting that may not be visible in standard OTC reporting.
- Unbilled deliveries are a direct revenue completeness risk. Any goods issue posted but not yet invoiced beyond the standard billing cycle window represents a revenue recognition delay or leakage risk. Monitor and age unbilled deliveries as a revenue completeness control.
- Credit blocks released without documented authorization expose the business to credit risk. A credit block release process that allows the releasing user to also be the salesperson who entered the order — without a separate credit management approver — is an SoD gap in the credit risk control model.
- Aged order blocks (beyond the standard resolution target) indicate systemic process failures. Order blocks that age without resolution represent both revenue risk (delayed fulfillment) and customer experience risk. A pattern of aged blocks in a specific block reason category signals a root cause requiring process or configuration correction, not a block-by-block resolution approach.
- DSO above industry benchmark for the business model requires a multi-driver analysis. DSO elevation can stem from billing cycle timing, dispute volume and resolution cycle time, deductions backlog, cash application delays, or dunning process gaps. Attributing DSO solely to customer behavior without a process driver analysis is an incomplete finding.
- Evidence from user-provided artifacts or official SAP S/4HANA Sales and Finance documentation takes precedence over inference.
- Load only the reference needed for the OTC domain under review.

## Evidence rules

Label all claims with one of:

- `documentation-based` — grounded in SAP S/4HANA Sales and Distribution, SAP S/4HANA Finance Accounts Receivable, SAP FSCM Dispute Management, or SAP Help Portal OTC documentation, or SAP billing and revenue recognition guidance
- `user-provided evidence` — order block aging reports, billing schedule adherence reports, pricing procedure documentation, credit management configuration summaries, DSO analytics, dispute management reports, dunning procedure documentation, unbilled delivery reports, or written descriptions provided by the user
- `inference` — derived reasoning not directly confirmed by official docs or user evidence

## Live-environment rules

**This skill does not touch live systems.** There is no SAP S/4HANA RFC call, Fiori OData API invocation, SAP GUI session, BAPI execution, or direct database query in this skill's execution path. Users must supply configuration descriptions, order block aging reports, billing schedule adherence data, credit management summaries, DSO analytics, dispute management reports, or written descriptions of their OTC landscape for this skill to review.

**This skill never creates or releases OTC documents.** Recommendations describing remediation always apply to configuration, authorization, or process design — not to direct sales order creation, delivery release, billing document generation, credit limit change, or customer payment posting.

## References

Load only when needed:

- [Workflow and output contract](references/workflow-and-output.md) — OTC finding taxonomy, severity assignment, output format.
- [Safety checklist](references/safety-checklist.md) — non-negotiables, common OTC review mistakes, when to push back.
- [Official sources](references/official-sources.md) — SAP S/4HANA Sales and Distribution, credit management, billing, FSCM Dispute Management, and OTC documentation.

## Response minimum

Return, at minimum:

- **Problem classification**: OTC process domain(s) affected (order management / pricing / credit management / billing and revenue / order blocks / fulfillment exceptions / DSO drivers / revenue completeness) and specific finding(s) per domain.
- **Evidence used**: documentation-based / user-provided evidence / inference.
- **Risk level**: critical (unauthorized pricing at zero enabling fraudulent revenue leakage; credit block release without any credit management authorization; unbilled delivery backlog above a material revenue threshold) / high (pricing override without approval workflow; SoD gap in credit block release; billing plan milestone slippage creating revenue recognition risk; DSO elevation driven by unaddressed billing or dispute process gaps; aged order block backlog representing material revenue at risk) / medium (availability check gap causing fulfillment splits not aligned with customer requirements; dunning procedure without adequate escalation levels; incompletion procedure not covering required fields; cash application remittance matching inefficiency) / low (best practice deviation in billing document type design or partner function configuration).
- **Recommended action**: specific configuration or process remediation per finding (pricing override approval workflow, credit block SoD separation, billing schedule monitoring setup, unbilled delivery aging control, order incompletion procedure extension, availability check rule alignment, dunning level addition, dispute management configuration, cash application matching rule improvement, etc.).
- **Refusal / escalation triggers**: if unauthorized pricing at zero or revenue fraud indicators are found, escalate to the audit and finance team before any further OTC transactions are processed. If a finding requires live SAP S/4HANA system inspection, state that clearly and ask the user to supply the relevant export or description.
- **Business impact**: revenue leakage or deferral risk, credit loss exposure, customer service level impact from aged blocks or fulfillment exceptions, working capital impact from DSO elevation, audit finding risk for revenue completeness controls, IFRS 15 or ASC 606 compliance risk from billing plan slippage.
- **Next verification step**: confirm recommended configuration changes against the user's current SAP S/4HANA SD and FI setup in a non-production system before promoting to production.
