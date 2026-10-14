// Node hook for audit-trail: appends one JSON line per tool call to a per-session log.
// An honest append log, not tamper-evident: anything with write access can edit it.
import { appendFileSync, closeSync, existsSync, mkdirSync, openSync, readFileSync, readdirSync, unlinkSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { runHook, projectPath } from '../lib/runtime.mjs'
import { buildRecord, lastSequence, rawSessionId, safeSessionId } from '../lib/core.mjs'

const sleep = (ms) => Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, ms)

// Reuses the earliest date dir that already holds this session, so the sequence never
// resets at midnight.
// ponytail: scans every date dir, O(days); index it if slow.
function logFile(base, sid, today) {
  const days = existsSync(base) ? readdirSync(base).filter((d) => /^\d{4}-\d{2}-\d{2}$/.test(d)).sort() : []
  const hit = days.find((d) => existsSync(join(base, d, `${sid}.jsonl`)))
  return join(base, hit ?? today, `${sid}.jsonl`)
}

// Never steal a lock: its age does not prove the owner is dead. Explicit recovery is safer
// than racing two writers after an automatic stale-lock takeover.
function acquire(lock, file) {
  const deadline = Date.now() + 2000
  for (;;) {
    try {
      return openSync(lock, 'wx')
    } catch (err) {
      if (err.code !== 'EEXIST') throw err
    }
    if (Date.now() > deadline) throw new Error(`audit-trail: could not lock ${file} within 2s`)
    sleep(20)
  }
}

// Read directly instead of exists-then-read, so there is no check/use gap.
function readIfExists(file) {
  try {
    return readFileSync(file, 'utf8')
  } catch (err) {
    if (err.code === 'ENOENT') return ''
    throw err
  }
}

async function PostToolUse(payload, ctx) {
  const base = projectPath(process.env.VANGUARD_AUDIT_TRAIL_LOG_DIR || '.audit', ctx.harness, import.meta.dirname)
  const sid = await safeSessionId(rawSessionId(process.env.VANGUARD_AUDIT_TRAIL_SESSION_ID, payload))
  const timestamp = new Date().toISOString()
  mkdirSync(base, { recursive: true })
  // Serialize session-file selection as well as appends, including across midnight.
  const lock = join(base, `${sid}.lock`)
  const fd = acquire(lock, lock)
  try {
    const file = logFile(base, sid, timestamp.slice(0, 10))
    mkdirSync(dirname(file), { recursive: true })
    const sequence = lastSequence(readIfExists(file), file) + 1
    const agentId = process.env.VANGUARD_AUDIT_TRAIL_AGENT_ID
    const record = await buildRecord({ timestamp, sessionId: sid, agentId, payload, harness: ctx.harness, sequence })
    appendFileSync(file, JSON.stringify(record) + '\n')
  } finally {
    closeSync(fd)
    unlinkSync(lock)
  }
}

process.exitCode = await runHook({ name: 'audit-trail', handlers: { PostToolUse } })
