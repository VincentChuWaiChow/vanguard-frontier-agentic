# Orchestration

## Parent critical path
Draft the README, integrate review, run gates and publish the approved branch update.

## Packets
- 01-fact-check: independent read-only support/safety review.

## Delegation
One sidecar, no nested delegation. No separate writer or duplicate implementation.

## Agents
Native agent requested as gpt-6.1-sol at medium effort; runtime identity is unverified.

## Delegation limits
One cumulative agent run and one review wave; one bounded correction pass.
Reassess after 30 minutes rather than expanding scope.

## Wait points
Integrate the fact check before final verification and commit.

## Fallback
If delegation is unavailable, parent executes the bounded review and records it.

## Verification order
Review and links; lint; final integrity generation; full validation; commit/push;
independent remote-head verification.
