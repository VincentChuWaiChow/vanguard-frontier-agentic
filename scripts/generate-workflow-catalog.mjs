#!/usr/bin/env node
/**
 * Generator: catalog/workflows.json from .claude/workflows/*.js
 *
 * Workflow scripts are executable JavaScript whose body calls workflow globals
 * (`phase`, `agent`, `parallel`) that only exist inside the workflow runtime, so the
 * files cannot be imported to read their metadata — loading one outside the runtime
 * throws on the first `phase()` call.
 *
 * They can, however, be read statically. The workflow contract requires `meta` to be a
 * PURE LITERAL (no variables, calls, spreads, or interpolation), so this extracts the
 * object literal following `export const meta =` by brace matching and then PARSES it
 * with `parseLiteral` — it is only parsed. Executing it would mean `npm run
 * validate` executing code out of any workflow file a contributor adds; parsing both
 * removes that and turns the pure-literal contract into something actually enforced.
 *
 * The output exists so `tools/vfa-tui` can display available workflows without
 * re-implementing any of this: the TUI reads generated catalog JSON and never parses
 * JavaScript. Same contract as every other catalog file.
 *
 * Usage:
 *   node scripts/generate-workflow-catalog.mjs           # write
 *   node scripts/generate-workflow-catalog.mjs --check   # verify in sync (CI)
 */
import { readFileSync, writeFileSync, readdirSync, existsSync, realpathSync } from 'node:fs'
import { join, dirname, relative, sep } from 'node:path'
import { fileURLToPath } from 'node:url'

const ROOT = dirname(dirname(fileURLToPath(import.meta.url)))
const WORKFLOW_DIR = join(ROOT, '.claude', 'workflows')
const OUT = join(ROOT, 'catalog', 'workflows.json')

/**
 * Extract the balanced `{...}` literal following `export const meta =`.
 * Returns null when the marker is absent.
 */
function extractMetaLiteral(source) {
  const marker = /export\s+const\s+meta\s*=\s*/.exec(source)
  if (!marker) return null
  let i = marker.index + marker[0].length
  if (source[i] !== '{') return null

  // Brace matching that respects strings, template literals, and comments — a `{`
  // inside a description string must not be counted as nesting.
  let depth = 0
  let inS = null // "'" | '"' | '`'
  let inLine = false
  let inBlock = false
  for (; i < source.length; i++) {
    const c = source[i]
    const prev = source[i - 1]
    if (inLine) {
      if (c === '\n') inLine = false
      continue
    }
    if (inBlock) {
      if (c === '/' && prev === '*') inBlock = false
      continue
    }
    if (inS) {
      if (c === inS && prev !== '\\') inS = null
      continue
    }
    if (c === '/' && source[i + 1] === '/') { inLine = true; continue }
    if (c === '/' && source[i + 1] === '*') { inBlock = true; continue }
    if (c === "'" || c === '"' || c === '`') { inS = c; continue }
    if (c === '{') depth++
    else if (c === '}') {
      depth--
      if (depth === 0) return source.slice(marker.index + marker[0].length, i + 1)
    }
  }
  return null
}

/**
 * Parse a JavaScript *data* literal without executing it.
 *
 * Accepts exactly the pure-literal subset the workflow `meta` contract promises:
 * objects (quoted or bare identifier keys, optional trailing comma), arrays, single-,
 * double- and backtick-quoted strings with no `${}` substitution, numbers, `true`,
 * `false`, `null`. Anything else — a call, an identifier reference, a spread, a
 * template substitution, an operator — is a syntax error rather than something that
 * runs. Comments are skipped.
 *
 * Deliberately hand-written rather than delegated to a JavaScript code executor: the point
 * is that no code path here can execute the input.
 */
export function parseLiteral(src) {
  let i = 0

  const fail = (msg) => {
    throw new Error(`${msg} at offset ${i}`)
  }
  const skip = () => {
    for (;;) {
      while (i < src.length && /\s/.test(src[i])) i++
      if (src[i] === '/' && src[i + 1] === '/') {
        while (i < src.length && src[i] !== '\n') i++
      } else if (src[i] === '/' && src[i + 1] === '*') {
        i += 2
        while (i < src.length && !(src[i] === '*' && src[i + 1] === '/')) i++
        if (i >= src.length) fail('unterminated block comment')
        i += 2
      } else {
        return
      }
    }
  }

  const readString = () => {
    const quote = src[i++]
    let out = ''
    while (i < src.length) {
      const c = src[i]
      if (c === '\\') {
        const esc = src[i + 1]
        const simple = { n: '\n', t: '\t', r: '\r', b: '\b', f: '\f', v: '\v', 0: '\0' }
        if (esc === 'u') {
          const hex = src.slice(i + 2, i + 6)
          if (!/^[0-9a-fA-F]{4}$/.test(hex)) fail('bad unicode escape')
          out += String.fromCharCode(parseInt(hex, 16))
          i += 6
        } else if (esc === 'x') {
          const hex = src.slice(i + 2, i + 4)
          if (!/^[0-9a-fA-F]{2}$/.test(hex)) fail('bad hex escape')
          out += String.fromCharCode(parseInt(hex, 16))
          i += 4
        } else if (esc === '\n') {
          i += 2 // line continuation
        } else if (esc in simple) {
          out += simple[esc]
          i += 2
        } else {
          out += esc
          i += 2
        }
        continue
      }
      if (c === quote) {
        i++
        return out
      }
      // A template substitution would require running code to resolve, which is exactly
      // what this parser exists to avoid.
      if (quote === '`' && c === '$' && src[i + 1] === '{') fail('template substitution is not a literal')
      if (quote !== '`' && c === '\n') fail('unterminated string')
      out += c
      i++
    }
    return fail('unterminated string')
  }

  const readValue = () => {
    skip()
    const c = src[i]
    if (c === '{') return readObject()
    if (c === '[') return readArray()
    if (c === '"' || c === "'" || c === '`') return readString()
    if (src.startsWith('true', i)) { i += 4; return true }
    if (src.startsWith('false', i)) { i += 5; return false }
    if (src.startsWith('null', i)) { i += 4; return null }
    const num = /^-?\d+(\.\d+)?([eE][+-]?\d+)?/.exec(src.slice(i))
    if (num) { i += num[0].length; return Number(num[0]) }
    return fail(`unexpected token ${JSON.stringify(src.slice(i, i + 24))}`)
  }

  const readObject = () => {
    i++ // {
    const obj = {}
    skip()
    if (src[i] === '}') { i++; return obj }
    for (;;) {
      skip()
      if (src[i] === '.' ) fail('spread is not a literal')
      let key
      if (src[i] === '"' || src[i] === "'" || src[i] === '`') {
        key = readString()
      } else {
        const id = /^[A-Za-z_$][A-Za-z0-9_$]*/.exec(src.slice(i))
        if (!id) fail('expected property name')
        key = id[0]
        i += id[0].length
      }
      skip()
      if (src[i] !== ':') fail(`expected ':' after key ${JSON.stringify(key)}`)
      i++
      obj[key] = readValue()
      skip()
      if (src[i] === ',') { i++; skip(); if (src[i] === '}') { i++; return obj } continue }
      if (src[i] === '}') { i++; return obj }
      return fail("expected ',' or '}'")
    }
  }

  const readArray = () => {
    i++ // [
    const arr = []
    skip()
    if (src[i] === ']') { i++; return arr }
    for (;;) {
      skip()
      if (src[i] === '.') fail('spread is not a literal')
      arr.push(readValue())
      skip()
      if (src[i] === ',') { i++; skip(); if (src[i] === ']') { i++; return arr } continue }
      if (src[i] === ']') { i++; return arr }
      return fail("expected ',' or ']'")
    }
  }

  const value = readValue()
  skip()
  if (i !== src.length) fail('trailing content after literal')
  return value
}

function readWorkflow(file) {
  const abs = join(WORKFLOW_DIR, file)
  const src = readFileSync(abs, 'utf8')
  const literal = extractMetaLiteral(src)
  if (!literal) {
    throw new Error(`${file}: no 'export const meta = {...}' block found`)
  }
  let meta
  try {
    // PARSED, never executed. An earlier version compiled this literal as JavaScript,
    // which meant `npm run validate` executed code out of any workflow file a
    // contributor added — `description: (() => { …anything… })()` would have run with
    // the validator's privileges. In a repository whose subject is supply-chain
    // integrity that is not an acceptable way to read metadata, and it also let
    // nondeterministic values into a generated catalog. parseLiteral accepts only
    // JSON-shaped data and rejects everything else, which additionally makes the
    // "meta must be a pure literal" contract actually enforced rather than assumed.
    meta = parseLiteral(literal)
  } catch (err) {
    throw new Error(`${file}: meta is not a pure literal (${err.message})`)
  }
  if (!meta || typeof meta.name !== 'string' || typeof meta.description !== 'string') {
    throw new Error(`${file}: meta must declare string 'name' and 'description'`)
  }
  const expected = file.replace(/\.m?js$/, '')
  if (meta.name !== expected) {
    throw new Error(
      `${file}: meta.name '${meta.name}' must match the filename '${expected}' — ` +
      'the workflow is invoked by name, and a mismatch makes it unreachable',
    )
  }
  return {
    id: meta.name,
    name: meta.name,
    // Forward slashes always. `relative()` emits backslashes on Windows, which would
    // make generated output platform-dependent: `--check` would fail on one OS against
    // a catalog committed from another, and the Rust loader test asserts a
    // `.claude/workflows/` prefix. A repo-relative path in a committed artifact is a
    // repo path, not a filesystem path.
    path: relative(ROOT, abs).split(sep).join('/'),
    description: meta.description,
    when_to_use: typeof meta.whenToUse === 'string' ? meta.whenToUse : '',
    phases: Array.isArray(meta.phases)
      ? meta.phases.map((p) => ({
          title: String(p.title ?? ''),
          detail: typeof p.detail === 'string' ? p.detail : '',
          model: typeof p.model === 'string' ? p.model : '',
        }))
      : [],
  }
}

function build() {
  const files = existsSync(WORKFLOW_DIR)
    ? readdirSync(WORKFLOW_DIR).filter((f) => /\.m?js$/.test(f)).sort()
    : []
  const workflows = files.map(readWorkflow)
  return {
    version: '0.1.0',
    description:
      'Workflow definitions discovered in .claude/workflows/. Generated from each ' +
      "script's meta block by scripts/generate-workflow-catalog.mjs — never hand-edited.",
    workflows,
  }
}

function main() {
  const catalog = build()
  const serialized = JSON.stringify(catalog, null, 2) + '\n'

  if (process.argv.includes('--check')) {
    const current = existsSync(OUT) ? readFileSync(OUT, 'utf8') : ''
    if (current !== serialized) {
      console.error(
        'FAIL: catalog/workflows.json is out of sync with .claude/workflows/.\n' +
        '      Run: npm run workflow-catalog:write',
      )
      process.exit(1)
    }
    console.log(`OK: workflow catalog in sync (${catalog.workflows.length} workflow(s))`)
  } else {
    writeFileSync(OUT, serialized)
    console.log(`OK: wrote catalog/workflows.json (${catalog.workflows.length} workflow(s))`)
  }
}

// Only run when invoked directly. `parseLiteral` is exported so it can be tested and
// reused, and importing a module must never write a file as a side effect.
if (process.argv[1] && realpathSync(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main()
}
