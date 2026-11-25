---
layout: default
title: "Cilium Board: Value Model and KPIs"
permalink: /docs/architecture/cilium-board/05-value-model-and-kpis/
---

# Value Model and KPIs

> Status: **BUILT — design record.** Previous: [04-threat-model-and-governance.md](./04-threat-model-and-governance.md) ·
> Next: [06-red-team-and-release-readiness.md](./06-red-team-and-release-readiness.md)

**No value has been measured.** This repository ships contracts and gates, not a runtime, and has no production telemetry for these agents. Everything below is a model with named inputs, to be filled with measured data by an adopting organization. Any number in a worked example is labeled hypothetical.

## 1. Incident investigation capacity

`H_saved = N_incidents × (T_baseline − T_assisted)`, and `V_capacity = H_saved × C_hourly`.

- `N_incidents`: Cilium-attributable incidents in a fixed period, counted from the incident tracker with a consistent classification rule decided before measurement.
- `T_baseline`, `T_assisted`: time from incident declaration to **defensible diagnosis**, meaning a mechanism plus the discriminating observation that confirmed it, not time to mitigation. Measure the median and the p95 separately; the p95 is where the dataplane specialist is meant to help.
- Recovered capacity is not cash. Report `V_capacity` as engineering hours available for other work unless the organization's cost model shows the hours were actually released.

Hypothetical, for shape only: 24 incidents a year, p50 diagnosis 3.0 h falling to 2.0 h gives 24 h a year. At that scale the board's value is in the p95 tail and in avoided wrong fixes, not in the median.

## 2. Availability

`V_availability = Σ ΔT_i × C_i`, over incidents where the board's diagnosis demonstrably shortened downtime. Attribution is the hard part: count only incidents where the timeline shows the board's discriminating observation was the step that resolved the ambiguity, and never add this to `V_capacity` for the same hours.

## 3. Change safety

The lifecycle and policy specialists are preventive, so their value appears as a lower **network-related change failure rate**: failed or rolled-back Cilium upgrades, migrations and policy rollouts divided by those attempted. A blocked change is not value by itself; it is value only if the blocker was real, which is why false-positive rates are tracked.

## 4. Cost

> **Correction (pre-acceptance review, 2026-11-25).** The maintenance estimate below is understated. The FinOps reviewer counted about 150 ledger rows and about 350 version-keyed lines across the skills; re-verifying them is an estimated 1.5 to 4.5 engineer-days per Cilium minor (estimate, not measured). Three KPIs in §5 measure form rather than value (evidence-field completeness), rise with review volume (exposures found), or cannot be measured without a labeled corpus (false negatives). The maestro KPI needs routing logs that do not exist. Revised KPIs are in [07](./07-pre-acceptance-review.md).

`C_total = C_inference + C_tools + C_maintenance + C_review + C_infrastructure`.

`C_maintenance` dominates for this board. The ledgers are dated snapshots, so every Cilium minor release (about every six months) needs a refresh of at least the lifecycle skill's breaking-changes and advisory references. An estimate, which is a judgment and not a measurement: one to two engineer-days per minor release for the whole board, mostly in the lifecycle skill. `C_review` is the human review of every `approved` that gates a change; the design assumes it is never zero.

`ROI = (V_realized − C_total) / C_total`, computed only from measured `V_realized`.

## 5. KPI framework

| Objective | KPI | Baseline method | Owner | Interval | Limitation |
| --- | --- | --- | --- | --- | --- |
| Incident efficiency | p50 and p95 time to defensible diagnosis | Prior two quarters of Cilium incidents, re-read against the definition | SRE lead | Quarterly | Small samples; definition drift |
| Resilience | Cilium incident recurrence rate | Same classification rule | Incident manager | Quarterly | Recurrence attribution |
| Change safety | Network-related change failure rate | Change records for upgrades, migrations, policy rollouts | Release manager | Per release | Few upgrades a year |
| Security | Confirmed unauthorized-connectivity exposures found before an incident | Policy reviews | Security architect | Quarterly | Counts depend on review volume |
| Governance | Share of findings with complete evidence fields at handoff | Handoff records | Platform lead | Monthly | Measures form, not truth |
| Quality | False-positive and false-negative finding rates | Human review outcomes | Board maintainer | Quarterly | Needs labeled review data |
| Agent efficiency | Unnecessary specialist invocations per routed request | Maestro plans versus specialists whose output changed the verdict | Board maintainer | Monthly | Requires routing logs |
| Knowledge retention | Runbooks created from verified findings and still current at the last minor release | Confluence records | Platform lead | Per minor release | Stale pages count as zero |
| Economics | Cost per validated outcome | `C_total` divided by findings confirmed by human review | FinOps (`finops-maestro-agent`) | Quarterly | Needs cost attribution |

## 6. What would show the value is not there

- p95 diagnosis time does not move after two quarters of real use.
- Most `blocked` lifecycle verdicts are overridden by reviewers as wrong.
- The maestro rarely changes the first owner a user would have picked.
- Ledger refresh is skipped for two consecutive minors; at that point the version-specific references are worse than none and should be marked deprecated.
