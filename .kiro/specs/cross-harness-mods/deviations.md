# Cross-harness mods: approved deviations

The original [requirements](requirements.md), [design](design.md) and [tasks](tasks.md)
remain the historical spec. The approved [implementation plan](../../../.claude/plan/cross-harness-mods.md)
supersedes the conflicting details below. This record explains the implemented contract;
it does not claim every acceptance criterion in the original spec was satisfied literally.
See [mods/README.md](../../../mods/README.md) and each `mod.toml` for the capability matrix.

## API, adapters and generation

| Requirement | Original spec | Approved implementation and reason |
|---|---|---|
| 4.1–4.8 | Generated `claude-code/register.ts`, default `ModEventEmitter` export, Node APIs and event declarations in `hooks.json` | Hand-written `hooks/register.ts` exports `register: Register`, using the verified native runtime's web APIs and `$` capabilities. Generated `hooks/hooks.json` declares modules; subscriptions declare events. Native Audit Trail is unsupported. |
| 4.3 | TypeScript strict checking | Native `claude plugin validate` and `claude plugin test` run locally. CI has no native CLI; Node syntax and structural checks do not replace native validation. |
| 2.7 | Ten canonical events | Only `PreToolUse`, `PostToolUse` and `SessionStart` are admitted; these are the events the starter mods use with cross-harness mappings. |
| 3.6–3.9, 5.x | Kiro JSON ask/modify responses, POSIX companions and `.sh` adapters | Node `.mjs` entries under `bin/` share `lib/` with every adapter. Command surfaces require Node; syntax checks use `node --check`, not `bash -n`. Kiro/Codex asks become denies, and redactor Post events are declared unsupported. |
| 6.x, 12.2–12.3 | JSON data files imported by native modules; `live-patterns.json` | `patterns.json` remains source data; generated `lib/patterns.mjs` and `lib/live-agents.mjs` can be imported by the native runtime, which rejects JSON imports. Live agents are split into `covered` and `uncovered`. |
| 9.2 | Exclude executable mod scripts from scanning | Keep executable code scanned. Test credential-shaped strings are constructed at runtime; scanner exclusions are not a route around a security gate. |
| 16.1–16.2, 16.6 | Inline Python redaction/guard implementation in the validator | Production Node exports are exercised through `tests/mods.test.mjs` and property tests. A Python copy could pass while the shipped implementation fails. |
| 16.4–16.5 | Type-only catalog round-trip; same-input generator repeat | Test the supported TOML subset against a test-side serializer and byte-identical output under reordered mod inputs. Generation remains deterministic. |

The runtime also uses `node:path` for observer path resolution. Shared runtime helpers
read no mod environment and write no files; the entries perform declared file I/O.
Install-time `bin/`/`lib/` copies are not additional runtime capabilities.

## Security and behavioral changes

| Requirement | Original spec | Approved implementation and reason |
|---|---|---|
| 10.2 | Any base62 string of length 40 is an AWS secret; broad bearer and URL matches | Require AWS secret-key naming context, a bearer token of sufficient length, and a scheme-bounded password-in-URL pattern. The original forms overmatch hashes, prose and paths. |
| 10.3 | Rewrite credential-bearing Pre arguments | Ask before execution, or deny where ask is unavailable. Rewriting arguments would change the requested operation. Post output redaction is separate and only available on Claude Code surfaces. |
| 10.4–10.6 | Identical redaction across harnesses | Native redaction may return a redacted result or withhold it. Classic uses shape-compatible `updatedToolOutput`; failed tools are not covered, and a mismatched replacement may be ignored. Kiro/Codex redactor Post events remain unsupported. Codex's narrower MCP output replacement is not a general replacement contract. |
| 11.1, 12.1, 12.8–12.9 | Blast Radius Guard L1; Live Guard Gate L2; count generated files or possible future spawns | Both are L0: no runtime writes, spawns or egress. Build-time outputs and hypothetical capabilities do not increase declared reach. Context Burn and Audit Trail are L1 because they write files. |
| 11.4–11.6, 11.9–11.10, 15.3 | Shorter blast-radius env names and Kiro confirmation JSON | Use `VANGUARD_BLAST_RADIUS_GUARD_MODE` and `VANGUARD_BLAST_RADIUS_GUARD_PATTERNS`. Confirm asks only where supported; otherwise it denies. Unknown modes block. |
| 11.8 | Unanchored system-path expression | Anchor at the POSIX path start after lexical normalization of `.` and `..`. Relative paths remain unresolved against cwd; embedded `/usr/` in an ordinary project path is not treated as a system root. |
| 12.x live patterns | Include hcloud server/firewall verbs | Drop unverified hcloud verbs rather than encode model memory as CLI syntax. Covered/uncovered agent lists expose remaining coverage gaps. |
| 13.2–13.5 | `PostTaskExec`, a Kiro-only default, an estimated token accumulator and `max_tokens` option | Use `PostToolUse` and `SessionStart`; choose a status path per harness. Native reads real session context and toasts thresholds when usable data exists. Node adapters count calls only; no invented token counts or configured context window. |
| 14.x | Immutable, non-repudiable compliance evidence | An ordinary append log with argument/result hashes and per-session sequence numbers. No signatures, immutability, tamper evidence, non-repudiation or compliance guarantee. |
| 14.3, 14.7 | All payload fields guaranteed; Kiro result available | Missing agent/tool fields may be `null`. Kiro stdin fields are undocumented: hash the whole payload as arguments and `null` as the result, explicitly degraded. |
| 14.4, 14.9–14.10 | One daily file and `VANGUARD_AUDIT_*` names | `.audit/<UTC-date>/<session>.jsonl` by default, reusing an existing session file across dates. Use `VANGUARD_AUDIT_TRAIL_LOG_DIR`, `VANGUARD_AUDIT_TRAIL_SESSION_ID` and `VANGUARD_AUDIT_TRAIL_AGENT_ID`. |
| 14.8 | Native synchronous append | Native Audit Trail is unsupported: the verified native filesystem has no append. A whole-file rewrite would violate append-only behavior; spawning Node would add reach without needed coverage. Use the classic adapter. |
| 14.11–14.12 | PID/start-time session identity and POSIX hashing | Node hashes canonical JSON. Identity comes from env, then payload `session_id`, then `unsessioned`; unsafe filename characters are hashed. PIDs change on every command-hook invocation and do not identify a session. |
| Audit locking/framing | Shared-file line counts or automatic stale-lock takeover | Acquire a per-session lock before choosing the dated log. Never steal a lock based on age; reject unterminated records instead of corrupting the next append. Orphaned locks require explicit recovery. |
| 15.4 | Every write path contains the mod id | Retain the spec's `.audit/**` example and per-harness Context Burn defaults. The validator checks overlap among declared defaults; operator path overrides can exceed them. |
| 15.5 | Alphabetical filenames guarantee Kiro execution order | Generated hook filenames are deterministic. Do not infer a runtime ordering guarantee or composability proof from those names. |

Live Guard Gate names agents for manual review, not automatic dispatch. Regex matching
does not establish which account is live or provide complete DLP. Guard errors deny
only when execution reaches the hook; missing scripts, unavailable Node, startup
crashes and timeouts remain installation/enforcement gaps. Observer failures do not
block calls. Classic `PostToolUse` misses failed tools, affecting redaction, counts and audit.

## Installation and footprint

| Requirement | Original spec | Approved implementation and reason |
|---|---|---|
| 17.2–17.4 | Kiro `.sh` companions on PATH; `.claude/mods/`; Codex shell registration | Load a native plugin root with `claude --plugin-dir mods/<id>`, or copy Node `bin/` and `lib/` to `.<harness>/vanguard-mods/<id>/` and install generated hook JSON. No separate source pattern/catalog copy is needed. Never load native and classic for the same mod. |
| 17.1 | One install command without manual copying/merging | Explicit copy-and-merge instructions preserve existing settings and hooks. Kiro/Codex hook cwd is undocumented, so use absolute installed script paths; Codex also requires hook trust review. |

For installed Node observers, relative write paths resolve from the canonical installed
adapter location, otherwise from cwd. Native paths follow the native filesystem contract.
The footprint records default writes, not enforced containment:
`VANGUARD_CONTEXT_BURN_STATUS_PATH` and `VANGUARD_AUDIT_TRAIL_LOG_DIR` can redirect writes
outside declared globs. The reach taxonomy is a declaration, not sandbox enforcement.

## Verification boundary

`mods:write` regenerates manifests and data; `validate:mods` checks contracts and invokes
production-core tests; `test:fuzz` exercises properties. Asset integrity must be regenerated
last, after all source and generated files settle. Repository validation, spelling and
Markdown lint remain required. Native tests are local-only; live installation checks must
be reported separately from static/unit checks. This deviations record is not itself
evidence that a harness loaded or enforced a hook.
