import type { Register } from 'claude-code'
import { defaultStatusPath, freshStatus, parseStatus, recordCall, serializeStatus } from '../lib/core.mjs'

async function statusPath($: any, sessionId: string): Promise<string> {
  return (await $.env.get('VANGUARD_CONTEXT_BURN_STATUS_PATH')) || await defaultStatusPath('claude-code', sessionId)
}

export const register: Register = (on) => {
  // A new session resets the file; a failure must never block startup.
  on('session.start', async ($, e, next) => {
    const at = new Date(await $.clock.now()).toISOString()
    const sessionId = await $.session.id()
    await $.fs.write(await statusPath($, sessionId), serializeStatus(freshStatus(sessionId, at)))
    return next(e)
  }).catch(($, e, next) => next(e))

  // Observer: the tool runs first and its result is returned untouched, whatever happens here.
  on('tool.call', async ($, e, next) => {
    const r = await next(e)
    const sessionId = await $.session.id()
    const path = await statusPath($, sessionId)
    let prev
    try {
      prev = parseStatus(await $.fs.read(path))
    } catch {
      prev = undefined
    }
    const at = new Date(await $.clock.now()).toISOString()
    const { context } = await $.session.usage()
    const { status, messages } = recordCall(prev, sessionId, at, context)
    for (const m of messages) await $.ui.toast(m)
    await $.fs.write(path, serializeStatus(status))
    return r
  }).catch(($, e, next) => next(e))
}
