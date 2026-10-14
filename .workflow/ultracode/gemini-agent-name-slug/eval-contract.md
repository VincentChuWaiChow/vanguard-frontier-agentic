# Evaluation contract

## Level

Targeted, with a runtime probe in Gemini CLI if it can be installed and run
without credentials.

## Acceptance criteria

- Every `gemini.agent.md` `name` matches the rule Gemini CLI enforces, as read
  from its source, and names are unique.
- The rule for choosing the value is derived from committed data and the
  in-repository precedent, never chosen per agent.
- Every script that writes `gemini.agent.md` emits the new form; re-running each
  generator produces no diff against the committed exports.
- Negative probe: the gate fails on a Gemini export whose `name` is a display
  name, naming the file.
- Positive probe: the gate passes on the fixed tree.
- `npm run validate`, spell lint and markdown lint pass.

## Rejection criteria

Leave a part unchanged, and record why in `integration.md`, if Gemini CLI's
source shows display names are in fact accepted, or if a generator cannot be
re-run here to prove it emits the new form.

## Publication gate

The user asked for this work in PR #194. Commit and push to
`claude/stoic-meitner-h2aqr8`.
