# Orchestration

## Parent critical path

Freeze the baseline commit, dispatch one bounded read-only wave, integrate only
evidence-backed findings, implement fixes the parent owns, verify with paired
probes, run the full gate suite, then commit.

## Packets

- `01-export-detection-contract`
- `02-routing-fixture-independence`
- `03-secret-scan-coverage`

## Delegation

One parallel read-only review wave. No agent may edit repository source, run a
test suite, or invoke git. Every claim must carry a `file:line` citation.

## Agents

Three Haiku review agents via the native `Explore` type, per
`.claude/skills/agentic-delegation/SKILL.md`. The parent keeps architecture
decisions, all security-sensitive edits, verification, and the commit.

## Delegation limits

Three agents, one wave, bounded single-concern packets, evidence-only handoffs.
A self-report is not verification: the parent re-reads the diff and re-runs the
gates.

## Wait points

Integrate after all three reports arrive, or on a concrete reported blocker.

## Fallback

The parent reproduces any disputed or thin finding locally before acting on it.

## Verification order

Targeted probe for the changed behaviour, then the owning gate, then
`npm run validate`, then spell and markdown lint, then the cargo gates when
`tools/vfa-tui` changed, then asset integrity last and alone.
