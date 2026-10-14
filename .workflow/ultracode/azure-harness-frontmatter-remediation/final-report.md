# Final report

Sixteen Azure agents, including the Azure router `azure-maestro-agent`, now load
as distinct, described agents in Claude Code, and a gate stops a nameless agent
export from being committed again.

## What was wrong

Their harness exports were verbatim copies of `AGENT.md`, whose frontmatter
holds only `metadata`. With no `name`, Claude Code names a plugin agent after
its file; every export is `claude-code.agent.md`, so the 16 collapsed into one
agent and 15 were dropped. With no `description`, none could be chosen by
automatic delegation. The same gap existed in the Cursor, Copilot, Gemini and
Kiro IDE exports. It shipped in v3.12.1 and predates PR #194.

## What changed

80 harness files gained `name` and `description`, derived from committed
`metadata.json` by the rule the healthy agents follow; bodies are unchanged.
`validate:agent-schema` now checks every markdown harness export for both
fields and requires claude-code names to be unique.

## Evidence

The gate fails on exactly the 80 files before the fix and passes after; a
planted duplicate name fails; a real Claude Code install loads all 735 agents
with no duplicate warning.

## Open

Gemini's documentation requires slug-form agent names, and every Gemini export
in the repository uses display names. That is pre-existing, repository-wide
and unprobed here, and needs a naming decision before it is changed.
