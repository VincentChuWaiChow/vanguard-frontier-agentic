---
name: model-registry-refresh
description: "Re-verify and extend catalog/model-registry.json — the fail-closed model-name and reasoning-effort matrix scripts/model-policy.mjs validates against — via delegated Context7-backed research, orchestrator-owned registry edits, and the full validation chain; use when a policy check fails on an unregistered model, a provider ships new models, or the registry has gone stale."
allowed-tools: ["Agent", "Read", "Edit", "Bash"]
---

# Model Registry Refresh

## Doctrine

`catalog/model-registry.json` is the single source of truth `scripts/model-policy.mjs` fails closed
against. Every model name and reasoning-effort value it accepts must trace to official documentation
with a citation — never to memory, never to a plausible-sounding guess at a slug. This skill is the
repeatable workflow for keeping that registry accurate without letting research cost dominate the
orchestrator's context.

## When to run

- `npm run model-policy:check` fails with an error naming a model "not in the verified model
  registry" — the registry is missing a model the policy (or an operator) wants to use.
- A provider (OpenAI, Anthropic, Cursor) ships new models or retires old ones and the catalog
  needs to reflect current reality.
- Quarterly staleness check — `last_refreshed` in `catalog/model-registry.json` is more than
  ~3 months old.

## Step 1 — delegate research to Haiku Explore agents

Fan out one Haiku `Explore` agent per harness (or per namespace, for codex) using the Context7 MCP
tools (`mcp__Context7__resolve-library-id` then `mcp__Context7__query-docs`) plus official docs
URLs already cited in the registry. Each research task must:

- Ask for **exact slugs/IDs**, not families — `gpt-5.5` not "the gpt-5 line".
- Ask for **reasoning-effort support per model**, not per harness — some models in a family
  predate newer effort levels (see `o1`/`o3`/`o4-mini` lacking `none`/`minimal`/`xhigh` in the
  current registry).
- Ask for **failure-mode evidence** — what error shape a bad model name or unsupported effort
  actually produces (HTTP status, error code/type), so `docs/model-policy-matrix.md`'s failure
  table stays accurate.
- **Require a source citation per claim** — a Context7 library ID + section, or an official docs
  URL. A finding without one is not actionable.
- **Require an explicit `UNVERIFIED` flag** on anything the agent could not confirm from a primary
  source (e.g. inferred from a changelog mention, or contradicted between two docs). Do not let
  an agent silently round an uncertain claim into a confident one.

Example prompt template (adapt per harness/namespace):

```
Research current [codex OpenAI models | codex Ollama routing | codex OpenRouter routing |
claude-code subagent model/effort fields | cursor subagent model field] using Context7
(resolve-library-id then query-docs) and official docs. Report, for each model/field:
exact slug or ID, supported reasoning-effort values (if any), and the error shape observed
or documented for an invalid value (HTTP status + error code/type). Cite the Context7 library
ID + section or the exact docs URL for every claim. If you cannot confirm a claim from a
primary source, prefix it UNVERIFIED and say why. Do not guess slugs from training data.
```

Run these Explore agents in parallel; each is scoped to one harness or namespace so the
citations stay traceable to a narrow question.

## Step 2 — orchestrator updates the registry

The orchestrator, not a delegate, edits `catalog/model-registry.json`:

- Add new models with `last_verified` (today's date) and a `source` where the schema allows it;
  update the relevant namespace's `sources` array if a new canonical URL was used.
- **Prefer the readable alias over a dated snapshot ID.** Anthropic's convention
  ([model-ids-and-versions](https://platform.claude.com/docs/en/about-claude/models/model-ids-and-versions)):
  from the 4.6 generation on, IDs are dateless *and are themselves the pinned
  snapshot* (`claude-opus-5`, `claude-sonnet-4-6`) — there is no alias to add.
  Before 4.6, the canonical ID carries a snapshot date and the API also exposes
  a shorter alias pointing at the most recent dated snapshot: register **both**,
  and write the alias (`claude-sonnet-4-5`) as the entry an operator reaches
  for, keeping the dated form (`claude-sonnet-4-5-20250929`) for when an exact
  snapshot is required. Never invent an alias for a dateless ID, and never drop
  the dated entry. Apply the same instinct to other providers: register the
  form a human can recognize, not only the fully-qualified one.
- **A capability is only real on the surface this registry governs.** The
  registry validates `codex.toml`, subagent frontmatter and `.agent.md` — not
  every API a provider ships. A field documented on one route, present in an
  enum, or shown in a web UI is not evidence the configured surface accepts it.
  Three concrete cases this rule came from: Ollama documents `reasoning_effort`
  on `/v1/chat/completions` but omits it from `/v1/responses`, which is the
  route the namespace configures (so it stays fail-closed); OpenRouter *does*
  document it on its Responses route, but with a narrower four-value list than
  its chat-completions surface (so the narrower list is what is registered);
  and `ultra` is in the Codex `ReasoningEffort` enum and the ChatGPT desktop
  picker, but the CLI effort list stops at Max (so it is excluded). Ask "which
  surface, and does *that* one document it?" before widening any vocabulary.
- Bump the registry-level `last_refreshed` date.
- **Never remove a model still referenced by `catalog/model-policy.json`** without first
  migrating the policy rule(s) that reference it to a replacement model — check with
  `npm run model-policy:report` before deleting anything.
- Treat every `UNVERIFIED`-flagged finding from Step 1 as a blocker, not a data point to
  merge as-is — either verify it directly or leave the registry unchanged for that item.
- Validate the edit against `schemas/model-registry.schema.json` structurally (required fields,
  anchored `match` patterns, `last_verified` date format) before moving on.

## Step 3 — sync the human-readable matrix

Delegate to a Sonnet writer subagent to update `docs/model-policy-matrix.md` so its tables match
the registry exactly (namespace tables, verified-model tables, failure modes, enforcement
boundaries). Give the delegate the exact diff you made to `catalog/model-registry.json` in Step 2
and instruct it to touch only `docs/model-policy-matrix.md` — no other file, no commits.

## Step 4 — verify

Run in order, orchestrator-owned:

```bash
npm run model-policy:check          # registry schema + policy resolves against it
npm run validate                    # full gate suite
npm run asset-integrity:write       # LAST — after every other write has settled
```

The orchestrator reviews the full diff (registry, matrix doc, any touched harness projections)
and is the only one who commits. A delegate's self-report that research or writing is "done" is
not verification — read the diff and run the gates yourself before accepting.

## Delegation defaults

- **Haiku** — research only (Step 1): Context7 lookups, docs reading, citation gathering. Never
  writes to `catalog/model-registry.json` or any tracked file.
- **Sonnet** — writing only (Step 3): syncing `docs/model-policy-matrix.md` prose/tables to a
  registry diff the orchestrator already made. Never edits `catalog/model-registry.json` itself.
- **Orchestrator** — owns `catalog/model-registry.json` edits, schema/gate verification, and the
  commit. This is the same split `.claude/skills/agentic-delegation/SKILL.md` codifies more
  generally: cheap parallel research to Haiku, bulk writing to Sonnet, judgment and commits stay
  with the orchestrator.
