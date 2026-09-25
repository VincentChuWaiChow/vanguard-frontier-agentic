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

## Results (2026-09-25)

Two iterations, each 3 prompts x 2 configurations x 3 trials, all trials on the same
model. Iteration 2 is the authoritative run. Iteration 1 leaked two answers to the
baseline. The prompt said "running on Claude Opus 5.5 at its default effort", which gave
away `medium`, and the `plan.json` schema listed `fable-5.1` as a model value. Iteration 2
removed both hints (the model field became free text, normalized by the grader), added
R5, and tested the skill after the gate-order fix below.

```text
EVAL REPORT: agentic-delegation (iteration 2)
=============================================
Capability evals (new skill; pass@3 / pass^3 per task, trial pass rate)
  C1 code edits on opus-5.5          PASS 2/2 / 2/2   6/6   (baseline 5/6)
  C2 reading delegated to haiku/son  PASS 2/2 / 2/2   6/6   (baseline 6/6)
  C3 model named on every subagent   PASS 2/2 / 2/2   6/6   (baseline 6/6)
  C4 no effort on haiku              PASS 3/3 / 3/3   9/9   (baseline 9/9)
  C5 default effort medium           PASS 2/2 / 2/2   6/6   (baseline 0/6)
  C6 stuck task escalates to Fable   PASS 1/1 / 1/1   3/3   (baseline 0/3)
  C7 returns to Opus 5.5 at medium   PASS 1/1 / 1/1   3/3   (baseline 0/3)
  C8 switch at a break, cache reason PASS 1/1 / 0/1   2/3   (baseline 0/3)
  C9 plan mode for multi-file work   PASS 1/1 / 1/1   3/3   (baseline 3/3)
Regression evals (pass^3)
  R1 R2 R3 R4 R5                     1.00 each        (baseline 1.00 each)
Metrics
  Trial pass rate: new 98% +/- 5%, baseline 78% +/- 16%, delta +0.21
  Time per trial: new 295 s, baseline 285 s; tokens ~104k both
  Discrimination: 5 of 9 capability assertions fail on the baseline at least once
  (target: at least half)
Status: capability pass@3 >= 0.90 on all nine; regression pass^3 = 1.00
```

Iteration 1 (leaky prompts): new 100% +/- 0%, baseline 84% +/- 16%, delta +0.16. The two
leaks show in the baseline's scores: C5 went from 4/6 in iteration 1 to 0/6 in
iteration 2, and C6 from 3/3 to 0/3.

What the evidence supports:

- The skill changes four behaviors reliably: start at `medium`, not `high`; escalate a
  repeated identical failure to Fable 5.1, not up the effort ladder to `max`; come back to
  Opus 5.5 at `medium` afterwards; and keep code edits off Sonnet.
- C8 is the soft spot. In one of three trials the plan put the model switch at a break
  but justified it by per-model effort calibration, not by cache cost.
- C2, C3, C4, C9 and R1 to R5 do not discriminate: the baseline passes them too. They are
  kept as regression guards, not counted as evidence that the skill helps.
- The trials found a real defect in both versions. The Gate-run template refreshed asset
  integrity after `npm run validate`, but `validate` checks integrity itself. Every trial
  that noticed followed `CLAUDE.md`'s order instead, which is why R5 scored 1.00 even
  before the fix. The skill now matches `CLAUDE.md`.

Grader corrections made during the run, each with a negative probe: R5 originally could
not order two commands inside one step. It now compares (step, offset). The Sonnet grader
for C7/C8 was blind to the configuration, and its quotes were spot-checked by grep; one
absence claim ("cache never appears") was wrong in wording but right in verdict (the only
hit was a build cache).

Remaining threats to validity: skills authored and trialled on the same model; the new
skill's description was visible in every trial's available-skills list; in iteration 2
`git log` showed the commit that introduced the new doctrine. The last two bias the
baseline toward the new behavior, so the measured delta is conservative.

Artifacts (session scratchpad, not committed): `evals.json`, per-trial `plan.md` /
`plan.json` / `grading.json` / `timing.json`, `benchmark.json` per iteration, and the
static review viewer.
