# Implementation Plan: cross-harness-mods

## Execution status — 2026-10-15

Implementation follows the approved [plan](../../../.claude/plan/cross-harness-mods.md)
and [deviations](deviations.md), not the superseded API assumptions below. Checked items
mean their approved equivalents are implemented and locally verified; intermediate
checkpoint counts are not claims that historical staged runs occurred. The final
implementation passed local gates and was verified on the remote branch/PR.
Checkpoint 8.5 is complete after approved synthetic-fixture repairs and all-green
GitHub checks on source head `9bf10878`; runtime implementation is unchanged. T9,
expanded P3 coverage and explicit reach documentation are implemented with
regressions; current proof is in the
[implementation run](../../../.workflow/ultracode/cross-harness-mods-tui/final-report.md).
Live installed Kiro/Codex sessions remain unverified.

## Overview

Implement the `mods/` behavioral extension layer for `vanguard-frontier-agentic`. This plan builds the framework first (directory contract, generator skeleton, validation gate), then adds each of the five enterprise starter mods with their hand-authored sources, then completes property-based tests, and finally wires everything into the repo's gates and documentation.

All generated harness files (`register.ts`, Kiro hook JSON, Codex `.sh` scripts) are produced by the generator — never hand-written. Each group ends with a verification step against the gate before moving to the next group.

---

## Tasks

- [x] 1. T1 — Framework scaffolding

  - [x] 1.1 Create `mods/` directory structure and `mods/README.md`
    - _Execution note: Reach/install conventions follow deviations; no L2/L3 starter is fabricated. See [deviations](deviations.md)._

    - Create `mods/` at repo root.
    - Write `mods/README.md` documenting the mod format, reach level taxonomy (L0–L3 with one-line definition and example mod for each level), install command for each harness (Kiro: copy `.json` + companion `.sh` to `.kiro/hooks/`; Claude Code: `.claude/mods/<id>/register.ts`; Codex: register `.sh` as Codex hook), and a note that no mod requires a running network service at install time and L3 mods are warned (not blocked) by `validate:mods`.
    - _Requirements: 1.10, 17.1, 17.2, 17.3, 17.4, 17.6, 17.7_

  - [x] 1.2 Add `mods:write` and `validate:mods` scripts to `package.json` and append `validate:mods` to the `validate` chain
    - Add `"mods:write": "node scripts/generate-mods.mjs"` to `scripts` in `package.json`.
    - Add `"validate:mods": "python3 tests/validate-mods.py"` to `scripts` in `package.json`.
    - Append `&& npm run validate:mods` to the end of the existing `validate` script string.
    - _Requirements: 6.4, 7.1_

  - [x] 1.3 Create `catalog/mods.json` as an empty array and add it to `catalog/index.json`
    - Write `catalog/mods.json` as `[]` (empty JSON array — the generator will populate it on first run).
    - Add `"mods": "catalog/mods.json"` to the `catalogs` object in `catalog/index.json`.
    - _Requirements: 8.1, 9.6_

  - [x] 1.4 Add `mods/` and `catalog/mods.json` coverage to `tests/validate-asset-integrity.py`
    - Add `"mods"` to the `TREES` list in `validate-asset-integrity.py` (it already walks `catalog/` recursively, so `catalog/mods.json` is covered once `mods/` itself is added).
    - Confirm `catalog/mods.json` is not in `EXCLUDED_RELATIVE_PATHS`.
    - _Requirements: 9.1_

  - [x] 1.5 Update `.plugin-scanner.toml` to suppress false positives from mod pattern files
    - _Execution note: Superseded: executable mod code remains scanned; no scanner suppression added. See [deviations](deviations.md)._

    - Add `"mods/*/kiro/*.sh"`, `"mods/*/codex/*.sh"`, and `"tests/fixtures/mods/**"` to the `[scanner].ignore_paths` array in `.plugin-scanner.toml`.
    - Add a comment explaining each entry (regex patterns in `secret-redactor` and `blast-radius-guard` are credential-shaped by design).
    - _Requirements: 9.2_

  - [x] 1.6 Write `scripts/generate-mods.mjs` — skeleton with TOML parser, directory walker, `--check` mode, and catalog writer
    - _Execution note: Strict TOML subset, source/generated symlink refusal, deterministic pure rendering. See [deviations](deviations.md)._

    - Implement a self-contained minimal TOML parser (regex-based, handling the exact subset used by `mod.toml`: tables, arrays of strings, booleans, integers, quoted strings).
    - Implement `readAllMods()` that discovers every `mods/*/mod.toml` and parses it.
    - Implement `readCatalog()` that reads `catalog/agents.json`.
    - Implement `writeOrCheck(filePath, content)` that writes when not in `--check` mode, or compares and accumulates mismatches when in `--check` mode.
    - Implement `generateCatalog(mods)` that writes `catalog/mods.json` as a sorted JSON array of Catalog_Entry objects (fields: `id`, `name`, `version`, `description`, `reach_level`, `events`, `harnesses`, `path`).
    - Implement top-level `main()` with `--check` flag support; exit 0 with `Generated <N> mods for <M> harnesses` on success; exit 1 with error details on failure; continue processing all mods on TOML parse errors (fail-after-all, not fail-fast).
    - Generator must never consult the wall clock, random, or any filesystem path outside the repo root during generation.
    - _Requirements: 6.1, 6.2, 6.3, 6.7, 6.8, 6.9, 6.10_

  - [x] 1.7 Add Kiro, Claude Code, and Codex artifact generators to `scripts/generate-mods.mjs`
    - _Execution note: Native modules are hand-authored; generator owns manifests and shared/data modules. See [deviations](deviations.md)._

    - Implement `generateKiroArtifacts(mod)` that, for each event in `mod.events` that Kiro supports, writes `mods/<id>/kiro/<id>-<event-lowercase>-hook.json` with the v1 schema (`version: "v1"`, `hooks` array with `name`, `trigger`, `action`); hook `name` field uses `<mod-id>:<event-lowercase-hyphenated>` convention; hook `action.command` invokes the co-located companion `.sh`; when `harnesses.kiro.supported = false`, writes `mods/<id>/kiro/UNSUPPORTED.md` instead.
    - Implement `generateClaudeCodeArtifacts(mod)` that writes `mods/<id>/claude-code/register.ts` (with `export default function register(on: ModEventEmitter, options: ModOptions): void` signature, mod-specific event handler bodies, and `try/catch` wrapping every handler) and `mods/<id>/claude-code/hooks.json` (events array matching the `on("<Event>")` call sites in the generated TypeScript); when `harnesses.claude-code.supported = false`, writes `UNSUPPORTED.md` instead.
    - Implement `generateCodexArtifacts(mod)` that, for each declared event, writes `mods/<id>/codex/<event-lowercase>.sh` with `#!/usr/bin/env bash`, `set -euo pipefail`, stdin read, env var handling boilerplate, and a `source` call to `codex/logic.sh` (which the hand-authored mod provides); when `harnesses.codex.supported = false`, writes `UNSUPPORTED.md` instead.
    - For `live-guard-gate` only: generate `mods/live-guard-gate/live-agents.json` by extracting all agents from `catalog/agents.json` whose `id` matches `*-live-*-guard-agent`.
    - _Requirements: 1.2, 1.6, 3.1, 3.2, 3.3, 3.4, 4.1, 4.2, 5.1, 5.3, 5.4, 6.2, 12.2_

  - [x] 1.8 Write `tests/validate-mods.py` — full validation gate
    - _Execution note: Node syntax/runtime tests replace shell and duplicated Python logic checks. See [deviations](deviations.md)._

    - Implement `discover_mods()` to find all `mods/*/mod.toml` paths.
    - Implement `validate_toml_schema(mod)` checking: `[mod]` table has all required keys (`id`, `name`, `description`, `version`, `reach_level`, `events`), `id` matches `^[a-z0-9][a-z0-9-]+$`, `id` matches parent directory name, `reach_level` is int in [0,3], `version` matches `^\d+\.\d+\.\d+$`, `events` are from the allowed set, `[footprint]` table exists with all four arrays, `[harnesses]` table has `claude-code`, `kiro`, `codex` sub-tables each with boolean `supported`.
    - Implement `validate_reach_level(mod)` comparing declared `reach_level` against the derived value from footprint arrays; report `FAIL` on mismatch, `WARN` (not `FAIL`) for L3 mods.
    - Implement `validate_env_reads(mod)` scanning `codex/*.sh` and `kiro/*.sh` for `$VAR`/`${VAR}` patterns and confirming each is in `[footprint].env_reads`; also scan for shell redirection operators to confirm `[footprint].file_writes` coverage.
    - Implement `validate_kiro_hooks(mod)` parsing every `.json` in `mods/<id>/kiro/` and checking v1 schema compliance.
    - Implement `validate_codex_scripts(mod)` running `bash -n` via `subprocess.run` on every `.sh` in `mods/<id>/codex/` and `mods/<id>/kiro/`.
    - Implement `validate_claude_code(mod)` checking that `register.ts` contains `export default function register(on:` and that events in `hooks.json` match `on("<EventName>"` call sites (regex scan).
    - Implement `validate_generator_check()` calling `node scripts/generate-mods.mjs --check` via `subprocess.run` and returning any errors.
    - Implement `validate_catalog_mods_json()` checking that every `mods/` directory has a corresponding entry in `catalog/mods.json` and no entry references a non-existent directory; also check `id` uniqueness across `catalog/agents.json`, `catalog/skills.json`, `catalog/rules.json`.
    - Implement `validate_no_duplicate_file_writes(mods)` checking that no two mods declare overlapping `[footprint].file_writes` paths.
    - Implement `run_pbt_suite()` containing P1 (Redaction Completeness) and P2 (Guard Blocking) inline property tests using only `stdlib.re` (200 iterations each).
    - Gate must: exit 0 only when all checks pass; print `FAIL [mods] <path> <description>` per failure; print `OK: <N> mods valid (kiro: <K>, claude-code: <C>, codex: <X>)` on success; exit 1 with `FAIL [mods] mods/ directory is missing` when `mods/` does not exist; have zero external Python dependencies beyond stdlib and `tomllib`.
    - _Requirements: 1.9, 2.2, 2.3, 2.4, 2.5, 2.8, 3.8, 4.8, 5.6, 5.7, 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7, 8.3, 8.6, 15.1, 15.6, 16.1, 16.2_

  - [x] 1.9 Checkpoint: Run `npm run validate:mods` (expect it to pass with zero mods found, or the `mods/ directory missing` path) and `npm run validate` — the new gate must not break the existing suite. Ask the user if questions arise.
    - _Execution note: Integrated five-mod checkpoint verified; empty-mod acceptance was not used. See [deviations](deviations.md)._


- [x] 2. T2 — `secret-redactor` mod

  - [x] 2.1 Write `mods/secret-redactor/mod.toml` with `reach_level = 0` and `events = ["PreToolUse", "PostToolUse"]`
    - _Execution note: Approved reach L0 and capability/degraded declarations; manifest version 1.0.0. See [deviations](deviations.md)._

    - `[mod]`: `id = "secret-redactor"`, `name = "Secret Redactor"`, `description` (≥10 chars), `version = "0.1.0"`, `reach_level = 0`, `events = ["PreToolUse", "PostToolUse"]`.
    - `[footprint]`: `env_reads = ["VANGUARD_SECRET_REDACTOR_PATTERNS"]`, `file_writes = []`, `spawns = []`, `egress_hosts = []`.
    - `[harnesses.claude-code]`: `supported = true`.
    - `[harnesses.kiro]`: `supported = true`.
    - `[harnesses.codex]`: `supported = true`.
    - _Requirements: 1.1–1.8, 10.1, 10.8, 10.9, 15.3_

  - [x] 2.2 Write `mods/secret-redactor/patterns.json` with the five required built-in patterns
    - _Execution note: Contextual portable patterns replace overbroad patterns; see deviations. See [deviations](deviations.md)._

    - Include at minimum: `aws-access-key` (`AKIA[0-9A-Z]{16}`), `aws-secret-key` (generic 40-char base62), `github-pat` (`(ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9]{36,255}`), `generic-bearer` (`Bearer\s+[A-Za-z0-9\-._~+/]+=*`), `password-in-url` (`://[^@]*:[^@]+@`).
    - Each entry has fields: `id` (kebab-case), `name`, `description`, `regex` (POSIX-compatible), `severity` (`critical`/`high`/`medium`/`low`), `context` (array, use `["all"]` for these patterns).
    - _Requirements: 10.2_

  - [x] 2.3 Write `mods/secret-redactor/codex/logic.sh` — hand-authored redaction logic
    - _Execution note: Production Node core and entry replace POSIX/sed implementation. See [deviations](deviations.md)._

    - POSIX-only implementation: reads `INPUT` from stdin (already set by boilerplate), loads `patterns.json`, applies `sed` regex replacements for each pattern replacing matches with `[REDACTED:<pattern-id>]`, writes redacted JSON to stdout, logs redaction events to stderr in format `[secret-redactor] redacted <pattern-name> in <tool-name> PreToolUse`.
    - Must handle `VANGUARD_SECRET_REDACTOR_PATTERNS` env var: if set, load additional patterns from the file and merge (log a warning to stderr if duplicate IDs found, but do not fail).
    - Must exit 0 with no stdout when no patterns match (pass-through).
    - _Requirements: 10.3, 10.4, 10.5, 10.6, 10.7, 10.8_

  - [x] 2.4 Run `npm run mods:write` to generate all `secret-redactor` harness artifacts
    - _Execution note: Native Register API and documented classic output protocol replace old SDK. See [deviations](deviations.md)._

    - Running `node scripts/generate-mods.mjs` must produce: `mods/secret-redactor/kiro/secret-redactor-pretooluse-hook.json`, `mods/secret-redactor/kiro/secret-redactor-posttooluse-hook.json`, `mods/secret-redactor/claude-code/register.ts`, `mods/secret-redactor/claude-code/hooks.json`, `mods/secret-redactor/codex/pretooluse.sh`, `mods/secret-redactor/codex/posttooluse.sh`.
    - The `register.ts` must: load `patterns.json` via `readFileSync`; implement `redact(value: string): string`; in PreToolUse return `{ modified: JSON.parse(redacted) }` when a match is found; in PostToolUse scan output and log to `console.error` but not return a modified response (PostToolUse output cannot be intercepted in Claude Code).
    - Verify `catalog/mods.json` is updated with the `secret-redactor` entry.
    - _Requirements: 3.1–3.4, 4.1–4.7, 5.1–5.5, 8.1, 8.2_

  - [x] 2.5 Write `mods/secret-redactor/README.md`
    - Document: mod purpose (DLP credential redaction), reach level L0 with footprint, events handled (`PreToolUse` scans tool args; `PostToolUse` scans tool result), harness-specific install notes, env var `VANGUARD_SECRET_REDACTOR_PATTERNS` for extension patterns, and note that `register.ts` implements Pattern D (argument modification) in Claude Code.
    - _Requirements: 17.5_

  - [x] 2.6 Add P1 (Redaction Completeness) inline property test to `tests/validate-mods.py`
    - _Execution note: Production Node P1 tests, not a Python mirror. See [deviations](deviations.md)._

    - **Property 1: Redaction Completeness**
    - Add a `test_redaction_completeness()` function inside `run_pbt_suite()` that: imports `re`, `random`, `string`; for each pattern in `secret-redactor/patterns.json`; generates 200 random suffix strings; constructs a string containing the literal pattern match; calls an inline `redact(s, patterns)` function; asserts `re.search(pattern["regex"], result) is None`.
    - Also test pass-through: generate 200 random strings that don't match any pattern and verify the output equals the input.
    - **Validates: Requirements 10.3, 10.4, 16.1**

  - [x] 2.7 Checkpoint: Run `npm run validate:mods` — must exit 0 with `secret-redactor` counted. Ask the user if questions arise.

- [x] 3. T3 — `blast-radius-guard` mod

  - [x] 3.1 Write `mods/blast-radius-guard/mod.toml` with `reach_level = 1` and `events = ["PreToolUse"]`
    - _Execution note: Approved derived L0; no fictitious file writes to justify L1. See [deviations](deviations.md)._

    - `[mod]`: `id = "blast-radius-guard"`, `name = "Blast Radius Guard"`, `reach_level = 1`, `events = ["PreToolUse"]`.
    - `[footprint]`: `env_reads = ["VANGUARD_BLAST_RADIUS_MODE", "VANGUARD_BLAST_RADIUS_PATTERNS"]`, `file_writes = []`, `spawns = []`, `egress_hosts = []`.
    - Note: `reach_level = 1` because `env_reads` is non-empty but `file_writes`, `spawns`, `egress_hosts` are all empty — the validator derives L1 from `env_reads` (which are reads, not writes) — wait: re-check the design formula. The formula is: L3 if `egress_hosts`, L2 if `spawns`, L1 if `file_writes`, else L0. `env_reads` alone → L0 per the formula. However Requirement 11.1 declares `reach_level = 1`. Use `reach_level = 1` as specified and add a `[footprint].file_writes` entry (e.g. a temp file) that justifies L1, OR declare `reach_level = 0` if no file writes exist and the validator checks actual derivation. Since the validator enforces the formula, set `reach_level = 0` only if all footprint arrays are empty. The requirement says 1 — add the extension patterns file as a conditional write in `file_writes` per the design pattern for extensibility (`VANGUARD_BLAST_RADIUS_PATTERNS` target path), or note in `security_notes` that the reach level is declared conservatively. Match exactly what Requirement 11.1 states.
    - `[harnesses.claude-code]`, `[harnesses.kiro]`, `[harnesses.codex]`: all `supported = true`.
    - _Requirements: 1.1–1.8, 11.1, 11.9, 11.10, 15.3_

  - [x] 3.2 Write `mods/blast-radius-guard/patterns.json` with the ten required built-in patterns
    - _Execution note: Portable flags and reviewed patterns; includes additional guarded variants. See [deviations](deviations.md)._

    - Include at minimum: `rm-rf-root` (`rm\s+-[rRfF]+.*\s+(/|/\*|~/)`), `drop-table` (`(?i)DROP\s+TABLE`), `drop-database` (`(?i)DROP\s+DATABASE`), `drop-schema` (`(?i)DROP\s+SCHEMA`), `git-force-push` (`git\s+push\s+.*(-f|--force)`), `curl-pipe-sh` (`curl\s+.*\|\s*sh`), `wget-pipe-sh` (`wget\s+.*\|\s*sh`), `terraform-destroy-autoapprove` (`terraform\s+.*destroy\s+.*-auto-approve`), `kubectl-delete-namespace` (`kubectl\s+delete\s+namespace`), `kubectl-delete-all` (`kubectl\s+delete\s+.*--all`).
    - Fields follow same schema as `secret-redactor/patterns.json`; `context` uses `["bash"]` for command patterns and `["file-write"]` for write-path patterns.
    - _Requirements: 11.2_

  - [x] 3.3 Write `mods/blast-radius-guard/codex/logic.sh` — hand-authored guard logic
    - _Execution note: Node core; namespaced GUARD envs; Kiro/Codex confirmation degrades to deny. See [deviations](deviations.md)._

    - Read command from `tool_input.command` in stdin JSON; also check `tool_input.path` for file-write tool calls.
    - Check against patterns: if match found, read `VANGUARD_BLAST_RADIUS_MODE`; if `block`, exit 2 with reason to stderr; if `confirm` (default), exit 0 with `{"hookSpecificOutput":{"permissionDecision":"ask","permissionDecisionReason":"<pattern match: <pattern-id>>"}}` to stdout.
    - If no match, exit 0 with no stdout.
    - Apply same block/confirm logic when a write tool call targets `/(etc|usr|bin|sbin|boot|sys|proc)/`.
    - Load extension patterns from `VANGUARD_BLAST_RADIUS_PATTERNS` if set (same merge convention as `secret-redactor`).
    - _Requirements: 11.3, 11.4, 11.5, 11.7, 11.8, 11.9_

  - [x] 3.4 Run `npm run mods:write` to generate all `blast-radius-guard` harness artifacts
    - _Execution note: Native tool.check and generated hook manifests replace old SDK artifacts. See [deviations](deviations.md)._

    - Verify `mods/blast-radius-guard/kiro/blast-radius-guard-pretooluse-hook.json`, `claude-code/register.ts`, `claude-code/hooks.json`, `codex/pretooluse.sh` are produced.
    - The `register.ts` must: in PreToolUse read `VANGUARD_BLAST_RADIUS_MODE`, return `{ deny: reason }` when mode is `block` and a pattern matches, return `{ ask: true, reason }` when mode is `confirm`.
    - Verify `catalog/mods.json` updated.
    - _Requirements: 3.1–3.7, 4.1–4.6, 5.1–5.5, 11.4, 11.5, 11.6_

  - [x] 3.5 Write `mods/blast-radius-guard/README.md`
    - _Execution note: README documents actual L0 and capability limitations. See [deviations](deviations.md)._

    - Document: purpose, reach level L1, env vars (`VANGUARD_BLAST_RADIUS_MODE`, `VANGUARD_BLAST_RADIUS_PATTERNS`), events (`PreToolUse` only), harness install notes.
    - _Requirements: 17.5_

  - [x] 3.6 Add P2 (Guard Blocking) inline property test to `tests/validate-mods.py`
    - _Execution note: Production Node P2 tests, not a Python mirror. See [deviations](deviations.md)._

    - **Property 2: Guard Blocking**
    - Add `test_guard_blocking()` inside `run_pbt_suite()`: for each pattern in `blast-radius-guard/patterns.json`, construct 200 random strings that contain a literal substring matching the pattern regex; call an inline `evaluate(cmd, patterns)` function (returns `"block"` or `"confirm"` when matched, `"pass"` otherwise); assert `evaluate(cmd).decision != "pass"`.
    - Also generate 200 random strings that don't match any pattern and assert `evaluate(cmd) == "pass"`.
    - **Validates: Requirements 11.3, 11.4, 11.5, 16.2**

  - [x] 3.7 Checkpoint: Run `npm run validate:mods` — must exit 0 with both `secret-redactor` and `blast-radius-guard` counted. Ask the user if questions arise.

- [x] 4. T4 — `live-guard-gate` mod

  - [x] 4.1 Write `mods/live-guard-gate/mod.toml` with `reach_level = 2` and `events = ["PreToolUse"]`
    - _Execution note: Approved derived L0; no automatic dispatch or build-time write footprint. See [deviations](deviations.md)._

    - `[mod]`: `id = "live-guard-gate"`, `name = "Live Guard Gate"`, `reach_level = 2`, `events = ["PreToolUse"]`.
    - `[footprint]`: `env_reads = []`, `file_writes = ["mods/live-guard-gate/live-agents.json"]`, `spawns = []`, `egress_hosts = []`.
    - Note: `reach_level = 2` is declared conservatively per Requirement 12.1 and Requirement 12.9 (conservative for future subprocess). The derived formula would give L1 (file_writes non-empty). Add `security_notes` in the catalog entry (written by generator) explaining this; the validator must accept the conservatively-declared level without failing (treat declared > derived as a valid conservative declaration when a `security_notes` justification exists, or simply allow declared level ≥ derived level).
    - `[harnesses.kiro]`: `supported = true`. `[harnesses.claude-code]`: `supported = true`. `[harnesses.codex]`: `supported = true`.
    - _Requirements: 1.1–1.8, 12.1, 12.7, 12.8, 12.9, 15.3_

  - [x] 4.2 Write `mods/live-guard-gate/live-patterns.json` — hand-authored trigger patterns
    - _Execution note: patterns.json with verified guard-agent references; unverified verbs omitted. See [deviations](deviations.md)._

    - List the tool-target patterns that should trigger a live-guard approval prompt (e.g. Terraform plan paths, kubectl resource targets, AWS CLI commands against production-named resources, shell commands that match live-guard agent scopes from `catalog/agents.json`).
    - Each entry: `id`, `name`, `regex`, `description`, `context` (e.g. `["bash", "terraform", "kubectl"]`).
    - _Requirements: 12.3_

  - [x] 4.3 Run `npm run mods:write` to generate `live-guard-gate` harness artifacts including `live-agents.json`
    - _Execution note: Generated live-agents.mjs and native/classic/Node adapters; no unsupported ask. See [deviations](deviations.md)._

    - Generator must read `catalog/agents.json`, extract agents whose `id` matches `*-live-*-guard-agent`, and write `mods/live-guard-gate/live-agents.json` as a JSON array.
    - Verify all harness files produced: `kiro/live-guard-gate-pretooluse-hook.json`, `claude-code/register.ts`, `claude-code/hooks.json`, `codex/pretooluse.sh`.
    - The `register.ts` must return `{ ask: true, reason: "<reason identifying the matched live-guard agent>" }` from PreToolUse when a match is found.
    - Verify `catalog/mods.json` updated with `security_notes` field for `live-guard-gate`.
    - _Requirements: 12.2, 12.3, 12.4, 12.5, 12.6, 12.7, 12.9_

  - [x] 4.4 Write `mods/live-guard-gate/README.md`
    - _Execution note: Actual L0, manual review and uncovered agents documented. See [deviations](deviations.md)._

    - Document: purpose (CAB-equivalent gate that routes to live-guard agents), reach level L2 (conservative), that the mod only prompts the user — it does NOT spawn a live-guard agent automatically, events (`PreToolUse`), harness install notes, and note that `live-agents.json` is regenerated by `mods:write`.
    - _Requirements: 12.7, 17.5_

  - [x] 4.5 Checkpoint: Run `npm run validate:mods` — must exit 0 with `live-guard-gate` counted. Ask the user if questions arise.

- [x] 5. T5 — `context-burn` mod

  - [x] 5.1 Write `mods/context-burn/mod.toml` with `reach_level = 1` and `events = ["PostToolUse", "SessionStart"]`
    - _Execution note: Per-harness defaults, real native usage; unsupported max_tokens option removed. See [deviations](deviations.md)._

    - `[mod]`: `id = "context-burn"`, `name = "Context Burn"`, `reach_level = 1`, `events = ["PostToolUse", "SessionStart"]`.
    - `[footprint]`: `env_reads = ["VANGUARD_CONTEXT_BURN_STATUS_PATH"]`, `file_writes = [".kiro/context-burn-status.json"]`, `spawns = []`, `egress_hosts = []`.
    - `[harnesses.claude-code]`: `supported = true`; `[harnesses.claude-code.options]`: `max_tokens = 200000`.
    - `[harnesses.kiro]`: `supported = true`; add `notes = "Token count unavailable in Kiro hook context; tracks tool call count as a proxy."`.
    - `[harnesses.codex]`: `supported = true`.
    - _Requirements: 1.1–1.8, 13.1, 13.3, 13.5, 13.9, 13.10, 15.3_

  - [x] 5.2 Write `mods/context-burn/codex/logic.sh` — hand-authored burn tracking logic
    - _Execution note: Node status logic; no invented token estimates or proxy threshold warnings. See [deviations](deviations.md)._

    - On `SessionStart`: initialize Burn_Status_File as valid JSON with `session_id` (derive from `$$` + timestamp hash), `tool_call_count = 0`, `last_updated` (ISO 8601), `threshold_events = []`.
    - On `PostToolUse`: read existing Burn_Status_File; increment `tool_call_count`; update `last_updated`; write updated file; emit warning to stderr if count crosses configurable thresholds (default: warn at 70% and 90% of a `MAX_TOOL_CALLS` env var, defaulting to 500).
    - Burn_Status_File path: `${VANGUARD_CONTEXT_BURN_STATUS_PATH:-.kiro/context-burn-status.json}`.
    - _Requirements: 13.2, 13.3, 13.8_

  - [x] 5.3 Run `npm run mods:write` to generate `context-burn` harness artifacts
    - _Execution note: Native session.usage context and toasts; Node adapters are counter-only. See [deviations](deviations.md)._

    - Verify all harness files produced; the `register.ts` must: on `SessionStart` initialize an in-memory `tokenAccumulator = 0` and read `options.max_tokens ?? 200_000`; on `PostToolUse` increment accumulator by `JSON.stringify(ctx.tool_input).length + JSON.stringify(ctx.tool_output).length` as a token estimate; emit `console.warn("[context-burn] WARNING: 70% context used ...")` at 70% and `console.warn("[context-burn] CRITICAL: 90% context used ...")` at 90% thresholds; never block (always return void).
    - Warning message format: `[context-burn] WARNING: 70% context used (<tokens> / <max> tokens)` and `[context-burn] CRITICAL: 90% context used (<tokens> / <max> tokens)`.
    - _Requirements: 13.2, 13.4, 13.5, 13.6, 13.7_

  - [x] 5.4 Write `mods/context-burn/README.md`
    - _Execution note: Capability differences and unlocked counter limitations documented. See [deviations](deviations.md)._

    - Document: purpose (FinOps token spend tracking), reach level L1, env vars (`VANGUARD_CONTEXT_BURN_STATUS_PATH`), events, Claude Code vs Kiro/Codex capability difference (token count vs tool-call count), threshold warning messages, `max_tokens` option.
    - _Requirements: 17.5_

  - [x] 5.5 Checkpoint: Run `npm run validate:mods` — must exit 0 with `context-burn` counted. Ask the user if questions arise.

- [x] 6. T6 — `audit-trail` mod (including extended coverage task 6.7)

  - [x] 6.1 Write `mods/audit-trail/mod.toml` with `reach_level = 1` and `events = ["PostToolUse"]`
    - _Execution note: Namespaced AUDIT_TRAIL envs and per-session default writes. See [deviations](deviations.md)._

    - `[mod]`: `id = "audit-trail"`, `name = "Audit Trail"`, `reach_level = 1`, `events = ["PostToolUse"]`.
    - `[footprint]`: `env_reads = ["VANGUARD_AUDIT_LOG_PATH", "VANGUARD_AUDIT_SESSION_ID", "VANGUARD_AUDIT_AGENT_ID"]`, `file_writes = [".audit/vanguard-audit-*.jsonl"]`, `spawns = []`, `egress_hosts = []`.
    - `[harnesses.claude-code]`, `[harnesses.kiro]`, `[harnesses.codex]`: all `supported = true`.
    - _Requirements: 1.1–1.8, 14.1, 14.9, 14.10, 15.3_

  - [x] 6.2 Write `mods/audit-trail/codex/logic.sh` — POSIX-only audit append logic
    - _Execution note: Node canonical hashes, per-session locking and append; no POSIX JSON parser. See [deviations](deviations.md)._

    - On PostToolUse: read stdin JSON; compute `args_hash` as `sha256(JSON.stringify(tool_input))` using `sha256sum` (Linux) or `shasum -a 256` (macOS) — try both; compute `result_hash` similarly; determine `session_id` from `VANGUARD_AUDIT_SESSION_ID` env var, fallback to `printf "%s%s" "$PPID" "$(date +%s)" | sha256sum | cut -c1-12`; determine `agent_id` from `VANGUARD_AUDIT_AGENT_ID` env var, fallback to `"unknown"`; determine `log_path` from `VANGUARD_AUDIT_LOG_PATH` env var, fallback to `.audit/vanguard-audit-$(date -u +%Y-%m-%d).jsonl`; `mkdir -p` the parent dir; count existing lines via `wc -l` to derive `sequence_number = line_count + 1`; build JSON entry using `printf`; append with `>>` (append-only, never truncate); exit 0.
    - Script must use only POSIX tools (`date`, `sha256sum`/`shasum`, `printf`, `wc`, file redirection) — no `jq` dependency.
    - Must exit 1 (not 0) if log file cannot be opened or written (fail-closed for audit).
    - _Requirements: 14.2, 14.3, 14.4, 14.5, 14.6, 14.7, 14.11, 14.12_

  - [x] 6.3 Run `npm run mods:write` to generate `audit-trail` harness artifacts
    - _Execution note: Native unsupported (no append); classic and Node adapters verified. See [deviations](deviations.md)._

    - Verify all harness files produced; the `register.ts` must: subscribe to `PostToolUse`; compute SHA-256 using Node.js `crypto.createHash("sha256")`; perform log append synchronously via `appendFileSync` (including `mkdirSync` for parent dir); derive `sequence_number` from `existsSync` + line-count of existing file content; never throw — wrap in `try/catch` and on failure log error to `console.error` then continue (soft-error, not hard block).
    - _Requirements: 14.2, 14.3, 14.7, 14.8_

  - [x] 6.4 Write `mods/audit-trail/README.md`
    - _Execution note: Ordinary append log; observer errors are non-blocking, not enforcement. See [deviations](deviations.md)._

    - Document: purpose (SOC 2 Type II / PCI-DSS Level 1 audit trail), reach level L1, env vars (`VANGUARD_AUDIT_LOG_PATH`, `VANGUARD_AUDIT_SESSION_ID`, `VANGUARD_AUDIT_AGENT_ID`), JSONL entry schema, events (`PostToolUse`), Codex limitation (POSIX-only, no jq), harness install notes, and the fail-closed behavior (exit 1 if log write fails).
    - _Requirements: 17.5_

  - [x] 6.5 Add P3 (Audit Completeness) property test to `tests/fuzz-properties.test.mjs`
    - _Execution note: P3 uses actual production audit hook/record code, not duplicated logic. See [deviations](deviations.md)._

    - **Property 3: Audit Completeness**
    - Invoke the production Node audit hook with isolated temporary session logs; import `buildRecord`, `lastSequence` and `canonicalJson` for comparison, not copied append logic.
    - Generate JSON-compatible payload dictionaries and use the wire key `tool_response`. Null-prototype dictionaries are valid JSON inputs; do not disable them to work around an unproven serialization problem.
    - Assert actual JSONL record count, sequence 1..N, trailing newline, released lock and canonical hashes. Original 6.5 coverage sampled N ∈ [1, 4]; completed 6.7 extends it to N ∈ [1, 50] with nested values and explicit boundaries (200 runs, seed 20261014).
    - **Validates: Requirements 14.2, 14.3, 16.3**

  - [x] 6.6 Checkpoint: Run `npm run validate:mods` — must exit 0 with all five mods not yet complete but `audit-trail` counted. Ask the user if questions arise.

  - [x] 6.7 Extend P3 coverage to nested JSON values and sequence boundaries
    - Owned file: `tests/fuzz-properties.test.mjs`; serialize edits with 7.1 and 7.2.
    - Use `fc.jsonValue({ maxDepth: 3 })` for nested input/result values, with 200 seeded random sequences sampling N ∈ [1, 50]. Keep the production hook invocation and temporary-log cleanup from 6.5.
    - Add deterministic N = 1 and N = 50 cases, plus JSON-compatible null-prototype input. Assert real record counts, sequence values and hashes for each supported command adapter.
    - Done: `rtk npm run test:fuzz` exits 0 with the boundary cases actually executed; report the seed and sample limits. Never claim this finite workload proves all possible sequences.
    - _Design: Property 3 and Testing Strategy / P3; Requirements: 14.2, 14.3, 16.3, 16.7_

- [x] 7. T7 — PBT extensions and full validation wiring

  - [x] 7.1 Add P4 (Manifest Validity Round-Trip) property test to `tests/fuzz-properties.test.mjs`
    - _Execution note: Production parser/render exports and test-side serializer oracle. See [deviations](deviations.md)._

    - **Property 4: Manifest Validity Round-Trip**
    - Import or inline the `parseModToml` and `extractCatalogEntry` functions from `scripts/generate-mods.mjs` (inline pure copies in the test file to avoid module coupling — the functions are pure string → object transforms).
    - Use `fc.record({ id: fc.stringMatching(/^[a-z0-9][a-z0-9-]+$/), name: fc.string({ minLength: 1, maxLength: 50 }), version: fc.tuple(fc.nat({ max: 9 }), fc.nat({ max: 99 }), fc.nat({ max: 99 })).map(([a,b,c]) => \`${a}.${b}.${c}\`), reach_level: fc.integer({ min: 0, max: 3 }), events: fc.array(fc.constantFrom("PreToolUse","PostToolUse","SessionStart","Stop"), { minLength: 1, maxLength: 4 }), harnesses: fc.array(fc.constantFrom("claude-code","kiro","codex"), { minLength: 1, maxLength: 3 }) })` as the arbitrary.
    - Serialize to a minimal TOML string, parse via the inline `parseModToml`, extract catalog entry, JSON.stringify → JSON.parse, then assert all fields have correct types and value ranges.
    - Run 200 iterations.
    - **Validates: Requirements 8.2, 16.4**

  - [x] 7.2 Add P5 (Generator Idempotency) property test to `tests/fuzz-properties.test.mjs`
    - _Execution note: Repeated generation/permutation invariance, not ill-typed generate(generate(t)). See [deviations](deviations.md)._

    - **Property 5: Generator Idempotency**
    - Use the same arbitrary as P4 (valid `mod.toml` content object).
    - Call an inline `generateKiroHookContent(mod)` (the pure string-building function from the generator) once and twice; assert the two outputs are byte-for-byte identical.
    - Also call `generateRegisterTsContent(mod)` and `generateHooksJsonContent(mod)` similarly.
    - Assert all `generate*Content(mod) == generate*Content(generate*Content(mod))` (the function is its own fixed point).
    - Run 200 iterations.
    - **Validates: Requirements 6.7, 16.5**

  - [x] 7.3 Run the full `npm run validate:mods` gate with all five mods present and confirm exit 0
    - Run `node scripts/generate-mods.mjs` (writes all artifacts).
    - Run `python3 tests/validate-mods.py` and confirm: exit 0; summary line shows 5 mods; all P1, P2 sub-tests pass; generator `--check` reports no drift; `catalog/mods.json` has 5 valid entries; no `FAIL` lines.
    - Fix any gate failures found before proceeding.
    - _Requirements: 7.2, 7.3, 7.4, 7.6_

  - [x] 7.4 Run `node tests/fuzz-properties.test.mjs` and confirm all properties pass
    - Execute the test file and verify all `PASS` lines print including P3, P4, P5.
    - Fix any failures.
    - _Requirements: 16.3, 16.4, 16.5, 16.7_

- [x] 8. T8 — Integration, asset integrity, and repo-level wiring

  - [x] 8.1 Run `npm run mods:write` and then `npm run asset-integrity:write` to add all mod files to the integrity manifest
    - Run `node scripts/generate-mods.mjs` (ensure all 5 mods × 3 harnesses are fully generated).
    - Run `python3 tests/validate-asset-integrity.py --write` to hash the settled tree including `mods/` and `catalog/mods.json`.
    - Confirm `catalog/asset-integrity.json` now has entries under the `mods` tree.
    - _Requirements: 9.1_

  - [x] 8.2 Run `npm run validate` in its entirety and fix any failures
    - Run the full validate chain: `npm run validate`.
    - Fix any `FAIL` lines across all gates before proceeding. Common failure modes to check: `validate:catalog` rejecting `catalog/mods.json`; `validate:asset-integrity` showing drift after generator re-run; `validate:links` finding broken links in `mods/README.md`; `validate:kiro-powers` (no impact expected since mods are not powers); `validate:codex-marketplace` (confirm `mods/` is not accidentally parsed as a codex plugin).
    - _Requirements: 7.1, 7.2_

  - [x] 8.3 Update `CLAUDE.md` with a `## Mods` section
    - _Execution note: Approved Node/native layout and capability matrix documented. See [deviations](deviations.md)._

    - Add a `## Mods` section to `CLAUDE.md` after the existing `## Marketplaces & export` section (or as a new top-level section under "What this repo contains") documenting:
      - The `mods/` directory layout (mod.toml, claude-code/, kiro/, codex/, README.md per mod).
      - The reach level taxonomy (L0–L3, one line each with an example mod).
      - The `mods:write` command (`node scripts/generate-mods.mjs`) — when to run it (after editing any `mod.toml`, `patterns.json`, or `logic.sh`).
      - The `validate:mods` gate — what it checks (schema, reach level, env reads, hook JSON, bash -n, register.ts signature, generator drift, catalog entries, P1+P2 PBT).
      - A contributor checklist item: when adding a new mod, run `npm run mods:write`, `npm run asset-integrity:write`, and `npm run validate` before opening a PR.
    - Update the "Quick start" section of `CLAUDE.md` to mention `npm run mods:write` and `validate:mods`.
    - _Requirements: 9.3, 9.4, 9.5_

  - [x] 8.4 Run `npm run lint:spell` and `npx --yes markdownlint-cli2 "**/*.md" "#node_modules"` — fix all failures
    - Spell-check new `.md` files in `mods/`, updated `CLAUDE.md`, and `mods/README.md`.
    - Add any legitimate technical terms tripped as false positives (e.g. `JSONL`, `POSIX`, `pretooluse`, `posttooluse`) to `.codespellrc` `ignore-words-list`.
    - Fix any markdown lint failures in the new files.
    - _Requirements: definition-of-done step 4_

  - [x] 8.5 Checkpoint: All gates green. Run `git status` — confirm tree is clean (all generated files committed). Confirm branch is `feat/cross-harness-mods`. Ask the user if questions arise.
    - Run after T9 and its integrated checks, not before. Earlier feature verification and publication do not establish completion of the later Rust additions. Preserve unrelated `.kiro/kiro-agents/`; report it instead of staging/deleting it or claiming a globally clean tree. See [deviations](deviations.md).
    - _Post-publication follow-up: approved fixture repairs `e50af843` and `9bf10878` preserve exact values and all 63 mod regressions. Full local validation/lints/secrets pass. The pinned production rule reports zero retained matches while its positive control still flags. All GitHub checks pass at exact remote/PR source head `9bf10878e3175673356fb4f120db4f500772f673`; scanner check 116451943347 succeeds with zero new alerts. No scanner suppression was added._
    - _Verified implementation: `3b73f6509ef243fcd3e1f3b39e1cdf8728b18e1e` on the remote branch and draft PR #212; tracked tree clean, unrelated untracked files preserved. Closeout changes only task/report metadata._
    - _Requirements: 18.12, repository definition of done_

  - [x] 8.6 Make each mod README's numeric reach declaration explicit
    - Owned paths: `mods/*/README.md`. State the current derived L0/L1 value and declared footprint; L0 must be written explicitly even when footprint arrays are empty.
    - Do not fabricate L2/L3 examples or change runtime reach to satisfy prose. Treat the footprint as disclosure, not isolation.
    - Done: compare each README with its `mod.toml`; run spelling and Markdown gates after the edits.
    - _Design: Data Models / mod.toml and reach taxonomy; Requirements: 2.1, 17.5, 17.6_

- [x] 9. T9 — Rust TUI (`vfa-tui`) integration for mods

  - [x] 9.0 Record the approved full-browser scope and footprint source
    - The user explicitly chose the visible browser with UI tasks and a defined footprint source. R8.7, R18.16–18.20, the visible-browser design and ADR-7 record the decision.
    - Generator-produced footprint metadata is the source; legacy absence is `not recorded`. This decision alone is not runtime evidence; implementation and regression checks are recorded under 9.1–9.10.
    - _Design: ADR-7 and Rust TUI integration / Approved scope; Requirements: 8.7, 18 user story_

  - [x] 9.1 Create `src/models/mod_catalog.rs` — `Mod` struct matching `catalog/mods.json`
    - Owned file: `tools/vfa-tui/src/models/mod_catalog.rs`. Use the eight generator fields (`id`, `name`, `version`, `description`, `reach_level`, `events`, `harnesses`, `path`) and optional defaulted `security_notes` and `footprint: Option<ModFootprint>`. Define strict `ModFootprint` with four required string arrays: `env_reads`, `file_writes`, `spawns`, `egress_hosts`. No invented `ModType`, `type` or `Provider` field; no `flatten`.
    - Pair with generated-entry deserialization tests and negative missing-required-field, unknown-field and wrong-type tests, including malformed/unknown nested footprint keys. Optional notes accept absence, null or string; optional footprint accepts absence/null without constructing an empty footprint.
    - Done: exact current generated entries deserialize; invalid shapes fail. Reach 0–3 enforcement is paired with the shared parser in 9.3, not inferred from `u8`.
    - _Design: Rust TUI integration / Mod struct; Requirements: 8.2, 18.1, 18.11_

  - [x] 9.2 Register `mod_catalog` module in `src/models/mod.rs` and re-export `Mod`
    - Add `pub mod mod_catalog;` to `src/models/mod.rs`.
    - Add `pub use mod_catalog::{Mod, ModFootprint};` to the re-exports block following the existing import style. Paths in this task are relative to `tools/vfa-tui/`.
    - Done: crate consumers import `crate::models::Mod`; no unused-import warnings.
    - _Design: Rust TUI integration / Mod struct; Requirements: 18.2_

  - [x] 9.3 Add `load_mods()` to `src/catalog/loader.rs`
    - Owned file: `tools/vfa-tui/src/catalog/loader.rs`. Implement public `load_mods() -> (Vec<Mod>, Vec<TuiError>)` over the design's shared `read_mods_snapshot()` contract, preserving the validated bytes' hash for store consumers.
    - Only OS `NotFound` means optional initial absence. Read at most 20 MiB + 1; reject excess, other I/O failures, invalid UTF-8, malformed/unknown-field input, reach outside 0–3 and tainted strings in every field/array, including footprint metadata. Reject the entire mod snapshot on failure. Do not copy the generic helper's all-errors-as-not-found mapping or use `exists()`.
    - Pair with missing/empty, directory-as-file, invalid UTF-8, size boundary/excess, malformed/unknown-field, reach boundary and control-byte fixtures. Preserve the existing permitted tab/newline policy. Use a deterministic injected read error if permissions cannot be tested reliably under elevated execution.
    - Test parse coordinates on multiline UTF-8 and EOF inputs; a `column()` value is not an absolute file-byte offset. Taint diagnostics identify the actual indexed field and byte position within its decoded string.
    - Done: focused loader tests pass for valid and invalid fixtures; no changes to unrelated loaders or errors.
    - _Design: Rust TUI integration / load_mods and Diagnostics; Requirements: 18.3, 18.4, 18.6, 18.15_

  - [x] 9.4 Add `mods` field to `CatalogStore` in `src/catalog/store.rs` and wire it into `load()`, `reload_file()`, and `content_hashes`
    - Add `pub mods: Vec<Mod>` field to the `CatalogStore` struct.
    - In `CatalogStore::load()`: use the shared validated snapshot pipeline, extend errors and sort by `(id.to_lowercase(), id)`. Record the hash of those same bytes; do not reread mods in the generic hashing loop or cache hashes after absence/failure.
    - Dispatch mods reloads before the generic unbounded read. Reuse startup validation and size limits; adopt the collection/hash together on success. Unchanged accepted bytes return `Unchanged`; missing or invalid reloads return `RetainedPrevious` without updating either collection or hash. A valid `[]` intentionally clears mods.
    - Add `pub fn mod_count(&self) -> usize { self.mods.len() }` query method.
    - Add `pub fn mod_by_id(&self, id: &str) -> Option<&Mod> { self.mods.iter().find(|m| m.id == id) }` query method.
    - Update `from_parts()` and exhaustive store literals in `tools/vfa-tui/src/test_fixtures.rs`, `src/app.rs`, `tests/property/reverse_lookup.rs`, and `tests/property/ui.rs` (last three relative to `tools/vfa-tui/`). Search for additional affected literals before editing; do not claim existing fixtures need no changes.
    - Pair with sorting, exact case-sensitive known/unknown lookup, valid empty collection and data/hash retention tests. Serialize this task with all other `store.rs` writers.
    - Done: focused store tests and existing fixture/property consumers compile and pass.
    - _Design: Rust TUI integration / CatalogStore wiring; Requirements: 18.5, 18.6, 18.7, 18.8, 18.9, 18.10, 18.13, 18.14_

  - [x] 9.5 Add unit and integration tests for mods loading to `src/catalog/store.rs`
    - Owned files: `tools/vfa-tui/src/catalog/store.rs` tests and `src/catalog/watcher.rs` tests (both crate-relative after the first). Use isolated existing workspace fixtures, not only the live checkout; attribute errors to mods rather than requiring unrelated catalog errors to be absent.
    - Exercise N = 0, 1, 5 and varied generated valid counts, reordered IDs, metadata preservation and known/unknown exact lookups. Keep a generated-catalog smoke test comparing input IDs/count, not a permanent `== 5` assertion.
    - Exercise load → unchanged → changed valid → repeated malformed/tainted/oversize/missing → corrected valid → valid empty. Check data and accepted hash after every transition; repeated failures must not be mislabeled `Unchanged`.
    - Test generic watcher classification of `catalog/mods.json` deterministically. Existing app dispatch should reach the store; do not add a filename allowlist or rewrite the watcher without contrary evidence.
    - Done: these tests fail when mods reload adoption or validation is disabled, and pass with the implementation. P8 is sampled evidence, not proven by one five-entry fixture.
    - _Design: Property 8 and Rust TUI integration / verification; Requirements: 18.6, 18.7, 18.11, 18.13, 18.14_

  - [x] 9.6 Run `rtk cargo test` in `tools/vfa-tui` — all tests must pass, zero clippy warnings
    - Run `rtk cargo fmt --check` — fix any formatting issues first.
    - Run `rtk cargo clippy --all-targets -- -D warnings` — fix all warnings.
    - Run `rtk cargo test` — all existing and new tests must pass. Run the generated-catalog integration smoke test after `rtk npm run mods:write`; missing/empty fixture tests must not depend on generation.
    - After all source/docs/generators settle, run asset integrity last on its own, then repository validation, spelling and Markdown gates. Earlier T8 results are historical, not verification of the new integrated revision.
    - Run after 9.7–9.10 as well as 9.1–9.5. Verify the visible browser, generated footprint parity, legacy unknown state and UI reload behavior; loader tests alone cannot complete T9. Record commands, exit codes and skipped checks.
    - _Design: Rust TUI integration / verification; Requirements: 18.11, 18.12, repository definition of done_

  - [x] 9.7 Add generated catalog footprint metadata and its validator contract
    - Owned paths: `scripts/generate-mods.mjs`, `tests/validate-mods.py`, `tests/mods.test.mjs`, generated `catalog/mods.json`.
    - Extend the catalog mapping in `renderAll()` to copy all four manifest footprint arrays, preserving their order. Update the Python validator's exact expected-entry comparison in the same change; do not hand-edit catalog output or add a required discriminator/provider.
    - Pair with production-generator tests comparing emitted arrays against each manifest, deterministic generation checks and a validator-negative fixture with a mismatched/missing footprint. Regenerate through one owner only.
    - Done: `rtk npm run mods:write`, `rtk npm run validate:mods`, generator `--check` and mod property tests pass; fresh entries carry footprint while Rust still accepts legacy entries without it. Refresh integrity only after all later changes settle.
    - _Design: Catalog entry schema and ADR-7; Requirements: 8.2, 8.4, 8.7, 18.1, 18.18_

  - [x] 9.8 Make the Mods list reachable in the primary Catalog tab
    - Owned files: `tools/vfa-tui/src/ui/nav.rs`, `tools/vfa-tui/src/app.rs`. Serialize app edits with 9.4 and 9.9.
    - Append a Mods sidebar entry; add `View::ModList` / `View::ModDetail(String)` and all required exhaustive dispatch, Catalog-tab synchronization, list-length/count and Enter/Back paths. Keep existing top-level tabs and sidebar indexes stable.
    - Render ordered ID/name/L0–L3 rows and a `No mods` empty state. Up/down selection and Enter must target the displayed mod; provider-only filters must not hide cross-provider mods. Do not add install/enable/export/run actions.
    - Pair with focused navigation and list-render tests. Done: primary `App::render` reaches the list via the Catalog tab, not only a directly invoked widget; existing navigation tests still pass.
    - _Design: Visible mods browser; Requirements: 18.16, 18.20_

  - [x] 9.9 Add mod detail rendering and reload-safe selection
    - Owned files: `tools/vfa-tui/src/ui/widgets/detail.rs`, `tools/vfa-tui/src/app.rs`; serialize with 9.8.
    - Add the typed detail renderer and show all R18.17 fields. Render all four declared footprint arrays; distinguish `none declared` from absent/null `not recorded`. Preserve scrolling and Back/Escape navigation.
    - Resolve displayed detail by ID after reload, clamp list indexes, and show a non-panicking `Mod no longer present` state when that ID is removed. Failed reload retains last valid metadata and surfaces its error.
    - Pair with field-preservation, legacy/empty/non-empty footprint, scroll/back and removed-ID tests. Done: displayed arrays equal the generated catalog and no manifest parsing, declared-path access or hook invocation occurs during browsing.
    - _Design: Visible mods browser and ADR-7; Requirements: 18.17, 18.18, 18.19, 18.20_

  - [x] 9.10 Exercise the full browser and reload path through the real App
    - Owned files: `tools/vfa-tui/tests/integration/tui_primary_render.rs`, `tools/vfa-tui/tests/integration/tui_reload.rs`. Extend existing registered integration modules rather than creating an unregistered test file.
    - Use isolated catalog fixtures and the existing Ratatui `TestBackend` to navigate Catalog → Mods → detail → Back, render identity/reach/events/harnesses/all footprint arrays, exercise scrolling, and cover missing/empty/legacy catalogs.
    - Reload changed metadata, a shorter list, removed selected ID, malformed/tainted data, and corrected valid data. Assert actual rendered state plus selection bounds and retained error/data behavior, not only store counts. Verify existing agent/skill/rule navigation still works.
    - Done: tests fail if the Mods route or reload refresh is disabled and pass with the implementation; finish with task 9.6 and final checkpoint 8.5.
    - _Design: Visible mods browser and R18 verification; Requirements: 18.11, 18.16, 18.17, 18.18, 18.19, 18.20_

---

## Notes

- All property-test tasks are required. No optional marker makes P3/P8 or an explicit R18 deliverable skippable.
- Generated files (`register.ts`, Kiro hook JSON, Codex `.sh` boilerplate) are produced by `npm run mods:write` — never hand-edit them. Only `mod.toml`, `patterns.json`, `logic.sh`, and `README.md` files are hand-authored.
- The generator `--check` mode is the drift gate: if on-disk files differ from generated, `validate:mods` fails.
- Run `npm run asset-integrity:write` LAST (after all generator runs and file edits), as described in CLAUDE.md.
- `catalog/index.json` must include the `mods` key before `validate:catalog` runs; Task 1.3 handles this.
- Reach follows the approved derived-footprint policy in [deviations](deviations.md); optional notes do not authorize aspirational over-declaration. TUI loading independently enforces the numeric 0–3 range.
- Only OS NotFound is optional initial absence. Reload deletion retains previous data/hash with an error; write `[]` to clear deliberately.
- The wave graph intentionally serializes shared generator outputs, fuzz-test edits and store-test edits. Safety takes precedence over a target parallelism percentage. Final checkpoint 8.5 follows T9, avoiding stale integrated-state evidence.
- The user retained the full visible browser. T9 is not complete until list/detail/navigation, generated footprint metadata and actual App reload/render tests pass; foundation-only loading is insufficient.

## Task Dependency Graph

Leaf-task waves preserve the existing ID convention; each shared writer is serialized.

```json
{
  "waves": [
    { "id": 0, "tasks": ["1.1"] },
    { "id": 1, "tasks": ["1.2"] },
    { "id": 2, "tasks": ["1.3"] },
    { "id": 3, "tasks": ["1.4"] },
    { "id": 4, "tasks": ["1.5"] },
    { "id": 5, "tasks": ["1.6"] },
    { "id": 6, "tasks": ["1.7"] },
    { "id": 7, "tasks": ["1.8"] },
    { "id": 8, "tasks": ["1.9"] },
    { "id": 9, "tasks": ["2.1"] },
    { "id": 10, "tasks": ["2.2"] },
    { "id": 11, "tasks": ["2.3"] },
    { "id": 12, "tasks": ["2.4"] },
    { "id": 13, "tasks": ["2.5"] },
    { "id": 14, "tasks": ["2.6"] },
    { "id": 15, "tasks": ["2.7"] },
    { "id": 16, "tasks": ["3.1"] },
    { "id": 17, "tasks": ["3.2"] },
    { "id": 18, "tasks": ["3.3"] },
    { "id": 19, "tasks": ["3.4"] },
    { "id": 20, "tasks": ["3.5"] },
    { "id": 21, "tasks": ["3.6"] },
    { "id": 22, "tasks": ["3.7"] },
    { "id": 23, "tasks": ["4.1"] },
    { "id": 24, "tasks": ["4.2"] },
    { "id": 25, "tasks": ["4.3"] },
    { "id": 26, "tasks": ["4.4"] },
    { "id": 27, "tasks": ["4.5"] },
    { "id": 28, "tasks": ["5.1"] },
    { "id": 29, "tasks": ["5.2"] },
    { "id": 30, "tasks": ["5.3"] },
    { "id": 31, "tasks": ["5.4"] },
    { "id": 32, "tasks": ["5.5"] },
    { "id": 33, "tasks": ["6.1"] },
    { "id": 34, "tasks": ["6.2"] },
    { "id": 35, "tasks": ["6.3"] },
    { "id": 36, "tasks": ["6.4"] },
    { "id": 37, "tasks": ["6.5"] },
    { "id": 38, "tasks": ["6.6"] },
    { "id": 39, "tasks": ["6.7"] },
    { "id": 40, "tasks": ["7.1"] },
    { "id": 41, "tasks": ["7.2"] },
    { "id": 42, "tasks": ["7.3"] },
    { "id": 43, "tasks": ["7.4"] },
    { "id": 44, "tasks": ["8.1"] },
    { "id": 45, "tasks": ["8.2"] },
    { "id": 46, "tasks": ["8.3"] },
    { "id": 47, "tasks": ["8.6"] },
    { "id": 48, "tasks": ["8.4"] },
    { "id": 49, "tasks": ["9.0"] },
    { "id": 50, "tasks": ["9.1"] },
    { "id": 51, "tasks": ["9.2"] },
    { "id": 52, "tasks": ["9.3"] },
    { "id": 53, "tasks": ["9.4"] },
    { "id": 54, "tasks": ["9.5"] },
    { "id": 55, "tasks": ["9.7"] },
    { "id": 56, "tasks": ["9.8"] },
    { "id": 57, "tasks": ["9.9"] },
    { "id": 58, "tasks": ["9.10"] },
    { "id": 59, "tasks": ["9.6"] },
    { "id": 60, "tasks": ["8.5"] }
  ]
}
```
