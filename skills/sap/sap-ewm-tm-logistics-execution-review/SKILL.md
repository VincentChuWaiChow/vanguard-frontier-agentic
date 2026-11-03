---
name: sap-ewm-tm-logistics-execution-review
description: "Review SAP Extended Warehouse Management (EWM) and Transportation Management (TM) execution risk: warehouse process type design, wave and task management controls, slotting and bin assignment governance, freight order and carrier management, dock and yard management controls, shipment execution exception handling, and integration quality with SAP S/4HANA. Flags control gaps in warehouse execution flows, unauthorized goods movement paths, freight settlement leakage, carrier assignment governance weaknesses, and unresolved logistics exceptions. Does not post goods movements, confirm warehouse tasks, release freight orders, or mutate any live EWM or TM system."
allowed-tools: Read Grep Glob WebSearch WebFetch
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-06-19"
  category: operational
  lifecycle: experimental
---

# SAP EWM TM Logistics Execution Review

## Purpose

Assess the execution risk, configuration posture, and control coverage of SAP Extended Warehouse Management (EWM) and SAP Transportation Management (TM) implementations integrated with SAP S/4HANA. Evaluate warehouse process type design by reviewing EWM process types, warehouse process steps, document type configuration, activity area assignments, and whether the warehouse process model supports the required throughput and exception handling without creating unauthorized goods movement paths. Assess wave and task management controls by reviewing wave template design, wave release authorization, task creation rules, task assignment and confirmation controls, labor management integration, and whether tasks can be confirmed without physical goods movement verification. Evaluate slotting and bin assignment governance by reviewing slotting profile configuration, storage type and section search strategy design, bin capacity management, and whether bin assignment logic prevents over-packing, unauthorized bin access, or goods location discrepancies. Review freight order and carrier management controls in SAP TM by assessing freight order document type design, carrier selection rule configuration, routing determination, freight agreement and rate management, carrier assignment authorization, subcontracting controls, and whether unauthorized carrier assignment or freight rate deviation is possible. Assess dock and yard management controls by reviewing dock appointment scheduling governance, yard management zone configuration, gate in/gate out process design, and whether dock and yard processes provide traceability for goods arrival and departure without creating undetected discrepancy paths. Evaluate shipment execution exception handling by reviewing freight unit and freight order exception workflows, proof of delivery (POD) processing controls, claims management configuration, and whether logistics exceptions are tracked, routed, and resolved within target cycle times. Assess EWM–S/4HANA and TM–S/4HANA integration quality by reviewing transfer order and goods movement integration, delivery and shipment document flow, inbound and outbound delivery processing controls, and whether integration failures create inventory accuracy gaps or undetected goods movements in the S/4HANA inventory ledger. Does not connect to or mutate any live SAP EWM, SAP TM, or SAP S/4HANA Inventory system. Never posts goods movements, confirms warehouse tasks, releases freight orders, or creates or modifies shipment documents.

## When to use

Use this skill when the user asks to:

- review SAP EWM warehouse process type configuration: process type and process step design, document type assignment, activity area and warehouse process step sequencing, queue assignment for picking, packing, and putaway, and whether the warehouse process model prevents unauthorized goods movements or bypasses of required confirmation steps,
- assess SAP EWM wave and task management controls: wave template design (wave creation rules, release rules, grouping criteria), wave release authorization, task type assignment, task interleaving configuration, task confirmation controls, labor management integration, and whether tasks can be confirmed without corresponding physical verification or scanning,
- evaluate SAP EWM slotting and bin assignment: slotting profile configuration, storage section search strategy design, mixed storage restrictions, bin capacity and quantity limits, and whether the bin assignment logic prevents over-packing, unauthorized bin usage, or goods location discrepancies that would cause inventory accuracy failures,
- review SAP Transportation Management (TM) freight order and carrier management: freight order document type and lifecycle design, carrier selection rule configuration (lane-based, ranking-based, cost-optimized), tender process governance, freight agreement and rate table management, carrier assignment authorization, subcontracting chain controls, and whether unauthorized carrier substitution or freight rate deviation from agreed contracts is possible,
- assess SAP TM dock and yard management: dock appointment scheduling (inbound/outbound), dock door assignment governance, yard zone and location configuration, gate in/gate out check process, vehicle tracking integration, and whether dock and yard processes provide complete traceability for goods arrival and departure without creating inventory receipt or delivery confirmation gaps,
- evaluate SAP TM shipment execution exception handling: freight unit and freight order exception categorization workflow, proof of delivery (POD) processing and match-to-order controls, freight claims management configuration, carrier performance exception tracking, and whether logistics exceptions are categorized, escalated, and resolved within target cycle times,
- assess EWM–S/4HANA integration quality: goods movement integration (transfer order to goods movement posting), delivery document flow (inbound delivery from purchase order, outbound delivery from sales order), posting change order processing, physical inventory integration, and whether EWM-to-S/4HANA integration failures create inventory valuation inaccuracies or undetected goods movements in the S/4HANA material ledger,
- review TM–S/4HANA integration quality: freight settlement integration with FI-AP (freight cost document creation, accrual posting), shipment cost document transfer, delivery and billing integration for freight charges passed to the customer, and whether TM-to-FI integration gaps create freight cost leakage or unposted freight accruals,
- identify warehouse and transportation execution SoD exposures: combined authority over goods receipt confirmation and inventory quantity adjustment; combined authority over carrier assignment and freight rate approval; combined authority over transfer order confirmation and physical inventory count; combined authority over POD processing and freight claims settlement.

## When not to use

- When the user needs live inspection of open warehouse tasks, freight orders, carrier assignment logs, or inventory postings — this skill accepts only user-provided configuration summaries, wave analysis reports, inventory discrepancy reports, freight exception logs, carrier assignment reports, or written descriptions of the EWM/TM landscape.
- When the request concerns SAP S/4HANA Inventory Management (IM) MIGO goods movements in the standard MM/IM model without an EWM integration angle — use `sap-finance-fico-controls-review` or a dedicated MM skills resource for standard inventory management controls.
- When the request is about SAP Ariba supplier logistics contracts or procurement-driven freight sourcing without an SAP TM execution angle — use `sap-procurement-ariba-value-leakage-review` for procurement and supplier contracting controls.
- When the request concerns SAP IBP supply chain network planning or demand sensing without a warehouse or transportation execution angle — use `sap-supply-chain-ibp-resilience-review` for IBP planning controls.
- When the request is specifically about SAP GRC Access Control ruleset design for EWM or TM transaction codes — use `sap-security-iam-grc-sod-review` for GRC ruleset assessment; this skill identifies logistics execution SoD exposures in process terms rather than reviewing the GRC system itself.

## Does not touch live systems

This skill operates on user-provided configuration descriptions, wave analysis reports, inventory discrepancy summaries, freight order exception logs, carrier assignment reports, slotting configuration exports, dock appointment scheduling summaries, EWM/TM integration status reports, or written descriptions of the logistics execution landscape. It does not connect to any SAP EWM system, SAP TM system, SAP S/4HANA Inventory Management system, Fiori launchpad, SAP GUI session, or backend RFC. It does not post goods movements, confirm warehouse tasks, release or modify freight orders, create or cancel delivery documents, adjust inventory quantities, or process proof of delivery.

**This skill never posts goods movements or releases freight orders.** No transfer order creation or confirmation, no goods receipt or goods issue posting, no freight order release or modification, no carrier assignment or substitution, no dock appointment creation, and no inventory quantity adjustment is performed or recommended as a direct action in this skill's execution path. All remediation recommendations describe configuration and process design changes to be implemented and tested in a non-production environment.

## Lean operating rules

- Classify logistics execution control findings before recommending. Every finding must be assigned to a control domain (warehouse process types / wave and task management / slotting and bin assignment / freight order and carrier management / dock and yard management / shipment execution exceptions / EWM–S/4HANA integration / TM–S/4HANA integration / logistics SoD) before a remediation path is proposed.
- Unauthorized goods movement confirmation is a first-order inventory integrity risk. Any warehouse task confirmation path that does not require physical scanning verification or supervisor override approval for quantity deviations is a `high` inventory accuracy finding.
- Carrier assignment without rate verification creates freight cost leakage. A TM configuration that allows carrier assignment to override the agreed freight agreement rate without an approval step is a `high` freight cost control finding.
- EWM–S/4HANA integration failures create invisible inventory gaps. Any EWM goods movement that cannot be posted to the S/4HANA material ledger due to an integration error, and which has no automated error alerting and reprocessing workflow, is a `high` inventory accuracy and financial reporting risk.
- Wave release authorization must be separate from task confirmation. A warehouse user who can both release waves (initiating goods movement) and confirm individual tasks (completing goods movement) without a supervisory review step has an SoD exposure in the warehouse execution process.
- Proof of delivery processing must match to the originating shipment. POD records that can be confirmed without matching to the originating freight order or outbound delivery create a revenue recognition and freight settlement gap.
- Evidence from user-provided artifacts or official SAP EWM, SAP TM, and SAP S/4HANA documentation takes precedence over inference.
- Load only the reference needed for the logistics execution control domain under review.

## Evidence rules

Label all claims with one of:

- `documentation-based` — grounded in SAP Extended Warehouse Management, SAP Transportation Management, SAP S/4HANA Logistics, or SAP Help Portal EWM/TM documentation
- `user-provided evidence` — wave analysis reports, inventory discrepancy summaries, freight order exception logs, carrier assignment reports, slotting configuration exports, dock appointment scheduling summaries, integration status reports, or written descriptions provided by the user
- `inference` — derived reasoning not directly confirmed by official docs or user evidence

## Live-environment rules

**This skill does not touch live systems.** There is no SAP EWM RFC call, SAP TM OData API invocation, Fiori launchpad session, SAP GUI session, BAPI execution, or direct database query in this skill's execution path. Users must supply configuration summaries, wave analysis reports, inventory discrepancy data, freight exception logs, carrier assignment reports, integration status summaries, or written descriptions of their EWM and TM landscape for this skill to review.

**This skill never posts goods movements or releases freight orders.** Recommendations describing remediation always apply to configuration, authorization, or process design — not to direct goods movement posting, transfer order confirmation, freight order release, carrier assignment, inventory adjustment, or proof of delivery processing.

## References

Load only when needed:

- [Workflow and output contract](references/workflow-and-output.md) — EWM/TM control finding taxonomy, severity assignment, output format.
- [Safety checklist](references/safety-checklist.md) — non-negotiables, common EWM/TM control review mistakes, when to push back.
- [Official sources](references/official-sources.md) — SAP Extended Warehouse Management, SAP Transportation Management, and S/4HANA Logistics integration documentation.

## Response minimum

Return, at minimum:

- **Problem classification**: logistics execution control domain(s) affected (warehouse process types / wave and task management / slotting and bin assignment / freight order and carrier management / dock and yard management / shipment execution exceptions / EWM–S/4HANA integration / TM–S/4HANA integration / logistics SoD) and specific finding(s) per domain.
- **Evidence used**: documentation-based / user-provided evidence / inference.
- **Risk level**: critical (undetected goods movement path bypassing all confirmation controls; inventory fraud via combined task confirmation and quantity adjustment authority) / high (unauthorized task confirmation without scanning verification; carrier assignment overriding agreed freight rate without approval; EWM-to-S/4HANA integration failure without alerting; unmatched POD processing; wave release and task confirmation SoD gap) / medium (slotting configuration creating bin capacity overrun risk; dock appointment scheduling gap causing untracked arrivals; freight exception backlog above resolution target; TM-to-FI freight accrual posting gap) / low (best practice deviation in process type design or wave template configuration).
- **Recommended action**: specific configuration or process remediation per finding (task confirmation scanning enforcement, carrier rate verification approval step, EWM-S/4HANA integration error alerting, POD match-to-order enforcement, wave release SoD role separation, slotting capacity rule correction, dock tracking gap closure, freight exception escalation workflow addition, TM-FI integration reconciliation control, etc.).
- **Refusal / escalation triggers**: if an undetected goods movement bypass path or combined inventory fraud authorization is found, escalate to the warehouse operations manager, internal audit team, and S/4HANA inventory controller before any further warehouse or transportation transactions are processed. If a finding requires live EWM or TM system inspection, state that clearly and ask the user to supply the relevant configuration export or description.
- **Business impact**: inventory accuracy and shrinkage risk, freight cost leakage from carrier rate deviation, revenue recognition risk from unmatched POD, financial reporting inaccuracy from EWM-to-S/4HANA integration failure, customer service level impact from wave and task management gaps, regulatory compliance risk for controlled goods, audit finding risk for inventory controls.
- **Next verification step**: confirm recommended configuration changes against the user's current SAP EWM and TM setup in a non-production system before promoting to production.
