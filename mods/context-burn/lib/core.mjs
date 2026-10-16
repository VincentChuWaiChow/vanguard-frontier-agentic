// Pure counter and threshold logic for context-burn. The native module and the Node hook
// both import it, so the status file means the same thing on every harness.
import { parseJson, sha256Hex } from './common.mjs'

const STATUS_DIR = Object.freeze({
  'claude-code': '.claude/context-burn-status',
  kiro: '.kiro/context-burn-status',
  codex: '.codex/context-burn-status',
})

// Never interpolate a raw session id into a filesystem path.
export async function defaultStatusPath(harness, sessionId) {
  return `${STATUS_DIR[harness]}/${await sha256Hex(sessionId)}.json`
}

const THRESHOLDS = [
  [70, 'WARNING'],
  [90, 'CRITICAL'],
]

// A new session starts at zero with no threshold events.
export function freshStatus(sessionId, at) {
  return { session_id: sessionId, tool_call_count: 0, last_updated: at, threshold_events: [] }
}

// Rejects anything that is not a context-burn status so a stray file is never extended.
export function parseStatus(text) {
  const s = parseJson(text, 'context-burn status')
  const ok =
    s !== null &&
    typeof s === 'object' &&
    !Array.isArray(s) &&
    typeof s.session_id === 'string' &&
    Number.isInteger(s.tool_call_count) &&
    s.tool_call_count >= 0 &&
    typeof s.last_updated === 'string' &&
    Array.isArray(s.threshold_events) &&
    s.threshold_events.every((t) =>
      t !== null && typeof t === 'object' && !Array.isArray(t) &&
      THRESHOLDS.some(([threshold]) => t.threshold === threshold) &&
      Number.isFinite(t.tokens) && t.tokens >= 0 &&
      Number.isFinite(t.window) && t.window > 0 &&
      typeof t.at === 'string',
    )
  if (!ok) throw new Error('status file is not a context-burn status')
  return s
}

// Counts one tool call and, when token data exists, records each threshold crossed for the
// first time. Never mutates prev.
export function recordCall(prev, sessionId, at, context) {
  const base = prev?.session_id === sessionId ? prev : freshStatus(sessionId, at)
  const status = {
    ...base,
    tool_call_count: base.tool_call_count + 1,
    last_updated: at,
    threshold_events: [...base.threshold_events],
  }
  const messages = []
  const window = context?.window
  const tokens =
    context?.tokens ??
    (context?.percent !== undefined && window > 0 ? Math.round((context.percent * window) / 100) : undefined)
  if (tokens !== undefined && window > 0) {
    const pct = context.percent ?? (tokens / window) * 100
    for (const [threshold, label] of THRESHOLDS) {
      if (pct >= threshold && !status.threshold_events.some((t) => t.threshold === threshold)) {
        status.threshold_events.push({ threshold, tokens, window, at })
        messages.push(`[context-burn] ${label}: ${threshold}% context used (${tokens} / ${window} tokens)`)
      }
    }
  }
  return { status, messages }
}

// Pretty JSON with a trailing newline; the whole file is rewritten each event.
export function serializeStatus(status) {
  return JSON.stringify(status, null, 2) + '\n'
}
