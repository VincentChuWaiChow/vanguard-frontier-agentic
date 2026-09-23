# Orchestration

## Parent critical path

Freeze the baseline, dispatch the read-only wave, decide the naming rule on the
evidence, change generators then exports, extend the gate, verify, run the full
gate suite, commit and push.

## Packets

- `01-gemini-cli-name-contract`
- `02-repository-writers-and-checks`

## Delegation

One parallel read-only wave of two Haiku agents via the native `Explore` type,
per `.claude/skills/agentic-delegation/SKILL.md`. No agent may edit files, run a
test suite, or invoke git. Repository claims need `file:line`; upstream claims
need a Context7 library id and file, or the URL fetched.

## What the parent keeps

The naming rule, every generator edit (catalog generators are load-bearing),
the rewrite of committed exports (a deterministic, re-runnable, refusing
script), the gate change, verification and the commit.

## Wait points

Packet 01 blocks the naming decision. Packet 02 blocks any edit: no file is
changed until every writer is known, so a regeneration cannot silently revert
the fix.

## Fallback

A contract claim packet 01 cannot source from Gemini CLI's code or docs is
treated as unknown, and the rule falls back to the documented regex.

## Verification order

Targeted probe, the owning gate, generator re-runs proving no drift, a Gemini
CLI load if available, then `npm run validate`, spell and markdown lint, then
asset integrity last and alone.
