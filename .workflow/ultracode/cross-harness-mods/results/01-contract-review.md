# 01 — Native harness contract review

Status: complete (orchestrator, read-only). Date: 2026-10-14.

## Claude Code (bundled plugin-authoring skill, build 2.1.296; types/claude-code.d.ts)

Verified:

- A mod is a plugin with three parts: `.claude-plugin/plugin.json`, `hooks/hooks.json` = `{ "modules": ["./register.ts"] }`, and a module that exports `register(on, options)`. The `Register` type comes from `'claude-code'`.
- A hook is `on('tool.call', { tool: 'Bash' }, ($, e, next) => …)`.
  - To block, return `{ deny }`.
  - To rewrite the input, call `next({ ...e, x })`.
  - To act after the tool runs, use `await next(e)` and then return `{ result }` (the hook's own result replaces the tool's).
  - A guard must also chain `.catch(($, e, next) => next.called ? next(e) : { deny })`.
- Runtime limits:
  - No Node and no `require`. Only web APIs are available, including `crypto.subtle`, which gives SHA-256.
  - `$.fs` has `read`, `write` (replaces the whole file), `list` and `stat`. **There is no append.**
- Options are declared in the manifest's `userConfig` and passed in as `options`.
- The classic settings-hook protocol is reachable as `classic.PreToolUse`, with `permissionDecision` = allow, deny or ask. `PostToolUse` supports `updatedToolOutput`.
- The API is labelled "early access; moves between releases".

The spec does not match this:

- It uses `export default function register(on: ModEventEmitter)`.
- It uses Node `readFileSync`, `crypto` and `appendFileSync`.

## Kiro (kiro.dev/docs/hooks.md, /docs/hooks/actions.md, /docs/hooks/types.md, /docs/cli/v3/hooks-migration.md)

Verified:

- Files are `.kiro/hooks/*.json` (global: `~/.kiro/hooks/`).
- The schema is `{ "version": "v1", "hooks": [{ name, description?, trigger, matcher?, action: { type: "command"|"agent", command|prompt }, timeout?, enabled?, confirm? }] }`. **The spec's schema is correct.**
- Triggers: SessionStart, UserPromptSubmit, PreToolUse, PostToolUse, PreTaskExec, PostTaskExec, PostFile{Create,Save,Delete}, Stop, Manual. **PostTaskExec is real.**
- Exit 2 blocks PreToolUse and PreTaskExec, and stderr goes to the LLM. On the IDE tab, any non-zero exit blocks.
- PostToolUse cannot block.

Not documented, so treated as unsupported:

- any stdout JSON protocol;
- `permissionDecision: "ask"`;
- modifying tool input;
- modifying tool output;
- the field names in the stdin payload.

The spec does not match this:

- Pattern C (ask JSON) and Pattern D (modify) are not in Kiro's docs. They are Claude Code's classic-hook protocol.

## Codex (Context7 /openai/codex — codex-rs/hooks, codex-rs/config, exec tests)

Verified:

- Configuration is `hooks.json` in the shape `{ "hooks": { "PreToolUse": [{ "matcher"?, "hooks": [{ "type": "command", "command" }] }] } }`.
- Events: PreToolUse, PermissionRequest, PostToolUse, Pre/PostCompact, SessionStart/End, UserPromptSubmit, Subagent{Start,Stop}, Stop, Interrupt.
- The stdin input is Claude-compatible: session_id, turn_id, cwd, hook_event_name, tool_name, tool_input, tool_use_id and others.
- PreToolUse decisions:
  - `deny` needs a non-empty reason.
  - `updatedInput` needs `permissionDecision: "allow"`.
  - **`ask` is rejected.**
  - Exit 2 with stderr blocks.
- PostToolUse supports `additionalContext` and block.
- **Hook trust gating:** a new or modified project hook does not run until it has been reviewed, or `--dangerously-bypass-hook-trust` is passed.

The spec does not match this:

- It says "POSIX .sh registered as Codex hooks" but gives no registration file and no output protocol.
- Live-guard "ask" cannot be expressed in Codex. The fallback has to be deny.

## Internal contradictions in the spec

1. The reach levels do not follow the formula:
   - blast-radius-guard declares L1 but derives L0;
   - live-guard-gate declares L2 but derives L1.
2. Req 13.1 (SessionStart/PostToolUse) and Req 13.2 (PostTaskExec) conflict.
3. audit-trail problems:
   - its "immutable / non-repudiable" claims are unsupported;
   - the per-session sequence comes from a line count in a shared daily file, which is racy;
   - Req 14.12 asks for POSIX with no jq, so it cannot parse JSON or reproduce the TS hashes;
   - Claude Code `$.fs` has no append.
4. secret-redactor problems:
   - a regex over serialized JSON breaks the structure;
   - a generic 40-char pattern matches git SHAs.
5. The `(?i)` and `\s` patterns do not port across Python, JS and `grep -E`.
6. P5 `generate(generate(t))` is ill-typed. Tasks 6.5 and 7.1 inline copies of the logic, which the eval contract forbids.
7. The env-var scan (Req 2.3) would flag shell locals.
8. Req 9.2 has the scanner ignore executable `mods/*/codex/*.sh` and `mods/*/kiro/*.sh`, which weakens the security scan.
9. Generation of the Kiro companion `.sh` is not specified.
10. `live-patterns.json` is called generated in the design and hand-authored in task 4.2.
11. The shebang is `bash`, but the spec says POSIX.

## Corrections to the earlier interim notes

- The Kiro `version: v1` hook schema and the `PostTaskExec` trigger were flagged as "likely invented". **Both are verified.**
- "PostToolUse output modification is not generally possible" was wrong for Claude Code, which has `updatedToolOutput` and a mod-returned `{ result }`. It is still unsupported or undocumented for Kiro and Codex.
