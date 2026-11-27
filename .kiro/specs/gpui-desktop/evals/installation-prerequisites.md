# Installation prerequisite continuation

## Scope and authority

The user explicitly requested installation-related work from this task plan.
This continuation targets task 1.4's existing GPUI bootstrap and task 1.6's
existing Desktop consumer contracts. It does not authorize commits, pushes,
remote CI, deployments, dependency upgrades or implementation of later tasks.
Historical no-install waves remain historical evidence, not current restrictions
on this explicitly authorized prerequisite work.

The user subsequently authorized committing and pushing the delivered work on
the current feature branch. This publication follow-up does not authorize a merge,
release, deployment or manual remote-CI dispatch, and does not close native gates.

Parent owns system package/Cargo state, verification and evidence integration.
Two native read-only agents reviewed task admission and native/prerequisite
sources; no worker edited files or ran installations/tests. Serving model/effort
and IDE authentication/profile are unverified; the separately inspected local
Codex CLI reports version 0.162.1 and ChatGPT authentication.

## Checks defined before installation

- Verify actual installed package metadata and compiler against the existing pin.
- Preserve both lockfiles and frozen requirements/design hashes.
- Run the bootstrap provenance/feature mutants and engine boundary graders.
- Run locked Desktop all-target check/clippy, UI callback and shared engine cases.
- Execute available Linux native bootstrap paths with backend-specific evidence;
  absent sessions/renderers/hosts are failures or unverified, never green skips.
- Review any actual compiler defect before one bounded repair; no unrelated UI,
  process/runtime or backlog implementation.

The detailed execution contract and command evidence are under
`.workflow/ultracode/gpui-install-prerequisites/`.

## Installation observations

Host: Ubuntu 24.04, WSL2 x86_64. Existing `stable` compiler bits match the recorded
Rust 1.99.1 pin; no compiler upgrade or numeric Rustup alias installation is needed.
The initial Cargo cache lacked the pinned GPUI source. The approved locked Linux
fetch completed successfully without changing dependency versions or lockfiles.

Sandboxed sudo initially failed at its privilege boundary. Escalated
`sudo -n apt-get update` then exited 1 because a password was required. No agent
supplied credentials or installed system packages. The user performed the corrected
four-package installation in their terminal and reported completion; direct
`dpkg-query` and `pkg-config` checks subsequently confirmed:

| Package | Installed version | pkg-config result |
| --- | --- | --- |
| libfontconfig-dev | 2.15.0-1.1ubuntu2 | fontconfig 2.15.0 |
| libwayland-dev | 1.22.0-2.1build1 | wayland-client 1.22.0 |
| libxkbcommon-x11-dev | 1.6.0-1build1 | xkbcommon and xkbcommon-x11 1.6.0 |
| libx11-xcb-dev | 2:1.8.7-1ubuntu0.1 | x11-xcb 1.8.7 |

The four-package apt simulation proposed 21 new packages, zero upgrades and zero
removals. This is a simulation, not an attestation of the user's complete apt
transaction. GCC/G++, make, pkg-config, OpenSSL development files, the Vulkan
loader and Mesa Vulkan drivers were already installed.

The initial suggested command unnecessarily included clang/cmake/libclang/zstd
development packages. After fetching the exact sources/graph, the parent corrected
that advice before the user's installation. Those packages remain absent and are
not claimed as requirements. Optional WebView/audio packages were excluded.

Primary setup reference: [tagged GPUI Kit installation documentation](https://github.com/longbridge/gpui-kit/blob/v0.9.1/website/docs/installation.md),
queried through Context7. The actual selected dependency graph and build results,
not the upstream all-feature package list, determine this consumer's requirements.

## Verification results

All Cargo commands use the unchanged locks, explicit verified `+stable` alias,
and offline resolution after the authorized locked fetches. Commands below are
prefixed with `rtk` (`rtk proxy` where raw output is needed).

| Check | Result | Evidence |
| --- | --- | --- |
| Installed prerequisites | PASS | Direct package metadata and pkg-config versions above |
| Desktop all-target check | PASS after repair | Initial exit 101: UI `update_window` lacked `AppContext`; importing the trait gives exit 0 |
| Desktop strict clippy | PASS | All-target invocation exits 0; explicit `--test native_smoke` strict clippy also exits 0 |
| Desktop standard suite | PASS | Exit 0: six shared engine cases and one real UI callback case; zero ignored/skipped tests; empty unit/doc targets |
| TUI shared cases | PASS | Six cases in default and six in no-default mode under the unchanged TUI lock |
| Callback mutation | PASS negative oracle | Isolated Unicode/space-containing copy with an empty callback exits 101 at `left: 0`, `right: 1`; original final suite rerun passes |
| Bootstrap provenance | PASS source/graph | Compiler/archive/VCS/lock identities, normal Kit features and 18 negative mutants pass; not runtime proof alone |
| Engine boundary | PASS both locks | Actual normal engine subtrees and 31 deterministic mutants pass |
| Native target compilation | PASS after approved repair | Explicit target initially exits 101: `rendered_scene` requires `bench-support`; existing dev-only `painted_quads` observation compiles without feature/dependency changes |
| Native X11 bootstrap | PASS scoped | Escalated actual WSLg handle `x11`, six painted quads, frame progress and event-loop return; parent/child exit 0 |
| Native Wayland bootstrap | PASS scoped | Escalated actual WSLg handle `wayland`, ten painted quads, frame progress and event-loop return; parent/child exit 0 |
| Native negative cases | PASS negative oracles | Unsupported scenario, wrong native host, file-as-evidence-directory and missing display each exit 1 with the expected diagnostic |
| Native path portability | PASS scoped | Copied executable and evidence paths contain spaces/Unicode; X11 bootstrap from unrelated `/tmp` CWD exits 0 with the same binary hash; not clean-machine packaging proof |
| Native sandbox attempt | MISS environment | X11 child could not connect to the display; retained as FAIL evidence, not reclassified as PASS |
| Format, spelling, Markdown and diff | PASS scoped | Desktop fmt, changed-file codespell, cached Markdownlint on both changed spec documents and diff whitespace checks exit 0 |
| Repository validation | PASS host rerun | `npm run validate` exits 0 through approved escalation; initial sandbox run exits 1 at install-coverage because spawnSync reports EPERM/empty output; no exporter changes |
| macOS ARM64 / Windows x64 MSVC | UNVERIFIED | No such native host exercised; Linux cannot substitute |

`test = false` excludes the explicit native target from ordinary checks; an
all-target success did not expose its initial compiler defect. Explicit native
compilation/clippy/run evidence is mandatory and was executed separately.

The initial one-repair limit was consumed by the UI import. The parent stopped the
native route and obtained the user's explicit approval for one additional bounded
native-test repair. That repair removes the unused import and replaces the
benchmark-only scene call with the selected SDK's `painted_quads` observation;
the emitted field and parent guard are named accordingly. No further repair,
dependency upgrade or feature expansion was silently admitted.

Both Linux observations report llvmpipe software emulation, Mesa
25.2.8-0ubuntu0.24.04.2 with LLVM 20.1.2. The X11 log warns that DRI3 acceleration
is unavailable. This proves neither hardware acceleration nor compositor pixels,
AT, IME, clipboard, performance, packaging or later process/runtime admission.

Raw command logs, the expected callback failure and backend-specific observation/
summary JSON live in the run's `results/` directory. Native summaries bind the
actual executable, unchanged lock, bootstrap source and toolkit provenance hashes.
Repository `*.log` ignores remain unchanged: raw logs are local-only; structured
JSON and `results/verification-transcript.md` retain versioned result identities
and labeled verbatim excerpts of the key positive/negative assertions.
The initial relative evidence destination resolved under Cargo's package CWD;
the parent moved only its generated evidence into the run root and subsequently
used absolute caller-selected evidence paths. No source or pre-existing files
were removed.

## Reproduction commands

```bash
desktop=tools/vfa-desktop/Cargo.toml
tui=tools/vfa-tui/Cargo.toml
rtk cargo +stable check --manifest-path "$desktop" --locked --offline --all-targets
rtk cargo +stable clippy --manifest-path "$desktop" --locked --offline --all-targets -- -D warnings
rtk cargo +stable clippy --manifest-path "$desktop" --locked --offline --test native_smoke -- -D warnings
rtk cargo +stable test --manifest-path "$desktop" --locked --offline
rtk cargo +stable test --manifest-path "$tui" --locked --offline --test core_contracts
rtk cargo +stable test --manifest-path "$tui" --locked --offline --no-default-features --test core_contracts
rtk proxy python3 tools/vfa-desktop/tests/check_bootstrap.py --manifest-path "$desktop" --toolchain stable --self-test
rtk proxy python3 tools/vfa-tui/tests/check_feature_boundary.py --manifest-path "$tui" --self-test
rtk proxy python3 tools/vfa-tui/tests/check_feature_boundary.py --manifest-path "$tui" --consumer-manifest "$desktop" --self-test
evidence_dir=$(mktemp -d)
rtk cargo +stable test --manifest-path "$desktop" --locked --offline --test native_smoke -- \
  --scenario bootstrap --backend x11 --evidence-dir "$evidence_dir"
# Repeat with --backend wayland and a new evidence directory.
```

On this host the successful native runs used the compiled explicit target directly
through approved escalation because sandbox display access failed. Exact executable
and arguments are recorded in workflow state/results; do not infer a sandboxed
Cargo launch succeeded. The commands are local only, not remote CI execution.

## Remaining gates

Read-only continuation discovery confirms a native Windows x64 host is reachable
through approved WSL interop. Rust/cargo are absent from PATH and their default
user location; the Visual Studio installer, known MSVC BuildTools locations and
standard Windows SDK library path are absent. This is standard-path discovery,
not an exhaustive inventory or Windows build evidence. No Windows installation
or configuration occurred. An appropriately licensed toolchain/setup route and
macOS ARM64 host access still require user input. The user has since confirmed
they do not have a Mac; Docker alternatives are documentation research, not
native acceptance evidence. See the run's
`results/windows-setup-handoff.md` and `windows-prerequisite-discovery.json`.

Tasks 1.4 and 1.6 remain unchecked: local installation/build/parity proof does not
waive remaining native-host obligations or task dependencies. No implementation
of task 1.5's foreground bridge, process/runtime admission or extensive UI is
claimed. Requirements/design and both lockfiles are unchanged. No staging,
commit, push, publication or deployment occurred during the verification phase;
the later explicit commit/push authorization is recorded above.
