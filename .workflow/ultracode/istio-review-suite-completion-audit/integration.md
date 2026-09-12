# Integration

## Accepted
- Both reproduced safety findings.
- Requirement-traceability and release evidence gathered by parent fallback after delegated agents exhausted workspace credits.
- Local safety corrections, regression fixtures/tests, and regenerated asset integrity.

## Rejected
- Treating structural/helper tests as actual LLM evaluations.
- Treating authored runtime scenario files as executed live acceptance.
- Treating a matching approval-binding tuple as execution authorization.

## Conflicts
Repository policy says finish means commit and push; the explicitly invoked Ultracode workflow forbids commit/push without an explicit user request. The stricter publication gate wins, so the verified corrective diff remains local pending permission.

## Decisions
- The deterministic live gate should fail safe for the six reproduced phrasing classes.
- Preflight must cover every resource on which the manifest grants write verbs.
- The workflow audit can finish while promotion readiness remains blocked.

## Final changes
- Expanded Istio live-intent routing and added six input/expected regression pairs.
- Completed RBAC negative and positive preflight coverage for five writable Istio resources.
- Added a parity regression test and regenerated asset integrity.

## Verification still needed
- Actual LLM behavior runs for all 55 specifications under a pinned model/harness matrix.
- Seven version-pinned live semantic acceptance runs in an independently authorized disposable environment.
- Commit/push verification if explicitly approved.

## Remaining risks
Regex routing is not a proof of open-ended language coverage. Bundle authorship is asserted but upstream comparison commits were unresolved.
