# Task 1.4 evaluation contract

## Objective and authority

Implement task 1.4 only: a pinned native GPUI Kit bootstrap and same-version UI/
native test entry points under `tools/vfa-desktop/`. Keep the separate Desktop
lockfile, `publish = false` and presentation-free `vfa-tui` path dependency.
The user explicitly authorized this task with Ultracode and Codex Agent Teams.
No task 1.5 bridge, application workflows, provider repair, root Cargo workspace,
TUI dependency/lock changes, approval-document edits, commit, push or publication.
Host system packages need separate authorization; absence must not produce PASS.

## Route and ownership

Ultracode owns the existing workflow records; Codex Agent Teams supplies routing.
Complexity B: snapshot/API/feature agreement and native prerequisites need source
verification before implementation. Parent owns version/graph resolution, all
crate/eval/workflow edits and serialized Cargo commands. One bounded read-only
Rust reviewer verifies release-specific facilities in parallel with discovery;
no recursive delegation or competing workflow tree. Configured Sol/high profile
has checked metadata and read-only defaults; serving identity is unobservable.
Two other disk candidates had incomplete schema metadata and are not dispatched.
Allow one implementation wave, one independent source-review wave and one parent
repair pass; unresolved external prerequisites remain explicit incomplete work.

## Deliverables and checks defined before implementation

1. `Cargo.toml`, `Cargo.lock` and `rust-toolchain.toml` select exact GPUI Kit,
   matching GPUI snapshots and compiler. Resolve actual archive checksum/source
   identities from registry/source bytes, not the research candidate alone.
   `provenance/toolkit.json` records selected version/source/checksums, lockfile
   and compiler identity, chosen/absent optional features and facility limits.
2. Shared bootstrap code and `src/main.rs` initialize Kit and bundled Assets,
   open a native Root-wrapped application view and handle failures explicitly.
   No remote assets, shell, WebView, Fast, editor/server or runtime bridge feature.
   Native paths/evidence output are caller-selected or discovered, not fixed to
   this checkout, user home or a skill installation path.
3. `tests/ui.rs` compiles the exact release's dev-only test-support imports and
   drives a real bootstrap control callback/state through a pointer/key event.
   A disconnected callback retaining the same label/metadata must fail.
   Synthetic state/semantics do not claim pixels, native clipboard/IME or AT.
4. `tests/native_smoke.rs` is an explicit `test = false`, `harness = false`
   main-thread entry, invoked with `--scenario bootstrap --backend <BACKEND>
   --evidence-dir <DIR>`. It launches the same Root/assets bootstrap in a real
   window, observes rendering/event-loop progress and bounded clean exit, and
   records actual backend/toolchain/build identity. Missing or contradictory
   backend/session/renderer, unsupported scenario, failed window or unwritable
   evidence destination must exit nonzero, never a successful skip/fallback.
   Full admission/process/runtime cases belong to task 2.6, not this task.
5. Record release-specific Linux X11/Wayland, macOS ARM64 and Windows x64 MSVC
   facilities honestly. Run the available host's native bootstrap paths; each
   unobserved required host remains UNVERIFIED, not inferred from Linux. Native
   admission across all required hosts remains a later mandatory gate. Windows
   headless pixels are unavailable upstream, not successful skipped pixel proof.
6. Desktop fmt, strict all-target clippy/check, UI tests and explicit native smoke
   run with the locked graph and selected compiler. Inspect the normal (not dev-
   unified) feature graph for forbidden optional layers/test-support; run the
   existing engine boundary checker under both engine and Desktop lock graphs.
   Retain existing engine contracts and frozen inputs. Record the existing 17/11
   full-suite baseline failures separately; do not repair or hide them here.
7. Grade provenance against actual lockfile/checksums/compiler/source files,
   including negative mutations for version/checksum/features/missing targets.
   Run spec structural mutants, local JSON/link/scope/approval hashes, spelling,
   Markdown and `npm run validate`. Update the AST graph after code edits; generated
   graph artifacts stay uncommitted and are not native visual acceptance evidence.

## Completion and stop conditions

Do not check task 1.4 merely because files or metadata exist. Bootstrap and test
imports must compile against the selected graph, normal dependency isolation must
pass, UI callbacks must be exercised, and the available native launch paths must
produce observed evidence. Missing platforms/facilities stay explicitly unverified
against the original feature acceptance gates. No scope reduction, fake stub UI
test, successful native skip, compile-only native claim or invented checksum.
Dependency/download denial, unresolved source agreement, prerequisite approval,
unsafe launch or failed grader stops that route, not unrelated safe progress.

## Initial evidence

- Worktree clean at `bedd0bb5d024f626255164d543a6977bab622302`; task 1.4 unchecked.
- Context7 resolves `/longbridge/gpui-kit/v0.9.1`; primary tagged manifests select
  Kit `0.9.1` and GPUI snapshots `=0.3.12`. The candidate's published archive and
  immutable source agreement still require verification before dependency pinning.
- Host compiler is Rust `1.99.1`, Linux x86_64; only the named stable toolchain is
  installed. Compiler bits and numeric pin must agree without claiming an
  uninstalled toolchain alias. Required system build tools/headers are absent;
  an authorization question is outstanding. X11 and Wayland socket indicators
  exist, but are not proof that a GPUI window can launch.
- Independent source research confirms Root wrapping is supplied by Kit's
  `open_window`; do not double-wrap. Tagged testing prose has a stale `0.3.8`
  sentence; actual manifest/source and compiled imports are authoritative.

## Current results: implementation incomplete

| Criterion | Result | Authoritative evidence |
| --- | --- | --- |
| Archive/source/snapshot pin | PASS | Registry checksum matches downloaded Kit archive, its VCS record selects the immutable commit, facade bytes match that commit, and acquired GPUI metadata identifies its upstream revision |
| Numeric compiler/source provenance | PASS scoped | Actual existing compiler release/commit/LLVM match recorded numeric pin; cached archive/VCS and facade bytes agree; this does not claim the numeric Rustup alias is installed |
| Pin/features/targets negative guards | PASS | Eighteen deterministic mutations rejected by `check_bootstrap.py --self-test`; actual normal Kit feature graph is assets/component only |
| Presentation-free engine | PASS graph | Existing checker passes actual engine and Desktop-consumer graphs, plus 31 deterministic cases; Fast/WebView/shell/test-support are absent from the normal graph |
| Existing engine regressions | PASS scoped | Five core cases in each mode, eleven default and ten no-default report cases pass under the unchanged engine lockfile; this is not task 1.6 Desktop-graph behavioral parity |
| Unicode/space relocation | PASS grader paths | Actual provenance and consumer boundary graders rerun from unrelated CWD in a relocated Unicode/space-containing root; native executable/evidence path behavior is still unverified |
| Desktop compilation | MISS prerequisite | Locked library check exits 101 in `yeslogic-fontconfig-sys`: the host has no `pkg-config`; no successful bootstrap/type-check claim |
| Required package installation | MISS environment | User explicitly approved build packages; `sudo -n apt-get update` exits 1 before apt runs because host sudo policy refuses; no packages changed |
| Desktop UI callback and disconnected-callback mutant | UNVERIFIED | Real callback-state test exists, but the consumer and actual mutant cannot run until native dependencies build |
| Desktop strict clippy/all-target build | UNVERIFIED | Owed after prerequisites; rustfmt parses/formats all current Rust targets but does not type-check them |
| Explicit native bootstrap and negative runtime scenarios | UNVERIFIED | Source exists; no X11/Wayland or other native launch occurred; unsupported/missing-renderer/evidence guards are not yet executed proof |
| Independent review | PASS process | Bounded source review found early-frame and provenance/target false positives; one parent repair addresses them, with added negative mutants; no second review/runtime success invented |
| Repository/spec/lint/AST | PASS scoped | Repository validation, structural task mutants, fmt, spelling and Markdown pass; AST updated, while oversized HTML visualization is not native visual evidence |
| Frozen engine/approvals and Git | PASS scope | Requirements/design, all engine files/manifest/lock and HEAD remain unchanged; no staging/commit/push |
| Full feature/native acceptance | UNVERIFIED incomplete | Other hosts, real pixels/compositor display, AT/IME/clipboard, runtime/process admission and packaging remain mandatory later gates, not inferred support |

### Checks actually executed

Commands use the discovered repository root and session-selected Cargo cache/target
directories. The available `stable` alias was checked against the recorded numeric
compiler release/commit; it is not permission to drift to a newer compiler.

```bash
manifest=tools/vfa-desktop/Cargo.toml
cargo +stable generate-lockfile --manifest-path "$manifest"
cargo +stable check --manifest-path "$manifest" --lib --locked
cargo +stable fmt --manifest-path "$manifest" --check
python3 tools/vfa-desktop/tests/check_bootstrap.py \
  --manifest-path "$manifest" --toolchain stable --self-test
python3 tools/vfa-tui/tests/check_feature_boundary.py \
  --manifest-path tools/vfa-tui/Cargo.toml --self-test
python3 tools/vfa-tui/tests/check_feature_boundary.py \
  --manifest-path tools/vfa-tui/Cargo.toml --consumer-manifest "$manifest" --self-test
# Repeat the engine contract command with --no-default-features.
cargo +stable test --manifest-path tools/vfa-tui/Cargo.toml --locked --offline \
  --test core_contracts --test report_service_contracts
python3 .kiro/specs/gpui-desktop/verify.py --tasks --self-test
npm run validate
git diff --check
graphify update .
```

The library check fails at the actual prerequisite; other listed completed checks
exit zero. The initial offline re-resolution also encountered an uncached optional
Fast/proptest requirement; normal online resolution succeeded without enabling
Fast or changing the engine lockfile. Unselected optional packages in a Cargo
lockfile are not a normal-feature leak.

Current logs are `vfa-task-1-4-{lock,check,provenance,normal-tree,engine-boundary,
engine-contracts,host-prerequisites,relocation,spec,npm-validate,spell,markdown,
graphify}.log` in the session's temporary run location. The final provenance
checker rerun records eighteen mutants, actual cached source checks and the
normal feature graph. Scoped spelling and offline Markdown commands cover the
new crate and changed hidden spec/workflow documents, excluding generated target
and lockfile spellings, not product source. No lint settings are changed.

### Review repair and remaining work

The selected GPUI implementation drains the first next-frame callback before
draw/present, contrary to its API comment. The original smoke draft could quit
without native rendering. It now refreshes and arms a second callback, requiring
a nonempty scene and reported Linux renderer after the intervening native turn.
macOS/Windows do not expose that GPU-info API; it is not fabricated there. This
remains scene/lifecycle progress, not proof of pixel/compositor display.

The review also reproduced acceptance of a false immutable-facade claim, a wrong
GPUI upstream commit and a redirected native target. The grader now rejects those
mutations and checks actual archive/extracted facade bytes and snapshot metadata.
The immutable source comparison was independently performed at acquisition; the
offline grader compares its recorded hash to the actual archive and cached source.

Install the approved prerequisites in a host context with appropriate privileges,
then run locked all-target builds/clippy, UI callbacks, an actual disconnected-
callback mutation, explicit X11/Wayland bootstrap and negative runtime cases.
Bind results to actual source/lock/binary/backend metadata; do not manufacture
PASS fields or check task 1.4 before those owed checks complete. The goal is blocked
after three consecutive prerequisite-blocked turns. The first implementation turn
made meaningful source/verification progress; two subsequent audits confirm the
same absent tools, refused installation and no live build/worker. Host-side setup
is required to resume. No completion or native success is inferred from this audit.

### Changed paths

- `tools/vfa-desktop/Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `README.md`.
- `tools/vfa-desktop/src/lib.rs`, `src/main.rs`.
- `tools/vfa-desktop/tests/ui.rs`, `tests/native_smoke.rs`, `tests/check_bootstrap.py`.
- `tools/vfa-desktop/provenance/toolkit.json`.
- This eval/index, task status/evidence and existing Ultracode workflow records.

Parent-only implementation plus one exact checked Rust specialist's read-only
research/review was the smallest useful route: source/graph selection stayed on
the critical path, while the independent review caught a real false native PASS.
No cost/latency saving, serving model/effort or native isolation claim is inferred.
