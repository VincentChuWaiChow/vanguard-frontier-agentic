// Node entry for Kiro, Codex and classic Claude Code. Reads the hook payload on stdin.
import { runHook } from '../lib/runtime.mjs'
import { checkToolUse, toolUseFromPayload } from '../lib/core.mjs'

const handlers = {
  PreToolUse: (payload) => checkToolUse(toolUseFromPayload(payload)),
}

// Fail closed: a payload we cannot read is denied rather than let through.
const onError = (err) => ({ decision: 'deny', reason: `live-guard-gate: check failed; denied (${err.message})` })

process.exitCode = await runHook({ name: 'live-guard-gate', handlers, onError })
