# Result 02 — repository writers and checks

Partly accepted. The packet found 4 of the 10 generators that write
`gemini.agent.md`; the parent found the rest by searching `scripts/` for the
file name.

- Ten generators write the file, all with `name` taken from the agent's display
  name. Eight share `md_harness()`: `gen_python_agents`, `gen_typescript_agents`,
  `gen_kotlin_agents`, `gen_databricks_agents`, `gen_terraform_agents`,
  `gen_snowflake_agents`, `gen_python_live_agents` and `gen_netsuite_agents`.
  `gen_azure_live_guards` and `gen_oci_live_guards` write the frontmatter
  inline. The packet listed only `gen_python_agents`, `gen_terraform_agents`,
  `gen_snowflake_agents` and `gen_azure_live_guards`.
- `scripts/model-policy.mjs` names the file in `HARNESS_CAPABILITIES` (line 108)
  with `model: false` and `reasoning_effort: false`, so it never edits a Gemini
  export.
- `tests/validate-catalog.py` and `tests/validate-azure-oci-live-guards.sh`
  check only that the file exists. `tests/validate-agent-frontmatter-schema.py`
  required a non-empty `name` with no format rule.
- The export CLI (`scripts/export-marketplace-agents.mjs`) writes
  `.gemini/agents/<id>.md`. It inserts a `# VFA-EXPORT:` YAML comment after the
  opening `---` and leaves `name` unchanged. The runtime probe confirms Gemini
  CLI accepts that comment.
- `tools/vfa-tui` never parses `name`. It detects a Gemini layout by the `.md`
  extension (`src/workspace/harness_layout.rs:130`). It matches an installed
  agent by file name, the `VFA-EXPORT` marker, and a line-overlap signature
  with a 40% threshold (`src/federation/scanner.rs:245-265`). The rename changes
  one frontmatter line and adds one, so it cannot break a match. The parent
  checked this; the packet had cited only the layout check.
