# Orchestration

## Parent critical path

Baseline/dependency readiness; models + secure loader/store + generator;
integrate UI/P3; all gates, independent review, task status and Git closeout.

## Packets

01-ui: app/nav/detail and registered render/reload integration tests.
02-p3-docs: fuzz test and mod README reach declarations.
03-review: read-only integrated refutation, dispatched after workers finish.

## Delegation

Two write workers with disjoint owned files; one integrated read-only reviewer.
Three cumulative native runs used. The parent performed the one bounded repair
after review found a busy-write navigation regression; no further wave needed.

## Agents

Available Codex `gpt-6.1-sol`, medium requested. Effective model/effort/isolation
beyond native tool evidence remain unverified; no isolation-dependent operation.

## Delegation limits

At most four cumulative runs (two implementation, one review, one bounded repair
follow-up), at most two active workers. No recursive fanout. Reassess at 90 minutes
before extending execution; do not duplicate a still-live process.

## Wait points

Rust tests are serialized by parent after baseline and foundations. Full fuzz
waits for settled generated data. Final review waits for integrated code.

## Fallback

If a worker is unavailable, parent implements its exact bounded ownership.

## Verification order

RED evidence → focused GREEN → integrated Cargo/fuzz → generation/integrity
last → full validate/lints → independent review → task evidence → commit/push.
