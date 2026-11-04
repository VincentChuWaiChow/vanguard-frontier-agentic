---
name: sap-supply-chain-ibp-resilience-review
description: "Review SAP Integrated Business Planning (IBP) and supply chain resilience posture: demand and supply planning configuration, Sales and Operations Planning (S&OP) process governance, inventory optimization and safety stock logic, statistical forecast accuracy and bias, IBP control tower alerts and exception management, scenario planning design, and supply chain risk coverage. Flags planning gaps, forecast blind spots, inventory policy misalignments, alert fatigue risks, and scenario planning deficiencies. Does not modify planning data, run supply plans, or mutate any live IBP system."
allowed-tools: Read Grep Glob WebSearch WebFetch
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-06-19"
  category: data
  lifecycle: experimental
---

# SAP Supply Chain IBP Resilience Review

## Purpose

Assess the planning configuration quality and supply chain resilience posture of SAP Integrated Business Planning (IBP) deployments. Evaluate demand planning configuration including statistical forecasting model selection, causal factors and promotions planning setup, lifecycle planning design, and consensus demand planning process governance. Assess supply planning configuration including capacity constraint modeling, network design alignment, supply heuristics and optimizer settings, and multi-echelon replenishment logic. Review Sales and Operations Planning (S&OP) process governance by evaluating S&OP cycle design in IBP for S&OP, meeting cadence governance, escalation and decision authority configuration, and plan version management. Analyze inventory optimization logic by reviewing safety stock calculation methods, service level target configuration, inventory stratification, and slow-moving or excess inventory monitoring. Assess statistical forecast accuracy and bias by reviewing error metric configuration (MAPE, WMAPE, bias), forecast accuracy dashboards, outlier correction process design, and whether forecast error drives planning model improvement. Evaluate IBP control tower alert design and exception management by reviewing alert rule completeness, alert threshold configuration, alert routing and ownership, and exception resolution governance to identify alert fatigue risks and coverage blind spots. Review scenario planning design by assessing what-if scenario configuration, scenario comparison capability, and whether scenario planning is integrated with the S&OP decision process. Does not connect to or mutate any live SAP IBP tenant, supply planning data, or integrated SAP S/4HANA system. Never modifies demand plans, supply plans, inventory targets, or control tower alert configurations.

## When to use

Use this skill when the user asks to:

- review SAP IBP demand planning configuration: statistical forecasting algorithm selection (ARIMA, exponential smoothing, croston for intermittent demand), causal factor and promotions planning setup, lifecycle planning for new product introduction and end-of-life, consensus demand planning process design, and forecast override governance,
- assess SAP IBP supply planning design: supply planning heuristic or optimizer configuration, capacity constraint modeling completeness (machine, labor, transport), network design alignment with physical supply chain, multi-echelon replenishment logic, and supply plan exception handling for capacity violations or supply shortfalls,
- evaluate SAP IBP for S&OP process governance: S&OP cycle and review meeting design, plan version management and version control configuration, collaborative review participation coverage, escalation and decision authority configuration, and whether the S&OP process produces a single consensus plan or competing plan versions,
- review IBP inventory optimization configuration: safety stock calculation method (statistical, time-series, demand-driven MRP), service level target assignment by product and location, inventory stratification (ABC/XYZ or equivalent), slow-moving and excess inventory identification logic, and inventory target review cadence,
- assess forecast accuracy and bias monitoring: IBP forecast error metric configuration (MAPE, WMAPE, bias, tracking signal), forecast accuracy dashboard completeness, outlier detection and correction process design, whether statistically significant forecast bias is tracked and acted upon, and whether accuracy metrics drive model selection or parameter tuning reviews,
- evaluate IBP control tower alert design and exception management: alert rule completeness across planning dimensions (demand deviation, supply shortfall, inventory below safety stock, transportation delay, supplier delivery performance), alert threshold calibration, alert ownership and routing configuration, exception resolution workflow, and whether alert volume creates alert fatigue that reduces actionable exception management,
- review scenario planning design in SAP IBP: what-if scenario configuration (demand shock, supply disruption, capacity constraint), scenario comparison and financial impact quantification capability, integration of scenario outputs into the S&OP decision process, and whether scenario planning is conducted reactively (crisis) or proactively (resilience governance),
- assess supply chain resilience coverage gaps: single-source-of-supply dependencies not covered by supply planning scenarios, geographic concentration in demand or supply not modeled as risk scenarios, lead time volatility not reflected in safety stock calculations, and whether IBP planning parameters are reviewed and updated in response to supply chain disruption events.

## When not to use

- When the user needs live inspection of SAP IBP planning data, active supply plans, demand forecasts, or control tower alerts — this skill accepts only user-provided configuration descriptions, planning parameter exports, accuracy metric reports, alert rule summaries, scenario descriptions, or written descriptions of the IBP landscape.
- When the request is about SAP S/4HANA Production Planning (PP) or Manufacturing Execution (ME) execution-level scheduling without a supply chain planning resilience angle — this skill focuses on IBP planning configuration and resilience, not production shop floor execution.
- When the request is about SAP Transportation Management (TM) route optimization or freight cost management rather than supply planning resilience — TM is a distinct execution platform.
- When the request concerns SAP Ariba procurement sourcing event design or SAP S/4HANA Procurement processes without an IBP supply planning integration angle — use `sap-procurement-ariba-value-leakage-review` for procurement-specific assessment.
- When the request is about SAP Analytics Cloud (SAC) advanced analytics modeling or data science pipeline design for demand forecasting outside the IBP platform — use a dedicated SAC or analytics skill.

## Does not touch live systems

This skill operates on user-provided configuration descriptions, planning parameter exports, S&OP process documentation, forecast accuracy metric reports, inventory policy summaries, control tower alert rule descriptions, scenario planning configuration notes, or written descriptions of the IBP and supply chain planning landscape. It does not connect to any SAP IBP tenant, SAP S/4HANA system, IBP Excel add-in session, IBP Fiori launchpad, or SAP Analytics Cloud. It does not create, modify, approve, or publish demand plans, supply plans, inventory targets, alert rules, or scenario configurations. All live inspection is out of scope.

**This skill never modifies planning data.** No demand plan creation or modification, no supply plan run, no inventory target change, no alert configuration update, and no scenario activation is performed or recommended as a direct action in this skill's execution path. All remediation recommendations describe configuration and process design changes to be implemented and tested in a non-production IBP tenant before promoting to production.

## Lean operating rules

- Classify resilience findings before recommending. Every finding must be assigned to a planning domain (demand planning / supply planning / S&OP governance / inventory optimization / forecast accuracy / control tower / scenario planning / resilience coverage) before a remediation path is proposed.
- Forecast bias is a first-order planning quality signal. Persistent systematic positive or negative bias in demand forecasts distorts supply plans, inventory targets, and S&OP decisions. Any confirmed forecast bias above a material threshold that is not being actively corrected is a `high` finding.
- Safety stock calculated without lead time variability is structurally incomplete. Safety stock that only accounts for demand variability but ignores supplier lead time variability or replenishment cycle uncertainty underestimates buffer inventory requirements — especially for disruption-prone supply lanes.
- Alert fatigue from excessive low-value alerts destroys the value of control tower monitoring. An alert design where more than 30% of daily alerts are resolved without action (noise) is a `medium` finding requiring threshold recalibration and alert hierarchy redesign.
- Single-source supply dependencies not modeled in scenario planning are a `high` resilience gap. Any product or location with a single-source-of-supply that has not been modeled as a disruption scenario in IBP is an unquantified resilience risk.
- S&OP without a single consensus plan is not S&OP. If the S&OP process produces competing plan versions across functions without a structured consensus and escalation path to a single approved plan, the S&OP process is a governance gap regardless of the cadence.
- Inventory stratification that is not refreshed at least quarterly becomes stale. ABC/XYZ classification that does not reflect current demand patterns leads to misaligned service level targets and safety stock levels for products whose demand profile has changed.
- Evidence from user-provided artifacts or official SAP IBP documentation takes precedence over inference.
- Load only the reference needed for the IBP planning domain under review.

## Evidence rules

Label all claims with one of:

- `documentation-based` — grounded in SAP Integrated Business Planning Help Portal documentation, SAP IBP configuration guides, SAP supply chain best practice documentation, or SAP S&OP governance guidance
- `user-provided evidence` — planning parameter exports, S&OP process documentation, forecast accuracy reports, inventory policy summaries, control tower alert rule descriptions, scenario planning configuration notes, or written descriptions provided by the user
- `inference` — derived reasoning not directly confirmed by official docs or user evidence

## Live-environment rules

**This skill does not touch live systems.** There is no SAP IBP API call, IBP Excel add-in session, Fiori OData request, SAP S/4HANA RFC invocation, or direct database query in this skill's execution path. Users must supply planning parameter exports, S&OP process documentation, forecast accuracy metric reports, inventory policy summaries, control tower alert rule descriptions, scenario planning configuration notes, or written descriptions of their IBP and supply chain planning landscape for this skill to review.

**This skill never modifies planning data.** Recommendations describing remediation always apply to configuration, planning parameter design, or process governance — not to direct demand plan modification, supply plan execution, inventory target change, or alert rule activation.

## References

Load only when needed:

- [Workflow and output contract](references/workflow-and-output.md) — IBP resilience finding taxonomy, severity assignment, output format.
- [Safety checklist](references/safety-checklist.md) — non-negotiables, common IBP resilience review mistakes, when to push back.
- [Official sources](references/official-sources.md) — SAP IBP demand and supply planning, S&OP, inventory optimization, control tower, and scenario planning documentation.

## Response minimum

Return, at minimum:

- **Problem classification**: IBP planning domain(s) affected (demand planning / supply planning / S&OP governance / inventory optimization / forecast accuracy / control tower / scenario planning / resilience coverage) and specific finding(s) per domain.
- **Evidence used**: documentation-based / user-provided evidence / inference.
- **Risk level**: critical (supply plan cannot respond to a high-probability disruption scenario due to missing coverage; S&OP producing conflicting unconstrained plans with no governance resolution path) / high (persistent forecast bias distorting supply plan; safety stock not accounting for lead time variability in a disruption-exposed lane; single-source supply not in scenario planning; critical alert threshold misconfigured causing missed exception) / medium (alert fatigue from excessive low-value alerts; inventory stratification stale; S&OP cadence without structured escalation; scenario planning conducted reactively only) / low (best practice deviation in forecasting model selection or S&OP template design).
- **Recommended action**: specific configuration or process remediation per finding (bias correction model parameter adjustment, safety stock calculation method update, single-source scenario addition, alert threshold recalibration, S&OP escalation path design, inventory stratification refresh schedule, scenario planning integration into S&OP cadence, etc.).
- **Refusal / escalation triggers**: if a finding requires live IBP tenant inspection (active plan versions, real-time control tower alerts, current forecast accuracy metrics), state that live inspection is out of scope and ask the user to supply the relevant export or description. If a resilience gap represents an immediate supply disruption risk, flag for supply chain leadership escalation.
- **Business impact**: service level risk (stockout probability increase), working capital impact (excess or insufficient inventory), supply chain disruption exposure (unmodeled single-source risk), S&OP decision quality degradation, or control tower effectiveness loss from alert fatigue.
- **Next verification step**: confirm recommended configuration changes against the current IBP setup in a non-production IBP tenant before promoting to the production planning environment.
