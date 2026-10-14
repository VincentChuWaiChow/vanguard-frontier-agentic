import type { Register } from 'claude-code'
import { checkToolUse } from '../lib/core.mjs'

export const register: Register = (on) => {
  // Never loosens the engine: its deny stays deny; a live mutation raises allow to ask.
  on('tool.check', async ($, e, next) => {
    const v = await next(e)
    if (v.decision === 'deny') return v
    return checkToolUse({ input: e.input }) ?? v
  }).catch(() => ({ decision: 'deny', reason: 'live-guard-gate: check failed; denied' }))
}
