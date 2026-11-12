# Task 1.5 evaluation: installation-free engine preparation

## Authorization and boundaries

Defined before implementation for the user-authorized installation-free wave.
This is preparation only: the full task checkbox and dependencies remain unchanged.
No installs, dependency/toolchain changes, remote CI, pushes, global configuration
or unrelated edits. Parent owns module/manifest wiring, evals, serialized Cargo
and separate no-reply local commits. Preserve other workers' files.

## Observable checks

- One dedicated host thread owns a long-lived multithread Tokio runtime with I/O
  and time enabled. Existing ReportService is the only ordinary public operation;
  owned requests/results, no domain recomputation or public arbitrary closures.
- Admission is nonblocking and bounded (including retained unacknowledged results).
  Busy is explicit; no unbounded completion map. Stable request IDs and immutable
  retained outcomes survive dropped observers and support explicit terminal release.
- Cancellation/control bypass ordinary saturation. Prelaunch cancellation performs
  no work. Started synchronous work is awaited, not abandoned by dropping/aborting
  a future. Its observed result wins with cancellation separately recorded.
- Wait drains accepted work; Cancel rejects queued work and awaits active work.
  Shutdown can escalate Wait to Cancel. Runtime-stopped acknowledgment follows
  actual work and runtime destruction; only off-UI host join proves thread exit.
  No timeout/detach fallback, blanket unwind recovery or UI-thread join assertion.
- Deterministic barrier tests prove saturation, cancellation, retained completion,
  terminal release, repeated shutdown, blocking-work wait and fail-closed panic.
  Private probes exercise real Tokio timer and piped helper I/O; no sleeps as
  ordering proof, no production test backdoors or uncontrolled real processes.
- Both TUI feature modes pass targeted contracts, strict clippy and fmt. Full
  failure names equal the existing 17/11 baseline. Check normal dependency closure
  under both locks, unchanged frozen approvals and normal package/archive bytes.

## Deferred full obligations

GPUI foreground bridge, weak-view delivery, native round trip, complete D10/session
identity and actual window lifecycle remain UNVERIFIED. No Desktop/native pass or
full task completion inferred from engine-only tests. Task 1.4 remains unchecked.

## Sources and initial status

Context7 `/tokio-rs/tokio` documents multithread Builder plus enable_all and that
started spawn_blocking work cannot be aborted. Its shutdown timeout allows work
to continue and is not cleanup. Verify selected cached source and actual tests.
Initial status before work: checks defined; implementation not yet graded.

## Results: engine-only preparation

| Criterion | Result | Evidence |
| --- | --- | --- |
| Persistent runtime/control | PASS Linux | Eight actual unit tests per feature mode exercise multithread Tokio timer, piped test-helper I/O, bounded admission/retention, cancel, escalation, panic and blocking-work ownership |
| Real report bridge | PASS | Two integration tests per mode call existing ReportService through owned runtime-neutral tickets; lost observers recover immutable outcomes and explicit terminal release returns budget |
| Source safety review | PASS repaired case | Configured Rust reviewer identified global Cancel/launch ordering; parent RED observed effects 1 rather than 0, final same regression passes with effects 0 |
| Existing targeted contracts | PASS | Five core, eight profile, thirteen journal and eleven default/ten no-default report contracts pass; no skipped or rewritten fixture expectations |
| Format and clippy | PASS | Both crate formats and both strict all-target engine clippy modes exit 0 |
| Full Rust regressions | MISS existing baseline | Actual --no-fail-fast runs exit 101; exact failure identities equal known seventeen default/eleven no-default cases; one existing ignored doctest |
| Engine dependency boundary | PASS | Both actual locked consumer metadata subtrees exclude normal presentation dependencies; 31 negative fixtures reject |
| Normal package | PASS | Tracked-HEAD snapshot plus only final engine module overlays passes normal cargo package --locked --offline verification; archived production bytes match; packaged no-default library checks |
| Native/foreground/full task | UNVERIFIED | GPUI bridge, native runtime/window round trip, both-consumer execution, full D10/session and journal/lifecycle integration remain deferred; task 1.4 and full 1.5 unchecked |
| Poison cleanup/MSRV/other OS | UNVERIFIED | Poison is not injected by public operations; other OS and original declared MSRV were not executed |

### Reproduced defect and bounded repair

Global Cancel originally updated only the shared mode and notification. A host
paused after its dequeue check could subsequently begin work after Cancel returned.
The private two-barrier regression forces this interleaving and records a real
side effect; parent ran it RED before production repair. The repair synchronously
cancels all retained request cells under the existing registry mutex, with
admission rechecked under the same synchronization. Per-cell CAS decides whether
work was prevented or already started. Started work still returns its observed
result with cancellation recorded; no abort/drop-as-cleanup shortcut was added.

On poisoned registry access, the guard is used solely to cancel for cleanup;
poison is not cleared and no state is reused. Runtime destruction precedes
StatePoisoned acknowledgment/join failure, not successful shutdown evidence. That
non-publicly-injectable branch is unexercised, not a tested recovery claim.

### Exact local commands

```bash
manifest=tools/vfa-tui/Cargo.toml
# Repeat test/clippy/full commands with --no-default-features.
cargo +stable test --manifest-path "$manifest" --locked --offline --lib engine::runtime::tests
cargo +stable test --manifest-path "$manifest" --locked --offline --test runtime_bridge --test core_contracts --test report_service_contracts --test profile_ownership --test journal_commits
cargo +stable clippy --manifest-path "$manifest" --locked --offline --all-targets -- -D warnings
cargo +stable test --manifest-path "$manifest" --locked --offline --no-fail-fast
cargo +stable fmt --manifest-path "$manifest" --check
cargo +stable fmt --manifest-path tools/vfa-desktop/Cargo.toml --check
# Select tracked-source snapshot manifest; no dirty or verification bypass.
cargo +stable package --manifest-path "$snapshot_manifest" --locked --offline
cargo +stable check --manifest-path "$unpacked_manifest" --locked --offline --lib --no-default-features
```

Logs/JSON are in the discovered checkout's ignored target evidence directory for
this installation-free wave: runtime-review-red.log, rust-results.json, both
feature-mode runtime-unit/contracts/clippy/full logs, package-results.json and
engine-source-hashes.json. Parent binds exact commands, exit codes, unchanged
full failure identities, source hashes and archive hash. Existing cached tools
were used without installation or compiler/dependency/pin changes.

One reused built-in implementation worker owned two new production modules and
one integration test; its serving model/effort are unobservable. One configured
Rust reviewer (gpt-6.1-sol/high, read-only) supplied the independent source
challenge; actual serving identity is unobservable. Parent owns shared module
wiring, actual serialized graders, source inspection, package checks and Git.
The fixture and static-CI tracks had disjoint writers; no parallel Cargo ran.

A started synchronous report may make shutdown wait indefinitely. EngineHost
start/join and blocking Drop fallback belong off the UI thread; no UI-thread
safety or bounded cleanup guarantee is inferred. No profile/journal opens or
action execution/replay were added. Explicit off-UI join proves std host-thread
exit; a stopped ticket proves only completed runtime destruction.

Verified compiler: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`.
Locked Tokio: `1.52.3`. Verified crate SHA-256:
`300ea70e0b6e9349248e8c4b13d73c679381fc59e7417a5b539911802b8c8c34`.

Final structural spec/negative mutants, scoped spelling/Markdown, lazy links,
unchanged frozen approvals/fixtures/manifests/locks and exact commit scope pass.
The full repository gate was already run for the CI slice; all covered inputs
remain byte-identical and final integrity readback passes. The final AST graph
update completed; HTML visualization exceeds its size limit and is not visual
or native evidence. No extra task checkbox was marked complete.
