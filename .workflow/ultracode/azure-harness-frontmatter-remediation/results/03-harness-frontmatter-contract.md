# Result 03 — harness frontmatter contract

Partly accepted. Two claims were wrong and were corrected by the parent.

| Harness | Source | `name` | `description` |
|---|---|---|---|
| Claude Code subagents | code.claude.com/docs/en/sub-agents | required | required |
| Claude Code plugin agents | code.claude.com/docs/en/plugins-reference | optional; missing means named after the file | optional; missing means a generic fallback |
| GitHub Copilot | docs.github.com custom-agents-configuration | optional display name | required |
| Gemini CLI local subagents | gemini-cli `docs/core/subagents.md` | required, slug only | required |
| Cursor | not documented at the URLs tried | — | — |
| Kiro IDE | not documented at the URLs tried | — | — |

Corrections:

- The packet reported that plugin agent names "must be in kebab-case with no
  spaces". Re-reading the page, that sentence applies to the plugin's own name
  in `plugin.json`, and the page says display names "may contain spaces and any
  casing". A runtime probe settled it: two agents named `"Alpha Agent"` and
  `"Beta Agent"` load as two distinct agents with no warning.
- The packet fetched Gemini's remote-agent page. The parent fetched the local
  subagent page, which requires `name` to use "only lowercase letters, numbers,
  hyphens, and underscores".
