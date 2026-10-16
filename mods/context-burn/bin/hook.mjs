// Command hook for classic Claude Code, Kiro and Codex: counter only, no token data.
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import { dirname } from 'node:path'
import { defaultStatusPath, freshStatus, parseStatus, recordCall, serializeStatus } from '../lib/core.mjs'
import { projectPath, runHook } from '../lib/runtime.mjs'

const target = async (ctx, sessionId) =>
  projectPath(process.env.VANGUARD_CONTEXT_BURN_STATUS_PATH || await defaultStatusPath(ctx.harness, sessionId), ctx.harness, import.meta.dirname)
const sessionOf = (p) => (typeof p.session_id === 'string' && p.session_id !== '' ? p.session_id : 'unsessioned')

function save(file, status) {
  mkdirSync(dirname(file), { recursive: true })
  writeFileSync(file, serializeStatus(status))
}

const handlers = {
  async SessionStart(payload, ctx) {
    const sessionId = sessionOf(payload)
    save(await target(ctx, sessionId), freshStatus(sessionId, new Date().toISOString()))
  },
  // ponytail: parallel hooks can lose an increment (read-modify-write, no lock); add the
  // audit-trail lock if exact counts matter.
  async PostToolUse(payload, ctx) {
    const sessionId = sessionOf(payload)
    const file = await target(ctx, sessionId)
    const log = []
    let prev
    try {
      prev = parseStatus(readFileSync(file, 'utf8'))
    } catch (err) {
      if (err?.code !== 'ENOENT') log.push(`[context-burn] status file unreadable; starting a fresh count: ${err.message}`)
    }
    save(file, recordCall(prev, sessionId, new Date().toISOString(), undefined).status)
    return log.length ? { decision: 'allow', log } : undefined
  },
}

process.exitCode = await runHook({ name: 'context-burn', handlers })
