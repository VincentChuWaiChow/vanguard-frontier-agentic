# Integration ledger

## Accepted

- **Naming rule: `name` is the agent id, and the readable name moves to
  `display_name`.** The agent id is the directory name. Every id is slug-valid
  and unique, and ids are already the in-repository precedent: the 19
  Salesforce exports used them. `display_name` is in Gemini CLI's schema and
  becomes the agent's `displayName`, so no readable name is lost. The rule comes
  from committed data, never from a per-agent choice.
- **Generators first.** A `gemini_harness()` helper was added next to
  `md_harness()` in the eight generators that share it. The two live-guard
  generators emit the same shape inline. `md_harness()` is unchanged, so every
  other harness keeps its display name.
- **Committed exports second.** 714 files were rewritten by a script that
  refuses anything unexpected. It sets `name` to the id and inserts the old
  value verbatim as `display_name` directly after it. It skips the 19 files
  already correct and leaves bodies and `description` untouched.
- **Gate in `validate:agent-schema`.** Each Gemini export must use its agent id
  as `name`, use only keys in Gemini CLI's local schema, and have `kind` omitted
  or `local`. Extending the existing gate keeps the gate count unchanged.

## Rejected or deferred

- **A slugified display name** (for example `azure-entra-id-specialist`).
  Rejected: it is not guaranteed unique, and it would add a second identifier
  to keep in sync with the id.
- **Changing other harnesses.** Out of scope. Their documentation allows
  display names, and a runtime probe in the previous run showed Claude Code
  loads them.
- **`gen_azure_live_guards.py` drift.** Before and after this change, re-running
  it rewrites 108 files: descriptions, bodies and skill references of six
  hand-edited live-guard agents. The drift is identical in both runs and comes
  from the committed files having been edited by hand since generation. The
  script's Gemini `name` and `display_name` lines match the committed files
  exactly. Recorded, not fixed: syncing the generator with those edits is a
  separate content decision.
- **Two agents with no Gemini export.** `nvidia-model-promotion-gatekeeper-agent`
  and `playwright-e2e-execution-run-agent` declare no Gemini harness. This
  predates the run and is recorded rather than widened into.

## Verification

- Gemini CLI runtime probe, using `@google/gemini-cli-core` 0.60.0
  `loadAgentsFromDirectory` on a real `vfa-export-agents --platform gemini --all`
  export:
  - Before (baseline `c8166696`): 19 loaded, 714 errors, every one "Name must
    be a valid slug".
  - After: 733 loaded, 0 errors, 733 unique names. `azure-maestro-agent` has
    `displayName` "Azure Maestro". The export's `# VFA-EXPORT:` comment is
    accepted.
- Gate probes:
  - Negative, on the unfixed tree: fails on exactly 714 files, all Gemini, with
    no other error across 3,669 exports.
  - Positive, on the fixed tree: passes.
  - Negative: a planted display-name `name`, a planted unknown key (`color`)
    and a planted `kind: "remote"` each fail and name the file.
- Generator re-runs: 9 of 10 produce byte-identical Gemini exports. The tenth,
  `gen_azure_live_guards.py`, has the pre-existing drift described above and no
  change to identity lines.
