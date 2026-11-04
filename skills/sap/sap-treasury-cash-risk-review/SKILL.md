---
name: sap-treasury-cash-risk-review
description: "Review SAP Treasury and Risk Management (TRM) and Cash Management configurations: cash position and liquidity forecasting controls, bank account management (BAM) governance, in-house cash and payment factory design, hedge and exposure management controls, fraud prevention in payment processing, and segregation of duties across treasury operations. Flags control gaps in cash visibility, unauthorized payment release paths, hedge documentation deficiencies, bank account governance weaknesses, and fraud control bypass risks. Does not execute payments, trades, or hedges, and does not access any live treasury or banking system."
allowed-tools: Read Grep Glob WebSearch WebFetch
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-06-19"
  category: finance
  lifecycle: experimental
---

# SAP Treasury Cash Risk Review

## Purpose

Assess the control posture, configuration design, and governance quality of SAP Treasury and Risk Management (TRM) and SAP Cash Management implementations within S/4HANA. Evaluate cash position and liquidity forecasting controls by reviewing cash management configuration, memo record governance, bank statement processing automation (electronic bank statement / MultiCash / EBAM), liquidity item hierarchy design, and whether cash position reporting provides an accurate and timely view of available liquidity. Review bank account management (BAM) governance by assessing SAP Bank Account Management configuration: bank account master data governance, account opening and closing approval workflows, signatory management, bank account authorization controls, and whether unauthorized bank accounts can be created or modified without a four-eyes approval. Assess in-house cash (IHC) and payment factory design by evaluating in-house bank configuration, internal account structure, payment request routing controls, netting design, and whether the payment factory aggregation logic prevents unauthorized outbound payment aggregation. Evaluate hedge and exposure management controls by reviewing financial instrument configuration (OTC derivatives, bonds, interest rate instruments), treasury position management, exposure determination logic, hedge designation and documentation workflow (hedge accounting under IFRS 9 / IAS 39), ineffectiveness calculation, and whether hedge documentation gaps create audit or accounting risk. Assess payment processing fraud controls by reviewing dual-control configuration in payment approval (payment request status management, payment release authorization, bank communication management channel controls), SWIFT or host-to-bank connectivity governance, and whether a single user can initiate, approve, and release a payment to an external bank without a second authorized approver. Review segregation of duties across treasury operations by identifying combined authorization over cash position manipulation, payment release, bank account management, and financial instrument trading. Does not connect to or mutate any live SAP S/4HANA Treasury, banking, or trading system. Never executes payments, trades, hedges, settlements, or bank transfers.

## When to use

Use this skill when the user asks to:

- review SAP S/4HANA Cash Management configuration: cash position structure, liquidity item hierarchy design, memo record governance, bank statement processing (electronic bank statement / BAI2 / MT940 / CAMT), cash concentration and zero-balancing configuration, and whether cash position reporting provides timely and complete visibility over the group's cash,
- assess SAP Bank Account Management (BAM) governance: bank account master data management, account lifecycle workflow (open, modify, close), signatory management and authority matrix, bank account authorization objects, four-eyes approval configuration for account creation and modification, and whether unauthorized bank accounts can be established or altered without a documented approval chain,
- evaluate SAP in-house cash (IHC) configuration and payment factory design: in-house bank structure, internal account assignment, payment request routing logic, netting and settlement controls, payment factory aggregation governance, and whether the in-house cash configuration allows internal payment requests to be aggregated and released externally without dual-control authorization,
- review SAP Treasury and Risk Management (TRM) financial instrument configuration: instrument type design (OTC derivatives, bonds, interest rate swaps, FX forwards, commodity derivatives), transaction flow configuration, position management controls, settlement processing governance, and whether the instrument configuration supports the entity's stated hedging strategy and accounting policy,
- assess hedge and exposure management controls: exposure determination source configuration (SD open items, MM purchase orders, financial forecasts), hedge designation workflow, hedge documentation completeness under IFRS 9 or IAS 39, ineffectiveness testing configuration, hedge accounting valuation controls, and whether hedge documentation gaps expose the entity to derecognition or restatement risk,
- evaluate payment processing fraud controls: dual-control configuration in SAP payment approval (payment request status transitions, payment release authorization objects, bank communication manager channel controls), SWIFT connectivity governance (SWIFT Alliance, bank communication management), host-to-bank file security, and whether a single treasury user has combined authority to create, approve, and release a payment run to an external bank,
- identify SoD exposures across treasury operations: combined authority to manage bank accounts and release payments; combined authority to create financial instruments and execute settlement; combined authority to modify cash position memo records and release payment requests; combined authority over FX exposure determination and hedge instrument execution; and whether treasury SoD controls are enforced at the SAP authorization object level or only at the process level,
- review treasury reporting and compliance controls: regulatory reporting configuration (EMIR, FRTB, Dodd-Frank position reporting), audit trail completeness for financial instrument transactions, treasury accounting entries in the general ledger, mark-to-market valuation controls, and whether treasury accounting is reconciled to the FI general ledger on a daily or period-end basis.

## When not to use

- When the user needs live inspection of cash positions, bank account balances, outstanding payment runs, or financial instrument positions — this skill accepts only user-provided configuration summaries, authorization object exports, bank account master data reports, hedge documentation summaries, payment run logs, or written descriptions of the treasury landscape.
- When the request is about SAP S/4HANA Finance Accounts Payable (FI-AP) payment run configuration (F110) without a treasury dual-control or fraud control angle — use `sap-finance-fico-controls-review` for FI-AP payment run controls in the FI context.
- When the request concerns SAP Ariba supplier payments or procurement-driven payment processing without a treasury management system angle — use `sap-procurement-ariba-value-leakage-review` for procurement payment controls.
- When the request is specifically about SAP GRC Access Control ruleset design for TRM transaction codes — use `sap-security-iam-grc-sod-review` for GRC ruleset assessment; this skill identifies treasury SoD exposures in process terms rather than reviewing the GRC system itself.
- When the request concerns SAP Analytics Cloud or SAP Datasphere treasury dashboards rather than the underlying TRM/Cash Management control configuration — visual analytics design is out of scope for this skill.

## Does not touch live systems

This skill operates on user-provided configuration descriptions, bank account master data reports, authorization object exports, payment run logs, hedge documentation summaries, in-house cash structure descriptions, bank statement processing configuration notes, financial instrument setup descriptions, or written descriptions of the treasury and cash management landscape. It does not connect to any SAP S/4HANA Treasury system, SAP Bank Communication Manager, SWIFT network, banking portal, Fiori launchpad, SAP GUI session, or backend RFC. It does not initiate, approve, release, cancel, or modify any payment, bank transfer, trade, hedge, settlement, or financial instrument transaction.

**This skill never executes payments, trades, or hedges.** No payment run initiation, no payment release, no bank transfer, no FX trade execution, no hedge designation or de-designation, no financial instrument creation or settlement, and no bank account opening or closing is performed or recommended as a direct action in this skill's execution path. All remediation recommendations describe configuration, authorization, or process design changes to be implemented and tested in a non-production environment.

## Lean operating rules

- Classify treasury control findings before recommending. Every finding must be assigned to a treasury control domain (cash position and liquidity / bank account management / in-house cash and payment factory / financial instruments / hedge and exposure management / payment fraud controls / treasury SoD) before a remediation path is proposed.
- Single-user payment release authority is a first-order fraud risk. Any configuration allowing one user to create, approve, and release a payment to an external bank account without a second authorized approver is a `critical` finding. Escalate before other remediation.
- Unauthorized bank account creation is a high-severity governance gap. A BAM configuration that allows bank account records to be created or modified without a documented four-eyes approval workflow exposes the entity to unauthorized external payment destination risk. Classify as `high`.
- Hedge documentation deficiency creates accounting and audit risk. Missing or incomplete hedge designation documentation under IFRS 9 or IAS 39 — including missing effectiveness testing results, incomplete hedge relationship designations, or undocumented risk component identification — is a `high` finding that may trigger derecognition or restatement.
- Cash position accuracy depends on memo record governance. Memo records that can be created or modified without authorization controls distort the cash position view and may support fraudulent cash concentration decisions. Assess memo record authorization as part of cash position review.
- Payment factory aggregation requires dual-control verification. In-house cash payment factories that aggregate and release external payments must enforce dual-control at the aggregation stage — not only at the individual payment request stage. A single approver at the aggregation level with no second reviewer is a fraud control gap.
- Treasury SoD must be enforced at the SAP authorization object level. Process-level SoD descriptions without corroborating SAP authorization object evidence (e.g., F_PAYR_BUK, F_REGU_BUK, TR_BANKI, TR_BANKA) are classified as `inference` and must be confirmed with user-provided role exports.
- Evidence from user-provided artifacts or official SAP S/4HANA Treasury and Cash Management documentation takes precedence over inference.
- Load only the reference needed for the treasury control domain under review.

## Evidence rules

Label all claims with one of:

- `documentation-based` — grounded in SAP S/4HANA Treasury and Risk Management, SAP Cash Management, SAP Bank Account Management, SAP Bank Communication Manager, or SAP Help Portal TRM documentation
- `user-provided evidence` — authorization object exports, bank account master data reports, payment run logs, hedge documentation summaries, in-house cash configuration descriptions, bank statement processing notes, financial instrument setup exports, or written descriptions provided by the user
- `inference` — derived reasoning not directly confirmed by official docs or user evidence

## Live-environment rules

**This skill does not touch live systems.** There is no SAP S/4HANA RFC call, Fiori OData API invocation, SAP GUI session, BAPI execution, SWIFT network connection, banking portal access, or direct database query in this skill's execution path. Users must supply configuration summaries, authorization object exports, bank account reports, payment run logs, hedge documentation notes, or written descriptions of their treasury and cash management landscape for this skill to review.

**This skill never executes payments, trades, or hedges.** Recommendations describing remediation always apply to configuration, authorization, or process design — not to direct payment execution, trade entry, hedge designation, bank account creation, or financial instrument settlement.

## References

Load only when needed:

- [Workflow and output contract](references/workflow-and-output.md) — treasury control finding taxonomy, severity assignment, output format.
- [Safety checklist](references/safety-checklist.md) — non-negotiables, common treasury control review mistakes, when to push back.
- [Official sources](references/official-sources.md) — SAP S/4HANA Treasury and Risk Management, Cash Management, Bank Account Management, and payment fraud control documentation.

## Response minimum

Return, at minimum:

- **Problem classification**: treasury control domain(s) affected (cash position and liquidity / bank account management / in-house cash and payment factory / financial instruments / hedge and exposure management / payment fraud controls / treasury SoD) and specific finding(s) per domain.
- **Evidence used**: documentation-based / user-provided evidence / inference.
- **Risk level**: critical (single-user payment release enabling external payment fraud; combined create-approve-release authority for external payments; unauthorized bank account creation path) / high (BAM four-eyes approval gap; hedge documentation deficiency creating derecognition risk; memo record manipulation without authorization; in-house cash aggregation without dual-control; treasury SoD in instrument creation and settlement) / medium (cash position accuracy gap from memo record governance weakness; bank statement processing automation gap; liquidity hierarchy design flaw; ineffectiveness testing configuration gap) / low (best practice deviation in financial instrument type design or reporting configuration).
- **Recommended action**: specific configuration or process remediation per finding (dual-control payment release configuration, BAM four-eyes approval workflow implementation, memo record authorization tightening, hedge designation workflow correction, in-house cash dual-control at aggregation, treasury SoD role separation, bank communication manager channel security, etc.).
- **Refusal / escalation triggers**: if single-user external payment release authority or unauthorized bank account creation path is found, escalate to the treasury, audit, and GRC team before any further payments are processed or bank accounts are modified. If a finding requires live SAP Treasury system inspection, state that clearly and ask the user to supply the relevant export or description. This skill never executes payments, trades, or hedges under any circumstances.
- **Business impact**: external payment fraud exposure, unauthorized cash transfer risk, hedge accounting restatement risk under IFRS 9 or IAS 39, regulatory reporting gap (EMIR, Dodd-Frank), bank account governance failure, audit finding risk, liquidity misstatement risk.
- **Next verification step**: confirm recommended configuration changes against the user's current SAP S/4HANA Treasury and Cash Management setup in a non-production system before promoting to production.
