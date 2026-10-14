# Blast Radius Guard

## What it does

Checks every tool call before it runs. It asks for confirmation, or blocks, when the call
looks destructive: a shell command from the built-in pattern list (recursive deletes of
system or home paths, SQL drops, force pushes to protected branches, piping a download
into a shell, and similar) or a write to a system path. Paths are normalized lexically
first, so `/tmp/../etc/passwd` is caught.

## Configuration

- `VANGUARD_BLAST_RADIUS_GUARD_MODE`: `confirm` (the default when unset) asks. Any other
  value, including a typo, blocks.
- `VANGUARD_BLAST_RADIUS_GUARD_PATTERNS`: optional path to a JSON file of extra patterns.
  They are appended to the built-in list and never replace it. If the file is set but
  cannot be read, is not valid JSON, is malformed, or reuses a built-in id, every call is
  denied.

The extension file is an array of entries with the same keys as `patterns.json`: `id`
(kebab-case, unique), `name`, `description`, `regex`, `flags` (`""` or `"i"`), `context`
(`["bash"]` for shell text or `["file-write"]` for normalized paths) and `severity`.

## Per-harness behavior

| Harness | Surface | Behavior |
| --- | --- | --- |
| Claude Code | native | `tool.check` guard. Never loosens the engine verdict; a hit asks (or denies in block mode). |
| Claude Code | classic | `PreToolUse` command hook; a hit asks (or denies in block mode). |
| Kiro | hook | Degraded. The whole payload is scanned, the system-path rule applies to the path keys whatever the tool, and confirm mode denies. |
| Codex | hook | Degraded. Codex rejects ask, so confirm mode denies with a reason. |

## Limitations

- It scans every string in the tool input, so a file write whose content mentions a
  destructive command also asks. This false positive is by design (fail closed).
- Relative paths are not resolved against a working directory.
- Kiro documents no stdin field names, so its whole payload is scanned.
- An `ask` is settled by the permission mode, so a bypass mode may auto-approve.
- Input over the size limit is not evaluated and counts as a hit.
- The native API is early access, verified against Claude Code 2.1.296.
- Classic, Kiro and Codex hooks need Node 22+.

## Install

See `mods/README.md`.
