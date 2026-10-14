# Orchestration

## Parent critical path

Verify harness APIs and reconcile the manifest/security contract before coding.

## Packets

- `01-contract-review`: read-only contract review agent.
- `02-integration-map`: read-only integration discovery agent.
- Parent: documentation verification, decisions, any implementation, final checks.

## Delegation

Native collaboration tools are available. No custom agent selector is exposed.
Use inherited native Codex workers, not an unverified Haiku/Sonnet substitution.

## Agents

Two native agents; requested model and effort: inherited. Actual model and effort:
UNVERIFIED. Agents are behaviorally read-only, not runtime-isolated read-only.

## Delegation limits

At most three open workers, five cumulative agent turns, one implementation wave,
one independent review wave, one bounded repair, and 60 minutes before replanning.
Initial reviews consume two turns. Token usage is unknown. No recursive fan-out.

## Wait points

Wait only when API evidence or a contract decision blocks implementation.

## Fallback

No hidden CLI runners, profile changes, paid providers, or escalated delegation.
If a native route fails, report it and keep safe parent-side investigation.

## Verification order

Primary documentation; contract review; focused runtime tests; generator drift;
repository validation; spelling and Markdown; diff and task-status review.

## Executed continuation

Four native Codex workers: runtime_tests, property_tests, docs_integration,
final_review. The last worker received one explicitly approved follow-up for
the anchored repair: five resumed agent turns total. Actual models/efforts and
tokens are unverified. No recursive delegates. Parent rejected raw-first redaction
and kept context-first matching with final standalone checks. Commit is authorized;
push and live installations are not.
