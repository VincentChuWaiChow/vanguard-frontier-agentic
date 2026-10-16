# 03 — Shared-core prototype probe (sonnet, scratchpad only)

Status: complete. Date: 2026-10-14.

These experiments were run against Claude Code 2.1.296. All artifacts are in the session scratchpad under `proto/`.

The orchestrator re-ran experiments C, D1 and F and got the same results.

## Experiments

| Exp | Result | Decisive line |
|---|---|---|
| A: zero-import `hooks/core.mjs`, imported as `./core.mjs` | PASS | `claude plugin validate` exit 0 |
| B: `core.mjs` at the plugin root, imported as `../core.mjs` | PASS | exit 0 |
| C: `../../shared/core.mjs`, outside the plugin | FAIL | `cannot import … it is outside the plugin's folder` |
| D: JSON import (with or without `with {type:'json'}`) | FAIL | `patterns.json is not named like code and was not loaded` |
| E: generated `patterns.mjs` that does `export default [...]` | PASS | exit 0 |
| F: `claude plugin test` with `lib/core.mjs` importing `./patterns.mjs` | PASS | `4 pass 0 fail` |
| G: Node `import()` of the same files | PASS | `{"block":true,"id":"rm-root"} {"block":false}` |

Extra probes:

- A symlink that points outside the plugin is refused. A symlink inside the plugin is accepted.
- `node:*` imports are refused: "imports its own files by relative path and "claude-code", nothing else".
- A transitive import, `register.ts` → `lib/core.mjs` → `lib/patterns.mjs`, validates.

## Type-checking

Ran with tsc 5.9.3, invoked through npx. The repo has no `typescript` dependency.

- The tsconfig must sit outside the plugin, with the types file listed in `files`.
- A plain `.mjs` import fails under strict mode with TS7016 (no declaration file for the module).
- Two fixes work:
  - `allowJs` + `checkJs` together with JSDoc `@param`/`@returns`;
  - a sibling `core.d.mts`.
- The checks are real: a wrong-typed call gives TS2345.

## Test-kit gotchas

- `$.env.get` and `$.session.id` have no implementation in the test world, so the hook throws.
  - Mock them with `mock.env(on, {...})` and `on('session.id', () => ({ value: 'x' }))`.
- A bottom `tool.call` hook is needed to prove that an allowed call actually passes through.
- `test(name, { options }, body)` supplies the `userConfig` values.

## API answers (line numbers in `claude-code.d.ts` 2.1.296)

1. **Env vars**
   - `$.env.get(name)` (3595-3616) takes a **string literal only**. A computed name fails validation, so every `VANGUARD_*` name must be spelled out.
   - The test mock is `mock.env` (15476).
2. **`tool.call` event input** (`ToolCallInput`, 12673)
   - It contains `tool`, `tool_use_id`, the tool's arguments spread in (e.g. `e.command`), and `agentId?`.
   - It does **not** contain `session_id`. Get it with `await $.session.id()` (2803).
3. **`ToolCallResult`** (12709-12795)
   - It is one of: `{deny}`, `{result, context?, ref?, text?, isReadOnly?}`, or `{isError:true, result, …}`.
   - To redact output: `const ran = await next(e); return {...ran, result: {...ran.result, stdout}}`. This type-checks.
   - A `{deny}` returned after `next` undoes nothing.
4. **Ask**
   - Native: `on('tool.check', {tool}, …)` returns `{decision:'ask', reason}` (12810, 12846-12900, docs 3963-3975).
     - The guard needs `.catch(() => ({decision:'deny'}))`.
   - Classic: `classic.PreToolUse` returns `{allow:true} | {ask} | {deny}`, plus `updatedInput?` and `additionalContext?` (7946-8008).
5. **`$.fs`** (3226-3340)
   - Available: `read`, `write` (replaces the whole file), `list`, `exists`, `stat`, `ancestors`.
   - **There is no append, mkdir or remove.** Reads and writes are limited to 4 MiB.
   - `$.store` holds JSON, up to 4 MiB in total.
6. **crypto** (14868)
   - Available: `subtle.digest`, `randomUUID`, `getRandomValues`.
   - Not available: HMAC, sign, encrypt.
7. **userConfig**
   - Field types: `string | number | boolean | directory | file`. There is no `integer`.
   - Numbers take `min` and `max`. Strings can take `options` for an enum picker.
   - Unknown keys are rejected.

## Recommendation (adopted for Ideation option 1)

- Put the shared core **inside** each plugin folder, as zero-import, Node-API-free ESM in `lib/*.mjs`.
- Generate the pattern data as `lib/patterns.mjs`, because JSON imports are refused.
- Import the same files from both `register.ts` and the Node hook scripts.
