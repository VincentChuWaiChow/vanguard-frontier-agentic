# Plan

## Objective

Make the 16 Azure agents whose agent files carry no `name` or `description`
loadable as distinct, selectable agents in every harness the repository ships,
and add a gate so a nameless agent file cannot be committed again.

## Evidence that opened this run

A real Claude Code install of the v3.13.0 release candidate (simulated from PR
#194 merged into `master`) loaded all 735 agent files, but logged
`Duplicate agent name 'vanguard-frontier-agentic:claude-code.agent' (plugin)`.
The official plugin reference states that a plugin agent without a `name` is
named after its file, and every harness file in this repository is called
`claude-code.agent.md`, so the nameless agents collapse into one scoped name
and at most one is reachable. With no `description`, none can be chosen by
automatic delegation.

A scan of every harness markdown file found the defect in exactly 16 agents,
all under `agents/azure/`, in all five markdown formats: `claude-code`,
`cursor`, `copilot`, `gemini` and `kiro-ide`. That is 80 files, and no other
provider is affected. The canonical `AGENT.md` of each of the 16 has the same
metadata-only frontmatter, and `validate:agent-schema` passes it, so the gap is
in the source file and in the gate, not only in the exports.

The defect predates PR #194: the files were last changed on 2026-07-31 and the
PR does not touch them. It ships in the current v3.12.1 plugin.

## Scope

In scope: the canonical frontmatter of the 16 agents, their five harness
markdown variants, whatever generator owns those files, and a gate covering
`name` and `description` on every agent markdown file.

Out of scope: `codex.toml` and `kiro-cli.agent.json`, which are not markdown
and carry identity differently; rewording any agent body; the `package-lock.json`
version lag noted during the release check, which the user did not ask to fix.

## Baseline

Commit `cddb98c1`, branch `claude/stoic-meitner-h2aqr8`, tree clean, all 21 CI
checks green, PR #194 mergeable.

## Method

One read-only wave establishes who writes these files, where the correct
values come from, and what each harness documents. The parent then derives the
values from committed data by the rule the healthy agents already follow,
regenerates through the owning tooling where it exists, extends the gate, and
verifies with a paired positive and negative probe plus a real Claude Code
install.
