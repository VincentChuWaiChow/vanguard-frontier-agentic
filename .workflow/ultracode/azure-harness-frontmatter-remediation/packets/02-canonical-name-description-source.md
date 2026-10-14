# Packet 02 — canonical name and description source

Owner: read-only Haiku agent (`Explore`). Write scope: none.

## Question

What exact rule produces `name:` and `description:` for Azure agents, so the
values for the 16 broken agents are derived rather than invented?

## Asks

1. For healthy controls (`azure-landing-zone-architect-agent`,
   `azure-live-aks-rollout-guard-agent`, `azure-entra-id-specialist-agent`):
   the `name`/`description` in `AGENT.md` and `claude-code.agent.md`, candidate
   source fields in `metadata.json`, and whether `name` equals the first `# `
   heading and `description` equals a metadata field byte-for-byte.
2. For each of the 16 broken agents: the first `# ` heading and the full
   metadata summary/description value, flagging any that is missing or needs
   YAML escaping.
3. Whether each broken agent's harness bodies match each other and `AGENT.md`.
4. The complete frontmatter of all five harness files for the healthy controls.

## Constraints

No edits, no test runs, no git. Values quoted exactly with `file:line`.
