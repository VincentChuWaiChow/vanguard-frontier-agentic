# EVAL DEFINITION: agentic-delegation — Opus 5.5 effort ladder and model routing

Target: `.claude/skills/agentic-delegation/SKILL.md`
Baseline: the skill as of `ac6c8032` (Haiku explores, Sonnet writes bulk and code)
Defined: 2026-09-25, before the skill was edited (eval-driven development)
Method: 3 prompts x 2 configurations (new skill, baseline) x 3 trials. Each trial
writes a delegation plan as `plan.md` plus a machine-readable `plan.json`, and
graders score the JSON with code wherever the property is structural.

## Verified facts the evals encode

Every expectation below traces to a primary source read on 2026-09-25. Nothing
from the Opus 5.5 guidance infographic is encoded unless a docs page backs it.

| Fact | Source |
| --- | --- |
| Opus 5.5 supports `low`/`medium`/`high`/`xhigh`/`max`; its default is `medium` (every other effort-capable model defaults to `high`, Opus 4.7 to `xhigh`) | <https://code.claude.com/docs/en/model-config>, <https://platform.claude.com/docs/en/build-with-claude/effort> |
| `max` applies to the current session only unless set via `CLAUDE_CODE_EFFORT_LEVEL`; it "is prone to overthinking. Test before adopting broadly" | <https://code.claude.com/docs/en/model-config> |
| Models absent from the effort table do not support effort; Haiku 4.5 is absent | <https://code.claude.com/docs/en/model-config>; `catalog/model-registry.json` (`haiku` alias carries empty `reasoning_efforts`) |
| The effort scale is calibrated per model, so the same level name differs across models | <https://code.claude.com/docs/en/model-config> |
| Changing effort invalidates the messages cache; caches are model-scoped | <https://platform.claude.com/docs/en/build-with-claude/prompt-caching> |
| Built-in Explore inherits the main conversation's model since v2.1.198 (no longer always Haiku); the per-invocation `model` parameter wins over every other source | <https://code.claude.com/docs/en/sub-agents> |
| Subagent `effort` defaults to the session's level; available levels depend on the model | <https://code.claude.com/docs/en/sub-agents> |
| Plan mode: Claude reads files and proposes a plan but makes no edits until approved | <https://code.claude.com/docs/en/common-workflows> |
| Model IDs `claude-opus-5-5`, `claude-fable-5-1`, `claude-sonnet-5`, `claude-haiku-4-5` | `catalog/model-registry.json` (verified 2026-09-05 to 2026-09-23) |

Guidance taken from the infographic as policy (judgment, not a docs fact):
search and log-reading delegates on Sonnet or Haiku, code edits on Opus 5.5;
start at `medium`, try `high` when medium stalls, `xhigh` when high cannot get
there, and switch to Fable 5.1 for a specific task when high hits the same
problem twice, then switch back.

## Prompts

1. **multi-file-gate** — add a new validation gate, wire it into
   `npm run validate`, and document it. Multi-file code plus docs.
2. **stuck-escalation** — a vfa-tui clippy failure that two fixes at `high`
   effort have failed the same way. What next?
3. **ci-logs-fix** — three red CI jobs: read the logs, root-cause, fix, push.

## Capability evals (expected to flip from the baseline)

- C1 (1, 3): every `code-edit` step runs on `opus-5.5`. Grader: code.
- C2 (1, 3): every `search`/`log-reading` step delegated to a subagent runs on
  `haiku-4.5` or `sonnet-5`. Grader: code.
- C3 (1, 3): every subagent step names its model explicitly (never `inherit`).
  Grader: code.
- C4 (1, 2, 3): no `haiku-4.5` step carries an effort level. Grader: code.
- C5 (1, 3): the plan's default effort is `medium`. Grader: code.
- C6 (2): the escalation path switches the stuck task to `fable-5.1`. Grader: code.
- C7 (2): the plan returns to the daily-driver model once the stuck task is
  solved. Grader: model.
- C8 (2): the plan changes effort or model at a break and says why (cache
  cost). Grader: model.
- C9 (1): the multi-file change is planned in plan mode. Grader: code.

## Regression evals (the baseline already does these; they must not break)

- R1 (1, 3): every delegated `search`/`log-reading` step must return citations.
  Grader: code.
- R2 (all): no subagent step may commit, and a commit step, when present, runs
  on the orchestrator. Grader: code.
- R3 (all): at least one orchestrator-run `verification` step names a concrete
  check with an endpoint (a command or gate). Grader: code.
- R4 (all): `default_effort` is never `max`. Grader: code.

## Success metrics

- Capability evals: pass@3 >= 0.90 per assertion for the new skill.
- Regression evals: pass^3 = 1.00 for the new skill.
- Discrimination: at least half of the capability assertions fail on the
  baseline in at least one trial; otherwise they are not measuring the change.

## Known threats to validity

- Every trial runs with the repository's `CLAUDE.md` in context. At eval time
  its delegation line still describes the baseline split (Sonnet writers), so
  the new skill is tested against contradicting ambient guidance. That biases
  toward the baseline, making new-skill passes conservative.
- The `plan.json` schema names the model and effort vocabulary for both
  configurations, which can inflate baseline scores. The delta, not the
  absolute baseline score, is the signal.
- Trials run on the same model that authored the skill.
