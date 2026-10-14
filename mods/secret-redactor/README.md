# Secret Redactor

## What it does

- **Before a tool runs**, it scans every string in the tool input against the credential
  patterns in `patterns.json`. These cover GitHub tokens, AWS access key IDs and secret
  keys, HTTP Bearer tokens and passwords embedded in URLs.
  - A hit asks for confirmation.
  - The reason names the pattern, never the matched text.
- **After a tool runs**, it replaces every match in the tool output with
  `[REDACTED:<pattern id>]` before the model reads it. This works only where the harness
  lets a hook replace output (see the table).

A secret is still found when it is split across a key and its value, for example
`aws_secret_access_key` paired with its value. The same holds for a secret split across
argument-list elements (`"Bearer"`, `"<token>"`).

Redaction repeats until the text stops changing. Text that never settles is replaced by
`[REDACTED:overflow]`.

## Configuration

`VANGUARD_SECRET_REDACTOR_PATTERNS` is an optional path to a JSON file of extra patterns.

- The extra patterns are appended to the built-in list. They never replace it.
- The file is an array of entries with the same keys as `patterns.json`:
  - `id`: kebab-case, and not a built-in id.
  - `name`
  - `description`
  - `regex`
  - `flags`: `""` or `"i"`.
  - `context`: `"tool-input"`, `"tool-output"` or both.
  - `severity`
- **The file fails closed.** If it is set and cannot be read, is not valid JSON, is
  malformed, reuses a built-in id or names any other context:
  - before a tool runs, every call is denied;
  - after a tool runs, the output is withheld.
- **Do not let an extension regex match a `[REDACTED:...]` marker.** Its redaction never
  settles, so the output comes back as `[REDACTED:overflow]`.

## Per-harness behavior

| Harness | Surface | Before a tool runs | After a tool runs |
| --- | --- | --- | --- |
| Claude Code | native | `tool.check`: asks on a hit, never loosens the engine verdict | `tool.call`: returns the redacted result |
| Claude Code | classic | `PreToolUse`: asks on a hit | `PostToolUse`: `updatedToolOutput` with the redacted output |
| Kiro | hook | Degraded: the whole payload is scanned and a hit is denied | Unsupported: Kiro documents no way to replace output |
| Codex | hook | Degraded: Codex rejects ask, so a hit is denied with a reason | Unsupported: Codex cannot replace output |

On the native surface, some results cannot be rebuilt from a redacted value:

- an errored call;
- a credential that shows only in the text the model reads;
- a credential in context added by another hook.

Instead of the result, the model reads a notice followed by the redacted text.

## Limitations

- **Classic `PostToolUse` fires only after a tool succeeds.** Output from a failed tool is
  not redacted there.
- **Classic Claude Code ignores a replacement whose shape differs from the tool's output.**
  The withheld output then cannot take effect in two cases:
  - the hook's stdin is invalid;
  - the redaction fails on output that is not an object.
- **Two keys can redact to the same marker.** They collapse into one. This loses data but
  may invalidate the tool's output schema; classic Claude Code may then ignore the
  replacement and retain the original output.
- **When redaction itself fails**, the withheld replacement keeps the output's keys and
  masks only its values. Credentials embedded in keys are not protected by that fallback.
- **An `ask` is settled by the permission mode**, so a bypass mode may auto-approve it.
- **Patterns are regexes, not proof.** A credential format they do not describe passes
  through, and a long uppercase run that starts like an AWS key is over-redacted.
- **The native API is early access**, verified against Claude Code 2.1.296.
- **Classic, Kiro and Codex hooks need Node 22+.**

## Install

See `mods/README.md`.
