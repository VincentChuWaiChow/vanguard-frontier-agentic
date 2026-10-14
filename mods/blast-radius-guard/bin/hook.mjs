// Command-hook entry for classic Claude Code, Kiro and Codex.
import { readFileSync } from 'node:fs'
import { checkToolUse, compileGuardPatterns, modeOf, toolUseFromPayload } from '../lib/core.mjs'
import { runHook } from '../lib/runtime.mjs'

const PreToolUse = (payload) => {
  const mode = modeOf(process.env.VANGUARD_BLAST_RADIUS_GUARD_MODE)
  const file = process.env.VANGUARD_BLAST_RADIUS_GUARD_PATTERNS
  const patterns = compileGuardPatterns(file ? readFileSync(file, 'utf8') : undefined)
  return checkToolUse(toolUseFromPayload(payload), patterns, mode)
}

const onError = (err) => ({
  decision: 'deny',
  reason: `blast-radius-guard: check failed; denied (${err instanceof Error ? err.message : String(err)})`,
})

process.exitCode = await runHook({ name: 'blast-radius-guard', handlers: { PreToolUse }, onError })
