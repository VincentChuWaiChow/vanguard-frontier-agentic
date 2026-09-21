# Vanguard Frontier Agentic

This repository is a curated marketplace for **cloud**, **zero-trust**, and **compliance-aware** AI workflows. Two things live here: the catalog itself (agents, skills, rules, MCP references, machine-readable indexes) and the tooling that keeps it honest (validation gates, generators, and the `vfa-tui` Rust terminal UI). Almost every mistake in this repo is caught by a gate — so the working loop is: make the change, run the gates, fix what they say, and never bypass one.

## Quick start

```bash
npm install                # one-time install
pip install jsonschema     # one-time; validate:promotion-gatekeeper imports it and fails without it
# ...make catalog/doc changes...
npm run validate           # 20+ gates (every validate:* script in package.json — catalog, schema, asset integrity, model policy, routing, marketplace)
npm run lint:spell         # codespell — separate CI gate, NOT part of validate
npx --yes markdownlint-cli2 "**/*.md" "#node_modules"   # markdown lint — separate CI gate
```

If you touched `tools/vfa-tui`, also run its gates (CI's `Gate` job runs all three and fails the PR independently):

```bash
cd tools/vfa-tui
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

If you touched the catalog (agents/skills/roles/providers), run `npm run manifest:write:all` then `npm run asset-integrity:write` **last, on its own** (see the ordering caveat at the bottom) before `npm run validate`.

## What this repo contains

- `skills/` — reusable workflows for recurring engineering tasks
- `agents/` — expert roles with judgment for review, architecture, and operations (each with per-harness variant files under `harnesses/`)
- `rules/` — harness-specific operating guidance
- `mcp/` — MCP references and trust-boundary notes
- `catalog/` — machine-readable indexes (agents, skills, model policy/registry/assignments, asset-integrity manifest)
- `schemas/` — metadata contracts
- `docs/` — governance, taxonomy, compatibility, and quality guidance (Jekyll site)
- `scripts/` — generators and the model-policy engine; `tests/` — the validation gates behind `npm run validate`
- `tools/vfa-tui/` — Rust terminal UI over the catalog (see its section below)
- `plugins/`, `powers/`, `templates/` — harness plugin bundles, Kiro Powers, and contribution templates (generated or scaffolding; regenerate, don't hand-edit generated output)
- `.claude/skills/` — project skills that codify how to work here (`agentic-delegation`, `model-registry-refresh`)

## Operating stance

- Prefer **official docs** and **live evidence** over memory.
- Default to **least privilege**, **zero trust**, and **safe rollback paths**.
- Separate **verified facts**, **judgment**, **assumptions**, and **unknowns**.
- Treat broad permissions, destructive automation, and MCP mutation paths as high risk.
- Do not add secrets, credentials, tokens, tenant IDs, or customer data.

## How to work here

- **Delegate by default.** Follow `.claude/skills/agentic-delegation/SKILL.md`: Haiku subagents for read-only exploration/research (require file:line or URL citations), Sonnet subagents for bulk writing against an exact file-scoped spec, and the orchestrator keeps architecture decisions, security-sensitive edits, verification, and the commit. A delegate's self-report is not verification — read the diff and run the gates yourself.
- **Run it as a pipeline when the work is big.** `.claude/workflows/agentic-delegation.js` is that skill as an executable `Workflow` — Haiku recon, a Context7 verification phase, Sonnet writers against file-scoped specs, an independent refuter per change, then the gate suite. Invoke as `Workflow({name: "agentic-delegation", args: {task, questions, libraries, specs}})`; see `.claude/workflows/README.md`. It never commits — the orchestrator still reads the diff and commits.
- **Ground version-sensitive API claims in Context7, not in service docs.** Provider documentation describes features; library documentation pins call signatures. Two wrong MLflow API claims reached this repo from correct-looking Databricks doc pages and were caught only by Context7. Treat absence from a Context7 result as *uncorroborated*, never as disproven — it serves retrieved snippets, not a complete API inventory, and "correcting" documented content on absence alone introduces the bug it was meant to prevent.
- **Orchestrator requirements.** Haiku must never be the orchestrator. When Sonnet orchestrates, run it at high reasoning effort at minimum (use the harness's maximum-thinking mode if available) — planning and delegation quality degrade below that, and a weak plan wastes every delegate downstream.
- **Verify external claims against primary sources** before encoding them (model names, retirement dates, API capabilities). Press coverage and launch blogs get details wrong; official docs pages are the bar. Anything unverifiable stays out — fail closed.
- **Commit and push as you go.** Sessions are ephemeral; work that isn't pushed to the branch does not exist. Commit messages are conventional commits with a scope (`feat(model-policy): …`, `fix(exporter): …`, `chore(codespell): …`) because semantic-release derives versions from them.
- **Never bypass a gate to go green.** Fix the cause, or extend the gate's config deliberately with a comment explaining why (e.g. a real API name added to `.codespellrc` `ignore-words-list`).

### Quality bar (any orchestrator model)

These are the behaviors that make the difference between passable and trustworthy work here. They are not optional at any model tier:

- **Probes over trust.** After any generated or delegated change, run one decisive positive probe (the thing now works) AND one negative probe (the invalid input now fails with the right message) — green gates alone don't prove semantics. Dry-run first on every mutating script.
- **Primary source or it doesn't exist.** Before encoding any external fact (model names, retirement dates, API capabilities), fetch the provider's official page yourself. Press, launch blogs, and even a delegate's "verified" label are leads, not evidence. If you can't verify it, leave it out and say so.
- **Correct the record.** When new evidence contradicts something you previously reported, say so explicitly and prominently — never let a superseded claim stand quietly.
- **Lead with the outcome.** First sentence of any report: what happened / what you found. Detail after. If tests fail, say so with the output; if a step was skipped, say that.
- **Deterministic over clever.** Generators and gates must never consult the wall clock or any other ambient state — behavior changes only on committed data, so builds are reproducible and CI can't change color without a commit.
- **Finish means pushed.** Follow the Definition of done below, in order, every time. A dirty tree or an unpushed branch is unfinished work, not a detail.

## When working in this repo

- Keep changes scoped and traceable to the task.
- Update catalog metadata when adding, moving, or removing cataloged assets.
- Run `npm run validate` before finishing. It runs 20+ gates (every `validate:*` script in `package.json`) — catalog integrity, schema compliance, asset integrity, model policy, maestro routing, and multi-harness marketplace consistency. Keep the count approximate (10+/20+), never exact — exact counts drift.
- If `skills/**` changed intentionally, also refresh `catalog/skill-manifest.json` with `npm run manifest:write`.
- Every `SKILL.md` must declare an `allowed-tools` field (least-privilege baseline) and conform to `schemas/skill.frontmatter.schema.json`.
- For agents that have a 1:1 companion skill, declare it explicitly via `companion_skills: [<skill-id>]` in the agent's `metadata.json`.

## Definition of done

Work is finished only when ALL of these hold — in this order, since the integrity manifest must hash the settled tree:

1. Generated files regenerated (`manifest:write:all` / `docs-data:write` / `model-policy:apply` as applicable to what you touched).
2. `npm run asset-integrity:write` run **last, on its own** if anything it hashes changed (agents/, plugins/, root files, package.json — see the canonical section below).
3. `npm run validate` — zero failures.
4. `npm run lint:spell` and `npx --yes markdownlint-cli2 "**/*.md" "#node_modules"` — zero failures.
5. `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` in `tools/vfa-tui` — required whenever `tools/vfa-tui/**` changed.
6. `git status` clean; changes committed with a conventional-commit message and pushed to the working branch.

A change that passes locally but was never pushed, or that leaves the tree dirty, is not done.

## Documentation & Version Sync (DRY)

**Never hardcode counts, versions, or provider/role lists in documentation.**

After any catalog change (agents, skills, roles, providers):
```bash
npm run readme-counts:write      # Update README inline count markers
npm run board-counts:write       # Update generated count markers in docs/ + index.md + tests/fixtures/README.md
npm run docs-data:write          # Update Jekyll docs/_data/catalog.yml
npm run plugin-manifest:write    # Sync .claude-plugin version + agents
npm run cursor-plugin:write      # Sync .cursor-plugin version + agents
npm run kiro-powers:write        # Regenerate Kiro Powers for all providers
python3 tests/validate-asset-integrity.py --write  # Refresh SHA256 hashes
```

Or all-in-one: `npm run manifest:write:all`

**Version parity:** `package.json`, `.claude-plugin/plugin.json`, `.cursor-plugin/plugin.json` must always show the same version. The generators read from `package.json` automatically.

**Jekyll docs:** All pages in `docs/` use `{{ site.data.catalog.X }}` Liquid variables sourced from `docs/_data/catalog.yml`. Never hardcode agent/skill/provider counts in markdown.

**Hand-written provider lists are NOT auto-generated.** A few docs enumerate providers *by name* in prose/bullets/tables and must be updated by hand when a provider is added or removed: `docs/taxonomy.md` (provider bullet list), `docs/language-stack-boards.md` (board enumeration + prose), and the Powers table in `docs/integrations/installation-guide.md`. **Counts are the exception and are never hand-written:** catalog numbers in `docs/language-stack-boards.md`, `docs/configuration.md`, `docs/marketplace-model.md`, `tests/fixtures/README.md`, and root `index.md` sit in marker spans generated by `npm run board-counts:write` and enforced by `validate:board-counts` — `<!-- count:board:<provider>:<key> -->N<!-- /count -->` for per-board figures (providers joinable with `+`), `<!-- count:global:<key> -->N<!-- /count -->` for repo-wide ones (agents, skills, providers, roles, gates, maestros, rules, mcp). Edit the catalog, run the generator — never retype a number. README.md has its own `count:<key>` namespace owned by `readme-counts:write`. Changelogs, ADRs, and dated plan/research documents are deliberately excluded: they record what was true when written, so regenerating their numbers would falsify the record. **Provider invariant:** `set(provider bullets in docs/taxonomy.md) == provider_list in docs/_data/catalog.yml == {distinct providers that have at least one agent}`. Skill-only providers (no agents) are not "boards" and must not be listed/counted — fix the miscategorization at the source (the skill's `provider` field) rather than inflating the metric.

**Releases:** semantic-release owns versioning. `feat:` → minor, `fix:` → patch. Never manually edit `"version"` in package.json.

## Adding a new provider

A `provider` value is hardcoded in several places that are **not** auto-derived from the catalog. When introducing a new provider (e.g. `sap`), update ALL of these or validation/CI will fail:

1. `schemas/agent.schema.json` — add to the `provider` enum.
2. `schemas/skill.schema.json` — add to the `provider` enum.
3. `tests/validate-catalog.py` — add to the `ALLOWED_PROVIDERS` set (a separate hardcoded list from the schemas — easy to miss; the `validate:catalog` gate fails without it).
4. `tools/vfa-tui/src/models/provider.rs` — add the variant to the Rust `Provider` enum (kebab-case serde). This is a **hard, load-bearing** requirement, not cosmetic: the TUI deserializes `catalog/agents.json`/`skills.json` with this strict enum, so a missing variant makes the catalog fail to load and breaks `cargo test` (the `Gate` job). CI's `Gate` is path-filtered to `tools/vfa-tui/**`, so a catalog-only PR will pass CI while leaving the TUI broken against the new provider — run `cargo test` in `tools/vfa-tui` after adding any provider.
5. `scripts/generate-docs-data.mjs` — add the provider to the correct category in the `taxonomy` array (drives `provider_taxonomy` in `catalog.yml`).
6. `scripts/generate-kiro-powers.mjs` — add a `PROVIDERS` entry **only if** the provider should ship a Kiro Power (optional; not every provider has one — e.g. netsuite/finance do not).
7. `docs/taxonomy.md` and `docs/language-stack-boards.md` — add the provider to the hand-written lists (see the provider invariant above). If the new provider gets its own board section, write its counts as `count:board:<provider>:<key>` marker spans rather than literals; `validate:board-counts` fails closed on an unknown provider or key, so a typo is caught rather than silently frozen.
8. Regenerate derived files: `npm run manifest:write:all` then `npm run docs-data:write`, then asset-integrity last (see the ordering caveat below).

## Marketplaces & export

This repo ships as an npm package AND as four harness marketplaces; each manifest is generated, validated, and must never be hand-edited:

- Claude Code plugin: `.claude-plugin/plugin.json` (`npm run plugin-manifest:write`; install: `/plugin marketplace add VincentChuWaiChow/vanguard-frontier-agentic`).
- Cursor plugin: `.cursor-plugin/plugin.json` (`npm run cursor-plugin:write`); Copilot CLI marketplace: `.github/plugin/marketplace.json` (repo root is the plugin root). Both validated together by `validate:multi-harness-marketplace`.
- Codex marketplace: `.agents/plugins/marketplace.json` — validated by `validate:codex-marketplace` (plugin name = folder name, kebab-case, `policy.{installation, authentication}` + `category` required, version parity with package.json).
- Kiro Powers: `powers/vanguard-*` (`npm run kiro-powers:write`). **Kiro frontmatter is strictly limited to five fields** (`name`, `displayName`, `description`, `keywords`, `author`) — any other field fails `validate:kiro-powers`.
- Export CLI: `vfa-export-agents --list-roles` to list role IDs; `vfa-export-agents --platform claude-code --all --repo <path>` installs all agents and auto-bundles companion skills.

## Adding a maestro / router agent

A `<provider>-maestro-agent` requires a routing fixture at `tests/fixtures/<provider>-maestro-routing/` containing `taxonomy.json` + `inputs/NN-name.json` + `expected/NN-name.json`. Every agent referenced must exist in `catalog/agents.json`. Generate the `expected/` files from the grader (`tests/validate-maestro-routing.py` → `evaluate(task, taxonomy)`) so they stay consistent, and list guarded-mutating-live agents under `live_guards` so they are never auto-dispatched (they only appear in `live-guard-gate` mode). The `validate:maestro-routing` gate enforces all of this.

## Model policy

Per-harness model/reasoning-effort assignment is policy-driven, not hand-edited. `catalog/model-policy.json` (schema: `schemas/model-policy.schema.json`) is the canonical source; `scripts/model-policy.mjs` resolves it into `catalog/model-assignments.json` and projects it into harness files (`codex.toml` `model`/`model_reasoning_effort`/`model_provider`; `claude-code` `.agent.md` `model:`/`effort:` frontmatter; `cursor` `.agent.md` `model:`). Rules scope to `all` | `provider:<id>` | `role:<id>` | `agent:<id>` per harness; precedence is agent > role > provider > all; `auto` clears the field.

- `npm run model-policy:report` / `model-policy:check` / `model-policy:apply` — inspect, validate, and project the policy (`check` also runs inside `npm run validate` as `validate:model-policy`, right after `validate:agent-schema`).
- After a non-dry-run apply, run `npm run asset-integrity:write` (the `vfa-tui` Model Policy Builder chains this automatically).
- Never hand-edit `model` / `model_reasoning_effort` / `model_provider` / `effort` lines in harness files directly — edit `catalog/model-policy.json` and run `model-policy:apply`.
- `catalog/model-registry.json` (schema: `schemas/model-registry.schema.json`) is the verified per-harness model-name and reasoning-effort matrix; `model-policy:check` fails closed on any model or effort value not registered there. Extend it via the `.claude/skills/model-registry-refresh/SKILL.md` workflow — never by guessing model names from memory. Human-readable companion: `docs/model-policy-matrix.md`.
- Codex `model_provider` is **derived** from the model's registry namespace (`gpt-*`/o-series → default; `name:tag` → `ollama`; `author/model` → `openrouter`) — it is never set by hand and never a policy field.
- **Model lifecycle:** registry entries may carry `status` (`available`|`retiring`|`retired`), `retirement_date`, and `successor`. `retiring` warns everywhere (CLI, assignments index, TUI) but projects unchanged; `retired` projects the documented successor with a persistent warning until the policy is migrated; `retired` without a successor is a hard error. Behavior flips **only on a committed status change** — the engine never consults the wall clock, so builds stay reproducible. Encode lifecycle data only from the provider's official deprecations page, never from press coverage.

## tools/vfa-tui (Rust TUI)

- **Read-first principle:** the TUI never duplicates business logic. It reads generated catalog files for display and shells out to the Node/Python scripts for every mutation (dry-run first, then real run). Rust-side validation is injection-safety only (`security/validate.rs`); semantics live in the scripts. If you find yourself re-implementing script logic in Rust, stop.
- **Toolchain:** `Cargo.toml` pins `rust-version = "1.97"` — the real dependency floor (`libsqlite3-sys` uses `cfg_select`). If the build fails on unstable-feature errors, run `rustup update stable` rather than downgrading dependencies, and bump the pin if a dependency raises the floor again.
- **Serde contracts are strict:** catalog-facing structs use `deny_unknown_fields`. Any new key emitted into a generated JSON (e.g. `catalog/model-assignments.json`) requires the matching struct field (use `#[serde(default)]` for optional additions) plus updates to every struct-literal in tests/fixtures — the compiler will point at them.
- **Gates:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (unit + integration + property tests). CI's `Gate` job runs these; a clippy warning is a failure.
- **Local artifact note:** `tools/vfa-tui/target/` is skipped in `.codespellrc` because generated build artifacts trip false positives after a local build; CI checks out fresh and never sees them.

## CI gates beyond `npm run validate`

`npm run validate` does **not** run spell-check, markdown lint, or the Rust gates — those are separate CI jobs that fail a PR independently. Before pushing, also run:

```bash
npm run lint:spell    # codespell. For false positives on real API names/acronyms
                      # (e.g. afterAll, AGS), add the lowercase term to
                      # ignore-words-list in .codespellrc — do NOT reword valid code.
npx --yes markdownlint-cli2 "**/*.md" "#node_modules"   # CI lints every markdown file
```

…and the cargo gates listed above when `tools/vfa-tui/**` changed (CI job: `Gate`).

## Cross-platform asset rule

This repo supports multiple harnesses without pretending they are identical.

- Keep portable logic in canonical specs and shared docs.
- Keep harness-specific behavior in the right adapter format.
- Do not invent unsupported metadata fields in executable agent files. Extend `HARNESS_CAPABILITIES` in `scripts/model-policy.mjs` only with officially documented harness support, verified against current docs (capabilities change — claude-code's `effort:` frontmatter field was added upstream after this repo's first policy engine landed).

## Important files

- `README.md` — human-facing vision and repository story
- `ROADMAP.md` — standing PRD + executable roadmap (the canonical work queue; keep it matching reality)
- `AGENTS.md` / `GEMINI.md` — pure pointers to this file (CLAUDE.md is canonical); they must never carry independent rules
- `CONTRIBUTING.md` — contributor onboarding and submission path
- `SECURITY.md` — vulnerability disclosure policy and SLA
- `CODE_OF_CONDUCT.md` — community standards
- `docs/compatibility.md` — harness support contract
- `docs/normalized-platform-matrix.md` — naming and platform normalization
- `docs/integrations/skills-cli.md` — install-path trust matrix
- `docs/model-policy.md` / `docs/model-policy-matrix.md` — model policy operator guide and verified capability matrix
- `schemas/skill.frontmatter.schema.json` — required SKILL.md frontmatter contract
- `schemas/agent.schema.json` — agent metadata contract (includes `companion_skills`)

## Asset integrity (canonical reference)

`catalog/asset-integrity.json` holds SHA256 hashes of all tracked assets. If any file in `agents/`, `plugins/`, `.github/plugin/`, `package.json`, or a root file changes and the manifest is not refreshed, the `validate:asset-integrity` gate fails and blocks release automation. Regenerate it whenever you add/move/remove agents, plugins, or skills; edit `.github/workflows/release.yml`; or change any root-level file (README.md, AGENTS.md, CLAUDE.md, package.json, …):

```bash
python3 tests/validate-asset-integrity.py --write   # or: npm run asset-integrity:write
git add catalog/asset-integrity.json
```

**Ordering caveat:** `npm run manifest:write:all` runs its generators in parallel (`&` … `wait`), so `asset-integrity:write` can hash the tree *before* the other generators (README counts, plugin manifests, Kiro powers) finish writing files it covers. After `manifest:write:all`, always run `npm run asset-integrity:write` once more **on its own, last**, so it hashes the settled tree. Always run `npm run validate` before finishing to catch staleness early.
