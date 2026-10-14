// Blast Radius Guard logic, shared by the native module and the Node hook so every
// harness reaches the same verdict. Pure: no I/O, no environment.
import { candidates, compilePatterns, matchPatterns, parseJson } from './common.mjs'
import builtin from './patterns.mjs'

const PATH_KEYS = new Set(['file_path', 'path', 'notebook_path', 'filePath'])
const WRITE_TOOL = /write|edit|create|patch|delete|move|rename/i

// Built-in patterns, plus the operator's extension appended (never replacing). Throws on
// bad JSON or shape, or an id reused from the built-ins; callers fail closed.
export function compileGuardPatterns(extraText, source = 'extension patterns') {
  if (extraText === undefined) return compilePatterns(builtin, 'built-in patterns')
  return compilePatterns([...builtin, ...parseJson(extraText, source)], source)
}

// Only an unset value or exactly `confirm` asks; any other value (typos included) blocks.
export function modeOf(value) {
  return value === undefined || value === 'confirm' ? 'confirm' : 'block'
}

// Lexical POSIX normalization: no cwd, `..` never climbs above an absolute root.
export function normalizePosix(p) {
  const abs = p.startsWith('/')
  const out = []
  for (const seg of p.split('/')) {
    if (seg === '' || seg === '.') continue
    if (seg !== '..') out.push(seg)
    else if (out.length > 0 && out[out.length - 1] !== '..') out.pop()
    else if (!abs) out.push('..')
  }
  const body = out.join('/')
  return abs ? '/' + body : body || '.'
}

// String values of path-like keys at any depth.
function pathValues(value, out = []) {
  if (Array.isArray(value)) value.forEach((v) => pathValues(v, out))
  else if (value !== null && typeof value === 'object') {
    for (const [k, v] of Object.entries(value)) {
      if (PATH_KEYS.has(k) && typeof v === 'string') out.push(v)
      else pathValues(v, out)
    }
  }
  return out
}

// undefined when nothing matched, else the ask/deny verdict naming every pattern hit.
export function checkToolUse({ tool, input }, patterns, mode) {
  const hits = new Set(matchPatterns(patterns, candidates(input), 'bash'))
  if (typeof tool !== 'string' || WRITE_TOOL.test(tool)) {
    for (const h of matchPatterns(patterns, pathValues(input).map(normalizePosix), 'file-write')) hits.add(h)
  }
  if (hits.size === 0) return undefined
  const names = [...hits].map((h) => h.name)
  return {
    decision: mode === 'confirm' ? 'ask' : 'deny',
    reason: `blast-radius-guard: ${[...new Set(names)].join('; ')}`,
  }
}

// Node harnesses: without a tool_input object (Kiro) the whole payload is scanned.
export function toolUseFromPayload(payload) {
  const ti = payload.tool_input
  const plain = ti !== null && typeof ti === 'object' && !Array.isArray(ti)
  return { tool: typeof payload.tool_name === 'string' ? payload.tool_name : undefined, input: plain ? ti : payload }
}
