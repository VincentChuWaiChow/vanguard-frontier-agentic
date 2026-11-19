# Task 2.4: approved additive bounded engine output

## Goal and authority

User explicitly approved one no-install engine-output wave: real byte-chunk
readers, shared8MiB raw budget, protocol-before-display and reliable terminal
status with TDD. Full dependencies remain unchanged. No installs, commits or
pushes. Prior blocked/scope audits are historical, not a denial of this new wave.

Parent owns contract/integration/evidence. One Rust execution specialist owns
subprocess/bounded.rs, additive executor.rs/mod.rs wiring, tests/output_budget.rs
and test helper output modes. Existing legacy capture and all prior preparation
must remain byte-identical, including every existing helper mode.

Forbidden: dependency/lock/catalog/frozen-spec changes, native backends, UI,
redaction/trust/journal admission, shell strings, unsafe/global env mutation,
changing old assertions or marking full tasks complete. Stop on a need for any.

## Observable success defined before implementation

| Criterion | Grader / expected signal | Owner |
| --- | --- | --- |
| TDD RED | Tests written first; missing API compile failure alone is insufficient: execute intended failure against a minimal unavailable API stub before actual implementation | Writer / parent |
| Real I/O |64MiB unterminated dual-pipe helper completes with both streams accounted; fixed-size byte reads, no lines/whole-output collection | Writer / parent |
| One raw budget | Limit exactly8388608; conservative fixed reader/frame/render/parser reservations plus actual RAII-charged payload capacities never exceed it. Both streams have capacity; held consumer tails remain charged | Writer / parent |
| Display pressure | Slow/closed consumers never stop draining; bounded message slots, UTF8-boundary truncation, explicit dropped-byte/record and truncated-record counters | Writer / parent |
| Protocol correctness | Reuse existing strict ExportProtocol on raw stdout, never route raw protocol to lossy display. Validated effects survive overflow/failure; failure latches reliable stop/status and requests root stop, never claims tree cleanup | Writer / parent |
| Lifecycle | Supervisor owns root and both pipe readers through observed completion; dropped wait future/closed display cannot detach readers or lose terminal result. Repeated waits observe one result outside lossy queues | Writer / parent |
| Regression | Both full Cargo suites, focused contracts, strict clippy, both crate/helper formats, boundary/spec/integrity/packaging-scope checks | Parent |
| Independent challenge | One reused Rust reviewer, one bounded integrated repair maximum; parent reruns graders after repair | Reviewer / parent |

One8MiB raw-output budget is not process RSS or a bound on arbitrary copies made
by outside callers. Charge conservative fixed-capacity reader/line/render buffers,
a conservative2MiB bounded parser frame/token/error-scratch reservation, and actual owned display payload
capacity. Application expected-plan/validated effect models are separate semantic
state, not raw transport. No uncharged generic callback may retain raw bytes.
Budget snapshots must state reservation accounting honestly. Output records are
not Clone; retained records keep their payload permits until drop. Queue message
counts are bounded even for empty records. No unused-primitives-only acceptance.

Use a concrete fresh export-plan configuration to construct the existing parser
before launch; reject invalid plan without starting a child. Protocol stdout is
engine-only, not display. Text/stderr records remain internal unredacted transport
pending task2.5; not safe UI/audit projections. Direct-root stop/reap is not Unix
group/Windows Job ownership; descendant cleanup and native acceptance remain
unverified and must be explicit in completion metadata. No full export admission.

## Team and integration

Parent owns immediate critical path, API contract and all independent grading.
One rust_beast_mode writer, configured gpt-6.1-sol/medium/workspace-write, handles
the tightly coupled transport/tests; no competing writers. Reused rust_reviewer
configured gpt-6.1-sol/high/read-only challenges afterward. Effective reviewer
sandbox is inherited workspace-write; read-only assignment is behavioral. Serving
identities/usage unverified. This is the smallest capable plan/execute/review route.
One implementation wave, one review, at most one integrated repair; no fan-out.

Primary Context7 Tokio docs confirm JoinHandle drop detaches rather than joins,
bounded try_send Full/Closed behavior and cancellation-safe byte-read/wait APIs.
Verify local locked APIs, not only documentation. Save raw evidence under ignored
tools/vfa-tui/target/output-budget-wave/. Full2.4/3.1 remain unchecked.

Status: contract defined before source implementation.


## Implemented surface and observed results

The additive SubprocessExecutor::spawn_bounded path owns one supervisor, both
4KiB byte readers, root wait, a stop channel and a retained supervisor JoinHandle.
Cancelling handle.wait does not move/drop that handle; repeated successful waits
return the same Arc completion. No per-pipe detached reader tasks. Reliable
terminal delivery is separate from both lossy display channels. Direct-root stop
is explicitly not descendant cleanup; every completion says so.

OutputProtocol::Export constructs the existing fresh strict parser before launch;
invalid plans never launch. Raw stdout feeds only that parser, not display.
Validated effects survive later frame/wire/schema failure. Text/stderr payloads
remain internal UNREDACTED transport, not trusted UI/audit projections.

The capacity ledger reserves24KiB for two read/prefix/construction paths and an
additional2MiB in export mode for bounded record/frame/token/error scratch. Each
4KiB owned record acquires a per-stream permit before allocation; field drop order
frees its box before releasing the permit. A held consumer tail remains charged.
The remaining payload quota is split between streams;1024 message slots per
stream bound empty records too. This is conservative reserved capacity, not an
allocator/RSS measurement or a bound on semantic plan/effect state or arbitrary
external copies. No raw callback can hide retained transport memory.

| Criterion | Status | Evidence |
| --- | --- | --- |
| Initial runtime RED/GREEN | PASS | Tests written before API; missing-API compile failure recorded separately. Runtime stub produced8 intended failures, exit101, before implementation; initial9 contracts then passed |
| Real pressure and ownership | PASS Linux | Final10 output contracts each mode:64MiB on EACH pipe, unterminated/empty/many records, closed/slow consumers, retained permits, UTF8 boundary/invalid bytes, cancelled wait, root stop/timeout/nonzero/repeated completion |
| Protocol/effects | PASS scoped | Valid raw protocol despite closed display;64KiB frame overflow and8MiB wire overflow retain earlier effects, including127 validated effects before total-wire failure; hostile escaped unknown fields reject without raw display |
| Independent review and repair | PASS scoped after repair | One Rust source review found overlapping truncation/drop counters. New closed-channel64MiB regression failed with67104768 truncated bytes instead of0; surgical condition fix then passed. Delivered-long-record assertions preserved. Parent independently reran all full graders; no second review claimed |
| Full default regression | PASS |1133 passed,0 failed,1 pre-existing ignored MockRunner doctest across14 targets; normal parallel harness |
| Full engine-only regression | PASS |912 passed,0 failed,1 same pre-existing ignored doctest across13 targets; normal parallel harness |
| Strict lint/format | PASS | Both all-target clippy modes with -D warnings, both crate formats, explicit helper format and standalone warning-clean helper metadata compile |
| Dependency boundaries | PASS graph only | TUI checker plus31 negative self-tests and normal engine subtree under Desktop lock pass; not Desktop behavioral execution |
| Packaging | PASS | Ordinary locked/offline package with normal verification, then packaged no-default library check; all96 Rust source files match current bytes, manifest original matches, test helper is excluded |
| Source preservation | PASS | Only admitted11-path wave; legacy executor/mod/helper exact after removing additive regions. Prior parser/session work, all other baseline files, manifests/locks and frozen inputs preserved; index empty |
| Spec/integrity/graph | PASS scoped | Task checker/self-tests, unchanged asset integrity and AST graph refresh; not native/visual/semantic proof |
| Root npm in checkout | MISS environment | Scanner recursively visits ignored Cargo cache and flags vendored zruntime workflow; no scanner change or source waiver |
| Root npm intended-source snapshot | PASS with context | Exact tracked+intended untracked source snapshot excludes ignored caches. First restricted run failed32 CLI checks with empty output; a direct probe reproduced Node spawnSync EPERM. Tool-approved outside-sandbox probe/rerun passed all gates. Separate original-checkout exporter control subsequently passed; transient restriction stability is not claimed fixed |
| Coverage/native/full tasks | UNVERIFIED | No measured coverage threshold, GPUI foreground/native Desktop, cross-platform descendant ownership or full dependency completion; tasks2.4/3.1 remain unchecked |

Commands used existing installed tools and ignored Cargo caches; no installations:

```bash
export CARGO_HOME="$PWD/tools/vfa-tui/target/consumer-cargo-home"
export CARGO_TARGET_DIR="$PWD/tools/vfa-tui/target/consumer-build"
cargo test --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --test output_budget --test process_contracts
cargo test --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --no-default-features --test output_budget --test process_contracts
cargo test --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --no-fail-fast
cargo test --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --no-default-features --no-fail-fast
cargo clippy --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --all-targets -- -D warnings
cargo clippy --manifest-path tools/vfa-tui/Cargo.toml --locked --offline --no-default-features --all-targets -- -D warnings
cargo fmt --manifest-path tools/vfa-tui/Cargo.toml --check
cargo fmt --manifest-path tools/vfa-desktop/Cargo.toml --check
rustfmt --edition 2021 --check tools/vfa-tui/tests/support/process_helper.rs
rustc --edition 2021 -D warnings --crate-name bounded_output_helper tools/vfa-tui/tests/support/process_helper.rs --emit metadata -o tools/vfa-tui/target/output-budget-wave/helper.rmeta
python3 tools/vfa-tui/tests/check_feature_boundary.py --manifest-path tools/vfa-tui/Cargo.toml --self-test
python3 tools/vfa-tui/tests/check_feature_boundary.py --manifest-path tools/vfa-tui/Cargo.toml --consumer-manifest tools/vfa-desktop/Cargo.toml
# Above metadata checks use CARGO_NET_OFFLINE=true.
cargo package --manifest-path "$SOURCE_SNAPSHOT/tools/vfa-tui/Cargo.toml" --locked --offline
cargo check --manifest-path "$CARGO_TARGET_DIR/package/vfa-tui-0.1.0/Cargo.toml" --locked --offline --lib --no-default-features
python3 .kiro/specs/gpui-desktop/verify.py --tasks --self-test
python3 tests/validate-asset-integrity.py
graphify update .
git diff --check
npm run validate
# Checkout MISS preserved. Same command in exact intended-source snapshot passes
# on tool-approved outside-sandbox rerun; no npm install or test weakening.
```

All shell invocations use rtk proxy. Context7 /tokio-rs/tokio plus locked local
Tokio1.52.3 source establish read/Child::wait/watch cancellation semantics and
[JoinHandle ownership](https://github.com/tokio-rs/tokio/blob/master/tokio/src/runtime/task/join.rs).
Raw RED/review-RED/GREEN/full/lint/boundary/package/npm/graph logs, source and
archive hashes, snapshot correspondence and final results are under ignored
tools/vfa-tui/target/output-budget-wave/. Failed attempts are preserved, not relabeled.

Status: READY FOR REVIEW for engine-output preparation after one bounded repair.
One configured Rust writer Sol/medium plus one reused Rust reviewer Sol/high,
parent-owned independent grading; serving identities/usage unverified. Five source
paths were sufficient; no competing writers or speculative native abstractions.
No installation, staging, commit, push or full-task/dependency completion.
