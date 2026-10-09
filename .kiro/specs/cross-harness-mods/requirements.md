# Requirements Document

## Introduction

This document specifies the `mods/` directory feature for the `vanguard-frontier-agentic` repository. A **mod** is a named, cross-harness behavioral extension package that ships harness-native implementations for Claude Code, Kiro, and Codex from a single harness-agnostic `mod.toml` manifest. The feature introduces a `mods/` top-level directory containing five enterprise-grade starter mods (`secret-redactor`, `blast-radius-guard`, `live-guard-gate`, `context-burn`, `audit-trail`), a generator script that compiles each mod to its target harness format, a `validate:mods` gate that joins `npm run validate`, and a new `catalog/mods.json` catalog file consistent with the existing `catalog/agents.json` schema conventions. The feature is designed to meet SOC 2 Type II, PCI-DSS Level 1, and zero-trust requirements for Fortune 50 enterprises.

## Glossary

- **Mod**: A named behavioral extension package living under `mods/<mod-id>/`. Contains a `mod.toml` manifest, per-harness subdirectories, and optional companion scripts.
- **Mod_ID**: A kebab-case string that uniquely identifies a mod (e.g. `secret-redactor`). Must match `^[a-z0-9][a-z0-9-]+$`.
- **mod.toml**: The harness-agnostic manifest file at the root of each mod directory. Declares identity, events, reach level, and per-harness capability flags.
- **Reach_Level**: An integer from 0 to 3 encoding a mod's environmental footprint. L0 = pure in-process/in-memory; L1 = reads or writes local files or env; L2 = spawns subprocesses or agent processes; L3 = initiates outbound network I/O.
- **Generator**: The Node.js script at `scripts/generate-mods.mjs` that reads every `mod.toml` and writes the corresponding harness-specific implementation files.
- **Harness**: A supported AI coding assistant runtime: `claude-code`, `kiro`, or `codex`. Cursor support is documented as future work.
- **Claude_Code_Mod**: The `claude-code/register.ts` TypeScript module inside a mod directory that exports a `register(on, options)` function conforming to the Claude Code mod SDK.
- **Kiro_Hook**: A JSON file under `mods/<mod-id>/kiro/` conforming to the Kiro hook schema (`version: "v1"`, `hooks` array, PascalCase trigger names).
- **Codex_Hook**: A POSIX-compliant shell script under `mods/<mod-id>/codex/` that implements the mod's behavior for the Codex hook model.
- **Catalog_Entry**: A JSON object in `catalog/mods.json` describing a mod's identity, reach level, harness support, and file paths.
- **Capability_Flag**: A boolean field in `mod.toml` under `[harnesses.<harness>]` indicating whether that harness can fully implement the mod's declared events.
- **validate:mods**: The npm script alias for `python3 tests/validate-mods.py`, which is a gate in `npm run validate`.
- **mods:write**: The npm script alias for `node scripts/generate-mods.mjs`, which regenerates all harness-specific mod files from their `mod.toml` sources.
- **PreToolUse_Hook**: A Kiro hook (or Claude Code mod event handler) that fires before a tool call executes. Exiting with code 2 (Kiro) or returning `{ deny }` (Claude Code) blocks the call.
- **PostToolUse_Hook**: A hook that fires after a tool call completes and receives the result.
- **Footprint_Declaration**: The `[footprint]` table in `mod.toml` that enumerates what env vars a mod reads, what paths it writes, what processes it spawns, and what hosts it contacts.
- **Redaction_Pattern**: A regular expression listed in `secret-redactor`'s pattern catalog that matches a known credential format (e.g. AWS access key, GitHub PAT, generic bearer token).
- **Blast_Radius_Pattern**: A shell command fragment or tool-argument substring listed in `blast-radius-guard`'s pattern catalog that indicates a potentially destructive operation.
- **Audit_Log**: The append-only JSONL file written by the `audit-trail` mod. Each line is a JSON object with `timestamp`, `session_id`, `agent_id`, `tool_name`, `args_hash`, `result_hash`, and `sequence_number` fields.
- **Burn_Status_File**: The local JSON file written by `context-burn` recording cumulative token counts and threshold breach events for the current session.
- **Live_Guard_Agent**: An agent whose ID matches the pattern `*-live-*-guard-agent` in `catalog/agents.json`. The `live-guard-gate` mod routes PreToolUse decisions through these agents.
- **Asset_Integrity_Manifest**: The file `catalog/asset-integrity.json` containing SHA-256 hashes for tracked assets, enforced by `validate:asset-integrity`.
- **POSIX_Script**: A shell script that uses only features defined in POSIX.1-2017, executable on both Linux (bash ≥ 5) and macOS (bash ≥ 3.2 / zsh).

---

## Requirements

### Requirement 1: Mod Directory Structure

**User Story:** As a repository contributor, I want every mod to follow a predictable directory layout with a machine-readable manifest, so that the generator, validator, and harness install commands can operate without per-mod configuration.

#### Acceptance Criteria

1. THE Mod_Directory SHALL have the path `mods/<mod-id>/` relative to the repository root, where `<mod-id>` matches `^[a-z0-9][a-z0-9-]+$`.
2. WHEN the Generator runs, THE Mod_Directory SHALL contain a `mod.toml` file, a `claude-code/` subdirectory, a `kiro/` subdirectory, and a `codex/` subdirectory.
3. THE `mod.toml` SHALL contain a `[mod]` table with the required keys `id`, `name`, `description`, `version`, `reach_level`, and `events`.
4. THE `mod.toml` SHALL contain a `[footprint]` table declaring `env_reads`, `file_writes`, `spawns`, and `egress_hosts` as arrays (empty arrays are valid).
5. THE `mod.toml` SHALL contain a `[harnesses]` table with sub-tables for each supported harness (`claude-code`, `kiro`, `codex`), each containing a boolean `supported` key.
6. WHEN a harness sub-table has `supported = false`, THE Generator SHALL create a `UNSUPPORTED.md` file in the corresponding harness subdirectory explaining why that harness cannot implement the mod's events, and SHALL NOT generate stub implementation files.
7. THE `mod.toml` `id` field SHALL exactly match the parent directory name.
8. THE `mod.toml` `reach_level` field SHALL be an integer in the range [0, 3] inclusive.
9. IF a `mod.toml` file is malformed or missing required keys, THEN THE `validate:mods` gate SHALL exit with a non-zero status and print a line beginning with `FAIL` identifying the file path and the missing or invalid field.
10. THE `mods/` directory SHALL contain a `README.md` documenting the mod format, the reach level taxonomy, and the install command for each harness.

---

### Requirement 2: Mod Manifest Schema (`mod.toml`)

**User Story:** As a security reviewer at a Fortune 50 enterprise, I want the mod's entire environmental footprint declared in machine-readable form before installation, so that I can approve or reject it through our change management process.

#### Acceptance Criteria

1. THE mod.toml `[mod]` table SHALL declare `reach_level` as an integer matching the actual maximum footprint: L0 when `[footprint].egress_hosts`, `[footprint].spawns`, and `[footprint].file_writes` are all empty; L1 when `file_writes` is non-empty but `spawns` and `egress_hosts` are empty; L2 when `spawns` is non-empty but `egress_hosts` is empty; L3 when `egress_hosts` is non-empty.
2. WHEN `validate:mods` runs, THE Validator SHALL verify that the declared `reach_level` matches the actual footprint and report a `FAIL` if there is a discrepancy.
3. THE `[footprint].env_reads` array SHALL enumerate every environment variable the mod's implementation files reference; THE Validator SHALL detect undeclared env reads by scanning `codex/*.sh` and `kiro/*.sh` files for `$VAR` and `${VAR}` patterns and confirming each is listed.
4. THE `[footprint].file_writes` array SHALL enumerate every file path (or glob pattern) the mod may write to; THE Validator SHALL detect undeclared write targets by scanning for shell redirection operators (`>`, `>>`, `tee`) in mod scripts.
5. WHERE an L3 mod is present, THE Validator SHALL emit a `WARN` (not `FAIL`) message identifying the mod and its egress hosts, and SHALL NOT block the gate.
6. THE `mod.toml` `version` field SHALL follow semantic versioning format `MAJOR.MINOR.PATCH`.
7. THE `mod.toml` `events` array SHALL contain only values from the set: `PreToolUse`, `PostToolUse`, `SessionStart`, `Stop`, `UserPromptSubmit`, `PreTaskExec`, `PostTaskExec`, `PostFileCreate`, `PostFileSave`, `PostFileDelete`.
8. IF a `mod.toml` declares an event in `events` but no harness sub-table implements a handler for that event, THEN THE Validator SHALL emit a `FAIL` identifying the unimplemented event.

---

### Requirement 3: Kiro Harness Implementation

**User Story:** As a Kiro IDE user at an enterprise, I want each mod to deploy as valid Kiro hook JSON files so that the mod's behavior activates automatically when I install it without any manual configuration.

#### Acceptance Criteria

1. WHEN the Generator runs for a Kiro-supported mod, THE Generator SHALL produce at least one `.json` file under `mods/<mod-id>/kiro/` for each declared event that Kiro supports.
2. THE Kiro_Hook JSON SHALL conform to the Kiro v1 hook schema: a top-level `version` field with value `"v1"` and a `hooks` array where each element has `name`, `trigger`, and `action` fields.
3. THE `trigger` value in every Kiro_Hook SHALL be a PascalCase string from the set of Kiro-supported triggers: `PreToolUse`, `PostToolUse`, `SessionStart`, `Stop`, `UserPromptSubmit`, `PreTaskExec`, `PostTaskExec`, `PostFileCreate`, `PostFileSave`, `PostFileDelete`.
4. WHEN a Kiro hook action type is `command`, THE hook JSON SHALL include a `command` string field containing the shell command to execute.
5. WHEN a Kiro PreToolUse hook needs to block a tool call, THE hook command SHALL exit with code 2 and write the reason to stderr.
6. WHEN a Kiro PreToolUse hook needs to prompt the user for confirmation, THE hook command SHALL exit with code 0 and write a JSON object `{"hookSpecificOutput":{"permissionDecision":"ask","permissionDecisionReason":"<reason>"}}` to stdout.
7. WHEN a Kiro PreToolUse hook needs to pass modified tool arguments to the next handler, THE hook command SHALL exit with code 0 and write the modified arguments as a JSON object to stdout.
8. THE `validate:mods` gate SHALL parse every Kiro hook JSON file and report a `FAIL` if any file fails JSON parsing or violates the v1 schema.
9. WHEN a Kiro hook references a companion shell script, THE companion script SHALL be co-located under `mods/<mod-id>/kiro/` and SHALL be POSIX-compliant.
10. THE Kiro hook files generated for a mod SHALL be installable by copying the `.json` files to `.kiro/hooks/` and the companion scripts to a path reachable from the hook's `command` field.

---

### Requirement 4: Claude Code Harness Implementation

**User Story:** As a Claude Code user, I want each mod to ship a valid TypeScript `register.ts` file that I can reference in my `.claude/mods/` directory so that the mod activates in-process with the full Claude Code SDK surface.

#### Acceptance Criteria

1. WHEN the Generator runs for a Claude Code-supported mod, THE Generator SHALL produce a `mods/<mod-id>/claude-code/register.ts` file.
2. THE `register.ts` file SHALL export a default function with the signature `export default function register(on: ModEventEmitter, options: ModOptions): void`.
3. THE `register.ts` file SHALL use only TypeScript language features compatible with TypeScript 5.x strict mode.
4. WHEN a Claude Code mod needs to block a tool call, THE `register.ts` SHALL call `on("PreToolUse", ...)` and return `{ deny: "<reason>" }` from the handler.
5. WHEN a Claude Code mod needs to request user confirmation before a tool call, THE `register.ts` SHALL return `{ ask: true, reason: "<reason>" }` from the PreToolUse handler.
6. WHEN a Claude Code mod needs to modify tool arguments before execution, THE `register.ts` SHALL return `{ modified: <newArgs> }` from the PreToolUse handler.
7. THE `mods/<mod-id>/claude-code/` directory SHALL also contain a `hooks.json` file declaring the event subscriptions for tooling that performs static analysis without loading the TypeScript module.
8. THE Validator SHALL check that the `hooks.json` event list matches the events registered in `register.ts` via static pattern matching on `on("<EventName>"` call sites.
9. THE `register.ts` file SHALL not import any module that requires a running network service or filesystem mutation at module load time (i.e., imports must be purely structural at load).
10. IF a `register.ts` file references an environment variable at runtime, THEN that variable SHALL be listed in the mod's `[footprint].env_reads` array.

---

### Requirement 5: Codex Harness Implementation

**User Story:** As a Codex CLI user, I want each mod to ship POSIX-compliant shell scripts that I can register as Codex hooks so that the mod's behavior works in my terminal-based AI workflow.

#### Acceptance Criteria

1. WHEN the Generator runs for a Codex-supported mod, THE Generator SHALL produce at least one `.sh` file under `mods/<mod-id>/codex/` for each declared event that the Codex hook model supports.
2. THE Codex hook scripts SHALL be POSIX-compliant and SHALL execute correctly on Linux (bash ≥ 5.0) and macOS (bash ≥ 3.2 / zsh).
3. THE Codex hook scripts SHALL begin with the shebang line `#!/usr/bin/env bash`.
4. THE Codex hook scripts SHALL set `set -euo pipefail` or equivalent error handling in the first executable line after the shebang.
5. WHEN a Codex PreToolUse script needs to block a tool call, THE script SHALL exit with code 2 and write the reason to stderr.
6. THE Validator SHALL check every Codex shell script with `bash -n` (syntax check only) and report a `FAIL` if the syntax check fails.
7. IF a Codex script uses an external command (e.g. `jq`, `sed`, `awk`), THEN the `mod.toml` `[harnesses.codex]` table SHALL declare that dependency in a `requires` array, and THE Validator SHALL confirm the array is present when such commands appear in the script.
8. THE Codex hook scripts SHALL NOT require a running network service at execution time unless the mod's `reach_level` is L3.

---

### Requirement 6: Generator Script (`scripts/generate-mods.mjs`)

**User Story:** As a repository maintainer, I want a single command to regenerate all harness-specific mod files from their `mod.toml` sources, so that I never maintain Claude Code TypeScript and Kiro JSON by hand out of sync with the manifest.

#### Acceptance Criteria

1. THE Generator SHALL be executable as `node scripts/generate-mods.mjs` from the repository root.
2. THE Generator SHALL read every `mods/*/mod.toml` file and, for each mod, write all harness-specific files to the appropriate subdirectories.
3. THE Generator SHALL support a `--check` flag: WHEN `--check` is passed, THE Generator SHALL compare the on-disk files against what it would generate and exit with code 1 printing the paths of files that are out of sync, without writing any files.
4. THE Generator SHALL be added to `package.json` as `"mods:write": "node scripts/generate-mods.mjs"`.
5. THE `validate:mods` gate SHALL invoke the Generator with `--check` as part of its validation to detect generator drift.
6. WHEN the Generator encounters a harness with `supported = false` in `mod.toml`, THE Generator SHALL skip harness-specific file generation for that harness and SHALL NOT report an error.
7. THE Generator SHALL be deterministic: WHEN given the same `mod.toml` contents on two separate runs, THE Generator SHALL produce byte-for-byte identical output files.
8. THE Generator SHALL exit with code 0 and print a summary line `Generated <N> mods for <M> harnesses` on success.
9. IF a `mod.toml` fails TOML parsing, THEN THE Generator SHALL print an error identifying the file and SHALL exit with code 1 after processing all remaining mods, not immediately.
10. THE Generator SHALL NOT invoke any network service, shell command, or filesystem path outside the repository root during generation.

---

### Requirement 7: Validation Gate (`tests/validate-mods.py`)

**User Story:** As a CI engineer, I want the `validate:mods` gate to be part of `npm run validate` so that any malformed or out-of-sync mod is caught before a pull request merges.

#### Acceptance Criteria

1. THE `validate:mods` script SHALL be added to `package.json` as `"validate:mods": "python3 tests/validate-mods.py"` and SHALL be appended to the `validate` script command chain.
2. THE `validate:mods` gate SHALL exit with code 0 if and only if all of the following conditions hold: every `mods/*/mod.toml` is valid; all Kiro hook JSON files are schema-conformant; all Codex shell scripts pass `bash -n`; all `register.ts` files contain the required export signature; the Generator `--check` reports no drift; `catalog/mods.json` entries match the on-disk mods.
3. THE `validate:mods` gate SHALL print exactly one line per failure beginning with `FAIL [mods]` followed by the file path and a human-readable description.
4. THE `validate:mods` gate SHALL print a summary line `OK: <N> mods valid (<harness breakdown>)` on success.
5. THE `validate:mods` gate SHALL have no external Python package dependencies beyond the standard library and `tomllib` (Python ≥ 3.11 standard library).
6. THE `validate:mods` gate SHALL complete in under 30 seconds on a repository checkout with the five starter mods.
7. WHEN the `mods/` directory does not exist, THE `validate:mods` gate SHALL exit with code 1 and print `FAIL [mods] mods/ directory is missing`.

---

### Requirement 8: Catalog Integration (`catalog/mods.json`)

**User Story:** As a harness plugin consumer, I want mods listed in a machine-readable catalog so that install tooling, documentation generators, and the asset-integrity manifest can enumerate mods programmatically.

#### Acceptance Criteria

1. THE `catalog/mods.json` file SHALL be a JSON array of Catalog_Entry objects, one per mod in `mods/`.
2. EACH Catalog_Entry SHALL contain the fields: `id` (string, kebab-case), `name` (string), `version` (string, semver), `description` (string), `reach_level` (integer 0–3), `events` (array of strings), `harnesses` (array of supported harness ids), and `path` (string, repo-relative path to the mod directory).
3. THE `validate:mods` gate SHALL verify that every mod in `mods/` has a corresponding entry in `catalog/mods.json` and that no entry references a non-existent mod directory.
4. THE `catalog/mods.json` file SHALL be regenerated by `node scripts/generate-mods.mjs` in the same run that regenerates harness files.
5. THE `validate:asset-integrity` gate SHALL cover `catalog/mods.json` and every file under `mods/` via the existing `npm run asset-integrity:write` pipeline.
6. EACH `id` in `catalog/mods.json` SHALL be unique and SHALL NOT collide with any `id` in `catalog/agents.json`, `catalog/skills.json`, or `catalog/rules.json`.

---

### Requirement 9: Asset Integrity and Repository Integration

**User Story:** As a release engineer, I want all mod files covered by the asset-integrity manifest and the `.plugin-scanner.toml` exclusion list updated so that CI gates do not produce false positives on the mod files that deliberately contain credential-shaped patterns.

#### Acceptance Criteria

1. WHEN `npm run asset-integrity:write` runs, THE Asset_Integrity_Manifest SHALL include SHA-256 hashes for every file under `mods/` and for `catalog/mods.json`.
2. THE `.plugin-scanner.toml` `[scanner].ignore_paths` array SHALL be updated to include `mods/*/kiro/*.sh`, `mods/*/codex/*.sh`, and `tests/fixtures/mods/**` to prevent secret-scanner false positives on regex patterns used by `secret-redactor`.
3. THE `CLAUDE.md` SHALL be updated with a `## Mods` section documenting the mod directory layout, the generator command, the `validate:mods` gate, and the reach level taxonomy.
4. THE `mods:write` and `validate:mods` scripts SHALL be documented in the "Quick start" section of `CLAUDE.md`.
5. WHEN a new mod is added to `mods/`, THE contributor checklist SHALL require running `npm run mods:write`, `npm run asset-integrity:write`, and `npm run validate` before opening a pull request.
6. THE `catalog/mods.json` file SHALL be listed in `catalog/index.json` under a `mods` key alongside the existing `agents`, `skills`, `rules`, and `mcp-references` keys.

---

### Requirement 10: `secret-redactor` Mod

**User Story:** As a DLP compliance officer at a Fortune 50 enterprise, I want every tool call's input and output automatically scanned and credential-shaped strings replaced with `[REDACTED]` before the model context is written, so that API keys, tokens, and passwords never appear in session logs or model prompts.

#### Acceptance Criteria

1. THE `secret-redactor` mod SHALL declare `reach_level = 0` and `events = ["PreToolUse", "PostToolUse"]` in its `mod.toml`.
2. THE `secret-redactor` mod SHALL ship a pattern catalog as a JSON file `mods/secret-redactor/patterns.json` containing at minimum: AWS access key (`AKIA[0-9A-Z]{16}`), AWS secret key (generic 40-char base62), GitHub PAT (`(ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9]{36,255}`), generic bearer token (`Bearer\s+[A-Za-z0-9\-._~+/]+=*`), and generic password-in-URL (`://[^@]*:[^@]+@`).
3. WHEN a PreToolUse event fires, THE `secret-redactor` implementation SHALL scan the serialized tool arguments against all patterns in `patterns.json` and replace every match with the literal string `[REDACTED:<pattern-name>]`.
4. WHEN a PostToolUse event fires, THE `secret-redactor` implementation SHALL scan the serialized tool result against all patterns in `patterns.json` and replace every match with the literal string `[REDACTED:<pattern-name>]`.
5. THE `secret-redactor` Kiro hook SHALL exit with code 0 and write the modified JSON to stdout when redactions are applied, preserving the original JSON structure with only the matched substrings replaced.
6. THE `secret-redactor` Kiro hook SHALL exit with code 0 and write nothing to stdout when no patterns match (pass-through).
7. IF a redaction pattern match occurs, THE `secret-redactor` implementation SHALL write a one-line log entry to stderr in the format `[secret-redactor] redacted <pattern-name> in <tool-name> <PreToolUse|PostToolUse>`.
8. THE `secret-redactor` pattern list SHALL be extensible: WHEN a `VANGUARD_SECRET_REDACTOR_PATTERNS` environment variable is set to a file path, THE implementation SHALL load additional patterns from that file and merge them with the built-in catalog.
9. WHERE the `VANGUARD_SECRET_REDACTOR_PATTERNS` feature is used, THE `mod.toml` `[footprint].env_reads` array SHALL declare `VANGUARD_SECRET_REDACTOR_PATTERNS` and the variable's file path SHALL be listed in `[footprint].file_writes` when it is a path the mod creates.

---

### Requirement 11: `blast-radius-guard` Mod

**User Story:** As a platform engineer at a Fortune 50 enterprise, I want destructive shell commands and infrastructure operations to be blocked or flagged for explicit confirmation before execution, so that a runaway agent cannot destroy production infrastructure.

#### Acceptance Criteria

1. THE `blast-radius-guard` mod SHALL declare `reach_level = 1` and `events = ["PreToolUse"]` in its `mod.toml`.
2. THE `blast-radius-guard` mod SHALL ship a pattern catalog as a JSON file `mods/blast-radius-guard/patterns.json` containing at minimum: `rm -rf` with a target of `/`, `/*`, or `~/`, `DROP TABLE`, `DROP DATABASE`, `DROP SCHEMA`, `git push --force` (or `-f`) to a protected branch pattern, `curl.*\|\s*sh`, `wget.*\|\s*sh`, `terraform.*destroy` with a `-auto-approve` flag, `kubectl delete namespace`, and `kubectl delete.*--all`.
3. WHEN a PreToolUse event fires for a Bash tool call, THE `blast-radius-guard` implementation SHALL scan the command string against all patterns in `patterns.json`.
4. WHEN a pattern match is found AND the `VANGUARD_BLAST_RADIUS_MODE` environment variable is set to `block`, THE implementation SHALL block the tool call by returning a deny decision with a human-readable reason identifying the matched pattern.
5. WHEN a pattern match is found AND the `VANGUARD_BLAST_RADIUS_MODE` environment variable is unset or set to `confirm`, THE `blast-radius-guard` Kiro hook SHALL exit with code 0 and write `{"hookSpecificOutput":{"permissionDecision":"ask","permissionDecisionReason":"<reason>"}}` to stdout to prompt the user for confirmation.
6. WHEN a pattern match is found AND the `VANGUARD_BLAST_RADIUS_MODE` environment variable is unset or set to `confirm` in the Claude Code implementation, THE `register.ts` SHALL return `{ ask: true, reason: "<reason>" }` from the PreToolUse handler.
7. WHEN no pattern match is found, THE `blast-radius-guard` implementation SHALL pass through the tool call without modification.
8. THE `blast-radius-guard` mod SHALL also intercept file-write tool calls: WHEN a write tool call targets a path matching the pattern `/(etc|usr|bin|sbin|boot|sys|proc)/`, THE implementation SHALL apply the same block-or-confirm logic.
9. THE `blast-radius-guard` pattern list SHALL be extensible via a `VANGUARD_BLAST_RADIUS_PATTERNS` environment variable pointing to an additional patterns JSON file, following the same merge convention as `secret-redactor`.
10. THE `mod.toml` `[footprint].env_reads` array SHALL declare `VANGUARD_BLAST_RADIUS_MODE` and `VANGUARD_BLAST_RADIUS_PATTERNS`.

---

### Requirement 12: `live-guard-gate` Mod

**User Story:** As a change management coordinator at a Fortune 50 enterprise, I want infrastructure mutation operations to be automatically routed through the repository's existing live-guard agents for approval, so that the mod formalizes the CAB-equivalent gate that the live-guard agents already provide.

#### Acceptance Criteria

1. THE `live-guard-gate` mod SHALL declare `reach_level = 2` and `events = ["PreToolUse"]` in its `mod.toml`.
2. THE `live-guard-gate` mod SHALL read the list of live-guard agents from `catalog/agents.json` at install time (not at hook invocation time) and persist it to `mods/live-guard-gate/live-agents.json` as part of the `mods:write` generation step.
3. WHEN a PreToolUse event fires, THE `live-guard-gate` Kiro hook SHALL check whether the tool call's target (command, path, or resource identifier) matches any pattern declared in `mods/live-guard-gate/live-patterns.json`.
4. WHEN a match is found, THE `live-guard-gate` Kiro hook SHALL exit with code 0 and write `{"hookSpecificOutput":{"permissionDecision":"ask","permissionDecisionReason":"<reason listing the matched live-guard agent>"}}` to stdout.
5. WHEN a match is found in the Claude Code implementation, THE `register.ts` SHALL return `{ ask: true, reason: "<reason>" }` from the PreToolUse handler with a message that identifies the live-guard agent the user should consult.
6. WHEN no match is found, THE `live-guard-gate` implementation SHALL pass through the tool call without modification.
7. THE `live-guard-gate` mod SHALL NOT spawn a live-guard agent subprocess automatically; it SHALL only prompt the user to invoke the appropriate live-guard agent manually.
8. THE `mod.toml` `[footprint].file_writes` array SHALL declare `mods/live-guard-gate/live-agents.json` since it is written by the generator.
9. THE `live-guard-gate` mod SHALL be listed in `catalog/mods.json` with a `security_notes` field warning that `reach_level = 2` is declared conservatively for future subprocess spawning capability and that the current implementation is effectively L1.

---

### Requirement 13: `context-burn` Mod

**User Story:** As a FinOps engineer at a Fortune 50 enterprise, I want token spend tracked per session with threshold warnings at 70% and 90% context fill, so that I can enforce cost center attribution and prevent runaway session costs.

#### Acceptance Criteria

1. THE `context-burn` mod SHALL declare `reach_level = 1` and `events = ["PostToolUse", "SessionStart"]` in its `mod.toml`.
2. THE `context-burn` Kiro hook for `PostTaskExec` SHALL read the current session's tool call count from the Burn_Status_File and write an updated entry after each task.
3. THE Burn_Status_File path SHALL default to `.kiro/context-burn-status.json` and SHALL be overridable via the `VANGUARD_CONTEXT_BURN_STATUS_PATH` environment variable.
4. THE `context-burn` Claude Code `register.ts` SHALL subscribe to PostToolUse events and maintain an in-memory token accumulator, emitting a warning message via `console.warn` when the accumulator crosses 70% and 90% of a configurable `max_tokens` option (default: 200 000).
5. THE `context-burn` mod SHALL accept a `max_tokens` configuration option in `mod.toml` under `[harnesses.claude-code.options]`; WHEN this option is set, THE `register.ts` SHALL use it instead of the default.
6. WHEN the 70% threshold is crossed, THE `context-burn` implementation SHALL emit a warning with the message `[context-burn] WARNING: 70% context used (<tokens> / <max> tokens)`.
7. WHEN the 90% threshold is crossed, THE `context-burn` implementation SHALL emit a warning with the message `[context-burn] CRITICAL: 90% context used (<tokens> / <max> tokens)`.
8. THE Burn_Status_File SHALL be a valid JSON file containing at minimum the fields `session_id`, `tool_call_count`, `last_updated` (ISO 8601), and `threshold_events` (array of threshold breach records).
9. THE `mod.toml` `[footprint].file_writes` array SHALL declare the default Burn_Status_File path.
10. THE `mod.toml` `[footprint].env_reads` array SHALL declare `VANGUARD_CONTEXT_BURN_STATUS_PATH`.

---

### Requirement 14: `audit-trail` Mod

**User Story:** As a SOC 2 compliance auditor, I want every tool call that an AI agent makes to be recorded in an immutable append-only log with timestamps and content hashes, so that I have non-repudiable evidence for SOC 2 Type II and PCI-DSS Level 1 audit purposes.

#### Acceptance Criteria

1. THE `audit-trail` mod SHALL declare `reach_level = 1` and `events = ["PostToolUse"]` in its `mod.toml`.
2. WHEN a PostToolUse event fires, THE `audit-trail` implementation SHALL append one JSON object (JSONL format) to the Audit_Log file.
3. EACH Audit_Log entry SHALL contain the fields: `timestamp` (ISO 8601 with millisecond precision), `session_id` (string), `agent_id` (string), `tool_name` (string), `args_hash` (SHA-256 hex of the serialized tool arguments), `result_hash` (SHA-256 hex of the serialized tool result), and `sequence_number` (monotonically increasing integer starting at 1 within a session).
4. THE Audit_Log file path SHALL default to `.audit/vanguard-audit-<YYYY-MM-DD>.jsonl` and SHALL be overridable via the `VANGUARD_AUDIT_LOG_PATH` environment variable.
5. THE Audit_Log file SHALL be opened in append mode only; THE `audit-trail` implementation SHALL NOT truncate or overwrite existing log content under any circumstances.
6. WHEN the Audit_Log file does not exist, THE `audit-trail` implementation SHALL create it (including parent directories) before appending the first entry.
7. THE `audit-trail` Kiro hook SHALL receive the tool call result on stdin as a JSON object and SHALL append the log entry before exiting with code 0.
8. THE `audit-trail` Claude Code `register.ts` SHALL subscribe to PostToolUse events and SHALL perform the log append synchronously before the handler returns.
9. THE `mod.toml` `[footprint].file_writes` array SHALL declare `.audit/vanguard-audit-*.jsonl` (glob).
10. THE `mod.toml` `[footprint].env_reads` array SHALL declare `VANGUARD_AUDIT_LOG_PATH`, `VANGUARD_AUDIT_SESSION_ID`, and `VANGUARD_AUDIT_AGENT_ID`.
11. WHERE a `VANGUARD_AUDIT_SESSION_ID` environment variable is not set, THE `audit-trail` implementation SHALL derive a session ID from a hash of the process start time and process ID.
12. THE `audit-trail` Codex shell script SHALL use only POSIX tools (`date`, `sha256sum` or `shasum -a 256`, `printf`, file redirection) to construct and append the log entry, with no dependency on `jq` unless declared in `mod.toml` `[harnesses.codex].requires`.

---

### Requirement 15: Mod Isolation and Composability

**User Story:** As a platform engineer deploying multiple mods simultaneously, I want mods to be isolated from each other so that installing two mods together does not produce conflicts, unexpected interactions, or ordering-dependent behavior.

#### Acceptance Criteria

1. THE Mod_ID namespace SHALL be globally unique: THE `validate:mods` gate SHALL fail if any two mods share the same ID.
2. EACH mod's Kiro hook JSON files SHALL use hook names prefixed with the Mod_ID (e.g. `secret-redactor:pre-tool-use`) to prevent name collisions when multiple mods are installed into `.kiro/hooks/`.
3. EACH mod's environment variable names SHALL be prefixed with `VANGUARD_` followed by the mod's ID in `SCREAMING_SNAKE_CASE` (e.g. `VANGUARD_SECRET_REDACTOR_PATTERNS`) to prevent cross-mod variable collisions.
4. EACH mod's log and status file paths SHALL use the mod's ID as a path component to prevent write collisions (e.g. `.audit/` for `audit-trail`, `.kiro/context-burn-status.json` for `context-burn`).
5. WHEN multiple mods are installed and both declare PreToolUse hooks for the same tool, THE Kiro hook execution order SHALL be determined by the alphabetical order of hook file names; THE Generator SHALL name hook files using a `<mod-id>-<event>-hook.json` convention that makes the ordering deterministic and documentable.
6. THE `validate:mods` gate SHALL verify that no two mods write to the same file path (or overlapping glob patterns) in their `[footprint].file_writes` declarations.
7. EACH mod's Claude Code `register.ts` SHALL be self-contained: it SHALL NOT import from another mod's directory.

---

### Requirement 16: Property-Based Correctness Properties

**User Story:** As a QA engineer, I want the five key correctness properties to be expressible as fast-check property tests so that the CI suite catches regressions in mod logic with minimal false negatives.

#### Acceptance Criteria

1. THE `tests/validate-mods.py` gate SHALL include a sub-suite that verifies **Redaction Completeness**: FOR ALL strings containing a substring matching any pattern in `secret-redactor`'s `patterns.json`, THE redaction function applied to the string SHALL return a string containing zero matches of that pattern.
2. THE `tests/validate-mods.py` gate SHALL include a sub-suite that verifies **Guard Blocking**: FOR ALL Bash command strings matching any pattern in `blast-radius-guard`'s `patterns.json`, the guard evaluation function SHALL return a non-pass decision (block or confirm).
3. THE `tests/fuzz-properties.test.mjs` file SHALL be extended with a property test verifying **Audit Completeness**: FOR ALL sequences of N tool-call event objects (N ∈ [1, 50]), the `audit-trail` log append function applied to each object in sequence SHALL produce a JSONL string containing exactly N lines, each parseable as valid JSON, with `sequence_number` values 1 through N in order.
4. THE `tests/fuzz-properties.test.mjs` file SHALL be extended with a property test verifying **Manifest Validity Round-Trip**: FOR ALL valid `mod.toml` content strings (constructed from the fast-check arbitrary), the TOML parse → Catalog_Entry extract → JSON serialize → JSON parse cycle SHALL produce an object with `id`, `name`, `version`, `reach_level`, `events`, and `harnesses` fields whose types and value ranges match the `mod.toml` source.
5. THE `tests/fuzz-properties.test.mjs` file SHALL be extended with a property test verifying **Generator Idempotency**: FOR ALL `mod.toml` inputs, running the Generator's per-mod transformation function twice on the same input SHALL produce byte-for-byte identical outputs on both runs.
6. THE **Redaction Completeness** and **Guard Blocking** property tests in `validate-mods.py` SHALL use only the Python standard library's `re` module and SHALL be self-contained inline within the test file.
7. THE property tests in `fuzz-properties.test.mjs` SHALL use the existing `fast-check` devDependency already present in `package.json`; no new devDependencies SHALL be added for these tests.

---

### Requirement 17: Install Commands and User-Facing Documentation

**User Story:** As a developer at an enterprise customer, I want a single documented install command per harness to deploy a mod, so that onboarding takes minutes rather than manual file copying.

#### Acceptance Criteria

1. THE `mods/README.md` SHALL document an install command for each supported harness in a `## Installation` section.
2. THE Kiro install command SHALL be: copy the mod's `.json` files from `mods/<mod-id>/kiro/` to `.kiro/hooks/` and any companion `.sh` scripts to a directory on `PATH` or to `.kiro/hooks/scripts/`.
3. THE Claude Code install command SHALL reference the Claude Code mods directory convention (`.claude/mods/<mod-id>/register.ts`).
4. THE Codex install command SHALL document how to register the mod's `.sh` scripts as Codex hooks.
5. EACH mod directory SHALL contain a `README.md` documenting: the mod's purpose, its reach level and footprint, the events it handles, and harness-specific install notes.
6. THE `mods/README.md` SHALL contain a reach level reference table with the four levels (L0–L3), their definitions, and an example mod for each level present in the starter set.
7. THE `mods/README.md` SHALL note that no mod requires a running network service at install time and that L3 mods are flagged (not blocked) during `validate:mods`.

---

## Correctness Properties (Formal Statements)

These properties are the ground truth for property-based testing. Any implementation that violates them is incorrect regardless of other acceptance criteria.

**P1 — Redaction Completeness:**
For all strings `s` and all patterns `p` in `secret-redactor/patterns.json`:
`redact(s, p)` does not match `p`.

Equivalently: `re.search(p.regex, redact(s, p)) is None` for all inputs `s`.

**P2 — Guard Blocking:**
For all command strings `cmd` matching any pattern `q` in `blast-radius-guard/patterns.json`:
`evaluate(cmd) ∈ {BLOCK, CONFIRM}`.

Equivalently: `evaluate(cmd).decision != "pass"` whenever `any(re.search(q.regex, cmd) for q in patterns)`.

**P3 — Audit Completeness:**
For all sequences of tool-call events `[e₁, e₂, …, eₙ]` where `n ≥ 1`:
`len(parse_jsonl(audit_log_after_n_appends)) == n`
AND `parse_jsonl(…)[i]["sequence_number"] == i + 1` for all `i ∈ [0, n-1]`.

**P4 — Manifest Validity Round-Trip:**
For all valid `mod.toml` content strings `t`:
`extract_catalog_entry(parse_toml(t))` produces an object with `id : str`, `name : str`, `version : str` matching `/^\d+\.\d+\.\d+$/`, `reach_level : int ∈ [0,3]`, `events : list[str]`, `harnesses : list[str]`.

**P5 — Generator Idempotency:**
For all `mod.toml` input `t`:
`generate(t) == generate(generate(t))`.

Equivalently: the output of the generator is a pure function of `mod.toml` content with no ambient state dependency.
