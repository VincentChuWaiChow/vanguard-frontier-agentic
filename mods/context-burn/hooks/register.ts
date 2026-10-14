import type { Register } from 'claude-code'
import { DEFAULT_STATUS_PATH, freshStatus, parseStatus, recordCall, serializeStatus } from '../lib/core.mjs'

async function statusPath($: any): Promise<string> {
  return (await $.env.get('VANGUARD_CONTEXT_BURN_STATUS_PATH')) || DEFAULT_STATUS_PATH['claude-code']
}

export const register: Register = (on) => {
  // A new session resets the file; a failure must never block startup.
  on('session.start', async ($, e, next) => {
    const at = new Date(await $.clock.now()).toISOString()
    await $.fs.write(await statusPath($), serializeStatus(freshStatus(await $.session.id(), at)))
    return next(e)
  }).catch(($, e, next) => next(e))

  // Observer: the tool runs first and its result is returned untouched, whatever happens here.
  on('tool.call', async ($, e, next) => {
    const r = await next(e)
    const path = await statusPath($)
    let prev
    try {
      prev = parseStatus(await $.fs.read(path))
    } catch {
      prev = undefined
    }
    const at = new Date(await $.clock.now()).toISOString()
    const { context } = await $.session.usage()
    const { status, messages } = recordCall(prev, await $.session.id(), at, context)
    for (const m of messages) await $.ui.toast(m)
    await $.fs.write(path, serializeStatus(status))
    return r
  }).catch(($, e, next) => next(e))
}
