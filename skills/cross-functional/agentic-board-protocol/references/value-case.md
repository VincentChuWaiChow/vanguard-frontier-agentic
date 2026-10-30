# Value Case

Reference for [agentic-board-protocol](../SKILL.md). Load it when an automation-investment decision needs a value case, when you challenge one, or when you decide how far up the automation ladder a proposal has earned the right to go.

A value case answers one question before any agent is built or funded: does this beat the non-agent alternatives by enough, after the cost of running it, overseeing it, building it and its expected losses? It lives in the value_case payload of a decision-request, and an automation-investment decision-request must carry one (see [decision-record.md](decision-record.md)). The arithmetic and the promotion rules are checked offline; the truth of the inputs is not.

## Business automation ladder

Climb the ladder only as far as the evidence demands. The lowest rung that meets the requirement wins, and every rung that was considered and rejected is recorded with its reason.

| Rung | Question | Owner of the question | Where it is recorded |
|---|---|---|---|
| 1. Eliminate | Does the process or step need to exist at all? | This protocol | decision_request.alternatives, kind eliminate |
| 2. Redesign | Can the process be simplified before anything is automated? | This protocol | alternatives, kind redesign |
| 3. Existing process | Is the current process, perhaps with small fixes, good enough? | This protocol | alternatives, kind existing-process; counterfactual existing-process (required) |
| 4. Deterministic workflow | Does a fixed, coded flow with no model-chosen steps do the job? | The architecture agent (technical rung) | alternatives, kind deterministic-workflow; counterfactual deterministic-workflow (optional) |
| 5. Conventional automation or RPA | Do rules engines, ERP-native matching, scripts or robotic process automation do the job? | This protocol | alternatives, kind conventional-automation; counterfactual conventional-automation (required) |
| 6. Single agent | Does one model-driven agent with a fixed tool list do the job? | The architecture agent (technical rung) | alternatives, kind single-agent; counterfactual single-agent (required) |
| 7. Multi-agent | Is there evidence for several cooperating agents? | The architecture agent (technical rung) | alternatives, kind multi-agent; counterfactual multi-agent (optional) |

The architecture agent owns only the technical rungs, and its pattern-selection review decides among them. This reference owns the business rungs and the value case. Rungs 1 and 2 have no counterfactual kind in the schema, so they appear only as alternatives. No business-process agent is justified in this repository by a value case alone; see the honesty section below.

## The value case record

All money is integer minor currency units, such as cents, never floating point. A money range has low, point and high. The word verified in the owner's term "Verified Annual Benefit" is a claim to be tested: the confidence label says how far it has actually been verified, and at assumption or estimate it is still a claim.

| Field | Meaning |
|---|---|
| currency | Three-letter currency code. The schema checks the pattern of three uppercase letters, not membership in the ISO 4217 list. |
| baseline_process | The process as it runs today, in words. |
| annual_volume | Cases per year in scope. |
| baseline_labor_error_cost, baseline_loss_leakage | The yearly cost pools the investment claims to reduce, in minor units. |
| intervention_rate_bp, exception_rate_bp | Share of cases needing human intervention, and share ending as exceptions, in basis points from 0 to 10000. |
| verified_annual_benefit | Range of yearly benefit. |
| annual_run_cost | Range of the yearly cost of operating it: models, infrastructure, tools. |
| annual_oversight_cost | Range of the yearly human cost of reviewing, intervening and handling exceptions. |
| annualized_implementation_cost | Range of the build and rollout cost spread over a stated period. State the period in the measurement plan or the proposal. |
| expected_risk_loss | Range of likelihood-weighted loss from the failure scenarios the architecture and security reviews identified. |
| net_annual_value | The computed range, below. |
| runs, successful_runs | Runs in the year and the runs that met the quality bar. |
| cost_per_successful_task_minor | Annual run cost per successful run, below. |
| counterfactuals | The non-agent and other alternatives, each with its own net annual value range and basis. |
| confidence | assumption, estimate, benchmark or measured. |
| promotion_state | candidate, pilot, limited-production or production. |
| measurement_plan | How each number will be measured, over what period, with what control. |

## Net annual value

The owner's formula is:

```text
Net Annual Value = Verified Annual Benefit - Annual Run Cost - Annual Human Oversight Cost - Annualized Implementation Cost - Expected Risk Loss
```

Each term is a range, and the result uses interval arithmetic:

- low is benefit.low minus the high of each of the four costs.
- point is benefit.point minus the point of each cost.
- high is benefit.high minus the low of each of the four costs.

Worked example, synthetic, in minor units (so 20,000,000 is 200,000.00 in the currency):

| Term | Low | Point | High |
|---|---|---|---|
| Verified annual benefit | 20,000,000 | 40,000,000 | 60,000,000 |
| Annual run cost | 8,000,000 | 10,000,000 | 14,000,000 |
| Annual oversight cost | 6,000,000 | 8,000,000 | 10,000,000 |
| Annualized implementation cost | 5,000,000 | 6,000,000 | 8,000,000 |
| Expected risk loss | 1,000,000 | 2,000,000 | 4,000,000 |
| Net annual value | -16,000,000 | 14,000,000 | 40,000,000 |

Low is 20,000,000 minus (14,000,000 + 10,000,000 + 8,000,000 + 4,000,000). High is 60,000,000 minus (8,000,000 + 6,000,000 + 5,000,000 + 1,000,000). The arithmetic is exact on integers with no rounding. The interval is a worst-case to best-case envelope built from the stated ranges. It is not a statistical confidence interval, and it ignores correlation between the terms. A negative low is the point of the exercise: it shows the downside the point estimate hides.

Two structural guards support the arithmetic. The benefit ceiling: verified_annual_benefit.high may not exceed baseline_labor_error_cost plus baseline_loss_leakage, because a benefit cannot exceed the cost pools it claims to remove. The oversight floor: a non-zero intervention or exception rate requires annual_oversight_cost.low above zero, because automation that needs human intervention is not free of human cost.

## Unit economics

Cost per successful task is annual run cost divided by successful runs, as an integer floor of the point, or 0 when there are no successful runs. It is aligned to the FinOps cost per successful outcome. Per the [FinOps for AI overview](https://www.finops.org/wg/finops-for-ai-overview/), teams should "Track cost per inference, per user, and per successful outcome for each use case, and report them as distributions", and the cost per successful outcome is defined as the "Cost of results that meet the quality bar", computed as "Total cost of all runs / Runs that meet the quality bar".

- The numerator covers all runs, including failed ones, which is why total run cost is divided by successful runs only.
- The record holds the single point figure the gate can recompute. The distribution belongs in the measurement evidence, and the measurement plan should say how it will be reported.
- What meets the quality bar is defined in the measurement plan. The gate cannot judge it.
- This unit cost covers run cost only. Oversight and implementation costs are separate lines in the net annual value, so a fully loaded cost per outcome is something the finance challenger assembles.
- Per the [FinOps Unit Economics capability](https://www.finops.org/framework/capabilities/unit-economics/), related unit measures include cost per token, cost per agent action and cost per case deflected. Model and token unit prices are owned by finops-ai-economist-agent.
- The number of successful runs cannot exceed the number of runs.

## Counterfactuals

A counterfactual is what the organization would get from an alternative, valued the same way.

- Required: existing-process, conventional-automation and single-agent.
- Optional: deterministic-workflow and multi-agent.
- Each has a net annual value range and a basis in words saying where the number came from.
- When net annual value is measured as an increment over the status quo, the existing-process counterfactual is zero by construction. Say so in its basis.

The governance basis is NIST AI RMF MANAGE 2.1, "viable non-AI alternative systems, approaches, or methods" ([NIST AI RMF core](https://airc.nist.gov/airmf-resources/airmf/5-sec-core/)). This is a candidate mapping that an accountable owner confirms; it does not mean a value case satisfies the subcategory. The AI RMF 1.0 is being revised, so check the current wording.

## Confidence and promotion

Confidence labels are protocol definitions:

| Label | Meaning |
|---|---|
| assumption | A stakeholder's belief with no data behind it. |
| estimate | Derived from partial or internal data with the reasoning stated. |
| benchmark | Taken from a comparable external or earlier system. |
| measured | Observed from the organization's own runs under the measurement plan. |

Promotion states are candidate (an idea on paper), pilot (a bounded trial, for example in shadow mode), limited-production (live for a bounded population) and production.

- Promotion to limited-production or production requires measured confidence and a net annual value low above every counterfactual's low.
- Recommending approve or conditional-approve on an investment requires the net annual value point above every counterfactual's point. These are different tests: the recommendation compares expected values, and promotion compares conservative bounds.
- A value case that does not beat its best counterfactual is a reject, a redesign, or a step down the ladder, not a conditional approval with hopes attached.

## Honesty about evidence

- Repository fixtures are synthetic and can never exceed pilot: a synthetic record may not claim measured confidence or promotion beyond pilot.
- A real promotion needs measured evidence held outside this repository and a human attestation by the decision owner that the evidence is real. The gate can check form and arithmetic. It cannot check that a measured number is true.
- Re-entry for business-process agents that this repository does not ship is the same: a sponsor runs the value case on measured data outside the repository, the record passes the gate structurally, and the owner attests the evidence.

## Counterfactual-attack checklist

For the finance challenger, finance-capital-allocation-advisor-agent. Each row is an attack to attempt, with the evidence that would defeat it.

| Attack | Question to ask | Evidence to request |
|---|---|---|
| Benefit double-counting | Is the same hour, error or loss counted in more than one benefit line, or already claimed by another initiative? | Line-item benefit derivation tied to specific cost pools; the other initiatives' claims. |
| Baseline inflation | Is the baseline the true current cost, or a high, stale or hand-picked figure? Inflating it also raises the benefit ceiling. | Finance-owned actuals for a named period. |
| Ignoring oversight cost | Who reviews, approves and corrects outputs, how many minutes per case, at what loaded rate? | A time study; the intervention rate and how it was measured. |
| Ignoring exception handling | What happens to the cases the system cannot handle, and are they harder or costlier than average? | The exception rate, its definition, and the cost of an exception under each alternative. |
| Comparing against a strawman workflow | Is the conventional or deterministic counterfactual the best realistic version, built with comparable effort, or a deliberately weak one? | Who designed it, what it assumes, vendor quotes, and the same cost categories as the agent case. |
| Survivorship in pilot data | Did the pilot include only easy cases or willing teams, drop failed runs, or run without real exceptions? | Cohort selection rules, the denominator of runs, successful runs versus all runs, and a control group. |

## What the gate does not check

The offline gate checks arithmetic and structure. It does not check:

- that any input is true, including a measured label;
- that counterfactual net annual values were computed correctly; only the range order of each is checked;
- that cost ranges are non-negative: a negative implementation cost or expected risk loss, entered consistently with the net annual value, is not rejected, and neither is a negative oversight cost when both rates are zero. A negative run cost is caught only indirectly, by the unit-cost rule, and only when there are successful runs;
- that runs relates to annual_volume, that the amortization period is stated, or that currency is a real ISO 4217 code;
- that the quality bar is defined, or that benefits are not double-counted inside the ceiling.

The finance challenger covers these. The checklist above is how.

## Rules checked by the gate

| Rule | Requirement and notes |
|---|---|
| `VC-ORDER` | Every money range satisfies low <= point <= high. This covers the counterfactual ranges too. |
| `VC-NAV-ARITHMETIC` | net_annual_value equals benefit minus run, oversight, implementation and risk loss under interval arithmetic. |
| `VC-BENEFIT-CEILING` | verified_annual_benefit.high does not exceed baseline labor and error cost plus baseline loss and leakage. |
| `VC-OVERSIGHT` | A non-zero intervention or exception rate requires annual_oversight_cost.low > 0. |
| `VC-RUNS-BOUND` | successful_runs does not exceed runs. |
| `VC-UNIT-COST` | cost_per_successful_task_minor equals floor(annual_run_cost.point / successful_runs), or 0 with no successful runs. |
| `VC-COUNTERFACTUALS` | Counterfactuals include existing-process, conventional-automation and single-agent. |
| `VC-PROMOTION` | limited-production or production requires measured confidence and net_annual_value.low above every counterfactual's low. |
| `VC-SYNTHETIC` | A synthetic record never claims measured confidence or promotion beyond pilot. |
| `VC-RECOMMEND-INCREMENTAL` | Recommending approve or conditional-approve on an investment requires net_annual_value.point above every counterfactual's point. |
