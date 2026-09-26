---
name: agentic-delegation
description: "Plan who does each step of a task in this repo, on which model, at which effort: Haiku or Sonnet subagents for search, recon, and log-reading; code edits on Opus 5.5; start at medium effort and escalate on evidence (high, then xhigh, then Fable 5.1 for one stuck task); the orchestrator keeps design, security-sensitive edits, verification, and the commit. Use at the start of any task that spans more than one file or needs research or CI-log triage, whenever a fix keeps failing at the current effort, and whenever you are about to spawn a subagent or pick a model or effort level — even if nobody says 'delegate'."
allowed-tools: ["Agent", "TaskCreate", "TaskUpdate"]
---

# Agentic Delegation

The orchestrator's context window is the scarcest resource in a session. Every file dump,
log tail, and grep result read directly is context that can no longer hold the plan, the
spec, or the diff under review. Delegation exists to keep that window for judgment.

Opus 5.5 is the daily driver: it is the orchestrator, and it makes the code edits. Reading
work goes to cheaper models.

## Route each step by the kind of work

| Work | Who does it | Model | Effort |
| --- | --- | --- | --- |
| Search, recon, mapping call sites, "where does X live" | subagent (`Explore`) | `haiku`; `sonnet` when interpreting what it finds takes judgment | none on Haiku |
| Reading CI or gate logs, running the gate suite | subagent | `haiku` or `sonnet` | none on Haiku |
| Bulk prose: docs pages, guides, templates, against an exact spec | subagent | `sonnet` | session level |
| Code edits: scripts, tests, generators, gates, workflow code, Rust | orchestrator, or `opus` subagents for independent parallel edits | Opus 5.5 | session level |
| Architecture, schema shapes, security-sensitive and load-bearing edits | orchestrator only | Opus 5.5 | session level |
| Verification of delegate output, and the commit | orchestrator only | Opus 5.5 | session level |
| One task that `high` has failed twice in the same way | orchestrator, after switching model | Fable 5.1 | then switch back |

Search and log-reading are summarization jobs: a cheaper model reads a lot and returns a
little, and that is exactly the trade you want. Code edits are the opposite. They are where a
subtle mistake costs a CI cycle and a reviewer's trust, and a spec rarely captures every
invariant the surrounding code depends on. Keep them on the model that holds the whole plan.

Docs prose is the judgment call in this table. It is not code, so a Sonnet writer with an
exact file-scoped spec is still the default. Anything that encodes facts (model names,
command flags, API shapes) gets verified by the orchestrator before acceptance, whoever wrote it.

## Name the model on every delegate call

Pass `model` explicitly each time you spawn a subagent. Since Claude Code v2.1.198 the
built-in `Explore` agent inherits the main conversation's model instead of always running on
Haiku, so an `Explore` call without `model: "haiku"` runs the reconnaissance on Opus 5.5. That
defeats the point of delegating it. The per-invocation `model` parameter wins over the
subagent's frontmatter, over `CLAUDE_CODE_SUBAGENT_MODEL`, and over the session model. An
`opus` alias from an Opus 5.5 session resolves to Opus 5.5 itself, which is what you want for
parallel code-edit delegates.

## Effort: start at medium, escalate on evidence

Opus 5.5 defaults to `medium`, and well-scoped daily work belongs there. Raise effort only
when the work shows it needs more, and in this order:

1. **Give the model a way to check its work first.** A test, gate, or script with a clear
   endpoint (`npm run validate`, `cargo test`, a one-line negative probe) often catches at
   `medium` what you would otherwise need `high` to find. Add the check before touching the
   effort dial.
2. **`medium` stalls → `high`.** More thinking per turn catches what `medium` misses.
3. **`high` still cannot get there → `xhigh`.** Use this when each attempt makes progress but
   falls short.
4. **`high` hits the same problem twice → Fable 5.1 for that task.** An identical repeated
   failure means more thinking on the same model is not the fix. Switch that specific task to
   Fable 5.1, solve it, then switch back to Opus 5.5 at `medium`. Do not leave the session
   running on the escalated setting.
5. **`max` is for one hard task, never a standing default.** Claude Code applies `max` to the
   current session only unless it is forced through `CLAUDE_CODE_EFFORT_LEVEL`. The docs
   warn it can show diminishing returns and is prone to overthinking. Turn it on for the task,
   then lower it again.

Change effort or model at a break, such as after a commit or between tasks. Changing effort
invalidates the messages cache, and caches are model-scoped, so the next turn after a switch
pays full price for the whole conversation. Switching in the middle of a debugging loop pays
that cost at the worst time.

Effort for delegates:

- **Haiku 4.5 does not support effort.** Do not assign a Haiku delegate an effort level; it
  means nothing there.
- **Effort is calibrated per model.** `high` on Sonnet is not `high` on Opus. Choose the
  delegate's model first, then its effort.
- **Delegates inherit the session's effort** unless their subagent definition sets `effort:`
  or the Workflow `agent()` call passes one. A recon or gate-run delegate rarely needs more
  than the session level.

## Keep the session lean

- Use plan mode (`/plan`, or Shift+Tab) for changes that span multiple files. The plan is
  reviewed before any edit touches disk, which is cheaper than unwinding a wrong edit.
- `/clear` between unrelated tasks, so the old task's context does not ride along.
- `/compact` at a natural break, with a note on what to keep, for example
  `/compact keep the spec, the failing test names, and the open review threads`.
- When a cost or model choice is in doubt, run one real task on each option and compare
  what `/usage` reports. Your own numbers beat anyone's benchmark, including this skill's
  table.

## What the orchestrator keeps

These stay with the orchestrator because each needs the whole picture, and a delegate sees
only its slice:

- Architecture and design decisions: schema shapes, scope boundaries, precedence rules.
- Security-sensitive code: auth, secrets handling, trust-boundary logic.
- Surgical edits to load-bearing logic: validation gates, schemas, catalog generators.
- Final verification and the commit itself.

Haiku never orchestrates; it explores and runs gates. If Sonnet orchestrates instead of Opus
5.5, run it at `high` effort at minimum. Planning quality degrades below that, and a weak
plan wastes every delegate downstream.

## Every delegate gets

A delegate knows only what its prompt says. Hand over:

- **The model**, named explicitly (see above).
- **Exact file paths**, absolute rather than "somewhere in docs/".
- **Acceptance criteria**, stated concretely enough to check.
- **Citations as the price of a finding.** Recon and log-reading delegates return
  `file:line` or a URL for every claim. A report without them is not actionable; re-run it
  with a tighter prompt rather than accepting it.
- **A "do NOT" list**: files not to touch and commands not to run. By default that means no
  `npm run validate`, no `cargo test`, and never `git commit`. Delegates write files; only
  the orchestrator commits.

## Verify before accepting

A delegate's self-report is a lead, not evidence. Read the diff in full, then run the gates
relevant to the touched files (`npm run validate`, `cargo test` for `tools/vfa-tui`,
`npx markdownlint-cli2`, `npm run lint:spell`). Treat a green gate as necessary, not sufficient. Also
run one positive probe (the thing now works) and one negative probe (bad input now fails with
the right message).

## Workflow templates

Three shapes cover most multi-step tasks here. Reach for one before inventing a bespoke plan.
For large work, `.claude/workflows/agentic-delegation.js` runs the same doctrine as an
executable `Workflow` (see `.claude/workflows/README.md`).

### Recon sweep

Parallel `Explore` agents on `model: "haiku"`, one narrow question each, one area of the tree
each, all launched in the same message so they run concurrently. Every finding carries a
`file:line` citation. Use this when you do not yet know where something lives. It is
read-only: no edits, no commits. If a sweep comes back thin or off-target, tighten the prompt
and re-run it.

### Spec-driven change

The orchestrator writes the spec first: exact paths, the shape of the change, the conventions
to mirror, and checkable acceptance criteria.

- **Code:** the orchestrator implements on Opus 5.5. When several edits are independent,
  split them across `opus` subagents, one spec each, with disjoint file lists.
- **Prose:** hand the spec verbatim to a `sonnet` subagent. A one-line summary of a spec
  produces a one-line-quality result.

Either way, the orchestrator reads the whole diff before running any gate, then verifies as
above. Delegates touch exactly the files their spec lists and never commit.

### Gate run

A `haiku` subagent, with no effort level, runs the suite and reports pass or fail with raw
failure output verbatim, not a paraphrase like "some tests failed". The orchestrator needs the
actual error to decide the next move.

Order matters because `npm run validate` includes the asset-integrity check. The orchestrator
runs the generators its change needs first. The gate run then refreshes the manifest with
`npm run asset-integrity:write`, on its own, as the last write. Only then does it run the
checks: `npm run validate`, `npm run lint:spell`, and `npx markdownlint-cli2`, plus
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` when
`tools/vfa-tui` changed. Run it the other way round and `validate` fails on the stale
manifest before the refresh ever happens. The only file a gate run may write is
`catalog/asset-integrity.json`, and it never commits.
