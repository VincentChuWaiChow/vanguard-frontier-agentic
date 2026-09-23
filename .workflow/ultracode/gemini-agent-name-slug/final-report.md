# Final report

All 733 Gemini agent exports now load in Gemini CLI. Before this run, 714 were
skipped at load time and only 19 loaded. A gate now stops a non-slug `name`
from being committed again.

## What was wrong

Gemini CLI validates local agent frontmatter with a strict schema, and `name`
must match `/^[a-z0-9-_]+$/`. A file that fails is skipped, and the only record
is an entry in the loader's error list. Every generator wrote the agent's
display name, such as `"Azure Maestro"`, into `name`. Only the 19 Salesforce
exports, which no generator writes, used the agent id.

## What changed

- The ten generators that write `gemini.agent.md` now emit `name: <agent id>`
  and `display_name: <readable name>`.
- The 714 committed exports were rewritten to that form, with bodies unchanged.
- `validate:agent-schema` now requires each Gemini export to use its agent id
  as `name` and only the keys Gemini CLI's schema allows.

## Evidence

Gemini CLI's own loader (`@google/gemini-cli-core` 0.60.0) was run on a real
export. It loaded 19 agents with 714 errors before the change, and 733 agents
with 0 errors after. The gate fails on exactly the 714 files before the change
and passes after, and each planted violation is caught.

## Open

- `gen_azure_live_guards.py` is stale against six hand-edited Azure live-guard
  agents. The drift predates this run.
- Two agents ship no Gemini export.
