# Task 2.1: installation-free process-contract preparation

## Goal and authorization

User requested next implementable tasks using Ultracode and Codex Agent Teams.
The full DAG has only native task 1.4 ready, with host prerequisites still absent.
Explicitly admit this additive engine/helper preparation based on verified engine
boundary/runtime portions; do not bypass full dependencies or mark 2.1 complete.
No installs, dependency/compiler changes, native/Desktop acceptance, push, global
settings, real operator writes or OS descendant-ownership implementation.

## Owned integration and consumers

One worker owns subprocess/program.rs, additive executor.rs wiring and
 tests/support/process_helper.rs plus tests/process_contracts.rs under tools/vfa-tui.
Parent owns subprocess/mod.rs, graders, records, package checks and local Git.
Existing executor/handle behavior and secret filtering remain compatible. Future
native backends/trust resolution consume ProgramSpec, not shell strings.

## Capability checks defined before work

- Validated resolved executable/cwd plus Vec<OsString> arguments and explicitly
  selected environment build a literal shell-free launch. Reuse existing secret
  key filtering; no implicit ambient inheritance. Invalid paths/NULs/environment
  names and denied secret keys fail explicitly. Native paths are not identifiers.
- Keep native values owned; JSON conversion rejects unrepresentable native
  strings, never lossy conversion. No command/version probe at construction.
- Additive spawn_spec executes the validated request with existing owned child
  handle/capture. Preserve legacy spawn behavior; unchanged line readers/queues
  are NOT D7 output-budget or descendant-cleanup acceptance.
- A standalone std-only helper compiles with installed Rust into test-owned
  temporary storage; no publish-invalid Cargo target referencing excluded tests.
  Finite modes report argv/cwd/selected env, stdout/stderr, exit codes and a
  stdin-gated exit. Any self-spawn tree is finite and fully joined; no orphan mode.
- Real tests prove spaces/Unicode, injection-like argv stays literal and creates
  no marker, selected cwd/env, denied ambient variables, nonzero exit, both streams
  and actual stdin-gated delay with channels/observations rather than sleep proof.
- Negative cases cover missing/non-file executable, invalid cwd/NUL/env and
  nonrepresentable JSON paths/arguments. No unsafe/global environment mutation,
  broad panic recovery, test-only production backdoors or changed fixed oracles.

## Regression and stop conditions

Parent serializes both-mode process/core/report/profile/journal/runtime tests,
strict all-target clippy and both crate formats. Full suites may fail only for
exact known 17/11 identities and solely --lib. Normal cargo package verification,
archive/source bytes, packaged no-default library and both locked normal engine
subtrees must pass without dirty/no-verify bypass. Preserve pins/frozen approvals.

One implementation wave, independent Rust source review and one parent-integrated
repair pass for this two-track wave. Stop if descendant semantics, stream budgets,
new dependencies or native libraries become necessary. Return exact changes/tests
and uncertainty. Per-task local no-reply commits only; no push.

Initial status: checks defined before implementation; full task remains pending.

### Artifact and source constraints

The helper compiler's temporary output must use a unique owned directory under
the discovered ignored Cargo target, not the nearly-full host temporary tmpfs.
Other full-suite fixture temporaries stay outside the checkout to preserve
workspace-detection negatives. Diagnostics omit argv/environment values; explicit
JSON conversion is internal, not a log sink. The typed launcher defaults stdin
to null while preserving legacy stdin behavior.

Primary [Rust process documentation](https://doc.rust-lang.org/std/process/)
describes implicit Windows batch-file handling; this slice restricts typed Windows
launch to resolved .exe files rather than claiming any Command input is shell-free.
Context7 /tokio-rs/tokio confirmed native OsStr args, env_clear/stdin builders and
child-only kill_on_drop semantics. Native Windows execution remains UNVERIFIED.

## Scoped implementation results

| Criterion | Status | Evidence |
| --- | --- | --- |
| Typed native launch | PASS Linux | Six real process_contracts tests per feature mode: literal argv/Unicode/cwd, real ambient PATH excluded, selected env retained, finite streams/nonzero exit and stdin-gated actual exit |
| Invalid inputs/private data | PASS | Missing/nonfile/invalid cwd, NUL/env keys, secret-key rejection, Debug omission, native non-Unicode preservation and explicit JSON failure; no process-global env mutation |
| Legacy/runtime contracts | PASS | Three existing subprocess tests and ten runtime unit tests per mode; all core/report/profile/journal/bridge targeted contracts pass |
| Format and strict clippy | PASS | Both crate formats and both all-target -D warnings clippy modes exit 0 |
| Normal package | PASS | Tracked-source snapshot plus exact three production overlays passes ordinary cargo package --locked --offline, exact archived bytes and packaged no-default library check; no test-helper Cargo target |
| Independent source review | PASS scoped | Configured Rust reviewer found no production blocker and confirmed original spawn/capture body byte-identical; parent completed/verified the noted helper target-storage allocation |
| Full Rust regressions | MISS existing baseline | Both full --no-fail-fast commands exit 101 for exactly known 17/11 identities, with only --lib failing; no new failure waived |
| Both normal engine graphs | PASS metadata only | Actual locked TUI/Desktop subtrees and 31 feature-boundary negative fixtures; no Desktop behavioral parity claim |
| Native/D7/trust/full task | UNVERIFIED | Windows/macOS, descendant ownership, safe reader lifetime/output budget, execution grants and full2.1 acceptance remain deferred |

One reused built-in writer (serving model/effort unobservable) owned four files;
parent owns module export, helper artifact allocation, actual serialized Cargo and
Git. Configured Rust reviewer is gpt-6.1-sol/high with read-only default; effective
isolation/serving identity are unobservable. No helper/runtime resolver or full
ProcessBackend state forest was invented. Existing capture is still legacy
display-only behavior, not a reliable protocol transport for the new Node producer.

### Exact parent commands and source evidence

```bash
manifest=tools/vfa-tui/Cargo.toml
# Repeat test/clippy/full commands with --no-default-features.
cargo +stable test --manifest-path "$manifest" --locked --offline --test process_contracts --test runtime_bridge --test core_contracts --test report_service_contracts --test profile_ownership --test journal_commits
cargo +stable test --manifest-path "$manifest" --locked --offline --lib subprocess::
cargo +stable test --manifest-path "$manifest" --locked --offline --lib engine::runtime::tests
cargo +stable clippy --manifest-path "$manifest" --locked --offline --all-targets -- -D warnings
cargo +stable test --manifest-path "$manifest" --locked --offline --no-fail-fast
cargo +stable fmt --manifest-path "$manifest" --check
cargo +stable fmt --manifest-path tools/vfa-desktop/Cargo.toml --check
cargo +stable package --manifest-path "$snapshot_manifest" --locked --offline
cargo +stable check --manifest-path "$unpacked_manifest" --locked --offline --lib --no-default-features
python3 tools/vfa-tui/tests/check_feature_boundary.py --manifest-path "$manifest" --self-test
python3 tools/vfa-tui/tests/check_feature_boundary.py --manifest-path "$manifest" --consumer-manifest tools/vfa-desktop/Cargo.toml
```

Helper compilation uses the installed Rust compiler's explicit absolute path.
No rustup/toolchain/package install is attempted. Parent independently verified
the old spawn/capture body remains byte-identical. Package archive bytes match
the three current production modules; the helper stays test-only and no new
manifest target references excluded tests. Both manifests/lockfiles stay unchanged.

Raw evidence in the discovered ignored target wave directory: rust-pre-results.json,
rust-full-early.json, per-mode process/legacy/runtime/contracts/clippy/full logs,
package-results.json, process-source-hashes.json and boundary-*.log.

Verified crate SHA-256: `e7d904a45d8cd0e8906b90f76ee9d2cd157d56488ad166b2881eff1bb8185fd4`.
