# Packet 01 — harness generator ownership

Owner: read-only Haiku agent (`Explore`). Write scope: none.

## Question

Which scripts write `agents/<provider>/<id>/harnesses/claude-code.agent.md`
and `cursor.agent.md`, and where do their `name:` and `description:`
frontmatter fields come from?

## Asks

1. Every script under `scripts/` or `tests/` that writes these files, with the
   write call's `file:line`, whether it writes the whole file or edits keys,
   and which keys it sets.
2. How `scripts/model-policy.mjs` projects `model:`/`effort:` into claude-code
   frontmatter, and whether it preserves other keys.
3. Whether `scripts/gen_azure_live_guards.py` writes Azure harness files, and
   the frontmatter it emits.
4. Which gate validates agent frontmatter, which files it checks, which fields
   it requires, and whether it would catch a harness file with no `name`.
5. The files and frontmatter of `agents/azure/azure-maestro-agent/`.

## Constraints

No edits, no test runs, no git. `file:line` for every claim; "not found" with
what was searched when absent.
