# Result 02: Security, routing, and live-guard refutation

## Summary
MISS found and corrected locally: six common live-mutation phrasings bypassed the live gate, and RBAC preflight coverage omitted delete-denial checks for three writable Istio resources.

## Evidence
- Before correction, resource-before-verb, `set`, `kubectl apply -f`, multiline, plural/API-qualified, and Gateway mutations routed to a static reviewer or remained unclassified.
- The taxonomy now recognizes those forms only with an explicit live-target marker and retains the existing negative static-review fixture.
- Six native routing regression pairs now require `kubernetes-live-mesh-policy-guard-agent` in `live-guard-gate` mode.
- The RBAC manifest deliberately grants create/patch but not delete on five policy/traffic kinds. Both the documented preflight and executable guard now test delete denial and create/patch presence for all five.
- A native contract regression test enforces parity between the documented and executable checks.

## Handoff
Handoff:
- Summary: Reproduced and locally fixed both safety misses.
- Changed surfaces: Istio routing taxonomy/fixtures, mesh-policy RBAC preflight docs/script, native contract tests, asset integrity.
- Contracts satisfied: Fail-closed live handoff and preflight parity for every writable Istio kind.
- Assumptions: The deterministic router is a safety gate, not proof of model intent classification.
- Local checks: 840 routing scenarios; 83 Istio tests; full validation; spell and Markdown lint.
- Integration evidence: New fixtures reproduce every reported bypass.
- Risks: Natural-language intent remains open-ended; actual model/adversarial evaluation is still required.

## Files changed
Parent-owned corrective diff only; the delegated reviewer remained read-only.

## Decisions
Keep the local binding helper's `delete` vocabulary unchanged: it reports consistency only and explicitly returns `execution_authorization: not-granted`; RBAC and the live guard remain the enforcement boundary.

## Risks
The gate is regex-based. Passing the committed counterexamples does not establish exhaustive semantic coverage.

## Verification run
Direct evaluator reproduction before and after, `validate:maestro-routing`, `validate:istio-review-suite`, and full `npm run validate`.

## Open questions
Actual model behavior on the 55 adversarial specifications remains unverified.
