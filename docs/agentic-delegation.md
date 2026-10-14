---
layout: default
title: "Agentic Delegation"
permalink: /docs/agentic-delegation/
---

# 🧭 Agentic Delegation

Most work in this repository decomposes into a large amount of cheap, parallelizable effort
plus a small amount that genuinely needs judgment. This page documents the delegation model
that split applies to, and the executable workflow that encodes it.

Two artifacts implement it:

| Artifact | Path | What it is |
|---|---|---|
| Doctrine | `.claude/skills/agentic-delegation/SKILL.md` | The rules a contributor or agent follows by hand |
| Executable form | `.claude/workflows/agentic-delegation.js` | The same rules as a runnable multi-agent workflow |

Both live under `.claude/`, which is excluded from the published site in `_config.yml` — this
page is the public description of what they do.

---

## The model

### Route by cost, not by habit

| Work | Delegate to | Why |
|---|---|---|
| Read-only reconnaissance — locating files, mapping call sites, summarizing structure | Haiku (Sonnet when interpreting the findings takes judgment) | Cheap, parallel, and mechanical. Findings are checkable because citations are mandatory |
| Reading CI and gate logs, running the gate suite | Haiku | Deterministic commands; the value is raw output, not interpretation. Haiku takes no effort level |
| Bulk prose — docs, guides, templates against an exact spec | Sonnet | The shape of the change is already decided; what remains is faithful execution |
| Code edits — scripts, tests, generators, gates, workflow code, Rust | Opus 5.5 (the orchestrator, or `opus` subagents for independent parallel edits) | A subtle mistake here costs a CI cycle; it stays on the model that holds the whole plan |
| Verifying an external claim against primary documentation | Sonnet, with Context7 | Requires judging whether a retrieved snippet actually supports a claim |

Haiku **never orchestrates**. It explores and it runs gates; it does not plan, decompose, or
accept work. Opus 5.5 is the default orchestrator and runs at its default `medium` effort,
escalating on evidence (the skill holds the ladder: a check with a clear endpoint first,
then `high`, `xhigh`, and Fable 5.1 for one task that `high` failed twice in the same way).
When Sonnet orchestrates instead, it runs at high reasoning effort at minimum — a weak plan
wastes every delegate downstream.

### What never gets delegated

- Architecture and design decisions — schema shapes, scope boundaries, precedence rules.
- Security-sensitive code — auth, secrets handling, trust-boundary logic.
- Surgical edits to load-bearing logic — validation gates, schemas, catalog generators.
- Final verification and the commit itself.

### A delegate's self-report is not verification

This is the rule that makes the rest safe. A subagent reporting success is evidence that it
believes it succeeded. Before accepting any delegated output the orchestrator reads the diff
and runs the repo's own gates — `npm run validate`, `npm run lint:spell`, `markdownlint-cli2`,
and `cargo test` when `tools/vfa-tui/` changed. A passing gate is *necessary*, not sufficient.

---

## The executable workflow

`.claude/workflows/agentic-delegation.js` is the same doctrine as a runnable workflow: the
skill states the split, the workflow enforces it. It runs seven phases, and each phase names
the model tier it runs at — the tier is the point, not an implementation detail.

| Phase | Tier | Why that tier |
|---|---|---|
| Resolve sources | Haiku | Mechanical Context7 lookup, no judgment |
| Recon | Haiku | Read-only, one narrow question per agent, citations mandatory |
| Spec | Session model, no override | Architecture never delegates downward |
| Implement | Opus (Sonnet for Markdown-only specs) | Code stays on the daily driver; a spec whose every file is Markdown is prose and goes to a Sonnet writer |
| Regenerate | Haiku | Settles generated output before anything verifies it |
| Verify | Sonnet | Adversarial, so it cannot be the cheapest tier |
| Gate | Haiku | Runs commands and reports raw output verbatim |

Two design choices carry most of the weight.

**Context7 IDs resolve once, centrally.** Resolution is capped per question, so a fleet of
delegates each resolving the same library independently spends its budget learning the same
identifier repeatedly. The first phase resolves once and injects the IDs downstream, turning a
per-agent cost into a fixed one. A fact that grounds in neither a resolved library nor fetched
vendor documentation is left out — the workflow fails closed.

**Verification re-reads the files.** The verify agent is a different agent, told that the
implementer's report is a claim rather than evidence, and it checks both external facts and
internal fidelity — a document describing a script is checked against the script. Its verdicts
are `CONFIRMED` / `CONTRADICTED` / `UNVERIFIABLE`; a claim true in spirit but wrong in detail is
`CONTRADICTED`, which is where most real defects live.

Implementation and verification run as a pipeline rather than a barrier, so one spec can be
under verification while another is still being written. The one real barrier is between recon
and spec, and it belongs there: the spec phase splits file scope across delegates and cannot do
that from a partial map.

Full phase-by-phase description, including the input contract:
[`docs/agentic-delegation-workflow.md`](https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/blob/master/docs/agentic-delegation-workflow.md).

The TUI lists these workflows too. `catalog/workflows.json` is generated from each script's
`meta` block by `scripts/generate-workflow-catalog.mjs` and validated by
`npm run validate:workflow-catalog`; `vfa-tui` reads that generated file, exactly like every
other catalog surface, and never parses JavaScript itself.

---

## Worked example

The Snowflake board's verification pass used the model directly: three parallel Haiku sweeps
(volatile-claim inventory, cross-reference resolution, reference-quality audit) alongside a
Context7 verification of seven encoded vendor claims.

Six claims verified `AGREE` with quoted evidence. One returned a real defect — the encoded
statement that no Snowpipe Streaming retirement date is published was accurate but omitted a
documented *"Expected timeline and migration window"* section stating that a formal
announcement carrying the final end-of-life date was planned, with an 18-month sunset period
following it. Accurate, and materially under-informative: a planner reading the original text
would conclude the migration could not be scoped, when in fact an 18-month clock starts on
announcement.

That is the class of defect the adversarial phase exists to find, and it is exactly the class
a confirmation-seeking review misses.

---

## Cross-references

- [Architecture](architecture/) — the three-layer system the delegated work produces
- [Testing](testing/) — the validation gates the gate phase runs
- [Contributing](contributing/) — conventions a writing delegate is told to mirror
- [Execution Tiers](execution-tiers/) — the trust posture every generated asset must declare
