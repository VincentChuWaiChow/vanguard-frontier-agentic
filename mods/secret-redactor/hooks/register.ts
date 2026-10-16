import type { EngineInterface, Register } from 'claude-code'
import { WITHHELD, checkInput, loadPatterns, redactResult } from '../lib/core.mjs'

// Top level, not inside register: `claude plugin validate` refuses a $-taking helper there.
async function patterns($: EngineInterface) {
  const file = await $.env.get('VANGUARD_SECRET_REDACTOR_PATTERNS')
  return loadPatterns(file ? await $.fs.read(file) : undefined, file)
}

export const register: Register = (on) => {
  on('tool.check', async ($, e, next) => {
    const v = await next(e)
    if (v.decision === 'deny') return v
    return checkInput(e.input, await patterns($)) ?? v
  }).catch(() => ({ decision: 'deny', reason: 'secret-redactor: check failed; denied' }))

  // The tool has already run when redaction fails, so the catch withholds rather than replays.
  on('tool.call', async ($, e, next) => redactResult(await next(e), await patterns($))).catch(() => ({ deny: WITHHELD }))
}
