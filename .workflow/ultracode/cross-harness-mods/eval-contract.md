# Eval contract

## Goal

Ship real, testable native mods rather than artifacts validated against invented APIs.

## Success criteria

1. Every supported harness/event/decision has current official evidence.
2. Unsupported behavior is explicitly rejected, not silently downgraded.
3. Reach declarations match actual execution, including helper processes.
4. Runtime tests exercise exported production logic or generated adapters, not copies.
5. Real audit appends preserve JSON validity and per-session sequence under the
   agreed concurrency contract; no unsupported immutability claim.
6. Redaction preserves JSON structure and states interception limitations.

## Integration surfaces

`mod.toml` to generator to adapters/catalog; Python validator to generated artifacts;
package validation chain; catalog index; integrity inventory; installation docs.

## Downstream consumers

Claude Code, Kiro, Codex, CI gates, mod installers, and human security reviewers.

## Required checks

- Official API evidence before adapter implementation.
- Positive and negative runtime probes, including malformed input and write failure.
- Deterministic generation and read-only drift detection.
- Existing and new property tests using production code.
- `npm run validate`, `npm run lint:spell`, repository Markdown lint.

## Deliverables

Spec-defined implementation when its contract is verified; otherwise a precise
blocking report and a clarification request, with no fabricated completions.

## Blocking conditions

Unverified SDK/protocol, conflicting acceptance criteria, approval-dependent scope
change, unsafe secret exposure, or tests unable to support the claimed behavior.
