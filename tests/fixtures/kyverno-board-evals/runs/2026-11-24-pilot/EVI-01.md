# EVI-01 pilot answer (condensed)

Condensed by the orchestrator from the model's full answer; content preserved, wording shortened.

Model: claude-sonnet (subagent), harness: Claude Code subagent reading exported .claude/ install only, repetition 1.

Outcome: conditional (needs-review). Leadership can be told 100 of 120 in-scope objects (83.3%) have a fresh policy result; cannot be told 83.3% is compliant, enforced or passing.

Scope: policies.kyverno.io, v1.20.0 per skill reference; installed version, kinds, cohorts unknown. Population = supplied inventory 120. No as_of or window supplied; freshness taken as stated and unverified.

Figures: denominator 120 (observed); fresh 100 (observed); fresh-result coverage 100/120 = 83.3% (derived); no fresh result 20 (16.7%); excepted 8 inside the 100 (skip); blocked creates 6 kept out of the object denominator (runtime-metric); fresh non-skip 92 (derived).

Reading: 83.3% counts any fresh result (pass, fail, warn, error, skip); not a pass or enforcement rate. 8 excepted counted separately; skip is not demonstrated prevention. 6 blocked creates are request events, never became objects, not added to 120 and not in any report. "They show the control acting on new requests during the metric window." Not a savings or incident-prevention claim. 92 is not a passing count.

Can tell: 100/120 fresh evaluation result; 20 blind spot; 8 excepted reported separately, not counted as enforced; "The control blocked 6 create requests in the metric window", separate from object coverage.
Cannot claim: compliance %, estate-wide assertion, exception validity/currency (expired exceptions no longer applied), cause of the 20, savings/prevention from the 6, freshness of the 100.
Provisional conclusion + easiest way wrong; counterexample (even if all 92 pass, "100% compliant" wrong). Falsifier stated.
Unknowns with settling items: as_of/window; per-result breakdown for the 92; exception list with expiresAt vs as_of; root cause of the 20; retention; blocked-create window and labels matched to Events.
Suggested action vs authority: request items, recompute; route the 20 to policy-review or reliability; exceptions to exception-governance; live change to kubernetes-live-admission-policy-guard-agent (not dispatched); no approval or attestation.
Evidence owner: assurance analyst.
Files read: agent file; SKILL.md; evidence-vocabulary.md; population-and-coverage.md; report-and-event-streams.md
