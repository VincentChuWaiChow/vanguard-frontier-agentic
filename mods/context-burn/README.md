# context-burn

Counts tool calls per session in a small status file. Where the harness reports token use,
it also warns once at 70% and once at 90% of the context window. It only observes: it never
blocks a tool and never changes a tool result.

## Reach and declared footprint

**L1 (file writes).** The manifest declares default writes to
`.claude/context-burn-status/*.json`, `.kiro/context-burn-status/*.json` and
`.codex/context-burn-status/*.json`, and reads `VANGUARD_CONTEXT_BURN_STATUS_PATH`.
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
| Claude Code | `.claude/context-burn-status/<session-hash>.json` |
| Kiro | `.kiro/context-burn-status/<session-hash>.json` |
| Codex | `.codex/context-burn-status/<session-hash>.json` |

`<session-hash>` is the full lowercase SHA-256 of the UTF-8 `session_id`; raw session IDs
never appear in the path. Missing or empty Node session IDs use `unsessioned`, so those
events share one fallback file rather than isolated session state.

`VANGUARD_CONTEXT_BURN_STATUS_PATH` (when non-empty) is an exact path override. Using the
same override across sessions opts out of default isolation. It can point outside the
declared `file_writes` globs. Old single-file state is not migrated automatically.

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
  the same session status file and would double count.
- Updates are unlocked read-modify-write operations, so same-session parallel hooks can
  lose increments. There is no automatic cleanup of old status files.
- Classic PostToolUse fires only after a tool succeeds, so failed calls are not counted.
- An unreadable or invalid status file, including malformed threshold event objects or
  fields, starts a fresh count; the Node hook says so on stderr.
- The native API was verified against Claude Code 2.1.296. The Node hooks need Node 22+.

## Install

See `mods/README.md`.
