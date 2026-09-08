# vfa-tui

Enterprise-grade terminal UI for the **Vanguard Frontier Agentic** marketplace catalog.

[![Crates.io](https://img.shields.io/crates/v/vfa-tui.svg)](https://crates.io/crates/vfa-tui)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

> **Status: v0.1.0** — Functional, secure, and tested. APIs and UI may change before 1.0.

## Installation

```bash
cargo install vfa-tui
```

Requires Rust 1.75 or later.

## Important Limitations

**`vfa-tui` is designed for use inside the [`vanguard-frontier-agentic`](https://github.com/VincentChuWaiChow/vanguard-frontier-agentic) monorepo.** The binary auto-detects the workspace root by walking up from the current directory and looking for the `catalog/` directory and `package.json`. Running it outside a checked-out copy of the repository will produce an error at startup.

**Library API stability:** The `vfa_tui` library crate is an internal implementation detail. All modules are marked `pub` only to allow integration and property tests in sibling crates to import them directly. The library API is **not covered by semver guarantees** — it may change between any releases.

## Pre-built Binaries

Tagged releases publish pre-compiled binaries for all supported platforms via GitHub Releases. Download from [Releases](https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/releases) and place the binary on your `$PATH`.

Available targets per release:

| File | Target |
|------|--------|
| `vfa-tui-x86_64-linux-gnu.tar.gz` | `x86_64-unknown-linux-gnu` |
| `vfa-tui-aarch64-linux-gnu.tar.gz` | `aarch64-unknown-linux-gnu` |
| `vfa-tui-x86_64-linux-musl.tar.gz` | `x86_64-unknown-linux-musl` (static) |
| `vfa-tui-x86_64-macos.tar.gz` | `x86_64-apple-darwin` |
| `vfa-tui-aarch64-macos.tar.gz` | `aarch64-apple-darwin` |

Each release also includes a `checksums.sha256` file and an SPDX 2.3 SBOM (`vfa-tui.spdx.json`).

## Overview

`vfa-tui` provides a fast, keyboard-driven terminal interface for browsing the VFA catalog of 300+ security agents across 30+ providers. It supports fuzzy search, provider/harness filtering, role-based views, validation gate execution with real-time streaming output, export command building with dry-run preview, and structured audit logging.

Key capabilities:

- Browse agents, skills, roles, providers, MCP references, and rules interactively
- Fuzzy search across all catalog entities (powered by nucleo-matcher)
- Run any of the 17+ validation gates with streaming output
- Build and preview export commands before execution
- Manage per-harness model/reasoning-effort policy for providers, roles, and agents — batch assignment with dry-run preview and automatic integrity refresh
- View asset integrity manifests with SHA-256 hashes
- Full audit trail via structured tracing logs

The TUI is a read-first interface — it parses catalog JSON files directly for browsing and wraps existing Node.js/Python validation and export scripts via subprocess execution. Write operations (exports) require explicit operator confirmation with dry-run as the default.

## Usage

Run from anywhere inside the checked-out repository (workspace is auto-detected):

```bash
vfa-tui
```

Or specify the workspace explicitly:

```bash
vfa-tui --workspace /path/to/vanguard-frontier-agentic
```

### CLI Flags

| Flag | Description | Default |
|------|-------------|---------|
| `--workspace <PATH>` | Path to the workspace root | Auto-detected from CWD |
| `--report <TYPE>` | Headless report (no TUI): `summary`, `coverage`, `all` | — |
| `--validate-config` | Validate config files and exit (0=ok, 2=invalid) | — |
| `--export-audit <FMT> <PATH>` | Export audit log to file | — |
| `--log-file <PATH>` | Path to the structured audit log file | None (stderr only) |
| `--log-level <LEVEL>` | Verbosity: `trace`, `debug`, `info`, `warn`, `error` | `info` |
| `--no-color` | Disable ANSI color codes in all output | `false` |
| `--theme <MODE>` | Color theme: `auto`, `dark`, `light` | `auto` |
| `--quiet` | Suppress informational messages | `false` |
| `--help` | Show help message and exit | — |
| `--version` | Show version and exit | — |

### Keyboard Navigation

| Key | Action |
|-----|--------|
| `↑`/`↓` or `j`/`k` | Move up/down in lists |
| `g`/`G` | Jump to top/bottom of list |
| `Enter` | Select item / confirm action |
| `Escape` | Go back / cancel |
| `Tab` | Switch between panels |
| `/` | Activate search |
| `p` | Cycle provider filter (agent list) |
| `h` | Cycle harness filter (agent list) |
| `m` | Open model policy builder (scope from current view) |
| `?` | Toggle keyboard shortcut overlay |
| `q` or `Ctrl+C` | Quit |

### Examples

```bash
# Basic launch — auto-detects workspace
vfa-tui

# Headless report (no TUI, exits immediately)
vfa-tui --report summary

# Run with debug logging to a file
vfa-tui --log-level debug --log-file /tmp/vfa-tui.log

# Run without color (useful for piping or screen readers)
vfa-tui --no-color

# Validate configuration files and exit
vfa-tui --validate-config

# Export audit log to JSON
vfa-tui --export-audit json /tmp/audit.json
```

## Model Policy

The **Model Policy** sidebar section and the Model Policy Builder manage per-harness model and reasoning-effort assignments for agents — batch scope changes across providers, roles, or individual agents without hand-editing `codex.toml` keys or `.agent.md` frontmatter. Agent detail views show a **Models** section with the resolved model/reasoning per harness and the rule that won.

Press `m` from anywhere to open the builder. Scope is prefilled from context — an agent detail or list view targets that agent, a provider view targets that provider, a role view targets that role, and any other view defaults to `all`. Consistent with the "no business logic duplication" design principle above, the builder never re-implements policy resolution: it shells out to `scripts/model-policy.mjs` for validation and apply, and on a successful non-dry-run apply it automatically chains `npm run asset-integrity:write`.

Builder flow:

- **Scope** — `Space`/`Enter` cycles kind (`All` → `Provider` → `Role` → `Agent`); type the ID
- **Harness** — cycles `codex` → `claude-code` → `cursor`
- **Model** — `Space`/`Enter` cycles the models `catalog/model-registry.json` verifies for the selected harness (closed-namespace allowlists first, then the advisory examples of open namespaces such as local Ollama tags), wrapping through a free-text slot; typing still accepts any value, since open namespaces are too large to enumerate. `auto` clears the managed field, empty leaves it untouched
- **Reasoning** — cycles `(unchanged)` / `auto` / the efforts the registry verifies for the selected harness *and* model. The vocabulary narrows model-first: a model's own `reasoning_efforts` wins, else its namespace's, else the harness's. Harnesses with no reasoning field (`cursor`) collapse the cycle and say why; a model that supports no effort (for example `claude-haiku-4-5`) keeps `auto`, since clearing an inherited effort is what the engine asks for in that case
- **Dry Run** — defaults to on; previews changes before writing
- **Refresh Integrity** — defaults to on; runs `npm run asset-integrity:write` after a successful apply
- `[ Continue ]` shows the exact command to be run, then the output view streams the subprocess

The builder reads that same registry to populate both pickers, so the values it offers are the values the script will accept. Everything entered here is still validated against the registry by `scripts/model-policy.mjs` before any file changes — an unknown model name or a reasoning effort the resolved model doesn't support is rejected up front, not discovered later as a provider-side failure. For `codex`, the builder accepts local Ollama models (`name:tag`) and OpenRouter models (`author/model`) in addition to OpenAI slugs, projecting the matching `model_provider` line automatically. See [`docs/model-policy-matrix.md`](../../docs/model-policy-matrix.md) for the full per-harness matrix.

## Workflows

The **Workflows** entry in the sidebar lists the executable workflows in `.claude/workflows/` — each one's name, invocation, description, when-to-use guidance, and per-phase model tiers — with a detail view per workflow.

The TUI does not parse those JavaScript files. `scripts/generate-workflow-catalog.mjs` extracts each script's `meta` block into `catalog/workflows.json` (`npm run workflow-catalog:write`, checked in CI by `npm run validate:workflow-catalog`), and the TUI deserializes that generated file like every other catalog surface. This is the read-first principle from the Architecture section: parsing and validation live in the Node generator; the TUI displays.

Display is read-only. The TUI never runs a workflow — that remains the harness's job.

## Supported Platforms

| Target | Architecture | Notes |
|--------|-------------|-------|
| `x86_64-unknown-linux-gnu` | Linux x86_64 | Primary development target |
| `aarch64-unknown-linux-gnu` | Linux aarch64 | ARM64 Linux (Graviton, RPi) |
| `x86_64-apple-darwin` | macOS x86_64 | Intel Macs |
| `aarch64-apple-darwin` | macOS aarch64 | Apple Silicon (M1/M2/M3) |
| `x86_64-unknown-linux-musl` | Linux musl | Static binary for WSL and containers |

### WSL Notes

Build with `--target x86_64-unknown-linux-musl` for a fully static binary. Windows Terminal provides the best experience (true color, proper resize events). Use `--no-color` for terminals with limited color support.

## Architecture

```
┌─────────────────────────────────────────────────────┐
│              Presentation Layer                       │
│  Terminal Manager (crossterm) ← UI Renderer (ratatui)│
│  Navigation State Machine ← Keybinding Dispatcher   │
├─────────────────────────────────────────────────────┤
│              Application Layer                        │
│  Browse Controller │ Validation Controller           │
│  Export Controller │ Search Engine (nucleo-matcher)  │
├─────────────────────────────────────────────────────┤
│              Domain Layer                             │
│  Catalog Store (in-memory, read-only)                │
│  Data Models (serde structs, deny_unknown_fields)    │
│  Security Module (sanitize + validate + redact)      │
├─────────────────────────────────────────────────────┤
│              Infrastructure Layer                     │
│  Subprocess Manager (tokio::process)                 │
│  Workspace Detector │ Audit Logger (tracing)         │
│  Terminal Manager (setup/restore/panic hook)          │
└─────────────────────────────────────────────────────┘
```

### Design Principles

1. **No business logic duplication** — wraps existing scripts via subprocess; never re-implements validation or export logic.
2. **Read-first, confirm-before-write** — catalog access is read-only. Exports require explicit confirmation with dry-run default.
3. **Security by construction** — no shell interpolation, no network access, no credential exposure, terminal escape sanitization on all rendered content.
4. **Deterministic rendering** — same inputs produce same outputs.
5. **Graceful degradation** — partial catalog loading on file errors; never panics on recoverable errors.

### Technology Stack

| Concern | Crate | Purpose |
|---------|-------|---------|
| Terminal rendering | `ratatui` 0.30 | Immediate-mode TUI framework |
| Terminal backend | `crossterm` 0.28 | Cross-platform terminal abstraction |
| CLI parsing | `clap` 4.x (derive) | Type-safe argument handling |
| JSON deserialization | `serde` + `serde_json` | Strict schema validation |
| Async runtime | `tokio` (rt-multi-thread) | Subprocess management |
| Structured logging | `tracing` + `tracing-subscriber` | Audit events |
| Error handling | `thiserror` + `anyhow` | Domain + application errors |
| Fuzzy matching | `nucleo-matcher` 0.3 | High-performance search |
| SQLite | `rusqlite` (bundled) | Audit log persistence |
| Property testing | `proptest` (dev) | Correctness verification |

## Development

### Prerequisites

- Rust 1.75+
- A checkout of the `vanguard-frontier-agentic` repository
- Node.js 18+ (for validation gate and export command execution)

### Build

```bash
cd tools/vfa-tui
cargo build --release
```

For a statically-linked binary:

```bash
cargo build --release --target x86_64-unknown-linux-musl
```

### Running Tests

```bash
cd tools/vfa-tui
cargo test
```

Property tests live in `tests/property/` and cover catalog loading, search invariants, security sanitization, workspace detection, and deterministic rendering across randomized inputs:

```bash
PROPTEST_CASES=1000 cargo test
```

### Linting and Formatting

```bash
cargo fmt --check   # Check only
cargo fmt           # Auto-format
cargo clippy -- -D warnings
```

### Full CI Check (local)

```bash
cd tools/vfa-tui
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo build --release
cargo publish --dry-run
```

## Security

The TUI enforces several security invariants:

- **No shell interpolation** — all subprocesses are spawned with arguments as arrays, never concatenated strings
- **No network access** — the binary makes zero network requests under any circumstance
- **No credential exposure** — secret environment variables are stripped before subprocess spawning and redacted from all output
- **Terminal escape sanitization** — all catalog data and subprocess output is sanitized against terminal injection attacks
- **Path validation** — all user-provided paths are canonicalized and checked for directory traversal

See [SECURITY.md](SECURITY.md) for the vulnerability disclosure policy.

## License

Licensed under either of:

- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your option.
