# Executable workflows

Scripts here are run by the `Workflow` tool (multi-agent orchestration), not read
by a human. They are the executable counterpart to `.claude/skills/` — a skill
tells one agent how to behave, a workflow deterministically orchestrates many.

Do not confuse this directory with `.claude/workflow/` (singular), which holds
per-board **planning documents** (`m365-d365/`, `typescript-board/`). Those are
prose plans for humans; these are JavaScript pipelines for agents.

## `agentic-delegation.js`

Runs `.claude/skills/agentic-delegation/SKILL.md` as a pipeline, with Context7
MCP verification wired in as a first-class phase.

```text
Recon      Haiku Explore agents, one narrow question each, citations required
Context7   resolve + query per library surface, adjudicated CONFIRMED /
           CONTRADICTED / NOT_COVERED / NOT_AVAILABLE
Author     Sonnet writers against exact file-scoped specs, no commits
Verify     an independent refuter reads each authored change
Gates      the repo gate suite, raw failure output, asset-integrity last
Synthesis  an orchestrator-facing report
```

Recon and Context7 run as independent branches. Author and Verify run as a
`pipeline()`, so a spec's refuter starts as soon as that spec is written rather
than waiting for every other spec.

### Invoking it

```text
Workflow({ name: "agentic-delegation", args: { ... } })
```

With no `args` it runs a recon plus gate health check. Every field is optional:

| Field | Shape | Meaning |
|---|---|---|
| `task` | string | One line of context passed to every writer. |
| `questions` | string[] | Recon questions, one agent each. Max 5. |
| `libraries` | object[] | Context7 surfaces: `{name, query, claims[]}`. Max 5. |
| `files` | string[] | Paths the work is scoped to. Shown to recon and spec agents, and used to decide whether the cargo gates are in scope. |
| `generators` | string[] | Commands that turn generator inputs into shipped output. Run in their own phase between implementation and verification, so verification reads real output rather than stale artifacts. |
| `gates` | `false` \| string[] | `false` skips the gate phase entirely. An array replaces the default gate commands. Omitted runs the defaults. |

Anything beyond a cap is dropped **loudly** via `log()` — a truncated run must
never read as full coverage.

`specs` is **not** a caller input. Writing tasks are produced by the Spec phase,
which is orchestrator-tier by design: deciding what to change and which files it
touches is the judgment the doctrine keeps out of the delegates. Scope the run
with `files` and let the Spec phase derive the specs.

Skipping the gates does not make a run look finished. With `gates: false` the
returned `gatesGreen` is `false`, `missingGates` lists every command that was
never run, and `blocked` stays `true` — an author-only pass reports itself as
carrying no gate evidence rather than as green.

### Why Context7 is a phase and not a footnote

Service documentation describes features; library documentation pins call
signatures. This repo has already shipped two wrong MLflow API claims that
Databricks' own documentation pages did not catch — a non-existent
`evaluate(eval_data, ..., prediction_fn)` keyword pair, and built-in judges
imported from `mlflow.genai.judges` rather than `mlflow.genai.scorers`. Both
would have failed at runtime for anyone following the agent. Context7 caught
both.

The adjudication rule the phase enforces matters as much as the lookup:

- **`contradicted`** means Context7 positively shows something different. That is
  a defect, and the `actual` value is recorded.
- **`notCovered`** means Context7 returned no evidence. Context7 serves retrieved
  snippets, not a complete API inventory, so absence is **uncorroborated, never
  disproven**.

Conflating the two is not a harmless imprecision: it causes correct, documented
content to be "corrected" into incorrectness. A delegate that reports absence as
contradiction is wrong, and the schema is shaped to make that mistake hard.

If the Context7 tools cannot be reached, an agent returns `NOT_AVAILABLE` and
the affected claims stay labelled unknown. It never substitutes memory or a web
fetch for the MCP in that phase, and it never fabricates MCP evidence.

### What the workflow will not do

It never commits, never pushes, and never treats a green gate as sufficient. The
final report ends with an `orchestratorMustDo` list, and reading the actual diff
is always on it — a delegate's self-report is not verification.

## `plugin-security-scan.js`

The HOL AI Plugin Scanner loop, as a pipeline: scan, triage every finding against
the vendor's own rule definition, remediate or document each one, re-scan, then
the gate suite. It is the repeatable form of the work behind
[`.github/workflows/hol-plugin-scanner.yml`](../../.github/workflows/hol-plugin-scanner.yml)
and `.plugin-scanner.toml`.

```text
Inventory  resolve a runnable scanner + version, load the vendor rule list
Baseline   score/grade/findings per target, from real --format json output
Autofix    deterministic `lint --fix` on the fixable rules, before any judgment
Triage     orchestrator-tier disposition per finding, grounded in `lint --explain`
Remediate  Sonnet implements fixes and writes rationale-bearing suppressions
Rescan     barrier — the decisive probe that the disposition held
Refute     an independent reader attacks every suppression
Gate       the repo gate suite, asset-integrity first, raw failure output
```

Baseline and Rescan fan out per target; every other phase is a barrier, because
one rule firing on two targets is one decision, not two.

### Invoking it

```text
Workflow({ name: "plugin-security-scan", args: { ... } })
```

With no `args` it scans the Codex plugin bundle and the repo root against the
≥ 80 / no-high-or-critical bar. Every field is optional:

| Field | Shape | Meaning |
|---|---|---|
| `task` | string | One line of context passed to the triage and writing phases. |
| `targets` | (string \| object)[] | Scan targets as `{dir, config, blocking}` or a bare path. Max 6. Defaults to the two targets the CI workflow scans. |
| `scanner` | string | Scanner invocation. Defaults to `pipx run plugin-scanner`; pass a bare binary if it is on `PATH`. |
| `profile` | string | `default` \| `public-marketplace` \| `strict-security`. Empty by default, matching what CI actually passes. |
| `minScore` | number | The enforced score bar. Defaults to 80. |
| `autofix` | `false` | Skips `lint --fix`, sending every finding to triage instead. |
| `nonExecutablePaths` | string[] | Path fragments treated as suppressible. Widening this widens what may be silenced. |
| `gates` | `false` \| string[] | `false` skips the gate phase. An array replaces the default gate commands. |

As in `agentic-delegation`, anything beyond a cap is dropped **loudly** via
`log()`, a missing target is reported as not covered rather than quietly omitted,
and `gates: false` leaves `gatesGreen` false with every command listed in
`missingGates`.

### Why suppression is treated as a hostile act

A scanner score moves the same direction whether you fix the defect or tell the
scanner not to look, and the number cannot tell you which happened. That is the
whole reason this workflow exists, and it shapes every phase:

- suppressions are counted separately from fixes and never reported as
  remediation;
- the admission predicate is enforced in JavaScript, not requested in a prompt —
  a suppression is refused if it lands on an executable path, carries a blocking
  severity, or has a rationale under 40 characters;
- a refused suppression becomes an `escalate`, never a deletion, so the finding
  stays in the report instead of vanishing;
- the Refute phase reads `git diff` rather than the writer's claims, and its
  `silenced` verdict exists specifically for a suppression that covers a live
  issue or a placeholder file added only to close a "file missing" rule.

The path heuristic fails **toward** strictness: a finding with no file path at
all is treated as executable, because guessing wrong in the other direction
hides a real defect.

### The local-score trap

The action's `trust_repository_policy` input defaults to `"false"` — *"Allow
repository-owned scanner config and baselines to affect Action verdicts"* — and
`hol-plugin-scanner.yml` passes `config: .plugin-scanner.toml` without it. So a
local score bought with new suppressions is **not** evidence that CI will agree.

The workflow refuses to let a run read as finished on that basis: if every score
gain in a run came from suppression, that is appended to `blockers` in as many
words, and `readyToCommit` stays false.

### Known scanner constraints it encodes

Verified against the live CLI rather than assumed:

- `--diff-base` is **unimplemented upstream** — its guard exits with an error, so
  delegates are forbidden from passing it.
- `verify` accepts no `--config`, so verify-mode findings cannot be suppressed at
  all.
- `scan` prints its JSON document to stdout and may append a human-readable
  `Policy profile "..." failed.` line after it. That line is a *policy* verdict,
  independent of the score, and parsers must not fold it into either.
- `--online` and the `MCP_SCANNER_*` credentials stay off.
