# Task 2.1: finite controlled descendant fixture

## Contract before implementation

Continue the already admitted installation-free task2.1 preparation. Its existing
eval explicitly permits a finite, fully joined self-spawn tree. User declined
installation; native/full dependency gates and conditional Git delivery remain
unchanged. This is fixture capability, not process-backend cleanup acceptance.

Owned source paths: tools/vfa-tui/tests/support/process_helper.rs and
tools/vfa-tui/tests/process_contracts.rs. Parent owns integration and evidence.
Forbidden: production source, dependencies, manifests/locks, catalogs, frozen
specs, orphan/root-first modes, native ownership, installs and Git publication.
No unsafe code, shell strings or process-global environment mutation.

Observable success:

| Criterion | Grader and expected signal | Owner |
| --- | --- | --- |
| RED | New finite-tree contract actually fails with unknown helper mode before implementation | Parent |
| GREEN | Same contract observes depths0/2/3, success/nonzero exits, distinct native PIDs, correct parent-child links and one waited JOINED per edge before parent exit | Parent |
| Input rejection | Bad/negative/excessive depth, invalid exit and extra args produce exit64 without any tree-start output | Parent |
| Existing contracts | All process contracts pass in default/no-default modes; both strict clippy/fmt gates pass | Parent |
| Scope | Only helper/test plus existing spec/workflow evidence changes; current production/earlier preparation bytes preserved | Parent |
| Independent review | One reused Rust reviewer checks bounds, join ordering and oracle independence; at most one repair | Reviewer |

Implementation shape: synchronous std-only chain with at most three descendant
edges. Each process uses current_exe, native Command argv and stdin-null; every
parent retains its Child and waits before recording JOINED and its own exit.
Tree output is finite fixture output, not a new stream-budget implementation.
The test driver bounds observation and explicitly kills/waits its direct child
on failure; no OS ownership claim follows from dropping a future or root exit.

Risk: local test fixture only. Parent implementation plus one existing Rust
reviewer is the smallest capable route; no writer fan-out. Configured profile
gpt-6.1-sol/high/read-only, effective inherited sandbox not separate isolation;
serving identity unknown. Context7 standard-library docs confirm Child has no
implicit Drop wait and Child::wait observes termination. Only Linux execution is
available; Windows/macOS/Desktop and full tasks2.1/3.1 remain unverified.

Status: contract defined before editing helper/test implementation.


## Executed results

| Criterion | Status | Evidence |
| --- | --- | --- |
| Intended runtime RED | PASS | Positive tree contract actually failed with helper exit64/unknown mode instead of0 before helper change. Input-rejection case already passed; no claim both were RED |
| Real finite fixtures | PASS Linux | Eight process contracts in each feature mode; six valid tree scenarios (depths0/2/3, exit0/37), ten invalid input classes, real native PID links and joined exit ordering |
| Full default regression | PASS |1123 passed,0 failed,1 pre-existing ignored MockRunner doctest across13 targets; ordinary parallel harness |
| Full engine-only regression | PASS |902 passed,0 failed,1 same pre-existing ignored doctest across12 targets; ordinary parallel harness |
| Strict lint/format | PASS | Both all-target clippy modes with -D warnings, both crate fmt checks, explicit helper rustfmt and standalone rustc -D warnings metadata compilation exit0 |
| Spec/integrity/graph | PASS scoped | Task checker/self-tests, unchanged asset integrity and AST graph update; no semantic/native/visual inference |
| Scope/package target | PASS | Before-hash preservation of all10186 existing tracked/untracked files outside the admitted helper/test/evidence set. No helper Cargo target; manifests/locks and every production byte unchanged |
| Independent source review | PASS | Reused configured Rust reviewer gpt-6.1-sol/high found no blockers; parent executed graders. Serving identity unverified; no production repair |
| Full/native/closed-output failure acceptance | UNVERIFIED | No Unix ownership, signals, Windows Jobs, native Desktop, disconnected-output fault injection or full task dependency completion |

Exact shell commands ran through rtk proxy, using existing ignored
CARGO_HOME=tools/vfa-tui/target/consumer-cargo-home and
CARGO_TARGET_DIR=tools/vfa-tui/target/consumer-build resolved from checkout root:

```bash
cargo test --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --no-default-features --test process_contracts controlled_tree
# The preceding command was the actual pre-helper-edit RED attempt.
cargo test --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --test process_contracts
cargo test --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --no-default-features --test process_contracts
cargo test --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --no-fail-fast
cargo test --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --no-default-features --no-fail-fast
cargo clippy --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --all-targets -- -D warnings
cargo clippy --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --no-default-features --all-targets -- -D warnings
cargo fmt --manifest-path tools/vfa-tui/Cargo.toml --check
cargo fmt --manifest-path tools/vfa-desktop/Cargo.toml --check
rustfmt --edition 2021 --check tools/vfa-tui/tests/support/process_helper.rs
rustc --edition 2021 -D warnings --crate-name finite_tree_helper tools/vfa-tui/tests/support/process_helper.rs --emit metadata -o tools/vfa-tui/target/finite-tree-wave/helper.rmeta
cargo metadata --manifest-path tools/vfa-tui/Cargo.toml --format-version 1 --no-deps --locked --offline
python3 .kiro/specs/gpui-desktop/verify.py --tasks --self-test
python3 tests/validate-asset-integrity.py
graphify update .
git diff --check
```

Context7 /websites/doc_rust-lang_stable_std retrieved the official
[Child wait/Drop contract](https://doc.rust-lang.org/stable/std/process/struct.Child.html).
Raw RED/GREEN/full/lint/structural logs, before/source hashes and results are in
ignored tools/vfa-tui/target/finite-tree-wave/. Historical compatibility results
remain point-in-time evidence, not replaced or falsely relabeled. Coverage and
other native targets remain unverified. No installs, staging, commit or push.

Status: READY FOR REVIEW for the reviewed finite fixture only. Full task2.1 and
3.1 remain unchecked; no dependency or publication condition has been weakened.
Parent plus one reused reviewer was sufficient: two test-only source files,
no writer fan-out or speculative production abstractions. Token usage unknown.
