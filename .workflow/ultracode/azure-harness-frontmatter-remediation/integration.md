# Integration ledger

## Accepted

- **Give the 16 agents an identity in all five markdown exports.** 80 files. Each
  loses its copied `metadata:` block and gains `name` and `description` derived
  by the rule in result 02. Bodies are byte-identical before and after. The
  transformation ran as a script that refuses rather than guesses: it stops on
  any value needing escaping, any file that already has an identity, and any
  file whose first `# ` heading disagrees with `metadata.json` `name`. None
  tripped.
- **Gate harness identity in `validate:agent-schema`.** Every markdown harness
  export must declare a non-empty `name` and `description`, and claude-code
  names must be unique. Extending the existing gate keeps the gate count
  unchanged. `codex.toml` and `kiro-cli.agent.json` are out of its scope.

## Rejected or deferred

- **Adding `name` and `description` to the 16 `AGENT.md` files.** Withdrawn:
  all 735 `AGENT.md` files carry only `metadata`.
- **Adding Copilot `tools`, Cursor `model`/`readonly`, Gemini `kind`.** Not
  added: per-agent capability and tool-grant declarations with no source for
  these agents. Inventing tool grants would be a security decision, not a
  repair.
- **Gemini name format, repository-wide.** Gemini's local subagent
  documentation requires `name` to be a slug. Every Gemini export in the
  repository uses a display name such as `"Azure Entra ID Specialist"`, so all
  of them, not only these 16, may be rejected by Gemini CLI. The 16 follow the
  repository convention so that one future fix covers all of them uniformly.
  Not changed here: it is pre-existing, affects every agent, needs a naming
  decision (for example the agent id), and could not be probed because Gemini
  CLI is not installed in this environment.
- **Cursor and Kiro contracts.** Their documentation for custom agent files was
  not found; the repository convention is followed.

## Verification

- Negative probe: the extended gate on the unfixed tree fails on exactly the 80
  files — 16 agents, all Azure — with no other error across 3,669 exports.
- Positive probe: the gate passes on the fixed tree.
- Negative probe: a planted duplicate claude-code name fails, naming both files.
- Runtime probe: a real Claude Code install of the fixed tree loads 735 agents
  with 0 duplicate-name warnings. Before the fix, one duplicate warning hid 15
  dropped agents; a controlled probe showed the warning is logged once per
  name, not once per dropped agent.
- `model-policy:check` stays in sync.
