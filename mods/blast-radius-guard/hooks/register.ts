import type { Register } from 'claude-code'
import { checkToolUse, compileGuardPatterns, modeOf } from '../lib/core.mjs'

export const register: Register = (on) => {
  on('tool.check', async ($, e, next) => {
    const v = await next(e)
    if (v.decision === 'deny') return v
    const mode = modeOf(await $.env.get('VANGUARD_BLAST_RADIUS_GUARD_MODE'))
    const file = await $.env.get('VANGUARD_BLAST_RADIUS_GUARD_PATTERNS')
    const patterns = compileGuardPatterns(file ? await $.fs.read(file) : undefined)
    return checkToolUse({ tool: e.tool, input: e.input }, patterns, mode) ?? v
  }).catch(() => ({ decision: 'deny', reason: 'blast-radius-guard: check failed; denied' }))
}
