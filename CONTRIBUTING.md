# Contributing to Vanguard Frontier Agentic

This guide covers everything you need to add a skill, agent, rule, MCP reference, mod, schema fix, or doc improvement to this repository. Read it alongside the companion docs it links to — this file adds the contributor-specific glue, not the full governance narrative.

---

## Quick Start

```bash
git clone https://github.com/VincentChuWaiChow/vanguard-frontier-agentic.git
cd vanguard-frontier-agentic
npm install
npm run validate
```

All checks must pass before you open a pull request. If `npm run validate` fails on a clean clone, open an issue rather than working around it.

---

## What you can contribute

| Area | Where it lives | When to contribute |
|------|----------------|--------------------|
| **Skills** | `skills/<provider>/<name>/` | Reusable step-by-step workflows for recurring engineering tasks |
| **Agents** | `agents/<provider>/<id>/` | Expert roles with judgment for review, architecture, and operations |
| **Rules** | `rules/<harness>/` | Durable harness-specific operating instructions |
| **MCP references** | `mcp/` | Trusted notes for connecting tools to real systems |
| **Mods** | `mods/<id>/` | Tool-event guards and observers with explicit harness capabilities and runtime footprints |
| **Schema fixes** | `schemas/` | Corrections to JSON Schema metadata contracts |
| **Docs** | `docs/` | Governance, taxonomy, compatibility, and quality guidance |

Before writing anything, check `catalog/skills.json` and `catalog/agents.json` to confirm the asset does not already exist under a different name.

---

## Adding a Skill

### Directory layout

```
skills/<provider>/<skill-id>/
  SKILL.md          # required — the skill body with YAML frontmatter
  metadata.json     # required — matches schemas/skill.schema.json
  references/       # optional — source links, evidence templates, supporting context
```

Provider must be one of the values in the `provider` enum in `schemas/skill.schema.json` (see `schemas/agent.schema.json` for agents) — that enum is the authoritative contract, so adding a new provider means extending it there, not in this doc.

### SKILL.md frontmatter

Skills use YAML frontmatter at the top of `SKILL.md`. Required fields:

```yaml
---
name: <skill-id>               # lowercase, hyphen-separated; must match directory name
description: <one-line prose>  # what this skill does and when it is invoked
allowed-tools: Read Edit Write Grep Glob   # space-separated list of tools this skill may use
metadata:
  author: "github: <YourGitHubHandle>"    # use this exact format; do not use a top-level author key
  version: "0.1.0"                        # semver; do not use a top-level version key
---
```

The `allowed-tools` field is validated by `npm run validate:allowed-tools`. Omitting it or listing tools the skill body does not need will fail validation. List only the tools the skill actually uses. Common values: `Read`, `Edit`, `Write`, `MultiEdit`, `Grep`, `Glob`, `Bash`.

### metadata.json

Must satisfy `schemas/skill.schema.json`. Required fields: `id`, `name`, `version`, `type` (must be `"skill"`), `provider`, `harnesses`, `summary` (20+ chars), `source_type`, `official_docs` (array of URIs), `security_notes` (20+ chars), `last_verified` (YYYY-MM-DD), `path`.

Valid `source_type` values: `original`, `adapted`, `reference-only`.

Valid `harnesses` values: `codex`, `copilot`, `claude-code`, `cursor`, `gemini`, `kiro`, `other`.

### Exemplar skills

Study a well-structured skill before writing your own:

- `skills/aws/aws-iac-patch-executor/` — progressive-disclosure pattern, safety checklist references, `allowed-tools` declaration

### Catalog refresh

After adding or modifying any skill, run:

```bash
npm run manifest:write
```

This regenerates `catalog/skill-manifest.json` with updated SHA hashes. Do not skip this step — `manifest:check` will fail in CI if the manifest is stale.

Also update `catalog/skills.json` to include the new skill entry. See existing entries in that file for the expected shape.

---

## Adding an Agent

### Directory layout

```
agents/<provider>/<agent-id>/
  AGENT.md              # required — canonical agent body
  metadata.json         # required — matches schemas/agent.schema.json
  harnesses/            # required — one adapter per supported harness
    claude-code.agent.md
    codex.toml
    copilot.agent.md
    cursor.agent.md
    gemini.agent.md
    kiro-ide.agent.md
    kiro-cli.agent.json
```

You do not need to supply every harness adapter on day one, but every adapter you do ship must be in the correct format for that harness. See `docs/compatibility.md` and `docs/normalized-platform-matrix.md` for adapter shapes and naming conventions.

### metadata.json

Must satisfy `schemas/agent.schema.json`. Fields are the same as skills except `type` must be `"agent"`.

The `companion_skills` field is optional but recommended. It is an explicit array of skill IDs that pair with this agent. Set it to `[]` to declare intentional no-pair rather than leaving it absent.

```json
{
  "id": "my-new-agent",
  "type": "agent",
  "companion_skills": ["my-companion-skill"]
}
```

### Harness variants

Keep portable agent logic in `AGENT.md`. Keep harness-specific behavior in the matching adapter under `harnesses/`. Do not invent metadata fields in executable adapter files unless official docs for that harness verify the field is supported. See `AGENTS.md` for the cross-harness metadata rule.

### Role assignment

If your agent belongs to one of the six defined roles, add it to `catalog/install-roles.json`:

`cloud-security-engineer`, `cloud-platform-engineer`, `cloud-dba`, `cloud-finops-analyst`, `cloud-solutions-architect`, `cloud-devops-engineer`

An agent may appear in multiple roles. See `AGENTS.md` for the role-based pattern.

### Live-guard and review agents

Any agent that produces verdicts must emit all five required evidence fields: `verdict`, `evidence_level`, `blockers`, `safe_next_actions`, `open_questions`. See `docs/evidence-output-spec.md`.

---

## Adding a Mod

Read [mods/README.md](mods/README.md) for installation and the actual per-harness
capability matrix. A mod is a tool-event guard or observer, not an agent variant.

```text
mods/<mod-id>/
  mod.toml             # source: identity, events, footprint, harness capabilities
  README.md            # behavior, reach, configuration and limitations
  patterns.json        # source, when the mod uses patterns
  lib/core.mjs         # portable production logic; no Node imports
  bin/hook.mjs         # Node command adapter
  hooks/register.ts    # native module, only when supported
```

Generated manifests, shared runtime copies, pattern modules and `catalog/mods.json`
belong to `scripts/generate-mods.mjs`; do not hand-edit them.

Before opening a PR:

- [ ] Declare the exact runtime footprint and derived reach, with namespaced environment
  variables and non-overlapping default write paths. Do not count generator output as
  runtime reach or reserve a higher level for hypothetical behavior.
- [ ] Cover each declared event and native/classic surface explicitly; document degraded
  behavior and unsupported capabilities rather than silently dropping them.
- [ ] Keep portable logic in the production core; add focused positive/negative tests
  and relevant properties against that code, not a test-only implementation.
- [ ] Document installation, extension schemas, write-path overrides, hook failure gaps
  and permission-mode behavior. Do not claim complete DLP, immutable audit evidence or
  enforcement from a hook that never executes. Never install both Claude Code adapters
  for the same mod.
- [ ] Run `npm run mods:write`, `npm run validate:mods` and `npm run test:fuzz`.
- [ ] Run native `claude plugin validate mods/<id>` and `claude plugin test mods/<id>`
  locally for each affected native mod; CI does not provide the native CLI.
- [ ] Run positive and negative probes in each affected harness, or disclose which live
  installations remain unverified.
- [ ] After all generators settle, run `npm run asset-integrity:write` last on its own,
  then `npm run validate`, spelling and Markdown lint. Include the commands and results
  in the PR; never label an unrun check as passing.

---

## Validation gates

`npm run validate` runs the following checks in sequence. All must pass before merging.

| Script | What it checks |
|--------|----------------|
| `npm run validate:catalog` (`tests/validate-catalog.py`) | Every entry in `catalog/skills.json`, `catalog/agents.json`, `catalog/rules.json`, and `catalog/mcp-references.json` references a real path and satisfies the relevant JSON Schema in `schemas/`. |
| `npm run validate:aws` (`tests/validate-aws-skill-quality.py` + `tests/validate-aws-progressive-disclosure.py`) | AWS skills meet quality gates: progressive-disclosure structure, references directory, safety-checklist reference, and output-contract section present. |
| `npm run manifest:check` (`tests/validate-skill-manifest.py`) | `catalog/skill-manifest.json` SHA hashes match current file contents. Fails if you edited skills without running `npm run manifest:write`. |
| `npm run validate:allowed-tools` (`tests/validate-skill-allowed-tools.py`) | Every `SKILL.md` that declares `allowed-tools` in frontmatter uses only recognized tool names. |
| `npm run validate:links` (`tests/validate-links.py --offline`) | Internal relative links in Markdown files resolve to real paths. Offline mode only — no HTTP calls during CI. |
| `npm run validate:mods` (`tests/validate-mods.py`) | Mod manifests, derived reach, event/surface coverage, footprints, generated adapters, syntax and production-core tests agree. |

For an additional pre-release check that validates external URLs, run `python3 tests/validate-links.py` (online mode) before tagging.

---

## Catalog refresh

The catalog is the machine-readable index that consumers use to discover and install assets. When your change affects skills, run:

```bash
npm run manifest:write
```

When your change adds, moves, or removes any cataloged asset (skill, agent, rule, MCP reference), also update the relevant JSON file in `catalog/`:

- `catalog/skills.json`
- `catalog/agents.json`
- `catalog/rules.json`
- `catalog/mcp-references.json`
- `catalog/install-roles.json` (if role membership changed)

Do not rely on validators to catch missing catalog entries after the fact — add catalog entries before opening the PR.

---

## Provenance rules

- Use `source_type: original` for assets created specifically for this repository.
- Use `source_type: adapted` when derived from another public project; cite the source and confirm license compatibility with Apache-2.0.
- Use `source_type: reference-only` for catalog entries that point to official or third-party resources without bundling their content.

---

## Pull Request expectations

- **Small and scoped.** One skill, one agent, or one coherent doc change per PR. Large batches are hard to review safely.
- **Linked to an issue.** Open an issue first for new skills and agents. PRs without an issue reference are acceptable for doc fixes and small corrections.
- **Validation evidence included.** Paste the output of `npm run validate` in the PR description. The PR template provides a slot for this.
- **No secrets or credentials.** Do not commit API keys, tokens, tenant IDs, account IDs, or customer data in any form. See `SECURITY.md` if you discover a secret already in the repo.
- **Docs updated.** If your change touches a behavior documented in `docs/`, update the relevant doc in the same PR. Do not leave docs in a state that contradicts the code.
- **Catalog updated.** If you added, moved, or removed a cataloged asset, the catalog JSON files must be updated in the same PR.

---

## Code of Conduct

A `CODE_OF_CONDUCT.md` will be added to this repository separately. Until it is present, contributors are expected to engage constructively, assume good faith, and follow the operating stance described in `CLAUDE.md` and `AGENTS.md`.

---

## Security issues

Do not report security vulnerabilities through public GitHub issues. See `SECURITY.md` for the responsible disclosure process.
