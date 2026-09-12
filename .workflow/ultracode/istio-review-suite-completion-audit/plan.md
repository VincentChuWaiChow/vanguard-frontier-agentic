# Istio review suite completion audit

## Goal
Independently determine whether commit `d3b0693294fd410f6de9a15efc6c898ac2627228` satisfies the supplied integration contract and is honestly ready for promotion.

## Success criteria
- Map every material bundle requirement to repository evidence or a named gap.
- Adversarially test live-mutation routing and guard boundaries.
- Verify native catalogs, harness projections, exports, generated assets, and relevant gates.
- Report PASS, MISS, UNVERIFIED, and skipped checks without inflating structural tests into model/runtime evidence.

## Current context
The implementation is on `feat/istio-review-suite`, based on `24c409c79b7d9b7298711ba93cfc910988fb1f19`, and currently matches its remote branch.

## Constraints
- Read-only audit apart from local workflow notes under this run root.
- No cluster mutations, deploys, force operations, commits, or pushes.
- Do not treat fixture shape validation as actual LLM or live-runtime evaluation.

## Risk level
High: public catalog surfaces, multi-harness generation, and a Kubernetes mutation guard are involved.

## Approval gates
No approval is needed for local read-only inspection and validation. Any corrective source edit, external publication, or live operation requires a separate decision.

## Mode
Delegated: three independent read-only review packets plus parent integration and verification.

## Work packets
1. Requirement traceability.
2. Security, routing, and live-guard refutation.
3. Packaging, generated artifacts, provenance, and release gates.

## Eval contract
Full contract in `eval-contract.md`.

## Integration policy
Accept only findings backed by exact paths, commands, or reproducible cases. Conflicting findings are resolved by direct parent verification.

## Verification plan
Re-run targeted Istio tests, routing tests, export/asset checks, and the repository validation required by the integration contract when safe.

## Completion criteria
The audit is complete when all packets are reconciled and every required check is classified with evidence. Promotion readiness may still be blocked by authorized-but-unavailable live or model evaluations.
