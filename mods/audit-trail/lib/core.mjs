// Pure record logic for audit-trail. The hook adds the file I/O around it.
import { canonicalJson, sha256Hex } from './common.mjs'

export const SESSION_RE = /^[A-Za-z0-9._-]{1,128}$/

const isObject = (v) => v !== null && typeof v === 'object' && !Array.isArray(v)

// Picks the session id: env override, then the payload, then a shared fallback log.
export function rawSessionId(envValue, payload) {
  if (typeof envValue === 'string' && envValue) return envValue
  const id = isObject(payload) ? payload.session_id : undefined
  return typeof id === 'string' && id ? id : 'unsessioned'
}

// Ids become file names, so anything outside SESSION_RE is replaced by a hash prefix.
export async function safeSessionId(raw) {
  return SESSION_RE.test(raw) ? raw : (await sha256Hex(raw)).slice(0, 32)
}

// Last sequence number in an existing log. A corrupt tail throws: restarting at 1 would fake
// a clean sequence.
export function lastSequence(text, name = '') {
  if (text !== '' && !text.endsWith('\n')) throw new Error(`audit log ${name} has an unterminated last line`)
  const lines = text.split('\n').filter((l) => l.trim() !== '')
  if (lines.length === 0) return 0
  let n
  try {
    n = JSON.parse(lines[lines.length - 1]).sequence_number
  } catch {
    n = undefined
  }
  if (!Number.isInteger(n) || n < 1) throw new Error(`audit log ${name} has a corrupt last line`)
  return n
}

// One log line. Hashes cover the canonical JSON of the arguments and the result; Kiro's
// stdin field names are undocumented, so there the whole payload is hashed.
export async function buildRecord({ timestamp, sessionId, agentId, payload, harness, sequence }) {
  const p = isObject(payload) ? payload : {}
  const kiro = harness === 'kiro'
  return {
    timestamp,
    session_id: sessionId,
    agent_id: agentId || null,
    tool_name: typeof p.tool_name === 'string' ? p.tool_name : null,
    args_hash: await sha256Hex(canonicalJson(kiro ? (payload ?? null) : (p.tool_input ?? null))),
    result_hash: await sha256Hex(canonicalJson(kiro ? null : (p.tool_response ?? null))),
    sequence_number: sequence,
  }
}
