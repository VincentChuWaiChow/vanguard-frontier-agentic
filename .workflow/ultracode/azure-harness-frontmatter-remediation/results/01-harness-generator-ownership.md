# Result 01 — harness generator ownership

Accepted, verified by the parent where it changed a decision.

- Ten generators write whole harness files with `name` and `description` taken
  from the agent's `name` and `summary`: the eight that share `md_harness()`
  (`scripts/gen_python_agents.py:312-313` and siblings), plus
  `scripts/gen_azure_live_guards.py:1404-1405` and
  `scripts/gen_oci_live_guards.py:1491-1492`.
- The 16 affected agents are hand-maintained: no generator writes their harness
  files. Their exports are verbatim copies of `AGENT.md`, frontmatter included.
- `scripts/model-policy.mjs` edits only `model` and the reasoning key through
  `editFrontmatterKey()` (lines 955-992) and leaves other keys untouched, so it
  cannot have caused or undone this.
- `tests/validate-agent-frontmatter-schema.py:225` reads `AGENT.md` only, and
  `name` and `description` are optional there. No gate read the harness
  exports, so nothing could catch a nameless one.
