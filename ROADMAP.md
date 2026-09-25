# ROADMAP — standing PRD and executable roadmap

This is the canonical work queue for this repository. It is written so a capable model
(Opus or Sonnet) can execute it task by task without the author present. Read
[`CLAUDE.md`](./CLAUDE.md) in full before touching anything — it is the canonical operating
guide; this file only says *what* to build, never *how to work*.

**Maintenance rule:** this is a standing document. When a milestone completes, mark it done,
refine the next milestone's outline into tasks using the same task format, and update the
success-criteria checkboxes. Never let it describe a state that no longer exists.

## 1. OBJECTIVE

Vanguard Frontier Agentic is a curated marketplace of AI skills, agents, and rules for
cloud, zero-trust, and compliance-aware engineering, consumable from Claude Code, Codex,
Cursor, Copilot, Gemini, and Kiro.

The user is an engineer (any seniority) who installs these assets into their harness and
gets expert, evidence-grounded behavior without writing prompts themselves. The outcome we
sell is **trust**: every asset is verified against primary sources, least-privilege by
default, and guarded by machine gates — not a prompt dump.

The next phase (this roadmap) advances all four growth axes in this order: verification
depth → tooling/automation → provider breadth → distribution. Depth comes first because
every later axis multiplies whatever quality level exists when it lands.

## 2. CONTEXT

What already exists and works (do not rebuild, do not break):

- **Catalog**: several hundred skills and agents across dozens of providers; machine indexes
  under `catalog/`; JSON-schema contracts under `schemas/`. **Never restate the counts here** —
  `docs/_data/catalog.yml` is generated from the catalog and is the only place they are current
  (`npm run docs-data:write`). The board roster is enumerated in `docs/language-stack-boards.md`.
- **Gate suite**: 20+ deterministic gates behind `npm run validate` (see `package.json`
  `validate:*`), plus codespell, markdownlint, and the Rust `Gate` job (fmt/clippy/test in
  `tools/vfa-tui`) as separate CI jobs.
- **Model policy system**: `catalog/model-policy.json` → `scripts/model-policy.mjs` →
  harness files + `catalog/model-assignments.json`, validated fail-closed against
  `catalog/model-registry.json` (verified model/reasoning matrix with lifecycle statuses).
- **Asset integrity**: `catalog/asset-integrity.json` SHA256-hashes agents/, plugins/,
  root files, package.json; must be regenerated last, from the repo root.
- **Project skills** (`.claude/skills/`): `agentic-delegation`, `model-registry-refresh`,
  `pr-babysit`, `definition-of-done`. Use them; they encode how work ships here.
- **Marketplaces**: Claude Code plugin, Cursor plugin, Copilot marketplace, Codex
  marketplace, Kiro Powers — all generated, all validated (see CLAUDE.md "Marketplaces &
  export").
- **Docs site**: Jekyll under `docs/`, data-driven from `docs/_data/catalog.yml`.

Hard invariants the executor must not break: every rule in CLAUDE.md, the provider
invariant, the generated-files-are-never-hand-edited rule, version parity, and the
deterministic-gates rule (see Constraints).

## 3. SUCCESS CRITERIA

The roadmap phase is done when every box below can be checked by running the named command
or inspecting the named artifact:

- [x] `validate:skill-coherence` gate exists, runs inside `npm run validate`, and passes:
      every fenced shell command in every `SKILL.md` is covered by that skill's
      `allowed-tools` declaration. *(Shipped: Task 1.1, 2026-07-10.)*
- [ ] `validate:live-guard-coverage` gate exists, runs inside `npm run validate`, and
      passes: every agent with `execution_tier: mutating-runtime` is listed in
      `live_guards` of its provider's maestro routing fixture (or the gate's documented
      exemption list, each entry with a reason).
- [ ] A scheduled GitHub Actions workflow audits provenance freshness (`last_verified`
      ages) and maintains exactly one rolling issue with the oldest assets — no gate in
      `npm run validate` consults the wall clock.
- [ ] A scheduled GitHub Actions workflow flags model-registry staleness
      (`last_refreshed` > 90 days) via the same single-rolling-issue pattern.
- [ ] `.claude/skills/provenance-refresh/SKILL.md` exists and has been exercised once:
      the oldest provider from the first freshness report re-verified with citations,
      `last_verified` bumped, gates green.
- [ ] Milestones 2–4 are refined from outlines into numbered tasks using the Milestone 1
      task format (done at each milestone boundary, not in advance).
- [ ] Every milestone shipped as one reviewed PR, merged by the author; `npm run validate`,
      codespell, markdownlint, and the Rust Gate green on every PR head.

## 4. CONSTRAINTS

- **Deterministic validate, scheduled freshness.** Gates inside `npm run validate` must
  never consult the wall clock or network — they compare committed data only. Anything
  time- or web-dependent (freshness ages, link liveness, staleness nags) lives in
  scheduled CI workflows that produce reports/issues, never CI failures on unrelated PRs.
- **Fail closed on external facts.** Model names, API capabilities, retirement dates:
  primary sources only, encoded with `last_verified` + source, per the quality bar in
  CLAUDE.md. Unverifiable claims stay out.
- **No mass rewrites.** The existing 619 skills are audited-clean (2026-07-10). Do not
  reword, reformat, or "improve" them in bulk; touch a marketplace skill only when a gate
  or a verified fact demands it.
- **No new providers until Milestone 3.** Breadth waits for the depth gates.
- **Generated files are never hand-edited**; policy/registry changes go through their
  scripts and skills.
- **Shipping model: one PR per milestone.** Executor branches `claude/m<N>-<slug>` from
  `master`, works task by task with conventional commits, keeps the PR green via the
  `pr-babysit` skill, and stops for author review at the milestone boundary. Direct pushes
  to `master` are out of scope. semantic-release owns versions (`feat:` minor, `fix:`
  patch) — never edit versions by hand.
- **Out of scope for this roadmap**: rewriting the Jekyll theme; TUI rewrites (only the
  listed TUI features); any LLM-graded eval infrastructure (deterministic structural
  checks only — LLM-as-judge is a future decision for the author); changing the license
  or contribution model.

## 5. MILESTONES

1. **M1 — Verification depth** (fully specified below): mechanical trust gates + freshness
   automation over the existing catalog. Goal: every asset's least-privilege and
   provenance claims are machine-checked, and staleness surfaces itself.
2. **M2 — Tooling & automation** (outline): make the maintenance loops cheaper. Candidates:
   TUI catalog-health view (surface the freshness report + registry warnings already in
   `catalog/model-assignments.json`); auto-PR for asset-integrity refresh after merges;
   TUI policy-diff preview (dry-run rendering); scheduled registry refresh dispatcher that
   opens a pre-filled issue for the `model-registry-refresh` skill.
3. **M3 — Provider & domain breadth** (outline): new providers and deeper coverage, riding
   the M1 gates. Candidates: shortlist from author (entry criterion: author names 2–4
   providers); each addition follows CLAUDE.md "Adding a new provider" end-to-end incl.
   maestro fixture; parity pass so every provider with agents has ≥1 skill and vice versa
   where sensible.
4. **M4 — Distribution & adoption** (outline): make the verified catalog findable and
   installable. Candidates: docs-site search; per-harness quickstart funnels; release
   notes automation from conventional commits; marketplace listing copy refresh sourced
   from catalog data (never hardcoded counts).

## 6. TASK BREAKDOWN — Milestone 1

Work the tasks in order; each is finishable in one session by a capable model. Every task
ends by running the `definition-of-done` skill (it knows the gate order) and committing
with the stated conventional-commit type. All tasks land on branch `claude/m1-verification-depth`.

**Task 1.1 — Skill/allowed-tools coherence gate.**
Build `tests/validate-skill-coherence.py`: for every `skills/**/SKILL.md`, extract fenced
`bash`/`sh` code blocks, map each command's leading token(s) to the skill's
`allowed-tools` entries (`Bash(npm run x:*)`-style patterns included), and fail listing
every uncovered command with file:line. Add `"validate:skill-coherence"` to `package.json`
scripts and append it to the `validate` chain. Expect real findings: fix them by
correcting the skill's `allowed-tools` (least-privilege: add the narrowest matching
pattern), never by deleting the command. If findings exceed ~30 files, land the gate with
an explicit `EXEMPT` list in the test (each entry commented with a reason) and burn the
list down in follow-up commits within this task.
*Touches*: `tests/validate-skill-coherence.py` (new), `package.json`, affected
`skills/**/SKILL.md` frontmatter, `catalog/skill-manifest.json` (via `manifest:write`).
(CLAUDE.md needs no edit — it deliberately avoids per-gate enumeration; counts stay
approximate.)
*Done when*: `npm run validate` green including the new gate; exemption list empty or each
entry reasoned; conventional commit `feat(gates): ...`.

**Task 1.2 — Live-guard coverage gate.**
Build `tests/validate-live-guard-coverage.py`: for every agent whose metadata declares
`execution_tier: mutating-runtime` (grep `agents/**/metadata.json`), require its id in the
`live_guards` array of its provider's `tests/fixtures/<provider>-maestro-routing/taxonomy.json`
when that fixture exists; providers without a maestro fixture are reported (not failed)
in a summary line. Wire as `validate:live-guard-coverage` into `package.json` + the
`validate` chain. Fix real gaps by adding the agent ids to `live_guards` and regenerating
`expected/` files from the grader per CLAUDE.md "Adding a maestro / router agent".
*Touches*: `tests/validate-live-guard-coverage.py` (new), `package.json`, possibly
`tests/fixtures/*-maestro-routing/taxonomy.json` + `expected/*`, `CLAUDE.md` gate list.
*Done when*: gate green in `npm run validate` with zero unexplained gaps; commit
`feat(gates): ...`.

**Task 1.3 — Scheduled provenance-freshness audit.**
Build `.github/workflows/provenance-audit.yml` (cron weekly + `workflow_dispatch`) running
a new `scripts/provenance-audit.mjs` that scans every `metadata.json` + SKILL.md
`last_verified` date, ranks the 25 oldest, and creates-or-updates ONE issue titled
"Provenance freshness report" (find by title; update body; never open duplicates). The
report groups by provider and links each asset path. No `validate:*` wiring — this is
wall-clock territory and stays out of the deterministic suite.
*Touches*: `.github/workflows/provenance-audit.yml` (new), `scripts/provenance-audit.mjs`
(new), `package.json` (script entry `provenance:audit` for local runs).
*Done when*: `npm run provenance:audit -- --dry-run` prints the report locally;
`workflow_dispatch` run produces/updates the single issue; commit `feat(ci): ...`.

**Task 1.4 — Registry staleness nag.**
Extend the same pattern: `.github/workflows/registry-staleness.yml` (cron weekly) checks
`catalog/model-registry.json` `last_refreshed`; if older than 90 days, create-or-update
one issue titled "Model registry staleness" whose body links
`.claude/skills/model-registry-refresh/SKILL.md` as the remediation workflow. Reuse the
issue-upsert helper from Task 1.3 (factor it into the script, don't duplicate it).
*Touches*: `.github/workflows/registry-staleness.yml` (new), `scripts/provenance-audit.mjs`
(shared helper), `.github/workflows/` only — note `release.yml` edits require an
asset-integrity refresh; these new workflow files do not, but run definition-of-done anyway.
*Done when*: dispatch run with a mocked >90d date produces the issue; real run is a no-op
today; commit `feat(ci): ...`.

**Task 1.5 — Provenance-refresh skill.**
Write `.claude/skills/provenance-refresh/SKILL.md` mirroring the frontmatter and voice of
`model-registry-refresh`: input = one provider (usually the oldest from the freshness
report); Step 1 Haiku researchers re-verify each asset's `official_docs` URLs and factual
claims against primary sources (citations mandatory, UNVERIFIED = blocker); Step 2
orchestrator updates `last_verified` + fixes only what evidence contradicts; Step 3 gates
via definition-of-done. Add the `.gitignore` negation line for the new skill directory.
*Touches*: `.claude/skills/provenance-refresh/SKILL.md` (new), `.gitignore`, `CLAUDE.md`
(add to the `.claude/skills` list line).
*Done when*: skill registers (appears in the harness skill list), markdownlint + codespell
green; commit `feat(skills): ...`.

**Task 1.6 — Pilot the refresh loop end-to-end.**
Run Task 1.3's audit locally, take the single oldest provider, and execute the
`provenance-refresh` skill on it for real: delegated research, `last_verified` bumps with
sources, `manifest:write` + integrity refresh, gates green. This proves the M1 loop
(detect → refresh → verify) before the milestone PR goes to review.
*Touches*: the chosen provider's `agents/**/metadata.json` / `skills/**` dates,
`catalog/skill-manifest.json`, `catalog/asset-integrity.json`.
*Done when*: freshness report no longer ranks that provider oldest; all gates green;
commit `chore(provenance): ...`; milestone PR opened with the `pr-babysit` skill armed
and author requested for review.

## 7. HANDOFF NOTES

Read in this order before your first task: `CLAUDE.md` (all of it — especially the Quality
bar, Definition of done, and Asset integrity sections), then the four project skills under
`.claude/skills/`, then this file. That is the entire cold-start surface.

- **Work style**: delegate per `agentic-delegation` (Haiku or Sonnet explore, read logs,
  and run gates; Sonnet writes prose to a locked spec; code edits stay on Opus 5.5; you
  keep architecture, verification, commits). Opus 5.5 runs at its default medium effort
  and escalates on evidence; a Sonnet orchestrator runs at high or above; Haiku never
  orchestrates.
- **Finishing**: every task ends with the `definition-of-done` skill. Its footguns section
  is earned knowledge — believe it.
- **The ordering trap**: asset-integrity refresh runs LAST, alone, from the repo root.
  Most historical CI churn in this repo came from getting this wrong.
- **Probes over trust**: after building each gate, prove it both ways — one real violation
  it catches (inject, observe failure message, revert) and the clean tree passing.
- **Warnings vs failures**: `model-policy` warnings are exit-0 by design; scheduled-audit
  findings are issues, not failures. Only the deterministic suite blocks merges.
- **When blocked**: if a task's premise no longer matches the tree (files moved, gate
  renamed), fix the premise in this file in the same commit as the work — the roadmap must
  never drift from reality. If a decision genuinely needs the author (new provider names,
  deleting anything, LLM-judged evals), stop and ask; do not improvise scope.
