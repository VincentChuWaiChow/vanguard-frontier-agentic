# Orchestration

## Parent critical path

Freeze the baseline, dispatch one bounded read-only wave, derive values only
from committed data, make the edits and the gate change, verify with paired
probes and a real install, run the full gate suite, then commit and push.

## Packets

- `01-harness-generator-ownership`
- `02-canonical-name-description-source`
- `03-harness-frontmatter-contract`

## Delegation

One parallel read-only wave of three Haiku agents via the native `Explore`
type, per `.claude/skills/agentic-delegation/SKILL.md`. No agent may edit files,
run a test suite, or invoke git. Every repository claim needs a `file:line`
citation; every documentation claim needs the URL fetched.

## What the parent keeps

The derivation rule, every file edit, the gate change (load-bearing validation
logic), verification, and the commit. The 80 edits are a deterministic
transformation of committed data, so the parent performs them with a script it
can re-run and diff, rather than handing them to a writing delegate.

## Wait points

Integrate after all three reports arrive. Packet 02 blocks the edit: no value
is written until its derivation rule is confirmed byte-for-byte on healthy
controls.

## Fallback

The parent reproduces any disputed or thin finding locally before acting on
it. A documentation claim a packet cannot source is treated as unknown.

## Verification order

Targeted probe for the changed behaviour, the owning gate, a real Claude Code
install from the simulated post-merge tree, then `npm run validate`, spell and
markdown lint, then asset integrity last and alone.
