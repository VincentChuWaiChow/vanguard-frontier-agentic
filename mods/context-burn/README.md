# context-burn

Counts tool calls per session in a small status file. Where the harness reports token use,
it also warns once at 70% and once at 90% of the context window. It only observes: it never
blocks a tool and never changes a tool result.

## Reach and declared footprint

**L1 (file writes).** The manifest declares default writes to
`.claude/context-burn-status.json`, `.kiro/context-burn-status.json` and
`.codex/context-burn-status.json`, and reads `VANGUARD_CONTEXT_BURN_STATUS_PATH`.
It declares no subprocess spawns or network egress. This is capability disclosure,
not isolation; the status-path override can write outside these defaults.

## Status file

Fields (pretty-printed JSON, rewritten in full on every event):

- `session_id`: the session the counts belong to.
- `tool_call_count`: tool calls seen in that session.
- `last_updated`: ISO 8601 time of the last event.
- `threshold_events`: `{ threshold, tokens, window, at }` for each threshold crossed, at most
  one per threshold per session. Empty where no token data exists.

Default paths, relative to the project:

| Harness | Path |
|---|---|
| Claude Code | `.claude/context-burn-status.json` |
| Kiro | `.kiro/context-burn-status.json` |
| Codex | `.codex/context-burn-status.json` |

`VANGUARD_CONTEXT_BURN_STATUS_PATH` (when non-empty) replaces the path. An override can point
outside the declared `file_writes` footprint, which lists only the three defaults.

## Per harness

| Harness | Behavior |
|---|---|
| Claude Code native | `session.start` resets the file. After each tool call it reads context use from the session, toasts the 70% warning and the 90% critical message once each, and updates the file. |
| Claude Code classic | Counter only. No token data, so no warnings. |
| Kiro | Counter only, as above. See `kiro/UNSUPPORTED.md`. |
| Codex | Counter only, as above. See `codex/UNSUPPORTED.md`. |

Messages: `[context-burn] WARNING: 70% context used (<tokens> / <window> tokens)` and
`[context-burn] CRITICAL: 90% context used (<tokens> / <window> tokens)`.

## Limitations

- Install either the native or the classic surface, not both: they share
  `.claude/context-burn-status.json` and would double count.
- The Node hook reads, then writes, without a lock, so parallel hooks can lose an increment.
- Classic PostToolUse fires only after a tool succeeds, so failed calls are not counted.
- A status file that cannot be read or parsed starts a fresh count; the Node hook says so on
  stderr.
- The native API was verified against Claude Code 2.1.296. The Node hooks need Node 22+.

## Install

See `mods/README.md`.
