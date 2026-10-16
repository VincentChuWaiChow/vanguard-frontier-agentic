# Packet 01-contract-review: contract contradictions

## Objective

Identify implementation-blocking contradictions in the requested spec.

## Context

Security-sensitive cross-harness hook implementation. Parent verifies external APIs.

## Sources

`.kiro/specs/cross-harness-mods/{requirements,design,tasks}.md`.

## Ownership

Read-only, no writes. Return findings to parent for workflow recording.

## Do

Review reach semantics, environment scanning, shell/JSON handling, audit sequencing,
redaction guarantees, and property-test validity. Cite exact file lines.

## Do not

Edit, commit, push, run the full suite, spawn more agents, or duplicate API research.

## Expected output

Ranked blockers, minimal defensible resolution, and any safe independent subset.

## Verification

Cross-check requirements against tasks/design; separate facts and inference.

## Handoff format

Concise summary, file:line evidence, recommended decisions, risks.
