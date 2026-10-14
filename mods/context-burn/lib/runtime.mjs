// Command-hook runtime for the mods under mods/. scripts/generate-mods.mjs copies this file
// byte-for-byte to mods/<id>/lib/runtime.mjs, so edit it here and run `npm run mods:write`.
//
// A mod's bin/hook.mjs hands its handlers to runHook, which reads the harness payload from
// stdin and renders the handler's decision in that harness's protocol. It writes no files
// and reads no environment; Claude Code's native runtime never imports it.
import { readFileSync } from 'node:fs'
import { isAbsolute, resolve, sep } from 'node:path'

const HARNESSES = new Set(['claude-code', 'kiro', 'codex'])
const EVENTS = new Set(['PreToolUse', 'PostToolUse', 'SessionStart'])
const DECISIONS = new Set(['allow', 'deny', 'ask', 'replace', 'context'])

const nodeIo = () => ({
  argv: process.argv.slice(2),
  readStdin: () => readFileSync(0, 'utf8'),
  out: (s) => process.stdout.write(s),
  err: (s) => process.stderr.write(s),
})

const message = (err) => (err instanceof Error ? err.message : String(err))

// JSON.parse errors quote the text they failed on, and stdin can carry a tool's input or
// output: report the failure without echoing any of it.
function parseStdin(text) {
  try {
    return JSON.parse(text)
  } catch {
    throw new Error('stdin is not valid JSON')
  }
}

export function parseArgs(argv) {
  const usage = 'usage: hook.mjs --harness <claude-code|kiro|codex> --event <PreToolUse|PostToolUse|SessionStart>'
  if (argv.length !== 4) throw new Error(usage)
  const args = {}
  for (let i = 0; i < 4; i += 2) {
    const key = { '--harness': 'harness', '--event': 'event' }[argv[i]]
    if (!key || key in args) throw new Error(usage)
    args[key] = argv[i + 1]
  }
  if (!HARNESSES.has(args.harness)) throw new Error(`unknown harness ${args.harness}; ${usage}`)
  if (!EVENTS.has(args.event)) throw new Error(`unknown event ${args.event}; ${usage}`)
  return args
}

// Where an observer writes `path`. A relative path resolves against the project root when
// the hook runs from its install location <root>/.<harness dir>/vanguard-mods/<id>/bin/,
// otherwise against the working directory. Never CLAUDE_PROJECT_DIR: mods read only their
// own VANGUARD_<ID>_* variables.
export function projectPath(path, harness, binDir) {
  if (isAbsolute(path)) return path
  const parts = resolve(binDir).split(sep)
  const n = parts.length
  const dir = { 'claude-code': '.claude', kiro: '.kiro', codex: '.codex' }[harness]
  const installed = n >= 5 && parts[n - 1] === 'bin' && parts[n - 3] === 'vanguard-mods' && parts[n - 4] === dir
  return resolve(installed ? parts.slice(0, n - 4).join(sep) || sep : process.cwd(), path)
}

// Rejects decisions the harness cannot express, so a handler bug surfaces as an error
// instead of a silently dropped verdict.
function normalize(d, { harness, event }) {
  if (d === undefined) return { decision: 'allow', log: [] }
  if (d === null || typeof d !== 'object' || !DECISIONS.has(d.decision)) {
    throw new Error(`handler returned an invalid decision: ${JSON.stringify(d)}`)
  }
  const log = d.log ?? []
  if (!Array.isArray(log) || !log.every((l) => typeof l === 'string')) throw new Error('decision log must be a string array')
  const needs = { deny: 'reason', ask: 'reason', context: 'text' }[d.decision]
  if (needs && (typeof d[needs] !== 'string' || d[needs] === '')) throw new Error(`${d.decision} needs a ${needs}`)
  if ((d.decision === 'deny' || d.decision === 'ask') && event !== 'PreToolUse') {
    throw new Error(`${d.decision} is only valid for PreToolUse, not ${event}`)
  }
  if (d.decision === 'replace' && (harness !== 'claude-code' || event !== 'PostToolUse')) {
    throw new Error(`replace is only valid for claude-code PostToolUse, not ${harness} ${event}`)
  }
  return { ...d, log }
}

function render(d, { harness, event }, io) {
  const json = (hookSpecificOutput) => io.out(JSON.stringify({ hookSpecificOutput: { hookEventName: event, ...hookSpecificOutput } }) + '\n')
  for (const line of d.log) io.err(line + '\n')
  switch (d.decision) {
    case 'allow':
      return 0
    case 'deny':
    case 'ask':
      if (harness === 'claude-code') {
        json({ permissionDecision: d.decision, permissionDecisionReason: d.reason })
        return 0
      }
      // Kiro and Codex have no confirmation step; exit 2 blocks the call in both.
      io.err(d.decision === 'ask' ? `${d.reason} (confirmation unsupported on ${harness}; denied)\n` : `${d.reason}\n`)
      return 2
    case 'replace':
      json({ updatedToolOutput: d.output })
      return 0
    case 'context':
      if (harness === 'kiro') io.err(d.text + '\n')
      else json({ additionalContext: d.text })
      return 0
  }
}

// Runs one hook invocation and returns the exit code; bin/hook.mjs sets process.exitCode
// with it rather than calling process.exit, so buffered stdout is never cut short.
//
// - Bad arguments, or an event this mod has no handler for: exit 2, which blocks a
//   PreToolUse call in every harness (a broken install must not allow silently).
// - Invalid stdin or a throwing handler: `onError(err, payload, ctx)` supplies the decision
//   (guards deny, the redactor withholds). Without onError: stderr and exit 1, which is
//   loud but does not block.
// - An onError that throws or returns an invalid decision: exit 2.
export async function runHook({ name, handlers, onError }, io = nodeIo()) {
  let ctx
  try {
    ctx = parseArgs(io.argv)
    if (!Object.hasOwn(handlers, ctx.event)) throw new Error(`no handler for ${ctx.event}`)
  } catch (err) {
    io.err(`${name}: ${message(err)}\n`)
    return 2
  }
  let payload
  let decision
  try {
    payload = parseStdin(io.readStdin())
    if (payload === null || typeof payload !== 'object' || Array.isArray(payload)) throw new Error('stdin is not a JSON object')
    decision = normalize(await handlers[ctx.event](payload, ctx), ctx)
  } catch (err) {
    io.err(`${name}: ${message(err)}\n`)
    if (typeof onError !== 'function') return 1
    try {
      decision = normalize(await onError(err, payload, ctx), ctx)
    } catch (err2) {
      io.err(`${name}: error handler failed: ${message(err2)}\n`)
      return 2
    }
  }
  return render(decision, ctx, io)
}
