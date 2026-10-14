// Shared helpers for the mods under mods/. scripts/generate-mods.mjs copies this file
// byte-for-byte to mods/<id>/lib/common.mjs, so edit it here and run `npm run mods:write`.
//
// Pure on purpose: no imports, no environment, no I/O. Claude Code's native mod runtime
// imports it and has no Node, and every harness must reach the same verdict on the same
// input. Regexes compile without the `u` flag so Node and Python `re.ASCII`
// (tests/validate-mods.py) read them the same way.

export const LIMITS = Object.freeze({
  // A redaction that still changes after this many passes is withheld as a whole.
  maxPasses: 8,
  // A guard candidate longer than this is not evaluated and counts as a hit.
  maxCandidateChars: 65536,
  // Guard candidates are not evaluated past this combined length; the call counts as a hit.
  maxTotalChars: 262144,
  // Only the tail of a leaf's context (key or previous sibling) is prefixed to it.
  maxContextChars: 256,
})

const MARKER_PREFIX = '[REDACTED:'
export const OVERFLOW_MARKER = '[REDACTED:overflow]'
const OVERFLOW_HIT = Object.freeze({ id: 'overflow', name: 'redaction did not converge' })
export const TOO_LARGE_HIT = Object.freeze({ id: 'too-large', name: 'input too large to evaluate' })

const ID_RE = /^[a-z0-9]+(-[a-z0-9]+)*$/
const PATTERN_KEYS = new Set(['id', 'name', 'description', 'regex', 'flags', 'context', 'severity', 'agents'])
const isStringArray = (v) => Array.isArray(v) && v.every((s) => typeof s === 'string' && s !== '')

// Parser diagnostics can quote credential-bearing data; never expose it in hook logs.
export function parseJson(text, source) {
  try {
    return JSON.parse(text)
  } catch {
    throw new Error(`${source}: invalid JSON`)
  }
}

// Validates a patterns.json-shaped list and compiles it. Throws on anything malformed so a
// bad extension file fails closed in the caller instead of silently matching nothing.
export function compilePatterns(list, source = 'patterns') {
  if (!Array.isArray(list)) throw new Error(`${source}: expected an array of patterns`)
  const seen = new Set()
  return list.map((p, i) => {
    const where = `${source}[${i}]`
    if (p === null || typeof p !== 'object' || Array.isArray(p)) throw new Error(`${where}: expected an object`)
    for (const k of Object.keys(p)) if (!PATTERN_KEYS.has(k)) throw new Error(`${where}: unknown key ${k}`)
    if (typeof p.id !== 'string' || !ID_RE.test(p.id)) throw new Error(`${where}: id must be kebab-case`)
    if (seen.has(p.id)) throw new Error(`${where}: duplicate id ${p.id}`)
    seen.add(p.id)
    if (typeof p.name !== 'string' || p.name === '') throw new Error(`${where}: name must be a non-empty string`)
    if (typeof p.regex !== 'string' || p.regex === '') throw new Error(`${where}: regex must be a non-empty string`)
    if (p.flags !== '' && p.flags !== 'i') throw new Error(`${where}: flags must be "" or "i"`)
    if (!isStringArray(p.context) || p.context.length === 0) throw new Error(`${where}: context must be a non-empty string array`)
    for (const k of ['description', 'severity']) {
      if (p[k] !== undefined && typeof p[k] !== 'string') throw new Error(`${where}: ${k} must be a string`)
    }
    if (p.agents !== undefined && !isStringArray(p.agents)) throw new Error(`${where}: agents must be a string array`)
    let rx, rxg
    try {
      rx = new RegExp(p.regex, p.flags)
      rxg = new RegExp(p.regex, 'g' + p.flags)
    } catch {
      throw new Error(`${where}: regex does not compile`)
    }
    return Object.freeze({
      id: p.id,
      name: p.name,
      severity: p.severity,
      context: Object.freeze([...p.context]),
      agents: p.agents ? Object.freeze([...p.agents]) : undefined,
      rx,
      rxg,
      marker: `${MARKER_PREFIX}${p.id}]`,
    })
  })
}

// Every string value (not key) in depth-first order. The order follows the input, so the
// same payload always yields the same list.
export function stringLeaves(value, out = []) {
  if (typeof value === 'string') out.push(value)
  else if (Array.isArray(value)) for (const v of value) stringLeaves(v, out)
  else if (value !== null && typeof value === 'object') for (const v of Object.values(value)) stringLeaves(v, out)
  return out
}

// Guard candidates: every string leaf, plus every all-string array joined with spaces so
// an argv-style command (["rm", "-rf", "/"]) reads like the shell line it runs.
export function candidates(value) {
  const out = stringLeaves(value)
  const joinArrays = (v) => {
    if (Array.isArray(v)) {
      if (v.length > 1 && v.every((s) => typeof s === 'string')) out.push(v.join(' '))
      v.forEach(joinArrays)
    } else if (v !== null && typeof v === 'object') {
      Object.values(v).forEach(joinArrays)
    }
  }
  joinArrays(value)
  return out
}

// The patterns tagged with `context` that match any text, in catalog order. Oversized input
// is not evaluated (regex cost is unbounded there) and reports TOO_LARGE_HIT instead.
export function matchPatterns(patterns, texts, context) {
  let total = 0
  for (const t of texts) {
    total += t.length
    if (t.length > LIMITS.maxCandidateChars || total > LIMITS.maxTotalChars) return [TOO_LARGE_HIT]
  }
  return patterns.filter((p) => p.context.includes(context) && texts.some((t) => p.rx.test(t)))
}

// One pattern pass after another until the text stops changing. Each pass can expose a new
// match (password-in-url can swallow an earlier marker), so a single pass is not enough.
function fixedPoint(text, patterns, hits) {
  let x = text
  for (let pass = 0; pass <= LIMITS.maxPasses; pass++) {
    let y = x
    for (const p of patterns) {
      y = y.replace(p.rxg, (m) => {
        if (m === '') return m
        hits.add(p)
        return p.marker
      })
    }
    if (y === x) return x
    x = y
  }
  hits.add(OVERFLOW_HIT)
  return OVERFLOW_MARKER
}

// Redacts `s` with `ctx` in front so a secret split across a key and its value, or across
// argv elements ("Bearer", "<token>"), still matches; then strips the context back off.
function redactLeaf(s, ctx, patterns, hits) {
  if (ctx === '') return fixedPoint(s, patterns, hits)
  const pre = ctx.slice(-LIMITS.maxContextChars) + ' '
  const y = fixedPoint(pre + s, patterns, hits)
  if (y === OVERFLOW_MARKER) return y
  const r = fixedPoint(pre, patterns, new Set())
  if (y.startsWith(r)) return fixedPoint(y.slice(r.length), patterns, hits)
  // A match spans the boundary: keep from the marker that covers it. Slicing changes
  // anchors and boundaries, so also check the standalone leaf after contextual matching.
  let d = 0
  while (d < r.length && y[d] === r[d]) d++
  const i = y.lastIndexOf(MARKER_PREFIX, d)
  return fixedPoint(i >= 0 ? y.slice(i) : y, patterns, hits)
}

function walk(v, ctx, patterns, hits) {
  if (typeof v === 'string') {
    const local = new Set()
    const out = redactLeaf(v, ctx, patterns, local)
    if (out !== v) for (const h of local) hits.add(h)
    return out
  }
  if (Array.isArray(v)) {
    let prev = null
    return v.map((el) => {
      const out = walk(el, typeof prev === 'string' ? prev : ctx, patterns, hits)
      prev = el
      return out
    })
  }
  if (v !== null && typeof v === 'object') {
    const named = typeof v.name === 'string' ? v.name : typeof v.key === 'string' ? v.key : null
    // fromEntries, not assignment: `out[k] = v` would drop an own "__proto__" key.
    return Object.fromEntries(
      Object.entries(v).map(([k, x]) => [
        walk(k, '', patterns, hits),
        walk(x, k === 'value' && named !== null ? named : k, patterns, hits),
      ]),
    )
  }
  return v
}

// Returns a copy of `value` with every match replaced by `[REDACTED:<id>]`, plus the
// patterns that fired (catalog order, unique). Keys are redacted too; two keys that redact
// to the same marker collapse into one, which loses data but never leaks it.
export function redactValue(value, patterns) {
  const hits = new Set()
  const out = walk(value, '', patterns, hits)
  const order = [...patterns, OVERFLOW_HIT]
  return { value: out, hits: order.filter((p) => hits.has(p)) }
}

// Same shape as `value`, every string replaced by `text`. The fallback when redaction
// itself failed: withhold everything rather than pass anything through.
export function maskValue(value, text) {
  if (typeof value === 'string') return text
  if (Array.isArray(value)) return value.map((v) => maskValue(v, text))
  if (value !== null && typeof value === 'object') {
    return Object.fromEntries(Object.entries(value).map(([k, v]) => [k, maskValue(v, text)]))
  }
  return value
}

// JSON with object keys sorted at every level, so equal values hash equally.
export function canonicalJson(value) {
  if (Array.isArray(value)) return '[' + value.map((v) => (v === undefined ? 'null' : canonicalJson(v))).join(',') + ']'
  if (value !== null && typeof value === 'object') {
    const keys = Object.keys(value).filter((k) => value[k] !== undefined).sort()
    return '{' + keys.map((k) => JSON.stringify(k) + ':' + canonicalJson(value[k])).join(',') + '}'
  }
  return JSON.stringify(value) ?? 'null'
}

export async function sha256Hex(text) {
  const digest = await globalThis.crypto.subtle.digest('SHA-256', new TextEncoder().encode(text))
  return Array.from(new Uint8Array(digest), (b) => b.toString(16).padStart(2, '0')).join('')
}
