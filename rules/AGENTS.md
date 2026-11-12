# AGENTS.md

## Purpose
- Store scoped, reusable behavior rules.

## Patterns
- Harness-specific rules use `rules/<harness>/<rule-id>.md`.
- Rust language rules use `rules/rust/<rule-id>.md`, not a harness directory.
- Adjacent `<rule-id>.metadata.json` mirrors catalog fields.
- `catalog/rules.json` path must match the rule file.

## Rules
- Keep rules short, enforceable, and scoped to their language or harness.
- Harness metadata records verified integrations; a language directory does not
  declare new cross-harness compatibility.
- Do not duplicate full skill procedures here.
- Run `npm run validate` after rule metadata edits.

