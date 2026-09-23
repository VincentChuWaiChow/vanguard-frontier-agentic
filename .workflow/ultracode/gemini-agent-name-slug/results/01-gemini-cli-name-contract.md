# Result 01 — Gemini CLI name contract

Accepted. The parent checked it against the published package, because the
packet read line numbers from `main`, which can differ from any release.

- `name` must match `/^[a-z0-9-_]+$/`, with the message "Name must be a valid
  slug". No length limit. Source:
  `packages/core/src/agents/agentLoader.ts` on `main`. Parent check:
  `@google/gemini-cli-core` 0.60.0, `dist/src/agents/agentLoader.js:30`.
- The local agent schema is `.strict()` (0.60.0 `agentLoader.js:87`). It
  allows only `kind`, `name`, `description`, `display_name`, `tools`,
  `mcp_servers`, `model`, `temperature`, `max_turns` and `timeout_mins`.
  `description` is required and non-empty. `kind` is `local` or omitted.
- A file that fails validation is recorded in `errors` and skipped
  (0.60.0 `agentLoader.js:542-545`). The rest of the directory still loads.
- The file name does not identify the agent; the frontmatter `name` does.
  `display_name` becomes the agent's `displayName` (0.60.0
  `agentLoader.js:418`, `475`).
- Package: `@google/gemini-cli` / `@google/gemini-cli-core`, 0.60.0 at the time
  of the run.

Not relied on: the packet's notes on settings-based agent enablement and on
interactive `/agents` commands. The parent loaded agents by calling
`loadAgentsFromDirectory` directly, which needs neither an API key nor an
interactive session.
