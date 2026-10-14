# Audit Trail

Appends one JSON line per tool call to a per-session log. It is an honest append log: it makes
no claim of immutability, carries no signatures, and any process with write access can edit or
delete the file.

## What it records

| Field | Value |
|---|---|
| `timestamp` | UTC ISO 8601 time of the hook run |
| `session_id` | The safe session id (see below) |
| `agent_id` | `VANGUARD_AUDIT_TRAIL_AGENT_ID`, or `null` |
| `tool_name` | The payload `tool_name`, or `null` |
| `args_hash` | SHA-256 of the canonical JSON of `tool_input` (`null` when absent) |
| `result_hash` | SHA-256 of the canonical JSON of `tool_response` (`null` when absent) |
| `sequence_number` | 1, 2, 3 ... per session file |

Only hashes are stored, never the arguments or results themselves.

## File layout

`<dir>/<YYYY-MM-DD>/<session>.jsonl`, where `<dir>` defaults to `.audit` and dates are UTC. If
the session already has a file under an earlier date directory, that file is reused, so the
sequence does not reset at midnight. A relative `<dir>` resolves against the project root when
the hook runs from its installed location, otherwise against the working directory.

## Session ids

1. `VANGUARD_AUDIT_TRAIL_SESSION_ID`, when set and non-empty.
2. The payload `session_id`, when a non-empty string.
3. Otherwise `unsessioned`.

An id that does not match `[A-Za-z0-9._-]{1,128}` is replaced by the first 32 hex characters of
its SHA-256, so it can never escape the date directory.

## Environment

| Variable | Purpose |
|---|---|
| `VANGUARD_AUDIT_TRAIL_LOG_DIR` | Log directory (default `.audit`). It can point outside the declared `.audit/**` write footprint; that is the operator's choice. |
| `VANGUARD_AUDIT_TRAIL_SESSION_ID` | Session id override |
| `VANGUARD_AUDIT_TRAIL_AGENT_ID` | Value for `agent_id` |

## Harness support

| Harness | Status |
|---|---|
| Claude Code native | Unsupported: the native API has no file append. Install the classic hook instead. |
| Claude Code classic | Supported. `PostToolUse` fires only after a tool succeeds, so failed calls are not logged. |
| Kiro | Degraded: stdin field names are undocumented, so `args_hash` covers the whole payload, `result_hash` hashes `null`, and without a session id env var every session shares the `unsessioned` log. |
| Codex | Supported. |

## Locking and failure

Each append takes an exclusive `<dir>/<session>.lock` before selecting the session file
(retrying for up to 2 s). Locks are never automatically stolen or removed based on age.
After a crash, confirm no writer remains before manually removing its orphaned lock.
A corrupt or unterminated last line, a lock timeout, or any I/O error exits 1 with a message on stderr and
appends nothing. A failure is loud but never blocks the tool call.

Requires Node 22+. Install: see `mods/README.md`.
