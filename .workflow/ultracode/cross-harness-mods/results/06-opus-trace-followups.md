# opus-trace follow-ups (orchestrator triage)

Fold into Wave A fixups (gate/plan):
- Gate: semver check on [mod].version (GAP 2.6).
- Gate: FAIL when an event is unsupported in every supported harness (GAP 2.8).
- Gate: register.ts and non-runtime lib files must not import runtime.mjs (C5: node:fs in native graph).
- Generator: hooks/hooks.json only when native (C4) — verify A2 output.
- Plan: fix C1 (live-patterns.json -> patterns.json), C2 (fail-closed wording: list fail-open causes), C6 (generator --check line format is wrapped by gate as FAIL [mods] lines).
- Deviations: rows for 2.7 3.9 4.1 10.2 10.3 11.8 12.2 12.3 13.3 13.4 14.4 17.2; add 11.5/12.4 to the ask->deny row.
- mod.toml: audit-trail + context-burn classic degraded "PostToolUse fires only on success"; audit-trail kiro degraded text add session_id/result_hash; blast-radius kiro text "every path key" -> the four keys; classic requires node.

Wave B design inputs:
- F1: lexically normalize POSIX paths (collapse . and ..) in blast-radius core before the system-path rule; relative paths documented as limitation.
- F2: live-guard uncovered list must be declared (README Limitations + lib/live-agents.mjs uncovered).
- F3: Codex payload fields unverified -> treat missing tool_input as whole-payload scan (fail closed).
- F5: verify $.env.get and native file read for *_PATTERNS in d.ts; else declare native degraded.
- F6: total candidate budget (LIMITS.maxTotalChars) in matchPatterns.
- Context-burn: specify 70%/90% message text and status fields (GAP 13.6-13.8).
- 15.4: audit default dir contains mod id, or deviation row.
