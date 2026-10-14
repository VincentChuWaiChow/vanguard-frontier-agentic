# Implementation Plan: cross-harness-mods

## Overview

Implement the `mods/` behavioral extension layer for `vanguard-frontier-agentic`. This plan builds the framework first (directory contract, generator skeleton, validation gate), then adds each of the five enterprise starter mods with their hand-authored sources, then completes property-based tests, and finally wires everything into the repo's gates and documentation.

All generated harness files (`register.ts`, Kiro hook JSON, Codex `.sh` scripts) are produced by the generator — never hand-written. Each group ends with a verification step against the gate before moving to the next group.

---

## Tasks

- [ ] 1. T1 — Framework scaffolding

  - [ ] 1.1 Create `mods/` directory structure and `mods/README.md`
    - Create `mods/` at repo root.
    - Write `mods/README.md` documenting the mod format, reach level taxonomy (L0–L3 with one-line definition and example mod for each level), install command for each harness (Kiro: copy `.json` + companion `.sh` to `.kiro/hooks/`; Claude Code: `.claude/mods/<id>/register.ts`; Codex: register `.sh` as Codex hook), and a note that no mod requires a running network service at install time and L3 mods are warned (not blocked) by `validate:mods`.
    - _Requirements: 1.10, 17.1, 17.2, 17.3, 17.4, 17.6, 17.7_

  - [ ] 1.2 Add `mods:write` and `validate:mods` scripts to `package.json` and append `validate:mods` to the `validate` chain
    - Add `"mods:write": "node scripts/generate-mods.mjs"` to `scripts` in `package.json`.
    - Add `"validate:mods": "python3 tests/validate-mods.py"` to `scripts` in `package.json`.
    - Append `&& npm run validate:mods` to the end of the existing `validate` script string.
    - _Requirements: 6.4, 7.1_

  - [ ] 1.3 Create `catalog/mods.json` as an empty array and add it to `catalog/index.json`
    - Write `catalog/mods.json` as `[]` (empty JSON array — the generator will populate it on first run).
    - Add `"mods": "catalog/mods.json"` to the `catalogs` object in `catalog/index.json`.
    - _Requirements: 8.1, 9.6_

  - [ ] 1.4 Add `mods/` and `catalog/mods.json` coverage to `tests/validate-asset-integrity.py`
    - Add `"mods"` to the `TREES` list in `validate-asset-integrity.py` (it already walks `catalog/` recursively, so `catalog/mods.json` is covered once `mods/` itself is added).
    - Confirm `catalog/mods.json` is not in `EXCLUDED_RELATIVE_PATHS`.
    - _Requirements: 9.1_

  - [ ] 1.5 Update `.plugin-scanner.toml` to suppress false positives from mod pattern files
    - Add `"mods/*/kiro/*.sh"`, `"mods/*/codex/*.sh"`, and `"tests/fixtures/mods/**"` to the `[scanner].ignore_paths` array in `.plugin-scanner.toml`.
    - Add a comment explaining each entry (regex patterns in `secret-redactor` and `blast-radius-guard` are credential-shaped by design).
    - _Requirements: 9.2_

  - [ ] 1.6 Write `scripts/generate-mods.mjs` — skeleton with TOML parser, directory walker, `--check` mode, and catalog writer
    - Implement a self-contained minimal TOML parser (regex-based, handling the exact subset used by `mod.toml`: tables, arrays of strings, booleans, integers, quoted strings).
    - Implement `readAllMods()` that discovers every `mods/*/mod.toml` and parses it.
    - Implement `readCatalog()` that reads `catalog/agents.json`.
    - Implement `writeOrCheck(filePath, content)` that writes when not in `--check` mode, or compares and accumulates mismatches when in `--check` mode.
    - Implement `generateCatalog(mods)` that writes `catalog/mods.json` as a sorted JSON array of Catalog_Entry objects (fields: `id`, `name`, `version`, `description`, `reach_level`, `events`, `harnesses`, `path`).
    - Implement top-level `main()` with `--check` flag support; exit 0 with `Generated <N> mods for <M> harnesses` on success; exit 1 with error details on failure; continue processing all mods on TOML parse errors (fail-after-all, not fail-fast).
    - Generator must never consult the wall clock, random, or any filesystem path outside the repo root during generation.
    - _Requirements: 6.1, 6.2, 6.3, 6.7, 6.8, 6.9, 6.10_

  - [ ] 1.7 Add Kiro, Claude Code, and Codex artifact generators to `scripts/generate-mods.mjs`
    - Implement `generateKiroArtifacts(mod)` that, for each event in `mod.events` that Kiro supports, writes `mods/<id>/kiro/<id>-<event-lowercase>-hook.json` with the v1 schema (`version: "v1"`, `hooks` array with `name`, `trigger`, `action`); hook `name` field uses `<mod-id>:<event-lowercase-hyphenated>` convention; hook `action.command` invokes the co-located companion `.sh`; when `harnesses.kiro.supported = false`, writes `mods/<id>/kiro/UNSUPPORTED.md` instead.
    - Implement `generateClaudeCodeArtifacts(mod)` that writes `mods/<id>/claude-code/register.ts` (with `export default function register(on: ModEventEmitter, options: ModOptions): void` signature, mod-specific event handler bodies, and `try/catch` wrapping every handler) and `mods/<id>/claude-code/hooks.json` (events array matching the `on("<Event>")` call sites in the generated TypeScript); when `harnesses.claude-code.supported = false`, writes `UNSUPPORTED.md` instead.
    - Implement `generateCodexArtifacts(mod)` that, for each declared event, writes `mods/<id>/codex/<event-lowercase>.sh` with `#!/usr/bin/env bash`, `set -euo pipefail`, stdin read, env var handling boilerplate, and a `source` call to `codex/logic.sh` (which the hand-authored mod provides); when `harnesses.codex.supported = false`, writes `UNSUPPORTED.md` instead.
    - For `live-guard-gate` only: generate `mods/live-guard-gate/live-agents.json` by extracting all agents from `catalog/agents.json` whose `id` matches `*-live-*-guard-agent`.
    - _Requirements: 1.2, 1.6, 3.1, 3.2, 3.3, 3.4, 4.1, 4.2, 5.1, 5.3, 5.4, 6.2, 12.2_

  - [ ] 1.8 Write `tests/validate-mods.py` — full validation gate
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

  - [ ] 1.9 Checkpoint: Run `npm run validate:mods` (expect it to pass with zero mods found, or the `mods/ directory missing` path) and `npm run validate` — the new gate must not break the existing suite. Ask the user if questions arise.

- [ ] 2. T2 — `secret-redactor` mod

  - [ ] 2.1 Write `mods/secret-redactor/mod.toml` with `reach_level = 0` and `events = ["PreToolUse", "PostToolUse"]`
    - `[mod]`: `id = "secret-redactor"`, `name = "Secret Redactor"`, `description` (≥10 chars), `version = "0.1.0"`, `reach_level = 0`, `events = ["PreToolUse", "PostToolUse"]`.
    - `[footprint]`: `env_reads = ["VANGUARD_SECRET_REDACTOR_PATTERNS"]`, `file_writes = []`, `spawns = []`, `egress_hosts = []`.
    - `[harnesses.claude-code]`: `supported = true`.
    - `[harnesses.kiro]`: `supported = true`.
    - `[harnesses.codex]`: `supported = true`.
    - _Requirements: 1.1–1.8, 10.1, 10.8, 10.9, 15.3_

  - [ ] 2.2 Write `mods/secret-redactor/patterns.json` with the five required built-in patterns
    - Include at minimum: `aws-access-key` (`AKIA[0-9A-Z]{16}`), `aws-secret-key` (generic 40-char base62), `github-pat` (`(ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9]{36,255}`), `generic-bearer` (`Bearer\s+[A-Za-z0-9\-._~+/]+=*`), `password-in-url` (`://[^@]*:[^@]+@`).
    - Each entry has fields: `id` (kebab-case), `name`, `description`, `regex` (POSIX-compatible), `severity` (`critical`/`high`/`medium`/`low`), `context` (array, use `["all"]` for these patterns).
    - _Requirements: 10.2_

  - [ ] 2.3 Write `mods/secret-redactor/codex/logic.sh` — hand-authored redaction logic
    - POSIX-only implementation: reads `INPUT` from stdin (already set by boilerplate), loads `patterns.json`, applies `sed` regex replacements for each pattern replacing matches with `[REDACTED:<pattern-id>]`, writes redacted JSON to stdout, logs redaction events to stderr in format `[secret-redactor] redacted <pattern-name> in <tool-name> PreToolUse`.
    - Must handle `VANGUARD_SECRET_REDACTOR_PATTERNS` env var: if set, load additional patterns from the file and merge (log a warning to stderr if duplicate IDs found, but do not fail).
    - Must exit 0 with no stdout when no patterns match (pass-through).
    - _Requirements: 10.3, 10.4, 10.5, 10.6, 10.7, 10.8_

  - [ ] 2.4 Run `npm run mods:write` to generate all `secret-redactor` harness artifacts
    - Running `node scripts/generate-mods.mjs` must produce: `mods/secret-redactor/kiro/secret-redactor-pretooluse-hook.json`, `mods/secret-redactor/kiro/secret-redactor-posttooluse-hook.json`, `mods/secret-redactor/claude-code/register.ts`, `mods/secret-redactor/claude-code/hooks.json`, `mods/secret-redactor/codex/pretooluse.sh`, `mods/secret-redactor/codex/posttooluse.sh`.
    - The `register.ts` must: load `patterns.json` via `readFileSync`; implement `redact(value: string): string`; in PreToolUse return `{ modified: JSON.parse(redacted) }` when a match is found; in PostToolUse scan output and log to `console.error` but not return a modified response (PostToolUse output cannot be intercepted in Claude Code).
    - Verify `catalog/mods.json` is updated with the `secret-redactor` entry.
    - _Requirements: 3.1–3.4, 4.1–4.7, 5.1–5.5, 8.1, 8.2_

  - [ ] 2.5 Write `mods/secret-redactor/README.md`
    - Document: mod purpose (DLP credential redaction), reach level L0 with footprint, events handled (`PreToolUse` scans tool args; `PostToolUse` scans tool result), harness-specific install notes, env var `VANGUARD_SECRET_REDACTOR_PATTERNS` for extension patterns, and note that `register.ts` implements Pattern D (argument modification) in Claude Code.
    - _Requirements: 17.5_

  - [ ] 2.6 Add P1 (Redaction Completeness) inline property test to `tests/validate-mods.py`
    - **Property 1: Redaction Completeness**
    - Add a `test_redaction_completeness()` function inside `run_pbt_suite()` that: imports `re`, `random`, `string`; for each pattern in `secret-redactor/patterns.json`; generates 200 random suffix strings; constructs a string containing the literal pattern match; calls an inline `redact(s, patterns)` function; asserts `re.search(pattern["regex"], result) is None`.
    - Also test pass-through: generate 200 random strings that don't match any pattern and verify the output equals the input.
    - **Validates: Requirements 10.3, 10.4, 16.1**

  - [ ] 2.7 Checkpoint: Run `npm run validate:mods` — must exit 0 with `secret-redactor` counted. Ask the user if questions arise.

- [ ] 3. T3 — `blast-radius-guard` mod

  - [ ] 3.1 Write `mods/blast-radius-guard/mod.toml` with `reach_level = 1` and `events = ["PreToolUse"]`
    - `[mod]`: `id = "blast-radius-guard"`, `name = "Blast Radius Guard"`, `reach_level = 1`, `events = ["PreToolUse"]`.
    - `[footprint]`: `env_reads = ["VANGUARD_BLAST_RADIUS_MODE", "VANGUARD_BLAST_RADIUS_PATTERNS"]`, `file_writes = []`, `spawns = []`, `egress_hosts = []`.
    - Note: `reach_level = 1` because `env_reads` is non-empty but `file_writes`, `spawns`, `egress_hosts` are all empty — the validator derives L1 from `env_reads` (which are reads, not writes) — wait: re-check the design formula. The formula is: L3 if `egress_hosts`, L2 if `spawns`, L1 if `file_writes`, else L0. `env_reads` alone → L0 per the formula. However Requirement 11.1 declares `reach_level = 1`. Use `reach_level = 1` as specified and add a `[footprint].file_writes` entry (e.g. a temp file) that justifies L1, OR declare `reach_level = 0` if no file writes exist and the validator checks actual derivation. Since the validator enforces the formula, set `reach_level = 0` only if all footprint arrays are empty. The requirement says 1 — add the extension patterns file as a conditional write in `file_writes` per the design pattern for extensibility (`VANGUARD_BLAST_RADIUS_PATTERNS` target path), or note in `security_notes` that the reach level is declared conservatively. Match exactly what Requirement 11.1 states.
    - `[harnesses.claude-code]`, `[harnesses.kiro]`, `[harnesses.codex]`: all `supported = true`.
    - _Requirements: 1.1–1.8, 11.1, 11.9, 11.10, 15.3_

  - [ ] 3.2 Write `mods/blast-radius-guard/patterns.json` with the ten required built-in patterns
    - Include at minimum: `rm-rf-root` (`rm\s+-[rRfF]+.*\s+(/|/\*|~/)`), `drop-table` (`(?i)DROP\s+TABLE`), `drop-database` (`(?i)DROP\s+DATABASE`), `drop-schema` (`(?i)DROP\s+SCHEMA`), `git-force-push` (`git\s+push\s+.*(-f|--force)`), `curl-pipe-sh` (`curl\s+.*\|\s*sh`), `wget-pipe-sh` (`wget\s+.*\|\s*sh`), `terraform-destroy-autoapprove` (`terraform\s+.*destroy\s+.*-auto-approve`), `kubectl-delete-namespace` (`kubectl\s+delete\s+namespace`), `kubectl-delete-all` (`kubectl\s+delete\s+.*--all`).
    - Fields follow same schema as `secret-redactor/patterns.json`; `context` uses `["bash"]` for command patterns and `["file-write"]` for write-path patterns.
    - _Requirements: 11.2_

  - [ ] 3.3 Write `mods/blast-radius-guard/codex/logic.sh` — hand-authored guard logic
    - Read command from `tool_input.command` in stdin JSON; also check `tool_input.path` for file-write tool calls.
    - Check against patterns: if match found, read `VANGUARD_BLAST_RADIUS_MODE`; if `block`, exit 2 with reason to stderr; if `confirm` (default), exit 0 with `{"hookSpecificOutput":{"permissionDecision":"ask","permissionDecisionReason":"<pattern match: <pattern-id>>"}}` to stdout.
    - If no match, exit 0 with no stdout.
    - Apply same block/confirm logic when a write tool call targets `/(etc|usr|bin|sbin|boot|sys|proc)/`.
    - Load extension patterns from `VANGUARD_BLAST_RADIUS_PATTERNS` if set (same merge convention as `secret-redactor`).
    - _Requirements: 11.3, 11.4, 11.5, 11.7, 11.8, 11.9_

  - [ ] 3.4 Run `npm run mods:write` to generate all `blast-radius-guard` harness artifacts
    - Verify `mods/blast-radius-guard/kiro/blast-radius-guard-pretooluse-hook.json`, `claude-code/register.ts`, `claude-code/hooks.json`, `codex/pretooluse.sh` are produced.
    - The `register.ts` must: in PreToolUse read `VANGUARD_BLAST_RADIUS_MODE`, return `{ deny: reason }` when mode is `block` and a pattern matches, return `{ ask: true, reason }` when mode is `confirm`.
    - Verify `catalog/mods.json` updated.
    - _Requirements: 3.1–3.7, 4.1–4.6, 5.1–5.5, 11.4, 11.5, 11.6_

  - [ ] 3.5 Write `mods/blast-radius-guard/README.md`
    - Document: purpose, reach level L1, env vars (`VANGUARD_BLAST_RADIUS_MODE`, `VANGUARD_BLAST_RADIUS_PATTERNS`), events (`PreToolUse` only), harness install notes.
    - _Requirements: 17.5_

  - [ ] 3.6 Add P2 (Guard Blocking) inline property test to `tests/validate-mods.py`
    - **Property 2: Guard Blocking**
    - Add `test_guard_blocking()` inside `run_pbt_suite()`: for each pattern in `blast-radius-guard/patterns.json`, construct 200 random strings that contain a literal substring matching the pattern regex; call an inline `evaluate(cmd, patterns)` function (returns `"block"` or `"confirm"` when matched, `"pass"` otherwise); assert `evaluate(cmd).decision != "pass"`.
    - Also generate 200 random strings that don't match any pattern and assert `evaluate(cmd) == "pass"`.
    - **Validates: Requirements 11.3, 11.4, 11.5, 16.2**

  - [ ] 3.7 Checkpoint: Run `npm run validate:mods` — must exit 0 with both `secret-redactor` and `blast-radius-guard` counted. Ask the user if questions arise.

- [ ] 4. T4 — `live-guard-gate` mod

  - [ ] 4.1 Write `mods/live-guard-gate/mod.toml` with `reach_level = 2` and `events = ["PreToolUse"]`
    - `[mod]`: `id = "live-guard-gate"`, `name = "Live Guard Gate"`, `reach_level = 2`, `events = ["PreToolUse"]`.
    - `[footprint]`: `env_reads = []`, `file_writes = ["mods/live-guard-gate/live-agents.json"]`, `spawns = []`, `egress_hosts = []`.
    - Note: `reach_level = 2` is declared conservatively per Requirement 12.1 and Requirement 12.9 (conservative for future subprocess). The derived formula would give L1 (file_writes non-empty). Add `security_notes` in the catalog entry (written by generator) explaining this; the validator must accept the conservatively-declared level without failing (treat declared > derived as a valid conservative declaration when a `security_notes` justification exists, or simply allow declared level ≥ derived level).
    - `[harnesses.kiro]`: `supported = true`. `[harnesses.claude-code]`: `supported = true`. `[harnesses.codex]`: `supported = true`.
    - _Requirements: 1.1–1.8, 12.1, 12.7, 12.8, 12.9, 15.3_

  - [ ] 4.2 Write `mods/live-guard-gate/live-patterns.json` — hand-authored trigger patterns
    - List the tool-target patterns that should trigger a live-guard approval prompt (e.g. Terraform plan paths, kubectl resource targets, AWS CLI commands against production-named resources, shell commands that match live-guard agent scopes from `catalog/agents.json`).
    - Each entry: `id`, `name`, `regex`, `description`, `context` (e.g. `["bash", "terraform", "kubectl"]`).
    - _Requirements: 12.3_

  - [ ] 4.3 Run `npm run mods:write` to generate `live-guard-gate` harness artifacts including `live-agents.json`
    - Generator must read `catalog/agents.json`, extract agents whose `id` matches `*-live-*-guard-agent`, and write `mods/live-guard-gate/live-agents.json` as a JSON array.
    - Verify all harness files produced: `kiro/live-guard-gate-pretooluse-hook.json`, `claude-code/register.ts`, `claude-code/hooks.json`, `codex/pretooluse.sh`.
    - The `register.ts` must return `{ ask: true, reason: "<reason identifying the matched live-guard agent>" }` from PreToolUse when a match is found.
    - Verify `catalog/mods.json` updated with `security_notes` field for `live-guard-gate`.
    - _Requirements: 12.2, 12.3, 12.4, 12.5, 12.6, 12.7, 12.9_

  - [ ] 4.4 Write `mods/live-guard-gate/README.md`
    - Document: purpose (CAB-equivalent gate that routes to live-guard agents), reach level L2 (conservative), that the mod only prompts the user — it does NOT spawn a live-guard agent automatically, events (`PreToolUse`), harness install notes, and note that `live-agents.json` is regenerated by `mods:write`.
    - _Requirements: 12.7, 17.5_

  - [ ] 4.5 Checkpoint: Run `npm run validate:mods` — must exit 0 with `live-guard-gate` counted. Ask the user if questions arise.

- [ ] 5. T5 — `context-burn` mod

  - [ ] 5.1 Write `mods/context-burn/mod.toml` with `reach_level = 1` and `events = ["PostToolUse", "SessionStart"]`
    - `[mod]`: `id = "context-burn"`, `name = "Context Burn"`, `reach_level = 1`, `events = ["PostToolUse", "SessionStart"]`.
    - `[footprint]`: `env_reads = ["VANGUARD_CONTEXT_BURN_STATUS_PATH"]`, `file_writes = [".kiro/context-burn-status.json"]`, `spawns = []`, `egress_hosts = []`.
    - `[harnesses.claude-code]`: `supported = true`; `[harnesses.claude-code.options]`: `max_tokens = 200000`.
    - `[harnesses.kiro]`: `supported = true`; add `notes = "Token count unavailable in Kiro hook context; tracks tool call count as a proxy."`.
    - `[harnesses.codex]`: `supported = true`.
    - _Requirements: 1.1–1.8, 13.1, 13.3, 13.5, 13.9, 13.10, 15.3_

  - [ ] 5.2 Write `mods/context-burn/codex/logic.sh` — hand-authored burn tracking logic
    - On `SessionStart`: initialize Burn_Status_File as valid JSON with `session_id` (derive from `$$` + timestamp hash), `tool_call_count = 0`, `last_updated` (ISO 8601), `threshold_events = []`.
    - On `PostToolUse`: read existing Burn_Status_File; increment `tool_call_count`; update `last_updated`; write updated file; emit warning to stderr if count crosses configurable thresholds (default: warn at 70% and 90% of a `MAX_TOOL_CALLS` env var, defaulting to 500).
    - Burn_Status_File path: `${VANGUARD_CONTEXT_BURN_STATUS_PATH:-.kiro/context-burn-status.json}`.
    - _Requirements: 13.2, 13.3, 13.8_

  - [ ] 5.3 Run `npm run mods:write` to generate `context-burn` harness artifacts
    - Verify all harness files produced; the `register.ts` must: on `SessionStart` initialize an in-memory `tokenAccumulator = 0` and read `options.max_tokens ?? 200_000`; on `PostToolUse` increment accumulator by `JSON.stringify(ctx.tool_input).length + JSON.stringify(ctx.tool_output).length` as a token estimate; emit `console.warn("[context-burn] WARNING: 70% context used ...")` at 70% and `console.warn("[context-burn] CRITICAL: 90% context used ...")` at 90% thresholds; never block (always return void).
    - Warning message format: `[context-burn] WARNING: 70% context used (<tokens> / <max> tokens)` and `[context-burn] CRITICAL: 90% context used (<tokens> / <max> tokens)`.
    - _Requirements: 13.2, 13.4, 13.5, 13.6, 13.7_

  - [ ] 5.4 Write `mods/context-burn/README.md`
    - Document: purpose (FinOps token spend tracking), reach level L1, env vars (`VANGUARD_CONTEXT_BURN_STATUS_PATH`), events, Claude Code vs Kiro/Codex capability difference (token count vs tool-call count), threshold warning messages, `max_tokens` option.
    - _Requirements: 17.5_

  - [ ] 5.5 Checkpoint: Run `npm run validate:mods` — must exit 0 with `context-burn` counted. Ask the user if questions arise.

- [ ] 6. T6 — `audit-trail` mod

  - [ ] 6.1 Write `mods/audit-trail/mod.toml` with `reach_level = 1` and `events = ["PostToolUse"]`
    - `[mod]`: `id = "audit-trail"`, `name = "Audit Trail"`, `reach_level = 1`, `events = ["PostToolUse"]`.
    - `[footprint]`: `env_reads = ["VANGUARD_AUDIT_LOG_PATH", "VANGUARD_AUDIT_SESSION_ID", "VANGUARD_AUDIT_AGENT_ID"]`, `file_writes = [".audit/vanguard-audit-*.jsonl"]`, `spawns = []`, `egress_hosts = []`.
    - `[harnesses.claude-code]`, `[harnesses.kiro]`, `[harnesses.codex]`: all `supported = true`.
    - _Requirements: 1.1–1.8, 14.1, 14.9, 14.10, 15.3_

  - [ ] 6.2 Write `mods/audit-trail/codex/logic.sh` — POSIX-only audit append logic
    - On PostToolUse: read stdin JSON; compute `args_hash` as `sha256(JSON.stringify(tool_input))` using `sha256sum` (Linux) or `shasum -a 256` (macOS) — try both; compute `result_hash` similarly; determine `session_id` from `VANGUARD_AUDIT_SESSION_ID` env var, fallback to `printf "%s%s" "$PPID" "$(date +%s)" | sha256sum | cut -c1-12`; determine `agent_id` from `VANGUARD_AUDIT_AGENT_ID` env var, fallback to `"unknown"`; determine `log_path` from `VANGUARD_AUDIT_LOG_PATH` env var, fallback to `.audit/vanguard-audit-$(date -u +%Y-%m-%d).jsonl`; `mkdir -p` the parent dir; count existing lines via `wc -l` to derive `sequence_number = line_count + 1`; build JSON entry using `printf`; append with `>>` (append-only, never truncate); exit 0.
    - Script must use only POSIX tools (`date`, `sha256sum`/`shasum`, `printf`, `wc`, file redirection) — no `jq` dependency.
    - Must exit 1 (not 0) if log file cannot be opened or written (fail-closed for audit).
    - _Requirements: 14.2, 14.3, 14.4, 14.5, 14.6, 14.7, 14.11, 14.12_

  - [ ] 6.3 Run `npm run mods:write` to generate `audit-trail` harness artifacts
    - Verify all harness files produced; the `register.ts` must: subscribe to `PostToolUse`; compute SHA-256 using Node.js `crypto.createHash("sha256")`; perform log append synchronously via `appendFileSync` (including `mkdirSync` for parent dir); derive `sequence_number` from `existsSync` + line-count of existing file content; never throw — wrap in `try/catch` and on failure log error to `console.error` then continue (soft-error, not hard block).
    - _Requirements: 14.2, 14.3, 14.7, 14.8_

  - [ ] 6.4 Write `mods/audit-trail/README.md`
    - Document: purpose (SOC 2 Type II / PCI-DSS Level 1 audit trail), reach level L1, env vars (`VANGUARD_AUDIT_LOG_PATH`, `VANGUARD_AUDIT_SESSION_ID`, `VANGUARD_AUDIT_AGENT_ID`), JSONL entry schema, events (`PostToolUse`), Codex limitation (POSIX-only, no jq), harness install notes, and the fail-closed behavior (exit 1 if log write fails).
    - _Requirements: 17.5_

  - [ ] 6.5 Add P3 (Audit Completeness) property test to `tests/fuzz-properties.test.mjs`
    - **Property 3: Audit Completeness**
    - Add import for `buildLogEntry` (inline the function in the test since it's not exported from a module yet — duplicate the pure logic from the `audit-trail` Claude Code register.ts).
    - Use `fc.array(fc.record({ tool_name: fc.string({ minLength: 1, maxLength: 30 }), tool_input: fc.object(), tool_output: fc.oneof(fc.string(), fc.object(), fc.integer()) }), { minLength: 1, maxLength: 50 })` as the arbitrary.
    - For each array, reduce by appending `JSON.stringify(buildLogEntry(evt, i + 1)) + "\n"` for each event with 1-based index.
    - Assert: `lines.length == events.length` and `lines[i].sequence_number == i + 1` for all i.
    - Run 200 iterations.
    - **Validates: Requirements 14.2, 14.3, 16.3**

  - [ ] 6.6 Checkpoint: Run `npm run validate:mods` — must exit 0 with all five mods not yet complete but `audit-trail` counted. Ask the user if questions arise.

- [ ] 7. T7 — PBT extensions and full validation wiring

  - [ ] 7.1 Add P4 (Manifest Validity Round-Trip) property test to `tests/fuzz-properties.test.mjs`
    - **Property 4: Manifest Validity Round-Trip**
    - Import or inline the `parseModToml` and `extractCatalogEntry` functions from `scripts/generate-mods.mjs` (inline pure copies in the test file to avoid module coupling — the functions are pure string → object transforms).
    - Use `fc.record({ id: fc.stringMatching(/^[a-z0-9][a-z0-9-]+$/), name: fc.string({ minLength: 1, maxLength: 50 }), version: fc.tuple(fc.nat({ max: 9 }), fc.nat({ max: 99 }), fc.nat({ max: 99 })).map(([a,b,c]) => \`${a}.${b}.${c}\`), reach_level: fc.integer({ min: 0, max: 3 }), events: fc.array(fc.constantFrom("PreToolUse","PostToolUse","SessionStart","Stop"), { minLength: 1, maxLength: 4 }), harnesses: fc.array(fc.constantFrom("claude-code","kiro","codex"), { minLength: 1, maxLength: 3 }) })` as the arbitrary.
    - Serialize to a minimal TOML string, parse via the inline `parseModToml`, extract catalog entry, JSON.stringify → JSON.parse, then assert all fields have correct types and value ranges.
    - Run 200 iterations.
    - **Validates: Requirements 8.2, 16.4**

  - [ ] 7.2 Add P5 (Generator Idempotency) property test to `tests/fuzz-properties.test.mjs`
    - **Property 5: Generator Idempotency**
    - Use the same arbitrary as P4 (valid `mod.toml` content object).
    - Call an inline `generateKiroHookContent(mod)` (the pure string-building function from the generator) once and twice; assert the two outputs are byte-for-byte identical.
    - Also call `generateRegisterTsContent(mod)` and `generateHooksJsonContent(mod)` similarly.
    - Assert all `generate*Content(mod) == generate*Content(generate*Content(mod))` (the function is its own fixed point).
    - Run 200 iterations.
    - **Validates: Requirements 6.7, 16.5**

  - [ ] 7.3 Run the full `npm run validate:mods` gate with all five mods present and confirm exit 0
    - Run `node scripts/generate-mods.mjs` (writes all artifacts).
    - Run `python3 tests/validate-mods.py` and confirm: exit 0; summary line shows 5 mods; all P1, P2 sub-tests pass; generator `--check` reports no drift; `catalog/mods.json` has 5 valid entries; no `FAIL` lines.
    - Fix any gate failures found before proceeding.
    - _Requirements: 7.2, 7.3, 7.4, 7.6_

  - [ ] 7.4 Run `node tests/fuzz-properties.test.mjs` and confirm all properties pass
    - Execute the test file and verify all `PASS` lines print including P3, P4, P5.
    - Fix any failures.
    - _Requirements: 16.3, 16.4, 16.5, 16.7_

- [ ] 8. T8 — Integration, asset integrity, and repo-level wiring

  - [ ] 8.1 Run `npm run mods:write` and then `npm run asset-integrity:write` to add all mod files to the integrity manifest
    - Run `node scripts/generate-mods.mjs` (ensure all 5 mods × 3 harnesses are fully generated).
    - Run `python3 tests/validate-asset-integrity.py --write` to hash the settled tree including `mods/` and `catalog/mods.json`.
    - Confirm `catalog/asset-integrity.json` now has entries under the `mods` tree.
    - _Requirements: 9.1_

  - [ ] 8.2 Run `npm run validate` in its entirety and fix any failures
    - Run the full validate chain: `npm run validate`.
    - Fix any `FAIL` lines across all gates before proceeding. Common failure modes to check: `validate:catalog` rejecting `catalog/mods.json`; `validate:asset-integrity` showing drift after generator re-run; `validate:links` finding broken links in `mods/README.md`; `validate:kiro-powers` (no impact expected since mods are not powers); `validate:codex-marketplace` (confirm `mods/` is not accidentally parsed as a codex plugin).
    - _Requirements: 7.1, 7.2_

  - [ ] 8.3 Update `CLAUDE.md` with a `## Mods` section
    - Add a `## Mods` section to `CLAUDE.md` after the existing `## Marketplaces & export` section (or as a new top-level section under "What this repo contains") documenting:
      - The `mods/` directory layout (mod.toml, claude-code/, kiro/, codex/, README.md per mod).
      - The reach level taxonomy (L0–L3, one line each with an example mod).
      - The `mods:write` command (`node scripts/generate-mods.mjs`) — when to run it (after editing any `mod.toml`, `patterns.json`, or `logic.sh`).
      - The `validate:mods` gate — what it checks (schema, reach level, env reads, hook JSON, bash -n, register.ts signature, generator drift, catalog entries, P1+P2 PBT).
      - A contributor checklist item: when adding a new mod, run `npm run mods:write`, `npm run asset-integrity:write`, and `npm run validate` before opening a PR.
    - Update the "Quick start" section of `CLAUDE.md` to mention `npm run mods:write` and `validate:mods`.
    - _Requirements: 9.3, 9.4, 9.5_

  - [ ] 8.4 Run `npm run lint:spell` and `npx --yes markdownlint-cli2 "**/*.md" "#node_modules"` — fix all failures
    - Spell-check new `.md` files in `mods/`, updated `CLAUDE.md`, and `mods/README.md`.
    - Add any legitimate technical terms tripped as false positives (e.g. `JSONL`, `POSIX`, `pretooluse`, `posttooluse`) to `.codespellrc` `ignore-words-list`.
    - Fix any markdown lint failures in the new files.
    - _Requirements: definition-of-done step 4_

  - [ ] 8.5 Checkpoint: All gates green. Run `git status` — confirm tree is clean (all generated files committed). Confirm branch is `feat/cross-harness-mods`. Ask the user if questions arise.

---

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP delivery. Core property tests (P1, P2 in validate-mods.py) are non-optional; only the fuzz-properties.test.mjs extensions are marked optional.
- Generated files (`register.ts`, Kiro hook JSON, Codex `.sh` boilerplate) are produced by `npm run mods:write` — never hand-edit them. Only `mod.toml`, `patterns.json`, `logic.sh`, and `README.md` files are hand-authored.
- The generator `--check` mode is the drift gate: if on-disk files differ from generated, `validate:mods` fails.
- Run `npm run asset-integrity:write` LAST (after all generator runs and file edits), as described in CLAUDE.md.
- `catalog/index.json` must include the `mods` key before `validate:catalog` runs; Task 1.3 handles this.
- Reach level validation note: the validator should accept declared reach level ≥ derived reach level (conservative declaration is valid) when `security_notes` is present in the mod entry, per the `live-guard-gate` design decision. Strict equality is enforced for non-justified over-declaration.

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "tasks": ["1.1", "1.2", "1.3", "1.4", "1.5"] },
    { "id": 1, "tasks": ["1.6"] },
    { "id": 2, "tasks": ["1.7"] },
    { "id": 3, "tasks": ["1.8"] },
    { "id": 4, "tasks": ["2.1", "3.1", "4.1", "5.1", "6.1"] },
    { "id": 5, "tasks": ["2.2", "3.2", "4.2", "5.2", "6.2"] },
    { "id": 6, "tasks": ["2.3"] },
    { "id": 7, "tasks": ["2.4", "3.4", "4.3", "5.3", "6.3"] },
    { "id": 8, "tasks": ["2.5", "3.5", "4.4", "5.4", "6.4"] },
    { "id": 9, "tasks": ["2.6", "3.6", "6.5", "3.3"] },
    { "id": 10, "tasks": ["7.1", "7.2", "7.3"] },
    { "id": 11, "tasks": ["7.4"] },
    { "id": 12, "tasks": ["8.1"] },
    { "id": 13, "tasks": ["8.2"] },
    { "id": 14, "tasks": ["8.3"] },
    { "id": 15, "tasks": ["8.4"] }
  ]
}
```
