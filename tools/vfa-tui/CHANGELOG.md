# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.19](https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/compare/vfa-tui-v0.0.18...vfa-tui-v0.0.19) - 2026-09-12

### Other

- *(vfa-tui)* document dynamic Istio discovery

## [0.0.18](https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/compare/vfa-tui-v0.0.17...vfa-tui-v0.0.18) - 2026-09-08

### Fixed

- *(security)* avoid credential-shaped test fixtures

## [0.0.17](https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/compare/vfa-tui-v0.0.16...vfa-tui-v0.0.17) - 2026-09-08

### Added

- *(model-policy)* per-model effort narrowing, model aliases, and Cursor parameter groups

## [0.0.16](https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/compare/vfa-tui-v0.0.15...vfa-tui-v0.0.16) - 2026-09-08

### Added

- *(model-policy)* refresh model registry and make the TUI pickers registry-backed

### Fixed

- *(model-policy)* close the Responses-route gap and the alias effort bypass

## [0.0.15](https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/compare/vfa-tui-v0.0.14...vfa-tui-v0.0.15) - 2026-09-01

### Other

- update Cargo.lock dependencies

## [0.0.14](https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/compare/vfa-tui-v0.0.13...vfa-tui-v0.0.14) - 2026-08-27

### Other

- update Cargo.lock dependencies

## [0.0.13](https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/compare/vfa-tui-v0.0.12...vfa-tui-v0.0.13) - 2026-08-22

### Other

- Merge origin/master into claude/snowflake-agent-board-arch-mb7ajm

## [0.0.12](https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/compare/vfa-tui-v0.0.11...vfa-tui-v0.0.12) - 2026-08-18

### Other

- update Cargo.lock dependencies

## [0.0.11](https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/compare/vfa-tui-v0.0.10...vfa-tui-v0.0.11) - 2026-08-18

### Added

- *(vfa-tui)* make the workflow catalog reachable, and normalize catalog paths
- *(vfa-tui)* surface workflows in the TUI via a generated catalog

## [0.0.10](https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/compare/vfa-tui-v0.0.9...vfa-tui-v0.0.10) - 2026-08-13

### Added

- *(schemas)* register the typescript provider across all code registration points

## [0.0.9](https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/compare/vfa-tui-v0.0.8...vfa-tui-v0.0.9) - 2026-08-04

### Fixed

- *(repo)* update GitHub owner references to VincentChuWaiChow

### Other

- *(cargo)* bump toml in /tools/vfa-tui in the cargo group

## [0.0.8](https://github.com/Raishin/vanguard-frontier-agentic/compare/vfa-tui-v0.0.7...vfa-tui-v0.0.8) - 2026-07-28

### Added

- *(python)* add static-review Python agent board (maestro + 4 specialists)

## [0.0.7](https://github.com/Raishin/vanguard-frontier-agentic/compare/vfa-tui-v0.0.6...vfa-tui-v0.0.7) - 2026-07-21

### Added

- *(kotlin)* add adversarial Kotlin agent board (16 agents + companion skills)

## [0.0.6](https://github.com/Raishin/vanguard-frontier-agentic/compare/vfa-tui-v0.0.5...vfa-tui-v0.0.6) - 2026-07-20

### Other

- update Cargo.lock dependencies

## [0.0.5](https://github.com/Raishin/vanguard-frontier-agentic/compare/vfa-tui-v0.0.4...vfa-tui-v0.0.5) - 2026-07-19

### Fixed

- *(vfa-tui)* classify java board assets in coverage provider inference
- *(vfa-tui)* register java in the Rust Provider enum; bump MSRV to 1.97

### Other

- Merge remote-tracking branch 'origin/master' into claude/java-agent-board-design-ym9ahq

## [0.0.4](https://github.com/Raishin/vanguard-frontier-agentic/compare/vfa-tui-v0.0.3...vfa-tui-v0.0.4) - 2026-07-17

### Fixed

- *(vfa-tui)* register php provider in enum, coverage inference, and property tests

## [0.0.3](https://github.com/Raishin/vanguard-frontier-agentic/compare/vfa-tui-v0.0.2...vfa-tui-v0.0.3) - 2026-07-11

### Added

- *(model-registry)* status-driven model lifecycle with successor fallback
- *(model-policy)* enforce verified model registry (fail-closed matrix)
- *(model-policy)* per-harness model/reasoning policy engine + vfa-tui builder

### Fixed

- *(security)* remediate PR #117 security + correctness review findings
- *(model-policy)* address CodeQL TOCTOU and Codex review feedback

### Other

- *(governance)* single canonical guide + orchestrator model rules

## [0.0.2](https://github.com/Raishin/vanguard-frontier-agentic/compare/vfa-tui-v0.0.1...vfa-tui-v0.0.2) - 2026-07-07

### Added

- *(tui)* detect frontend provider in vfa-tui

### Other

- *(cargo)* bump terminal-light in /tools/vfa-tui in the cargo group

## [0.0.1](https://github.com/Raishin/vanguard-frontier-agentic/compare/vfa-tui-v0.0.0...vfa-tui-v0.0.1) - 2026-06-25

### Fixed

- *(vfa-tui)* use RecommendedCache alias for debouncer field type
- *(vfa-tui)* fix watcher.rs type mismatch + immutable release uploads

## [0.0.0](https://github.com/Raishin/vanguard-frontier-agentic/releases/tag/vfa-tui-v0.0.0) - 2026-06-25

### Added

- *(vfa-tui)* prepare crates.io publication (v0.1.0)
- *(vfa-tui)* implement Task 15 — light/dark mode with system detection
- *(tui)* wire residual 1 (auto-persist coverage/drift) and residual 2 (v2 tab bar primary surface)
- *(vfa-tui)* wire watcher live-reload + render v2 operator-console tabs
- *(vfa-tui)* persist coverage/drift to SQLite + audit headless & trust overrides
- *(tui-v2)* test fixtures — reusable minimal valid examples
- *(tui-v2)* async event loop — tokio::select! multiplexing + dirty-flag rendering
- *(tui-v2)* integration + widgets — main.rs wiring + 6 TUI widgets
- *(tui-v2)* presentation tier — headless reporter, CLI, nav, fuzzy
- *(tui-v2)* engines tier — coverage, drift, versions, deps, integrity, policy
- *(tui-v2)* wave 5.5 + 7.14 — workspace scanner + gate DAG executor
- *(tui-v2)* wave 5 — catalog store + registry TOML parser
- *(tui-v2)* wave 3 — SQLite index, audit hash-chain, filesystem watcher
- *(tui-v2)* wave 3 — multi-harness workspace detection + fix gitignore
- *(tui-v2)* wave 1 — data models + fix catalog deserialization baseline
- *(tui-v2)* wave 1 — extend error hierarchy for new subsystems
- *(tui-v2)* wave 0 foundation — deps + module skeleton
- *(vfa-tui)* enhance UI layer with color detection and scroll support
- *(vfa-tui)* implement validation gate controller and export builder
- *(vfa-tui)* add tracing-subscriber time feature and structured audit logging
- validate rust tui workspace markers
- model skill catalog type
- model rule catalog type
- model mcp trust metadata
- model harness source metadata
- model agent catalog metadata
- expand tui structured errors
- *(tui)* add 8 TUI enhancements for v0.2.0
- integration tests, test fixtures, CI workflow, and TUI README
- UI layer - terminal manager, navigation, layout, theme, widgets, event loop, main entry point
- subprocess executor, export command model, validation gates, audit logging
- workspace detection, catalog store, security module, search engine, property tests
- scaffold tools/vfa-tui with CLI, error types, and data models

### Fixed

- *(vfa-tui)* remove unused Watcher import in watcher.rs
- *(vfa-tui)* update watcher.rs for notify-debouncer-full 0.7 API
- *(vfa-tui)* fix sha256_hex for sha2 0.11 — Array output lacks LowerHex
- *(vfa-tui)* add sap variant to Provider enum
- *(vfa-tui)* migrate deny.toml to cargo-deny v2 schema + allow MPL-2.0
- *(vfa-tui)* sync provider enum + gitignore artifact
- *(vfa-tui)* address code-review findings on Task 15 theme
- *(vfa-tui)* guard validation-gate spawn against missing Tokio runtime
- *(vfa-tui)* collapse sync_view_to_tab match guard for clippy 1.96
- *(vfa-tui)* address Codex PR review (4x P2)
- *(vfa-tui)* wire legacy catalog UI into tabs instead of dead-code allow
- *(vfa-tui)* green the v2 build and record spec verification status
- *(tui-v2)* resolve clippy 1.96 collapsible_match in policy parser
- *(tui-v2)* resolve clippy -D warnings, rustfmt, and codespell CI failures
- resolve 10 critical bugs identified by Codex review on PR #70
- *(tui-v2)* correct invalid proptest regex in catalog reload property
- *(vfa-tui)* security hardening and spec updates from review
- *(vfa-tui)* address 8 Codex code review findings
- render enum detail fields
- stringify validation error paths
- align provider catalog model
- normalize catalog loader error paths
- *(tui)* reduce max catalog file size from 100MB to 20MB
- *(tui)* truncate subprocess output lines exceeding 4096 bytes
- *(tui)* enable kill_on_drop for subprocess to prevent orphaned processes
- *(tui)* correct export args construction and harden gate extraction
- improve file size rejection test to exercise error path
- address 7 security findings in vfa-tui with tests
- address security review findings for vfa-tui

### Other

- *(vfa-tui)* lower version to 0.0.0 to bootstrap release-plz PR flow
- *(vfa-tui)* rustfmt import collapse in watcher.rs
- *(cargo)* bump the cargo group in /tools/vfa-tui with 7 updates
- *(vfa-tui)* apply rustfmt and fix codespell findings on theme tests
- *(vfa-tui)* de-flake watcher debounce test bound
- *(vfa-tui)* apply rustfmt to satisfy the 'check' (cargo fmt) CI gate
- *(vfa-tui)* add integration tests 13.3-13.6 + clean clippy
- *(vfa-tui)* backfill 17 missing property tests (rust-tui-v2 spec)
- *(fmt)* fix rustfmt formatting across vfa-tui module
- *(tui-v2)* wave 2 — security property tests (Req 20/21/22)
- *(deps)* update zerocopy 0.8.49 → 0.8.50 in vfa-tui
- *(vfa-tui)* apply cargo fmt to fix CI formatting check
- *(vfa-tui)* comprehensive README and mark spec tasks complete
- *(vfa-tui)* add property-based tests for 17 correctness invariants
- *(vfa-tui)* expand integration tests and fixtures
- strengthen workspace detection properties
- strengthen ui detail properties
- strengthen validation properties
- strengthen sanitization properties
- strengthen secret redaction properties
- re-export tui model types
- wire tui model exports
- *(tui)* commit proptest regression seeds for export args
- *(tui)* bump version to 0.2.0
- *(tui)* fix export args tests to match --agents=id1,id2 format
- *(tui)* use recursive value inspection for catalog taint checks

### Security

- *(tui)* use pidfd for race-free process signaling on Linux 5.3+
- *(tui)* harden secret redaction with JWT, PEM, Slack, and ANSI-aware detection
- *(tui)* extend escape sanitization to cover Unicode C1 controls (U+0080-U+009F)

## Versioning Policy (pre-1.0)

While this crate is below `1.0.0`, the public surface is the **binary CLI**, not
the library API. Per the pre-1.0 semver convention:

- **Minor** version bumps (`0.X.0`) may include breaking changes to the CLI or
  the internal library API.
- **Patch** version bumps (`0.1.X`) are reserved for backwards-compatible bug
  fixes and non-breaking additions.

The library API is internal and **not** covered by semver guarantees.

## [0.1.0] - 2026-06-19

### Added

- Initial public release on crates.io
- Interactive TUI for browsing the VFA catalog (agents, skills, roles, providers)
- Fuzzy search across all catalog entities
- Validation gate execution with streaming output
- Export command builder with dry-run preview
- Headless reporting mode (JSON/text/summary)
- Cross-platform support (Linux x86_64/aarch64, macOS x86_64/aarch64, musl)
- Structured audit logging
- Terminal escape sanitization for security
