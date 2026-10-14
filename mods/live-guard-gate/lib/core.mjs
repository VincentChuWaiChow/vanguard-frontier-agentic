// Pure decision logic for live-guard-gate. The native module and the Node hook both import
// it, so every harness reaches the same verdict on the same input.
import { compilePatterns, candidates, matchPatterns, TOO_LARGE_HIT } from './common.mjs'
import raw from './patterns.mjs'

const PATTERNS = compilePatterns(raw, 'patterns.json')

// Asks when the tool input holds a live cloud mutation, naming the guard agents to consult.
// The runtime renders ask as deny where a harness has no confirmation protocol.
export function checkToolUse({ input }) {
  const hits = matchPatterns(PATTERNS, candidates(input), 'bash')
  if (hits.length === 0) return undefined
  if (hits.includes(TOO_LARGE_HIT)) return { decision: 'ask', reason: 'live-guard-gate: input too large to evaluate.' }
  const agents = [...new Set(hits.flatMap((p) => p.agents ?? []))]
  const names = hits.map((p) => p.name).join('; ')
  return { decision: 'ask', reason: `live-guard-gate: live mutation (${names}). Review with ${agents.join(', ')} before running.` }
}

// Kiro's stdin field names are undocumented: without a tool_input object, scan the whole
// payload so the guard fails closed.
export function toolUseFromPayload(payload) {
  const ti = payload?.tool_input
  return { input: ti !== null && typeof ti === 'object' && !Array.isArray(ti) ? ti : payload }
}
