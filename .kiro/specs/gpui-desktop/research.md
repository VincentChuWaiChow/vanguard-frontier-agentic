# GPUI desktop evidence

Checked 2026-11-10. Repository baseline:
`6dda992a3dedd845731d795aaaed20d49346f57c`, on `feature/gpui-desktop-alpha`.
Facts below come from source inspection or primary documentation, not executed
native app tests. Requirements remain proposed until user approval.

## Upstream identity and constraints

- **Verified, upstream source:** the requested repository now presents GPUI Kit as
  the application dependency, composed over GPUI, a behavior foundation and the
  styled `gpui-component` layer. The optional JavaScript extension host is separate.
  This is not a request to install a coding-agent skill or embed a JS runtime.
  [Official repository](https://github.com/longbridge/gpui-kit).
- **Verified, release-source snapshot:** the
  [v0.9.1 manifest](https://github.com/longbridge/gpui-kit/blob/v0.9.1/Cargo.toml)
  declares the Kit family at `0.9.1` and pins matching `gpui-pre` snapshots to
  `=0.3.12`. The site installation page also presents `0.9.1`. An earlier README
  web snapshot showed `0.7`, and Context7 indexed `v0.8.0`: do not mix their APIs.
  Version `0.9.1` is a research candidate, not an installed dependency decision.
- **Updated at design phase:** the earlier commit lookup failure is resolved.
  The official release/commit pages identify `v0.9.1` at immutable commit
  [`39ab849a93b5f15507b969164d49891db8d5b470`](https://github.com/longbridge/gpui-kit/commit/39ab849a93b5f15507b969164d49891db8d5b470).
  Parent independently opened that source and the release manifest. Crate archive
  checksum and consumer lockfile/toolchain capture remain unresolved implementation
  gates. No toolchain, dependencies or system packages were installed in research.
- **Verified, documented development baseline:** upstream describes macOS 15+
  with Xcode tools; Windows 10+ with MSVC, Visual Studio C++/SDK and CMake; and an
  Ubuntu 24.04 setup needing a graphical X11/Wayland session and Vulkan driver.
  Its current locked graph needs Rust 1.92+. This does not establish our CPU
  support matrix or prove our app runs on those systems.
  [Installation](https://gpui-kit.com/docs/installation/).
- **Verified, API guidance:** initialization and window setup are supplied through
  Kit's re-exported APIs; Context7 `/longbridge/gpui-kit` corroborates that setup.
  Final design must verify signatures against the selected release, not paste
  mismatched examples. [Getting started](https://gpui-kit.com/docs/getting-started/).
- **Verified, accessibility limitation:** upstream explains that semantic-tree
  assertions cannot prove actual screen-reader announcements or visible focus;
  keyboard operation must also be implemented and checked.
  [Accessibility](https://gpui-kit.com/docs/accessibility/).
- **Verified, delivery distinction:** a compiled executable is not by itself a
  relocatable, dependency-complete or signed desktop distribution. Target-specific
  packaging and clean-machine checks are required.
  [Packaging](https://gpui-kit.com/docs/packaging/).
- **Verified, declared licensing:** upstream source/code examples use Apache-2.0;
  documentation has additional CC BY 4.0 terms, with third-party assets retaining
  their own terms. Do not assume icon/font redistribution follows the crate
  license. No upstream prose or illustrations are copied into this spec.
  [Repository licensing](https://github.com/longbridge/gpui-kit#license),
  [font guidance](https://gpui-kit.com/docs/fonts/).

No marketed frame-rate, scalability, production-readiness or accessibility claim
is adopted as evidence of this application's performance or support.

## Explicit Context7 MCP verification

The initial lookup used `resolve-library-id` for `gpui-kit`, returning
`/longbridge/gpui-kit` with an indexed `v0.8.0` version, followed by a setup query.
After the user's explicit Context7 request, three additional `query-docs` calls
covered platform/dependency constraints, UI-test boundaries and the asynchronous
runtime bridge. This section records retrieved evidence rather than treating a
library-ID match or an unreturned snippet as proof.

| Topic | Retrieved primary sources and cross-check | Requirements consequence |
| --- | --- | --- |
| Application dependency | Context7 points to upstream `AGENTS.md`/`CONTRIBUTING.md`: umbrella Kit dependency, exact matching GPUI snapshot; corroborated by the tagged manifest cited above | Keep R16.1 release/source/lockfile pinning; do not choose a release from the index label alone |
| Tokio versus GPUI executors | Context7 returns `website/docs/task.md` and `website/docs/image.md`; [task guide](https://gpui-kit.com/docs/task/) separates foreground/background work, while the [image example](https://gpui-kit.com/docs/image/) explicitly provides its own Tokio runtime | R9.9 requires runtime lifetime compatibility for existing Tokio services. Applying the upstream executor distinction to `vfa-tui`'s Tokio process/writer services is our inference, not an already tested bridge |
| UI test assertions | Context7 returns `crates/kit/TESTING.md`, the test reference and [testing guide](https://gpui-kit.com/docs/test/): metadata assertions do not prove application callbacks, pixels, OS clipboard/IME, packaged launch or AT behavior | R13.9 and revised evals require real interaction effects; native evidence remains separate |
| Renderer matrix drift | One Context7 snippet still names GPUI `0.3.8`. The live testing guide names `0.3.12`, reports macOS/Linux offscreen renderers and an explicit Windows pixel-test skip | R16.8–R16.9 require release-specific checks and native visual evidence where headless pixels are unavailable; this is not evidence that Windows native rendering is unsupported |
| Component-specific platform support | Context7 and the live [WebView guide](https://gpui-kit.com/docs/webview/) report experimental embedding with X11/XWayland requirements rather than native Wayland support | General GPUI Linux support cannot be inherited by every optional component. No embedded WebView is required by the proposed native catalog workflows; any such design choice needs a compatibility decision |

Context7 is retrieval, not compilation or native runtime proof. Attempted reads of
tag-specific test/WebView source pages returned browser cache misses, so those
sources are not claimed checked at an immutable release revision. The live guides
and retrieved `main` paths must be reconciled with the chosen source revision
before design relies on an exact API signature. No absence-of-snippet inference
is used to reject a documented capability.

## Existing repository facts

Paths and line ranges below refer to the baseline, not future extracted modules.

| Fact | Source evidence |
| --- | --- |
| Library exposes domain and terminal/application modules together and disclaims API stability | `tools/vfa-tui/src/lib.rs:3–25` |
| Terminal crates are unconditional dependencies; manifest declares Rust 1.75, not a measured locked-graph minimum | `tools/vfa-tui/Cargo.toml:1–6,22–44` |
| Catalog loading accumulates per-file errors, sorts entity IDs, and records hashes | `tools/vfa-tui/src/catalog/store.rs:47–62,77–141` |
| Invalid incremental reload keeps last valid data | `tools/vfa-tui/src/catalog/store.rs:144–160` |
| Search has reusable result/filter logic but no established benchmark | `tools/vfa-tui/src/search/fuzzy.rs:24–29,61–141` |
| Federation scanning, coverage, drift, dependency graphs and integrity services exist | `tools/vfa-tui/src/federation/scanner.rs:417,459`; `coverage.rs:108,221`; `drift.rs:114,182`; `dep_graph.rs:93,176–247`; `integrity.rs:93,122` in that directory |
| Policy evaluation and single-writer SQLite services exist | `tools/vfa-tui/src/policy/engine.rs:77–164`; `tools/vfa-tui/src/persistence/writer.rs:19–104` |
| Coverage, violations and audit TUI tabs currently receive empty data; settings falls through to overview | `tools/vfa-tui/src/app.rs:1144–1163` |
| Export defaults to dry-run; existing Node script remains authoritative | `tools/vfa-tui/src/models/export.rs:32–47,78–126` |
| Exporter dry-run currently lists selected names/counts, not per-path conflicts; destination expansion occurs afterward | `scripts/export-marketplace-agents.mjs:623–659` |
| Existing exporter documents residual symlink race exposure; preflight is not atomic write isolation | `scripts/export-marketplace-agents.mjs:275–285,663–708` |
| TUI validation runs existing npm scripts, not its separate DAG executor | `tools/vfa-tui/src/app.rs:691–702,732–770` |
| Headless reporting creates data directories/database and persists results | `tools/vfa-tui/src/headless/reporter.rs:399–416,502–506` |
| Existing roots require a VFA checkout; a desktop binary alone will not supply catalogs or scripts | `tools/vfa-tui/src/workspace/detect.rs:5–29`; `tools/vfa-tui/README.md:17–23` |
| Existing Windows path/argument support is incomplete | `tools/vfa-tui/src/paths.rs:73–120`; `tools/vfa-tui/src/security/validate.rs:5–8,31–41` |
| Unix cancellation targets process groups; other platforms kill only direct child | `tools/vfa-tui/src/subprocess/signal.rs:10–54` |
| Output is read as complete lines before truncation; full channels stop readers | `tools/vfa-tui/src/subprocess/executor.rs:60–119` |
| Existing Rust CI is Ubuntu-based; release matrix has Linux/macOS, no Windows | `.github/workflows/vfa-tui-ci.yml:32–76`; `.github/workflows/vfa-tui-release.yml:105–116` |
| Current writer drops individual SQL errors before flush acknowledgment; existing queue/flush success is not successful audit commit evidence | `tools/vfa-tui/src/persistence/writer.rs:199–230` |

**Inference:** shared engine reuse is credible, but directly depending on the whole
`vfa_tui::App` would couple the GUI to terminal behavior. A UI-independent shared
boundary needs design and tests; it does not exist merely because modules are
public. Reuse must preserve authoritative Node/Python logic rather than rewriting
it in Rust or a UI event handler.

## Proposed product decisions, not verified facts

1. Native desktop on Linux x86_64, macOS Apple Silicon and Windows x86_64; not a
   Windows-via-WSL substitute. Additional CPU targets need separate evidence.
2. Alpha uses an explicitly selected local VFA checkout and installed helper
   runtimes. Offline browsing does not require Node/Python or execute repo code.
   A self-contained bundled catalog/runtime product is a separate decision.
3. Include real backend capabilities, not just the currently populated TUI tabs:
   catalog, relationships/integrity, workspace coverage/drift/policy, gates,
   export preview/confirmation, audit and reports. Stage delivery without silently
   removing those requirements from the final feature.
4. No cloud operations, auto-updater, telemetry, web/mobile target, plugin runtime,
   background daemon or host configuration changes in this alpha.
5. Require native runtime and assistive-technology evidence per target; unsigned
   private alpha artifacts must be identified honestly. Public distribution and
   signing credentials require separate authorization.

The approval checkpoint covers these decisions and the exact acceptance criteria
in [requirements.md](requirements.md), not an unseen future implementation.
