# Task 5.2 Rust consumer preparation: eval contract

## Goal and boundary

Current user request admits a new installation-free parser slice of task 5.2.
Use `tasks.md` (the supplied `tasks.mda` does not exist). Full task dependencies
and unchecked status remain unchanged. Risk: shared engine data contract, no
external effects. Complexity A after producer/design inspection: exact wire
contract, additive isolated module, deterministic tests.

Parent owns integration, implementation and verification. One Rust reviewer
performs bounded discovery and one review follow-up; maximum one repair pass.
No dependency/lock, exporter, launcher, UI, journal, catalog, runtime configuration,
installation, commit, push, remote CI or operator-target changes are authorized.

## Surface and downstream consumers

Producer: `scripts/export-marketplace-agents.mjs` lines 1015-1139, unchanged.
Consumer: new `engine::export_protocol`, supplied an expected plan projection.
Future operation controller supplies raw stdout and actual child exit status.
This is validation, not authorization, filesystem verification or persistence.

The parser accepts byte chunks, retains verified file outcomes and a validated
summary, and fails closed with fixed diagnostics. It requires strict fields,
version/digest/sequence/order, exact planned path/hash/size/mode, consistent
summary arithmetic and newline framing. Maximum record including LF is 65536
bytes; cumulative wire bytes are at most 8388608. Errors remain latched and
cannot erase preceding effects or be followed by apparent success. EOF without
a summary, unterminated bytes, nonzero/unknown exit and FAILED cannot succeed.
Parser-local limits do not prove the future shared in-flight stdout/stderr budget.

## Graders and pass signals

| Criterion | Grader / expected signal | Failure owner |
| --- | --- | --- |
| RED | New Rust test target fails specifically for missing module before production edits | Parent |
| Capability | `cargo test --manifest-path tools/vfa-tui/Cargo.toml --locked [--no-default-features] --test export_protocol`; positive, hostile, boundary and actual Node fixture tests pass | Parent |
| Regression | Existing core/report/runtime/process/journal tests, both full library suites compared to baseline; no new failures | Parent |
| Quality | Cargo fmt, strict clippy in both feature modes; unchanged manifests/locks/producer | Parent |
| Coverage | Use existing coverage tooling if present; otherwise explicitly UNVERIFIED, never install implicitly or invent a percentage | Parent |
| Scope | Spec structural self-tests, integrity verifier and `git diff --check` pass; graph update attempted | Parent |
| Challenge | Independent Rust source review, one bounded repair maximum | Reviewer / parent |

## Deliverables and stop conditions

Only `tools/vfa-tui/src/engine/{mod.rs,export_protocol.rs}`,
`tools/vfa-tui/tests/export_protocol.rs`, existing GPUI spec/eval/workflow evidence
and generated local graph output. Stop for wire-contract ambiguity, required
dependency changes, native prerequisites or an unbounded integration requirement.
TDD checkpoint commits are intentionally not made: current request does not
authorize Git publication/checkpoints. Preserve RED/GREEN in the task report.
