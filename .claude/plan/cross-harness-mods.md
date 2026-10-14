# Cross-harness mods: implementation plan

Spec: `.kiro/specs/cross-harness-mods/` (requirements, design, tasks).
Branch: `feat/cross-harness-mods`. Baseline: `1ee35965`.

## Outcome

There will be five mods under `mods/<id>/`: secret-redactor, blast-radius-guard, live-guard-gate, context-burn and audit-trail.

Each mod is a Claude Code native plugin root, and the same folder also carries Node hook adapters for:

- Claude Code classic settings hooks;
- Kiro;
- Codex.

Supporting code:

- One shared, dependency-free core holds the security logic. The native `register.ts`, the Node hooks and the tests all import it.
- A deterministic generator emits the manifests and the data modules.
- A stdlib-only Python gate, `validate:mods`, enforces the contract.
- Any behavior a harness cannot express is denied (fail closed) or declared unsupported in that harness's `UNSUPPORTED.md`. Nothing is downgraded silently. The fail-open paths that remain (a hook that never runs) are listed under Install → Limitations.

## Correction of record

- During Ideation I said: "mods get the honest derived level (audit-trail L1, others L0)."
- **That is wrong.** context-burn is also L1, because the counter status file it writes counts as a file write.
- Corrected derived reach:
  - L0: secret-redactor, blast-radius-guard, live-guard-gate.
  - L1: context-burn, audit-trail.

## Facts verified in this phase

| Fact | Source |
|---|---|
| Classic hooks get `CLAUDE_PROJECT_DIR`, which is the project root where the session started. It stays fixed across worktrees. | code.claude.com/docs/en/hooks |
| Classic hooks "run in the current directory with Claude Code's environment". | same |
| `updatedToolOutput` is an object that must match the tool's output shape. For built-in tools, **a mismatched value is ignored and the original output is used**. | same, PostToolUse decision control |
| `updatedMCPToolOutput` applies to MCP tools only. The docs say "prefer `updatedToolOutput`". | same |
| Classic PostToolUse "fire[s] after a tool has already executed successfully". Tool failures are not covered. | same |
| `tool.check` can return `deny`, which refuses the call. An `ask` goes to the mode's decider. A hook may answer in either direction, and the last word up the chain wins. | `claude-code.d.ts` 2.1.296, lines 3963-3976 and 12860-12900 |
| Native `$.fs.read(path)` returns a file's text (rejects when missing or over 4 MiB) and `$.env.get(name)` reads env, so native guards can load `*_PATTERNS` extension files. | `claude-code.d.ts` 2.1.296, `fs:` at line 3230 |
| `claude plugin validate` and `claude plugin test` scan only the `register.ts` import graph. A `bin/hook.mjs` and a `lib/runtime.mjs` that import `node:fs` can sit in the same plugin folder and still pass validation, and Node runs them. | scratchpad probe `proto/expo`: validate exit 0, 4 pass |

Consequences of these facts:

- Redaction must preserve the shape of the output. Strings stay strings and every key is kept. Otherwise Claude Code silently discards the replacement.
- The classic adapter uses `updatedToolOutput` for every tool.
- Redaction of failed tool output on the classic surface is declared **degraded**.

## Decisions needing approval

1. **audit-trail on the native surface is UNSUPPORTED.**
   - `$.fs` has no append, and a whole-file rewrite breaks Req 14.5 (append-only, never truncate) and races.
   - The classic surface covers Claude Code instead.
   - The alternative is to spawn Node through `$.process`. That raises reach to L2 (spawns) for no extra coverage.
2. **Commit cadence.**
   - One commit per wave, pushed to `feat/cross-harness-mods`, with `validate`, spelling and markdown checks green at each one.
   - `validate:mods` joins the `validate` chain in Wave C, once all five mods exist.

## Deviations from the spec

These are recorded in `.kiro/specs/cross-harness-mods/deviations.md`. In `tasks.md`, each affected checkbox gets a note that points to it.

| Req | Spec says | Plan does | Why |
|---|---|---|---|
| 4.2, 4.4-4.6 | `export default function register(on: ModEventEmitter)` and Node APIs | `export const register: Register`; web APIs plus `$` only | the verified d.ts |
| 4.7, 4.8 | event list in `hooks.json` | `hooks.json` = `{"modules":["./register.ts"]}`; events come from `on()` calls | the verified d.ts |
| 5.x, 5.6, 5.7 | POSIX `.sh`, `bash -n`, requires `sh` | Node `.mjs`, `node --check`, `requires = ["node"]` | user decision |
| 3.6, 3.7, 10.5, 11.5, 12.4 | Kiro ask/modify JSON on stdout | Kiro: deny with exit 2, or declared unsupported | not documented by Kiro |
| 2.7 | ten allowed events | `PreToolUse`, `PostToolUse`, `SessionStart` | the only events the five mods use that all three harnesses can map |
| 3.9 | POSIX companion scripts under `kiro/` | Node `bin/` and `lib/` at the mod root, shared by every harness | user decision (Node `.mjs`), one core for all harnesses |
| 4.1 | generator writes `claude-code/register.ts` | hand-written `hooks/register.ts`; audit-trail ships none | the native layout in the verified d.ts; audit-trail native is unsupported (Decision 1) |
| 4.3 | TypeScript 5.x strict | type-checked by `claude plugin validate` locally only | CI has no `claude` CLI (see Tests) |
| 10.2 | AWS secret key as any 40-char base62; bearer and password-in-URL as given | the secret key must follow an AWS secret-key name; bearer needs 16+ token chars; password-in-URL needs a scheme and stops at whitespace and quotes | the spec regexes match ordinary hashes, prose and paths |
| 10.3 | Pre replaces matches in the arguments | Pre asks (deny where ask is unavailable); Post redacts | user decision: rewriting arguments changes what the tool does |
| 11.8 | unanchored `/(etc\|usr\|…)/` | anchored at the path start, after POSIX normalization of `.` and `..` | the unanchored form matches `/home/me/usr/notes` |
| 12.2 | `live-agents.json` | `lib/live-agents.mjs` with `covered` and `uncovered` | the native runtime refuses JSON imports |
| 12.3 | `live-patterns.json` | `patterns.json`, like the other pattern mods | one file name per role |
| 13.3 | default `.kiro/context-burn-status.json` | `.claude/`, `.kiro/` or `.codex/context-burn-status.json` per harness | a Kiro path is wrong on the other two harnesses |
| 13.4 | `console.warn` plus an in-memory accumulator | a toast plus `$.session.usage().context` | the engine reports real context use; an accumulator guesses |
| 13.5 | `max_tokens` option | dropped: the window comes from `$.session.usage()` | a configured window can only disagree with the real one |
| 14.4 | `.audit/vanguard-audit-<YYYY-MM-DD>.jsonl`, `VANGUARD_AUDIT_LOG_PATH` | `<dir>/<YYYY-MM-DD>/<session>.jsonl`, `VANGUARD_AUDIT_TRAIL_LOG_DIR` | one file per session keeps sequence numbers per session; 15.3 env naming |
| 15.4 | path contains the mod id | `.audit/**` kept, as in the spec's own example | the spec example; the gate's overlap check (15.6) prevents collisions |
| 17.2 | `.sh` scripts on `PATH` or in `.kiro/hooks/scripts/` | `bin/` and `lib/` copied to `.kiro/vanguard-mods/<id>/` | Node scripts, one layout for all harnesses |
| 16.1, 16.2, 16.6 | Python inline P1/P2 mirror in the gate | P1/P2 in `tests/mods.test.mjs` and the fuzz suite against production exports | a Python mirror tests a copy, not the shipped code |
| 11.1 | blast-radius-guard L1 | L0 | exact-match reach (user decision) |
| 12.1 | live-guard-gate L2 | L0 | the same |
| 12.8, 12.9 | generator output counted as footprint; `security_notes` | dropped: build-time output is not runtime reach | the same |
| 13.2 | PostTaskExec | PostToolUse and SessionStart only | conflicts with 13.1 |
| 14.x | immutable / non-repudiable log | an honest append log; no immutability claims | user decision |
| 14.11 | session id from pid and start time | env, then payload `session_id`, then `unsessioned` | pid differs on every hook call |
| 14.12 | POSIX with no jq | Node | cannot hash JSON portably otherwise |
| 9.2 | scanner ignores executable mod scripts | cut: the scanner keeps scanning code | `.plugin-scanner.toml` policy |
| 17.3 | install path | the paths listed under Install below | layout change |
| 15.3 | env names | `VANGUARD_<ID_SNAKE>_*` everywhere (e.g. `VANGUARD_BLAST_RADIUS_GUARD_MODE`) | the spec's own rule |
| live patterns | 40 patterns | 38: `live-hcloud-server` and `live-hcloud-firewall` dropped | verbs came from model memory, not from docs |

## Layout

Files marked G are generated. The generator never touches the hand-written files.

```text
mods/README.md                         install per harness, reach table L0-L3
mods/<id>/mod.toml                     source of truth
mods/<id>/README.md                    behavior, limitations, per-harness matrix
mods/<id>/patterns.json                source data (the three pattern mods)
mods/<id>/.claude-plugin/plugin.json   G (only when the native surface is supported)
mods/<id>/hooks/hooks.json             G {"modules":["./register.ts"]} (only when native)
mods/<id>/hooks/register.ts            hand-written native module
mods/<id>/hooks/register.test.ts       claude plugin test (local, not CI)
mods/<id>/lib/common.mjs               G byte copy of scripts/mods/common.mjs
mods/<id>/lib/runtime.mjs              G byte copy of scripts/mods/runtime.mjs
mods/<id>/lib/patterns.mjs             G export default [...] from patterns.json
mods/<id>/lib/live-agents.mjs          G covered and uncovered live agent ids
mods/<id>/lib/core.mjs                 hand-written mod logic; no node:* imports
mods/<id>/bin/hook.mjs                 hand-written thin Node entry
mods/<id>/claude-code/settings.hooks.json   G classic snippet
mods/<id>/kiro/<id>-<event-kebab>-hook.json G one per supported event
mods/<id>/codex/hooks.json             G
mods/<id>/<harness>/UNSUPPORTED.md     G whenever anything is unsupported or degraded
```

`lib/live-agents.mjs` exists only for live-guard-gate.

## mod.toml contract

```toml
[mod]
id = "blast-radius-guard"            # == directory name, kebab-case
name = "Blast Radius Guard"
description = "..."
version = "1.0.0"
reach_level = 0                      # must equal the derived level
events = ["PreToolUse"]              # subset of PreToolUse, PostToolUse, SessionStart

[footprint]
env_reads = ["VANGUARD_BLAST_RADIUS_GUARD_MODE", "VANGUARD_BLAST_RADIUS_GUARD_PATTERNS"]
file_writes = []                     # globs relative to the project root
spawns = []
egress_hosts = []

[harnesses.claude-code]
supported = true
surfaces = ["native", "classic"]
events = ["PreToolUse"]

[harnesses.kiro]
supported = true
requires = ["node"]
events = ["PreToolUse"]

[harnesses.kiro.degraded]
PreToolUse = "Kiro documents no confirmation protocol; confirm mode denies with a reason."

[harnesses.codex]
supported = true
requires = ["node"]
events = ["PreToolUse"]

[harnesses.codex.degraded]
PreToolUse = "Codex rejects ask; confirm mode denies with a reason."
```

Rules the gate enforces:

- **Derived reach:**
  - 3 if `egress_hosts` is non-empty;
  - else 2 if `spawns` is non-empty;
  - else 1 if `file_writes` is non-empty;
  - else 0.
  - The declared reach must equal the derived reach.
  - L3 gives a WARN.
- **Event coverage:** for each supported harness, every event in `mod.events` is in exactly one of `events` or `unsupported`.
- **Claude Code surfaces:** `surfaces` and the surface keys under `unsupported` together give exactly `{native, classic}`, with no overlap.
- **Degraded keys:** each key under `degraded` is a supported event or surface.
- **`UNSUPPORTED.md`:** it exists if and only if the harness has `supported = false`, any `unsupported` entry, or any `degraded` entry.
- **Env names:**
  - Every env read must start with `VANGUARD_<ID_SNAKE>_`.
  - Every literal env access in `lib/`, `bin/` and `hooks/` must be declared in `env_reads`.
  - A computed env access (`process.env[x]`) is a FAIL.

## Per-mod behavior

Canonical events map to the native API as follows:

- `PreToolUse` → `tool.check`
- `PostToolUse` → `tool.call` (after `await next(e)`)
- `SessionStart` → `session.start`

| Mod | Native | Classic | Kiro | Codex |
|---|---|---|---|---|
| secret-redactor (L0) | Pre: ask if the input contains a secret. Post: return a redacted `{result, context}` with no `ref` or `text`; an error result becomes `{deny: notice + redacted text}`. `.catch` withholds the result. | Pre: ask. Post: `updatedToolOutput` with the shape preserved; tool failures are degraded. | Pre: deny, exit 2 (degraded). Post: unsupported. | same as Kiro |
| blast-radius-guard (L0) | ask in confirm mode, deny in block mode | same | deny (confirm degrades to deny) | same as Kiro |
| live-guard-gate (L0) | ask, naming the guard agent or agents | same | deny, naming the agents (degraded) | same as Kiro |
| context-burn (L1) | `session.start` resets the state. Post reads `$.session.usage().context` (`percent`, else `tokens` over `window`), toasts each 70% and 90% threshold once per session, and writes the status file with `$.fs.write`. | counter only, no token data (degraded) | same as classic | same as classic |
| audit-trail (L1) | **unsupported** (no append) | appends a JSONL line | appends; degraded because the stdin field names are undocumented | appends |

Guard details:

- **Candidates:** every string leaf of the tool input, plus each array of strings joined with spaces. On Kiro, whose stdin field names are undocumented, the candidates are every string leaf of the whole payload. This is declared degraded.
- **System-path rule:**
  - Each path is first normalized lexically as POSIX (repeated `/` collapsed, `.` and `..` resolved), so `/tmp/../etc/passwd` is caught. Relative paths are not resolved against a working directory; that is a declared limitation.
  - It applies to the keys `file_path`, `path`, `notebook_path` and `filePath`.
  - It applies only when the tool name matches `/write|edit|create|patch|delete|move|rename/i`.
  - When the tool name is missing (Kiro), the rule applies anyway. This fails closed and is declared degraded.
- **Mode:**
  - `VANGUARD_BLAST_RADIUS_GUARD_MODE` is `confirm` (the default) or `block`.
  - Any other value means `block`.
- **Size limit:** a candidate over 65,536 characters, or candidates totalling over 262,144 characters, are not evaluated. The call counts as a hit with the reason "input too large to evaluate".
- **The native guard never loosens the engine's verdict:**

  ```text
  v = await next(e)
  if v.decision == deny: return v
  if hit: return ask or deny
  ```

  It has `.catch(() => ({ decision: "deny" }))`.
- **Missing tool input:** when a payload has no `tool_input` object (Kiro always; Codex if its field names differ from the documented ones), the candidates are the whole payload. Fail closed, never "nothing to scan".
- **Extension patterns** (`*_PATTERNS` points to a JSON file with the same shape; native reads it with `$.env.get` and `$.fs.read`):
  - They are appended to the built-in patterns and never replace them.
  - If the file is set but cannot be read or does not compile, the guards deny and the redactor's Post withholds.

Redactor details:

- Redaction is leaf-wise and keeps context:
  - the parent key;
  - the previous string sibling;
  - the `name` or `key` sibling when the leaf's key is `value`.
- Passes repeat until the output stops changing, up to 8. If it still changes, the leaf becomes `[REDACTED:overflow]`.
- Objects are rebuilt with `Object.fromEntries`.
- Each redaction logs to stderr: `[secret-redactor] redacted <pattern-name> in <tool-name> <event>`.

context-burn details:

- **Status file:** default `.claude/`, `.kiro/` or `.codex/context-burn-status.json` by harness; `VANGUARD_CONTEXT_BURN_STATUS_PATH` overrides it. The whole file is rewritten on each event.
- **Fields:** `session_id`, `tool_call_count`, `last_updated` (ISO 8601), `threshold_events` (an array of `{ threshold, tokens, window, at }`, empty where no token data exists).
- **Messages** (native only, once per threshold per session):
  - `[context-burn] WARNING: 70% context used (<tokens> / <window> tokens)`
  - `[context-burn] CRITICAL: 90% context used (<tokens> / <window> tokens)`
  - `tokens` is `context.tokens`, else `percent` × `window` / 100, rounded. With neither, no threshold is evaluated.
- **Node counter:** `SessionStart` resets the count for its session; `PostToolUse` increments it. An unreadable or unparsable status file starts a fresh count and says so on stderr. Ponytail: parallel hooks can lose an increment (read-modify-write, no lock); add the audit-trail lock if exact counts matter.

Write paths (both observers):

- A relative path resolves against the project root, taken from the install location `<root>/.<harness>/vanguard-mods/<id>/bin/hook.mjs`; outside that layout, against the working directory. Hooks never read `CLAUDE_PROJECT_DIR` (the env rule allows only `VANGUARD_<ID>_*`).
- An env override can point a write outside the declared `file_writes`; the footprint states the defaults, and each mod README says so.

audit-trail (Node) details:

- **Path:** `<dir>/<YYYY-MM-DD>/<session>.jsonl`.
  - `dir` comes from `VANGUARD_AUDIT_TRAIL_LOG_DIR`. The default is `.audit`.
  - If the session already has a file in an earlier date directory, that file is reused, so the sequence never resets at midnight. Ponytail: this scans every date directory, O(days); index it if that becomes slow.
- **Session id:**
  - Taken from env, then from the payload's `session_id`, then `unsessioned`.
  - It must match `^[A-Za-z0-9._-]{1,128}$`. Otherwise the first 32 hex characters of its sha256 are used.
- **Lock:** `<dir>/<session>.lock` is opened with `wx` before selecting the dated log file, and the open is retried for up to 2 s. Locks are never taken over based on age; orphaned locks require explicit recovery after confirming no writer remains. This tightens the original takeover design, which could race concurrent writers.
- **Write:** read the sequence from the last line, append the new line, then remove the lock.
  An existing non-empty file without a terminating newline is rejected without appending.
- **Record fields:**
  - `timestamp` (ISO, with milliseconds);
  - `session_id`, `agent_id`, `tool_name`;
  - `args_hash` = sha256 of the canonical JSON of `tool_input ?? null`. On Kiro it is the hash of the whole payload (degraded);
  - `result_hash` = sha256 of the canonical JSON of `tool_response ?? null`;
  - `sequence_number`.
- **Failure:** a write failure exits 1 with a message on stderr. It is loud, but it does not block.

## Shared code (orchestrator-owned)

### `scripts/mods/common.mjs`

- Pure ESM. No imports, no Node APIs, no env access. It runs inside Claude Code's mod runtime.
- Exports:
  - `compilePatterns`: validates the shape. Flags must be `''` or `'i'`; `u` is never added.
  - `stringLeaves`: walks leaves in a deterministic order.
  - `candidates`.
  - `matchPatterns(patterns, texts, context)`.
  - `redactValue(value, patterns)` → `{ value, hits }`.
  - `maskValue(value, text)`.
  - `sha256Hex(text)`: uses `globalThis.crypto.subtle`.
  - `canonicalJson`.
  - `LIMITS`.

### `scripts/mods/runtime.mjs`

- Imports only `node:fs`, for stdin. It writes nothing and reads no env.
- `runHook({ handlers, onError }, io)`:
  - Parses `--harness` (`claude-code`, `kiro` or `codex`) and `--event`. An unknown value exits 2, which blocks in all three harnesses.
  - Parses stdin.
  - Runs the handler.
  - Renders the returned decision as follows.

| Decision | claude-code classic | codex | kiro |
|---|---|---|---|
| allow | exit 0, no output | exit 0 | exit 0 |
| deny | `permissionDecision: "deny"` with a reason, exit 0 | stderr, exit 2 | stderr, exit 2 |
| ask | `permissionDecision: "ask"` | rendered as deny: reason plus "(confirmation unsupported on codex; denied)" | same as codex |
| replace | `updatedToolOutput` | rejected as a handler error (declared unsupported, so never registered) | same as codex |
| context | `additionalContext` | `additionalContext` | stderr |

If the handler throws, or stdin is not valid JSON:

- the result is `onError(err, payload, ctx)`;
- with no `onError`, the hook writes to stderr and exits 1;
- an `onError` that throws or returns an invalid decision exits 2.

Bad arguments, or an event the mod has no handler for, exit 2.

Each mod sets `onError` as follows:

- guards: deny;
- redactor Post: replace with `maskValue`;
- observers: none, so they exit 1 loudly.

## Generator: `scripts/generate-mods.mjs` (Sonnet)

- **TOML parsing:** a strict TOML-subset parser.
  - Supported: tables, dotted tables, bare keys, basic strings with escapes, integers, booleans, arrays of strings (single-line or multi-line) and comments.
  - Anything else throws `<file>:<line>: unsupported TOML`.
- **Code shape:**
  - The render functions are pure exports: `parseToml`, `loadMods`, `renderMod` and `renderAll`.
  - `main` runs only when `import.meta.url` is the entry script.
- **Modes:**
  - The default mode writes files.
  - `--check` compares against disk. It exits 1 and prints `FAIL: mods out of sync (<k> paths). Run: npm run mods:write` with the drifting paths. A clean check prints `OK: mods in sync`. The gate wraps that output, so the `validate` log still shows one `FAIL [mods]` line per failure (spec 7.3).
  - `--print-manifests` prints the parsed TOML as JSON, so the gate can cross-check it against `tomllib`.
- **Determinism:** no clock, no env, no network, no shell, no paths outside the repo.
  - Output is sorted.
  - JSON is pretty-printed with 2 spaces and a trailing newline.
- **Errors:** every mod is processed before exiting. A parse error exits 1 at the end. Bad CLI input exits 2.
- **Outputs:** everything marked G in the layout, plus `catalog/mods.json`.
  - The catalog is an array sorted by id, with the fields id, name, version, description, reach_level, events, harnesses (supported ids) and path.
  - It has no `official_docs` field.
- **Live agents:**
  - The selector is ids in `catalog/agents.json` matching `/-live-.*-guard-agent$/`, sorted.
  - `covered` lists the agents referenced by `patterns.json`.
  - `uncovered` lists the remaining selected ids.
  - An agent that is referenced but missing or unselected is an error.
- **Summary line:** `Generated <N> mods for <M> harnesses`.

## Gate: `tests/validate-mods.py` (Sonnet)

- **Constraints:** stdlib only, under 30 s.
- **Output:**
  - Each failure prints `FAIL [mods] <path> <desc>`.
  - L3 prints `WARN [mods] …`.
  - Success prints `OK: <N> mods valid (claude-code N, kiro N, codex N)`.
  - A missing `mods/` prints `FAIL [mods] mods/ directory is missing`.
- **Checks:**
  - `tomllib` parsing, plus every mod.toml rule above.
  - `[mod].version` is `MAJOR.MINOR.PATCH` (spec 2.6).
  - Every event in `[mod].events` is supported by at least one harness (spec 2.8).
  - The generator's `--print-manifests` output must equal `tomllib`'s parse.
  - `node scripts/generate-mods.mjs --check`. A non-zero exit becomes `FAIL [mods] scripts/generate-mods.mjs --check exited 1:` with its output indented below.
  - Kiro v1 schema: `name == "<id>:<event>"` and the file name.
  - The Codex and classic hook shapes. Every command must reference `bin/hook.mjs` and an `--event` the harness supports.
  - `node --check` on every `.mjs` file.
  - `register.ts`:
    - it contains `export const register`;
    - it imports only `claude-code` and relative `../lib/*.mjs`, never `../lib/runtime.mjs` (it imports `node:fs`, which the native runtime lacks);
    - its `on('<native event>'` calls equal the mapped supported events;
    - it has `.catch(` whenever it uses `tool.check`.
  - Capability scan:
    - Node imports are limited to `node:fs`, `node:path` and relative paths inside the mod, and only in `bin/` and `lib/runtime.mjs`. Every other `lib/` file imports only `./*.mjs`, never `./runtime.mjs`.
    - FAIL on `fetch(`, `WebSocket`, `eval(`, `Function(`, dynamic `import(`, `child_process`, `net`, `http`, `tls`, `dgram` and `worker_threads`, unless the matching `spawns` or `egress_hosts` entry is declared.
    - Write APIs require a non-empty `file_writes`. The write APIs are `writeFileSync`, `appendFileSync`, `openSync` with a `w` or `a` flag, `mkdirSync`, `renameSync` and `$.fs.write`.
    - The shared `lib/common.mjs` and `lib/runtime.mjs` must contain no write APIs at all.
  - Env scan, using the rules above.
  - `catalog/mods.json` matches.
  - Mod ids are unique against the agents, skills and rules catalogs.
  - No two mods have overlapping `file_writes`.
  - Patterns compile under Python `re.ASCII`.
  - P1 (redaction completeness) and P2 (guard blocking) run in `tests/mods.test.mjs` against the production exports, through the test run below, not as a Python mirror.
- **Test run:** finally, it runs `node --test tests/mods.test.mjs`.

## Tests

- **`tests/mods.test.mjs`** uses `node:test`, which is dependency-free.
  - It imports the production `lib/core.mjs`, `common.mjs` and the generator exports.
  - It spawns `bin/hook.mjs` for every harness and event, with a positive and a negative vector each.
  - Credential-shaped vectors are built at runtime (`'AKIA' + 'Z'.repeat(16)`), so the repo secret scan never sees one.
- **`tests/fuzz-properties.test.mjs`** gains these fast-check properties, all against production exports:
  - P1 (Node): `redactValue` reaches a fixed point with no residual match and preserves the shape.
  - P3: N events appended in a temp dir give N JSON lines with sequence 1..N.
  - P4: `parseToml` round-trips the values the subset allows, checked against a test-side serializer that acts as an oracle, not a copy.
  - P5: `renderAll` is byte-identical under any permutation of mod order.
- **`hooks/register.test.ts`** runs through `claude plugin test`.
  - It runs locally during Review, not in CI, because CI has no `claude` CLI.
  - This is a stated gap.

## Install (`mods/README.md`)

- **Native:** `claude --plugin-dir mods/<id>`.
- **Classic:**
  1. Copy `bin/` and `lib/` to `.claude/vanguard-mods/<id>/`.
  2. Merge `claude-code/settings.hooks.json` into `.claude/settings.json`.
  3. The command is `node "$CLAUDE_PROJECT_DIR/.claude/vanguard-mods/<id>/bin/hook.mjs" --harness claude-code --event <E>`.
  4. **Never install native and classic together.**
- **Kiro:**
  1. Copy `kiro/*.json` to `.kiro/hooks/`.
  2. Copy `bin/` and `lib/` to `.kiro/vanguard-mods/<id>/`.
  3. Hook commands use paths relative to the project root.
- **Codex:**
  1. Merge `codex/hooks.json` into `.codex/hooks.json`.
  2. Copy `bin/` and `lib/` to `.codex/vanguard-mods/<id>/`.
  3. The hook must pass Codex's hook trust review before it runs.
- **Limitations that apply everywhere:**
  - A hook that never reaches `runHook` fails open on the CLIs, as a non-blocking error: `node` not on `PATH`, a missing or moved script, a crash before `runHook`, or the harness's hook timeout. The Kiro IDE blocks instead.
  - The classic, Kiro and Codex surfaces need Node 22 or later on `PATH`; native needs nothing. The claude-code table has no `requires`, because it would misstate the native surface.
  - An `ask` is settled by the permission mode, so a bypass mode may auto-approve it.
  - No network is needed at install time.

## Integration edits

- `package.json`:
  - add `mods:write`, and add `validate:mods` (`python3 tests/validate-mods.py`);
  - add `validate:mods` to the `validate` chain (Wave C);
  - add `& npm run mods:write` to `manifest:write:all`;
  - never touch `"version"`.
- `tests/validate-asset-integrity.py`: add `"mods"` to `TREES`.
- `catalog/index.json`: add `"mods": "catalog/mods.json"`.
- `.github/workflows/ci.yml`, in the validate job:
  - add `actions/setup-node@820762786026740c76f36085b0efc47a31fe5020 # v7.0.0` (node 22);
  - add a `Validate mods` step.
- `CLAUDE.md`: a `## Mods` section, plus a Quick start line.
- `CONTRIBUTING.md`: a mod checklist.
- `.kiro/specs/cross-harness-mods/`: annotate the `tasks.md` checkboxes and add `deviations.md`.

## Waves

Every Sonnet writer works from a file-scoped spec, which is the relevant section of this plan. The orchestrator reads every diff and runs the probes itself.

| Wave | Owner | Files |
|---|---|---|
| A1 | orchestrator | `scripts/mods/common.mjs`, `scripts/mods/runtime.mjs` |
| A2 | Sonnet | `scripts/generate-mods.mjs` |
| A3 | Sonnet | `tests/validate-mods.py` |
| A4 | orchestrator | 5 × `mod.toml`, the 3 × `patterns.json` (hcloud dropped) |
| B1 | orchestrator | secret-redactor: `lib/core.mjs`, `bin/hook.mjs`, `hooks/register.ts` and test, README |
| B2-B5 | 4 × Sonnet | the same file set for blast-radius-guard, live-guard-gate, context-burn and audit-trail (audit-trail has no `register.ts`) |
| C1 | Sonnet | `tests/mods.test.mjs` |
| C2 | Sonnet | the fuzz additions |
| C3 | Sonnet | `mods/README.md`, the CLAUDE.md section, the CONTRIBUTING checklist |
| C4 | orchestrator | package.json, ci.yml, asset-integrity, index.json, the spec annotations |

After the waves:

- **Optimize:** three read-only reviewers run in parallel: a refuter, a security reviewer and a silent-failure hunter.
- **Review:** the Definition of done, in this order:
  1. `npm run mods:write`
  2. `npm run asset-integrity:write`, last and on its own
  3. `npm run validate`, `npm run lint:spell` and markdownlint
  4. local `claude plugin validate` and `claude plugin test` for the four native mods
  5. positive and negative probes per harness
  6. commit and push

## Known unknowns

- **U1 (Codex and Kiro hook cwd).** The working directory of these hook processes is not documented, so the relative command paths assume the project root. The README recommends absolute paths.
- **U2 (bypass mode).** An `ask` under a bypass mode may be auto-approved. This applies to native and classic alike, and is documented.
- **U3 (Kiro stdin field names).** These are undocumented. The mods scan the whole payload and declare the result degraded.
- **U4 (native API stability).** The native mod API is labelled early access, so `register.ts` may break across Claude Code releases. It is pinned to the verified build, 2.1.296, in the README.
