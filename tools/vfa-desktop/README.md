# VFA Desktop bootstrap

Task 1.4 adds the native GPUI Kit foundation, not the finished catalog application.
No engine workflow, trusted execution or runtime bridge is implemented here.
Kit owns the window's Root; the application supplies its content and bundled Assets.

## Build and test

Use the compiler pinned in `rust-toolchain.toml`. From the repository root:

```bash
manifest=tools/vfa-desktop/Cargo.toml
compiler=$(python3 -c 'import pathlib,sys,tomllib; print(tomllib.loads(pathlib.Path(sys.argv[1]).with_name("rust-toolchain.toml").read_text())["toolchain"]["channel"])' "$manifest")
cargo "+$compiler" check --manifest-path "$manifest" --locked --all-targets
cargo "+$compiler" clippy --manifest-path "$manifest" --locked --all-targets -- -D warnings
cargo "+$compiler" test --manifest-path "$manifest" --locked --test ui
cargo "+$compiler" run --manifest-path "$manifest" --locked
```

An already installed alias can be used only when the provenance grader verifies
its actual compiler bits against the numeric pin. For example, a `stable` alias
is acceptable only if that check passes; it must not permit future compiler drift.

```bash
python3 tools/vfa-desktop/tests/check_bootstrap.py \
  --manifest-path "$manifest" --toolchain stable --self-test
```

The checker proves pins, cached source/archive identity, declared targets and the
normal feature graph. It does not prove compilation, interaction or native launch.
The lockfile can contain unselected optional packages; only normal graph reachability
determines whether Fast, WebView, shell or test support leaked into production.

On Linux the native backend needs development libraries/build tools and a working
Vulkan driver plus an actual X11/Wayland session. Missing prerequisites fail, not
skip. Do not install system packages without explicit authorization. Windows needs
the MSVC toolchain and SDK; macOS needs its native Apple Silicon/Xcode environment.

## Explicit native bootstrap

Select an evidence directory yourself; the application does not embed a checkout,
user-home, skill or evidence path. Native paths may contain spaces and Unicode.

```bash
evidence_dir=$(mktemp -d)
cargo "+$compiler" test --manifest-path "$manifest" --locked --test native_smoke -- \
  --scenario bootstrap --backend wayland --evidence-dir "$evidence_dir"
```

Backends are `x11`, `wayland`, `macos` (ARM64), and `windows` (x64 MSVC), selected
on their actual native hosts. The explicit test is excluded from ordinary test
runs. It re-executes on a child process's main thread, requires the actual native
window handle to match, observes a nonempty scene across a native draw/present
turn and bounded event-loop exit, then writes build-bound evidence and a log.
It disables the other Linux selector in the child instead of silently accepting
XWayland. Unknown scenarios/backends, missing renderer/session, timeouts and file
errors fail. Existing evidence is never overwritten.

This is not pixel/compositor-display, AT, clipboard, IME, packaging or the later
runtime/process admission proof. The pinned implementation executes frame callbacks
before drawing despite its API comment, so the probe deliberately waits through
an intervening native draw turn. Linux requires reported renderer/GPU information;
that API is unavailable on macOS/Windows and is not fabricated there.

See `provenance/toolkit.json` for exact crate/source identities and facility limits,
and the active task eval under the repository's GPUI spec directory for actual
results. Unexecuted host capabilities remain UNVERIFIED.
