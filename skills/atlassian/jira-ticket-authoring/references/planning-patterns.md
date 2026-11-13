# Planning and Confluence handoff

These patterns are engineering conventions for this skill. Adapt their depth to the work.

## Choose the smallest useful pattern

- Use implementation for one known behavior, including a refactor that preserves an explicit invariant.
- Use bug when incorrect behavior is reported. Missing cause is normal; missing reproduction/evidence may make investigation the next step.
- Use spike when an unknown must be resolved before a responsible implementation choice. Completion is an evidence-supported answer, including an inconclusive answer with its limitations.
- Use epic when separate outcomes need a delivery sequence. Decompose by observable result, not merely “frontend / backend / tests,” unless those are independently usable contracts with integration checks.
- Use operational change when execution can affect deployed systems or persistent data. Scope the environment and recoverability explicitly.
- Use finding for work discovered outside the active assignment. Record it without silently expanding the assignment.

## Decomposition contract

For each proposed child, record:

| Local reference | Outcome | AC boundary | Depends on | Shared interface or data contract | Integration check |
| --- | --- | --- | --- | --- | --- |
| CHILD-A | One independently verifiable outcome | Criterion IDs owned by this child | Local refs or verified keys | Owner and contract version, if relevant | How it composes with its consumers |

Check dependencies for cycles. If several children modify the same interface, agree on an interface contract and ordering before describing them as parallel work. Keep an epic-level integration outcome so locally passing children cannot hide an end-to-end failure.

Do not fabricate Jira keys for proposed work. Replace local references with returned keys after successful creation. Creating parents and children can partly succeed; retain the mapping and repair only the missing operations.

## Optional modules

Add a module only when the request or inspected evidence makes it relevant.

| Work | Useful additional content |
| --- | --- |
| Refactor | Current externally observable behavior, invariant, compatibility check, deliberate exclusions |
| API/interface | Request/response contract, version compatibility, error behavior, retries/idempotency, consumer impact |
| Data/schema change | Source schema, ownership, compatibility, migration/backfill, integrity checks, recovery limits |
| Performance | Measured baseline, workload, measurement method, approved/proposed target, resource/cost tradeoff |
| UI/accessibility | Target user journey, specified accessibility needs, error/empty/loading behavior, visual evidence |
| Security-sensitive change | Threat/permission boundary, least-privilege behavior, negative tests, treatment of secrets |
| Operations | Preconditions, target environment, rollout steps, success/abort signals, rollback or forward recovery |

Never invent latency budgets, cost savings, severity, compliance obligations, or an easy rollback for an irreversible migration. Mark proposed values as proposals.

## Jira and Confluence division

| Artifact | Owns | Jira ticket carries |
| --- | --- | --- |
| TDD | Design, interfaces, alternatives, risks, delivery approach | Exact behavior relevant to the ticket plus verified TDD reference |
| ADR | One consequential decision and its rationale | Applicable decision and status; implementation dependency if still proposed |
| API documentation | Versioned interface contract | Contract version and changed behavior |
| Runbook/playbook | Maintained operational procedure | Change to the procedure or specific operational work and verification |
| Release notes | Evidence of delivered changes and upgrade implications | A verified link when available; issue closure alone is not release evidence |
| Meeting/retrospective notes | Recorded discussion, decisions, and actions | Assigned work only when actually agreed; unresolved owners remain unassigned |

Use traceability IDs when they help: source requirement `REQ-01` → design section/ADR → ticket `AC-01` → verification evidence. The identifiers are a local convention; they do not create native Jira links.

When sources conflict, show both versions and the affected requirement. Ask the appropriate decision owner if the conflict blocks implementation. Do not silently decide that the newest comment overrides an accepted design.

For reciprocal links, use verified URLs and IDs. If the page or ticket does not yet exist, leave an explicit pending reference. Add the second side only after the first object is successfully created and its returned identifier is known. Preserve access restrictions and do not copy restricted material into a broader destination.
