// Secret Redactor logic, shared by the native module and the Node hook so every harness
// reaches the same verdict. Pure: no I/O, no environment.
import { compilePatterns, parseJson, redactValue } from './common.mjs'
import builtin from './patterns.mjs'

const BUILTIN = compilePatterns(builtin, 'patterns.json')
const CONTEXTS = new Set(['tool-input', 'tool-output'])
const NOTICE = 'secret-redactor: credentials redacted from this result: '
export const WITHHELD = '[secret-redactor: output withheld; redaction failed]'

const inContext = (patterns, context) => patterns.filter((p) => p.context.includes(context))
const names = (hits) => hits.map((h) => h.name).join('; ')

// Built-in patterns plus the operator's extension, appended (never replacing). Throws on bad
// JSON or shape, a reused built-in id or an unknown context; callers fail closed.
export function loadPatterns(extraText, source = 'extension patterns') {
  if (extraText === undefined) return BUILTIN
  const extra = compilePatterns(parseJson(extraText, source), source)
  for (const p of extra) {
    if (BUILTIN.some((b) => b.id === p.id)) throw new Error(`${source}: id ${p.id} is a built-in pattern`)
    if (!p.context.every((c) => CONTEXTS.has(c))) throw new Error(`${source}: ${p.id} context must be tool-input or tool-output`)
  }
  return [...BUILTIN, ...extra]
}

// Asks before a call whose input carries a credential. The reason names the patterns, never
// the matched text.
export function checkInput(input, patterns) {
  const { hits } = redactValue(input, inContext(patterns, 'tool-input'))
  return hits.length ? { decision: 'ask', reason: `secret-redactor: tool input contains ${names(hits)}` } : undefined
}

export function redactOutput(output, patterns) {
  return redactValue(output, inContext(patterns, 'tool-output'))
}

// Native tool.call result. A clean result goes back untouched (core keeps its own messages).
// A redacted one becomes the hook's own { result }, which core re-validates and re-maps, so
// the model's text is rebuilt from the redacted value. What cannot be rebuilt that way is
// withheld with its redacted text instead: an error, a secret that shows only in core's
// text, or one in `context` (a hook may add context entries but never rewrite them).
export function redactResult(r, patterns) {
  const out = inContext(patterns, 'tool-output')
  if (typeof r.deny === 'string') {
    const d = redactValue(r.deny, out)
    return d.hits.length ? { deny: d.value } : r
  }
  const result = redactValue(r.result, out)
  const context = redactValue(r.context ?? [], out)
  const text = redactValue(r.text ?? '', out)
  if (result.hits.length + context.hits.length + text.hits.length === 0) return r
  if (r.isError || context.hits.length || result.hits.length === 0) {
    const shown = typeof r.text === 'string' ? text.value : typeof result.value === 'string' ? result.value : (JSON.stringify(result.value) ?? '')
    return { deny: [NOTICE + shown, ...context.value].join('\n') }
  }
  return r.context ? { result: result.value, context: r.context } : { result: result.value }
}

export function logLines(hits, tool, event) {
  return hits.map((h) => `[secret-redactor] redacted ${h.name} in ${tool ?? 'unknown tool'} ${event}`)
}
