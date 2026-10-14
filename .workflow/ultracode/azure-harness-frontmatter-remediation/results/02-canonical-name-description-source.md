# Result 02 — canonical name and description source

Accepted, then re-measured by the parent across every healthy Azure agent.

- Rule: `name` is `metadata.json` `name`, which equals the first `# ` heading of
  the body; `description` is `metadata.json` `summary` with any leading
  `Agent for <id>. ` removed.
- The rule holds for 15 of the 20 healthy Azure agents. The other five carry
  hand-edited descriptions, so the rule is the convention, not a law, and was
  applied only where nothing else exists.
- All 16 affected agents have a `name` and a `summary`, none carries the prefix,
  and no value contains a double quote, backslash or newline.
- Every healthy Azure export in all five formats carries `name` and
  `description`. Extra keys vary by agent (Copilot `tools`, Cursor `model` and
  `readonly`, Gemini `kind`) and were not added: they are per-agent capability
  declarations with no source for these 16, and none of the 16 has an
  execution tier that `validate:agent-tool-tiers` would police.
- All 735 `AGENT.md` files carry only `metadata`. The earlier plan to add the
  values to the 16 `AGENT.md` files was withdrawn.
