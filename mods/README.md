# Cross-harness mods

Mods are tool-event guards and observers, not agent prompts. Each `mods/<id>/mod.toml`
declares its events, harness support and runtime footprint. Shared logic lives in the
mod's `lib/core.mjs`; native Claude Code modules and Node command hooks import that
same logic. [The implementation plan](../.claude/plan/cross-harness-mods.md) records
the verified native API and [spec deviations](../.kiro/specs/cross-harness-mods/deviations.md).

## Capability matrix

Supported does not mean identical. Generated `<harness>/UNSUPPORTED.md` files explain
every declared unsupported or degraded event or surface.

| Mod | Reach | Claude Code native | Claude Code classic | Kiro | Codex |
|---|---|---|---|---|---|
| [secret-redactor](secret-redactor/README.md) | L0 | Ask on credential-shaped input; redact or withhold output | Ask on input; replace successful output; failed output is not covered | Deny matching input; output redaction unsupported | Deny matching input; output redaction unsupported |
| [blast-radius-guard](blast-radius-guard/README.md) | L0 | Ask in confirm mode; deny in block mode | Same decisions | Deny in either mode | Deny in either mode |
| [live-guard-gate](live-guard-gate/README.md) | L0 | Ask and name guard agents to consult | Same decision | Deny and name guard agents | Deny and name guard agents |
| [context-burn](context-burn/README.md) | L1 | Real session context usage; threshold toasts and status file | Tool-call counter only; successful calls only | Tool-call counter only | Tool-call counter only |
| [audit-trail](audit-trail/README.md) | L1 | Unsupported: no file append; use classic | Append a record after successful calls | Append with degraded payload interpretation | Append a record |

Guards run on `PreToolUse`; redaction and observers use `PostToolUse` where supported.
Context Burn also handles `SessionStart`. Native events are `tool.check`, `tool.call`
and `session.start`. Kiro's stdin field names are undocumented: guards fall back to the
whole payload when no input object is recognized. Audit hashes the whole payload as
arguments, hashes `null` as the result, and
needs `VANGUARD_AUDIT_TRAIL_SESSION_ID` to avoid the shared `unsessioned` fallback.

## Reach levels

The level is derived from the highest declared runtime capability. It is a disclosure,
not a sandbox or proof of safe behavior. Reading input or an extension file is allowed
at L0; launching Node to run an adapter is not a subprocess spawned by the mod itself.

| Level | Definition | Starter examples |
|---|---|---|
| L0 | No declared file writes, subprocess spawns or network egress | Secret Redactor, Blast Radius Guard, Live Guard Gate |
| L1 | File writes, but no declared subprocess spawns or network egress | Context Burn, Audit Trail |
| L2 | Subprocess spawns, but no declared network egress | None |
| L3 | Network egress | None; `validate:mods` warns, rather than blocks, on L3 |

## Installation

Start from a local checkout. No mod needs a running network service or a network call
to install. Native modules use the harness runtime; command adapters need Node 22+
on `PATH`. Set these absolute paths, and choose a mod from the matrix:

```bash
source=/absolute/path/to/vanguard-frontier-agentic
target=/absolute/path/to/your/project
mod=blast-radius-guard
```

**Never load both native and classic adapters for the same mod.** They duplicate checks
and observers; Context Burn would also share a status file and double count. Audit Trail
has no native adapter, so use its classic hook even if other mods use native modules.

### Claude Code native

Load the mod's plugin root for a session in the target project:

```bash
cd "$target"
claude --plugin-dir "$source/mods/$mod"
```

Do not use this command for `audit-trail`. Native modules use `hooks/register.ts` and
generated `hooks/hooks.json` plus `.claude-plugin/plugin.json`. The native API is early
access; check the dated implementation plan's verified build before upgrading it.

### Claude Code classic

Copy only `bin/` and `lib/`; generated pattern modules are already in `lib/`, so no
separate `patterns.json` or catalog copy is needed:

```bash
mkdir -p "$target/.claude/vanguard-mods/$mod"
cp -R "$source/mods/$mod/bin" "$source/mods/$mod/lib" "$target/.claude/vanguard-mods/$mod/"
```

Merge `mods/<id>/claude-code/settings.hooks.json` from the checkout into the target's
`.claude/settings.json`: append each event's hook entries under `hooks`, preserving
existing entries and other settings. Do not replace the entire settings file.
The generated command is:

```text
node "$CLAUDE_PROJECT_DIR/.claude/vanguard-mods/<id>/bin/hook.mjs" --harness claude-code --event <event>
```

`CLAUDE_PROJECT_DIR` stays fixed at the session's initial project root, including when
changing worktrees. The hooks themselves do not read it; observer write paths are
resolved from the installed adapter location.

### Kiro

```bash
mkdir -p "$target/.kiro/hooks" "$target/.kiro/vanguard-mods/$mod"
cp -R "$source/mods/$mod/bin" "$source/mods/$mod/lib" "$target/.kiro/vanguard-mods/$mod/"
cp "$source/mods/$mod/kiro/"*-hook.json "$target/.kiro/hooks/"
```

The generated command uses `.kiro/vanguard-mods/<id>/bin/hook.mjs` with
`--harness kiro --event <event>`. Hook process cwd is not documented: replace that
script path with the absolute installed path before relying on it. Copy only the
generated `*-hook.json` files, not `UNSUPPORTED.md`. A confirmation request is rendered
as a deny with exit 2; these adapters do not invent an ask or output-replacement protocol.

### Codex

```bash
mkdir -p "$target/.codex/vanguard-mods/$mod"
cp -R "$source/mods/$mod/bin" "$source/mods/$mod/lib" "$target/.codex/vanguard-mods/$mod/"
```

Merge `mods/<id>/codex/hooks.json` from the checkout into the target's `.codex/hooks.json`:
append each event's entries under `hooks` without replacing existing hooks. Replace
the generated relative `.codex/vanguard-mods/<id>/bin/hook.mjs` path with the absolute
installed path; hook process cwd is not documented. Commands use
`--harness codex --event <event>`. Complete Codex's hook trust review before relying on
them. Confirmation requests become denies with exit 2. This redactor's Post event is
unsupported on Codex: the general/built-in output replacement it needs is unavailable;
the narrower MCP output field is not used as a silent substitute.

### Verify the installation

In the actual harness, exercise one ordinary call and one harmless call carrying a
matching test pattern. Check the expected ask/deny, output replacement or observer
file—not just a process exit code. Repeat after changing paths, permissions, Node or
the harness version. Inspect unsupported/degraded notes before enabling additional mods.

## Limits and configuration

- **Not complete DLP.** Regexes miss formats they do not describe. Pre checks ask or
  deny; they never rewrite arguments. Approved arguments still reach the tool, and
  Post redaction happens after execution. It does not scrub other logs or network traffic.
  Classic output replacements may be ignored if their shape does not match the tool's
  output; see Secret Redactor's withholding limitations.
- **Not a cloud authorization system.** Live Guard Gate names agents for manual review;
  it never spawns one or verifies a cloud account is production. Guards scan string
  input, not shell semantics, and relative paths are not resolved against a cwd.
  An `ask` can be auto-approved by a bypass permission mode.
- **Fail closed only after execution reaches the hook.** Guard handler/input failures
  deny; redaction failures attempt to withhold output. Missing Node, moved scripts,
  startup crashes and hook timeouts can prevent enforcement. CLI hook errors can be
  non-blocking; Kiro IDE error handling differs. Verify the installed harness behavior.
- **Counters are not token spend.** Only native Context Burn reads real session context
  usage and emits threshold warnings when usable token/window data is present. Node
  adapters count tool events only. Their unlocked read-modify-write can lose increments.
- **Append is not immutability.** Audit Trail stores argument/result hashes and sequence
  numbers, not signed or tamper-evident evidence. Writers can edit or delete the log;
  hashes do not establish non-repudiation or compliance. Observer failures do not block
  the tool, and classic observers miss failed calls.
- **Footprints describe defaults.** Context Burn defaults to
  `.<harness>/context-burn-status.json` (`.claude` for Claude Code); Audit Trail defaults
  to `.audit/<UTC-date>/<session>.jsonl`, reusing an earlier file for the same session.
  `VANGUARD_CONTEXT_BURN_STATUS_PATH` and `VANGUARD_AUDIT_TRAIL_LOG_DIR` can point outside
  declared write globs. Overlap checks do not constrain those operator overrides.
- **Extensions add, never replace.** `VANGUARD_SECRET_REDACTOR_PATTERNS` and
  `VANGUARD_BLAST_RADIUS_GUARD_PATTERNS` name extra pattern files. An unreadable or invalid
  configured file denies guards or withholds redactor output. Use absolute extension
  paths to avoid cwd ambiguity. See each mod README for its schema and environment names.

## Development and gates

Edit `mod.toml`, source `patterns.json`, core logic, entries and native modules—not
generated adapters. `scripts/generate-mods.mjs` produces harness manifests, shared
runtime copies, pattern modules and `catalog/mods.json` deterministically.

```bash
npm run mods:write
npm run validate:mods
npm run test:fuzz
npm run asset-integrity:write  # last generator, on its own
npm run validate
```

The stdlib Python gate needs Python 3.11+ (`tomllib`). It checks declared reach,
event/surface coverage, namespace and footprint
contracts, generated-file drift, adapter syntax and production-core tests. Fuzz tests
exercise production exports, not a Python reimplementation. For each native mod, also
run `claude plugin validate mods/<id>` and `claude plugin test mods/<id>` locally;
CI does not provide the native CLI. Passing static checks does not prove a live harness
loaded a hook. Spelling and Markdown lint remain separate repository gates.
