// Command-hook entry for classic Claude Code (PreToolUse, PostToolUse), Kiro and Codex
// (PreToolUse only: neither can replace tool output).
import { readFileSync } from 'node:fs'
import { maskValue } from '../lib/common.mjs'
import { WITHHELD, checkInput, loadPatterns, logLines, redactOutput } from '../lib/core.mjs'
import { runHook } from '../lib/runtime.mjs'

const isObject = (v) => v !== null && typeof v === 'object' && !Array.isArray(v)

function patterns() {
  const file = process.env.VANGUARD_SECRET_REDACTOR_PATTERNS
  return loadPatterns(file ? readFileSync(file, 'utf8') : undefined, file)
}

// Kiro documents no stdin field names: without a tool_input object, scan the whole payload.
const PreToolUse = (payload) => checkInput(isObject(payload.tool_input) ? payload.tool_input : payload, patterns())

const PostToolUse = (payload, { event, harness }) => {
  if (harness !== 'claude-code') throw new Error(`PostToolUse is unsupported on ${harness}`)
  const { value, hits } = redactOutput(payload.tool_response, patterns())
  if (hits.length === 0) return undefined
  return { decision: 'replace', output: value, log: logLines(hits, payload.tool_name, event) }
}

// No error text in either verdict: a parse error can quote the payload it failed on.
const onError = (err, payload, { event }) =>
  event === 'PreToolUse'
    ? { decision: 'deny', reason: 'secret-redactor: check failed; denied' }
    : {
        decision: 'replace',
        output: maskValue(payload?.tool_response ?? WITHHELD, WITHHELD),
        log: ['[secret-redactor] redaction failed; output withheld'],
      }

process.exitCode = await runHook({ name: 'secret-redactor', handlers: { PreToolUse, PostToolUse }, onError })
