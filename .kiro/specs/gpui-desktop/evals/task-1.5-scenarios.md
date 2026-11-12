# Task 1.5 follow-up offline scenarios

Defined before test edits for the user request: run next scenarios.

## Scope and checks

Test-only work: add unit scenarios only inside the existing runtime test module.
One worker owns those tests; parent owns actual serialized Cargo, records and Git.
No production/API, dependency/compiler, fixed fixture or frozen approval changes.
No installs, native claims, remote CI, push or extra task completion.

- Poison the actual registry mutex in a joined controlled test thread with one
  held active probe and one queued probe. Cancel cannot acknowledge prematurely.
  Active work retains its actual result plus cancellation; queued work cannot
  begin. After release, shutdown ticket and host join return StatePoisoned.
- Race cloned-client admission against global Cancel while active work is held.
  Allowed admission outcomes: owned ticket or Busy/ShuttingDown/Stopped. Retained
  IDs remain unique/bounded; accepted queued work is CancelledBeforeStart and
  post-Cancel-return admission rejects. Join all threads; use gates, not sleeps.
- Run existing exporter and static workflow scenarios against temporary fixtures.
- Run all five fixed core cases under both TUI feature modes in a copied snapshot
  with spaces/Unicode, explicit manifest and unrelated CWD. Not Desktop parity.
- Existing runtime/report/core/profile/journal contracts, formats and strict
  all-target clippy pass. Full regression failure identities remain exactly the
  known 17/11 baseline. Production prefix, manifests/locks and fixtures unchanged.

One implementation/review wave and one bounded repair; preserve RED before any
out-of-scope production fix. Report exact commands/results and uncertainty.
Native/foreground/Desktop behavior, old MSRV and other OS remain UNVERIFIED.
Standing per-task commits use both GitHub no-reply identities. No push.

Initial status before execution: contract defined; execution pending.

## First executed results

- PASS: node --test tests/export-plan.test.mjs: 14 real CLI fixture tests, zero skips.
- PASS: node --test tests/gpui-ci-contracts.test.mjs: 27 actual-workflow checks, zero skips.
- PASS: node tests/export-cursor-kiro-skill-notice.test.mjs: Cursor, Kiro, Kiro IDE
  and Kiro CLI notice/no-skill-directory assertions against temporary targets.
- PASS: relocated copied-source core_contracts: five cases default and five
  no-default, explicit manifest, spaces/Unicode checkout and unrelated CWD.
- UNVERIFIED: Codex marketplace E2E was deliberately not run. The current opt-in
  script exits zero with SKIP when disabled (also when the CLI is unavailable);
  aggregate Node process-pass counts cannot establish marketplace installation.
  It can invoke installation/network actions, outside this no-install run.
- PASS: both new runtime scenarios and final Rust guards; detailed results below.

## Evidence integrity correction

The historical task 5.1 combined seventeen-pass Node tally includes an opt-in
marketplace E2E script. Its default branch is a successful process exit without
executing E2E, not a native installation pass. This source audit does not prove
what flags were set in a historical run; absent an explicit enabled-execution
log, actual marketplace-add/install acceptance remains UNVERIFIED. The current
run reports only actually executed fixtures and assertions, not that opt-in file.

## Final results

| Criterion | Result | Evidence |
| --- | --- | --- |
| Poison shutdown/retained results | PASS Linux | Actual joined-thread mutex poison; no early runtime-stop acknowledgment; actual active result/cancel flag, queued non-launch, identical owned observer results; stopped ticket and host join both StatePoisoned |
| Concurrent admission/Cancel | PASS Linux | Eight submitters with eight attempts plus Cancel share a barrier; unique retained IDs at most 32, allowed results only, post-Cancel rejection, every accepted queued outcome cancelled, actual active result retained; all spawned threads joined |
| Runtime and contracts | PASS | Ten runtime unit tests each feature mode, plus two bridge, five core, eight profile, thirteen journal and eleven default/ten no-default report tests; no skips |
| Export/static/relocation | PASS | Fourteen actual export-plan tests, 27 workflow checks, four Cursor/Kiro notice groups and five relocated core cases per feature mode |
| Format/clippy | PASS | Both crate formats and both strict all-target clippy modes exit 0 |
| Full Rust regressions | MISS known baseline | Both full runs exit 101; exact 17/11 failure identities unchanged AND sole failed Cargo target is --lib; existing ignored doctest unchanged |
| Scope/evidence | PASS | All original production/helpers/tests preserved byte-for-byte; only two appended unit cases; frozen approvals, fixtures, manifests/locks and covered root inputs unchanged |
| Native/install acceptance | UNVERIFIED | No GPUI/native/foreground/Desktop parity, original MSRV, other OS or Codex marketplace installation E2E executed; no full task completion |

### Commands and raw evidence

```bash
node --test tests/export-plan.test.mjs
node --test tests/gpui-ci-contracts.test.mjs
node tests/export-cursor-kiro-skill-notice.test.mjs
manifest=tools/vfa-tui/Cargo.toml
# Repeat Rust test/clippy/full commands with --no-default-features.
cargo +stable test --manifest-path "$manifest" --locked --offline --lib engine::runtime::tests
cargo +stable test --manifest-path "$manifest" --locked --offline --test runtime_bridge --test core_contracts --test report_service_contracts --test profile_ownership --test journal_commits
cargo +stable clippy --manifest-path "$manifest" --locked --offline --all-targets -- -D warnings
cargo +stable test --manifest-path "$manifest" --locked --offline --no-fail-fast
cargo +stable fmt --manifest-path "$manifest" --check
cargo +stable fmt --manifest-path tools/vfa-desktop/Cargo.toml --check
# Discovered copied-source manifest, with spaces/Unicode; CWD outside checkout.
cargo +stable test --manifest-path "$relocated_manifest" --locked --offline --test core_contracts
cargo +stable test --manifest-path "$relocated_manifest" --locked --offline --no-default-features --test core_contracts
```

Session-selected ignored target evidence contains preflight.json, original source,
scenario-results.json, relocation-results.json, rust-results.json and per-command
logs. Commands bind real exit codes; failed-target and exact-name comparisons
prevent a new failing binary from being hidden behind the known baseline. No
production change was needed, so normal package/native acceptance was not rerun
or relabeled from historical evidence. Root covered inputs match the last verified
repository gate; final asset-integrity readback is required before commit.

### Review and route

One reused built-in worker added only two test blocks. Its serving model/effort
are unobservable. One configured Rust reviewer (gpt-6.1-sol/high, read-only;
serving identity unobservable) found no blockers, confirming actual poisoning,
owned outcomes/acknowledgments, correctly sized barrier, joins and bounded IDs.
Parent independently compared source bytes and ran all graders, while the worker
prepared tests. This is the smallest useful split for two coupled Rust scenarios
and independent CLI/relocation runs; no extra implementation or review wave.

Barrier scheduling does not prove every possible interleaving, fairness or native
lifecycle. The intentional panic tests fail-closed cleanup, not poisoned-state
recovery. Existing GateOwner releases active work during assertion unwinding.
A local approval review initially timed out; the permitted single retry succeeded.
No install, external target/global config mutation, push or dependency change.

Final scoped spelling/Markdown, task-DAG negative fixtures, covered integrity
readback, original source/fixture/frozen hashes and exact Git scope pass. AST
maintenance completed; HTML visualization exceeds its size limit and is not
native/visual acceptance evidence. Full task checkboxes are unchanged.
