# Model Policy

How this repository decides which model (and, for Codex, which reasoning effort) each agent runs under, per harness — and how that decision stays enforced instead of drifting.

## What it is

Model policy is a small declarative layer sitting on top of the executable agent harness files. Instead of editing `model` keys inside hundreds of `codex.toml` files or `model:` frontmatter lines inside `.agent.md` files by hand, operators declare intent once in a policy file, and a projection engine writes the resolved values into the harness files that actually execute. The same engine also produces a resolved index that read-side tooling — the `vfa-tui` terminal UI — displays without re-deriving the policy logic itself.

This mirrors the rest of the repo's DRY posture: one canonical source, generated projections, and a CI gate that fails on drift between them.

## The policy file

`catalog/model-policy.json` (schema: `schemas/model-policy.schema.json`) is the canonical source. Each rule declares a `scope`, a `harness`, and a `model` and/or `reasoning_effort` value.

- **Scopes** — `all`, `provider:<id>`, `role:<id>` (an `install-roles.json` role), or `agent:<id>`. A rule applies per harness.
- **Precedence** — `agent` > `role` > `provider` > `all`. The narrowest matching tier wins for each field independently, so a rule can pin `model` at the provider tier while leaving `reasoning_effort` to the `all` tier.
- **`auto` semantics** — setting a field to `auto` clears the managed line from the harness file entirely, so the harness runtime's own default applies. Absence of any matching rule is equivalent to `auto`.
- **Conflicts are a hard error** — if two rules in the *same* tier disagree on the same agent, harness, and field (for example two `role` rules that both match one agent but specify different models), resolution fails rather than picking a silent winner. Roles are allowed to overlap and agree; they are not allowed to overlap and disagree. Settle a conflict with an `agent:<id>` rule, which always outranks role and provider rules.

## Harness capability matrix

Not every harness exposes a model-selection surface. Rules that target an unsupported field on a harness fail validation rather than being silently dropped or invented as new metadata — this repo's cross-platform asset rule forbids adding unsupported fields to executable agent files.

| Harness | Model field | Reasoning field | Executable file |
|---|---|---|---|
| `codex` | `model` (pattern `gpt-*`) | `model_reasoning_effort` (`minimal`\|`low`\|`medium`\|`high`) | `harnesses/codex.toml` |
| `claude-code` | `model:` frontmatter (`opus`\|`sonnet`\|`haiku`\|`inherit`\|`claude-*`) | not supported | `harnesses/claude-code.agent.md` |
| `cursor` | `model:` frontmatter (permissive, including `inherit`/`auto`) | not supported | `harnesses/cursor.agent.md` |
| `copilot` | not supported | not supported | `harnesses/copilot.agent.md` |
| `gemini` | not supported | not supported | `harnesses/gemini.agent.md` |
| `kiro` | not supported | not supported | `harnesses/kiro-ide.agent.md` |

For example, the `dotnet`, `generic`, `hr`, `legal`, `netsuite`, and `salesforce` providers currently carry a provider-tier rule pinning `codex` to a newer `gpt-*` model than the `all`-tier default — a normal use of the provider scope tier, not an exception to it.

## Verified model registry

The table above says which *field* each harness exposes; it does not say which exact model names and reasoning-effort values are safe to put in that field. `catalog/model-registry.json` (schema: `schemas/model-registry.schema.json`) is the verified answer to that — a per-harness matrix of model namespaces, membership rules, and supported reasoning efforts, sourced from official documentation. `scripts/model-policy.mjs` checks every `model` and `reasoning_effort` value against it and **fails closed**: a name or effort the registry has not verified is rejected before it can be projected into a harness file, rather than surfacing later as an HTTP 404 `model_not_found` at request time.

Two projections come directly from the registry: the claude-code `effort` frontmatter field (registry `reasoning_key: effort`, vocabulary `low`/`medium`/`high`/`xhigh`/`max`), and the codex `model_provider` line, which the engine derives automatically from the model's namespace (an `openai`-shaped model projects no line; an Ollama `name:tag` or OpenRouter `author/model` value projects `model_provider = "ollama"` / `"openrouter"`).

See [`docs/model-policy-matrix.md`](./model-policy-matrix.md) for the full human-readable matrix (namespaces, verified model tables, failure modes, enforcement boundaries), and [`.claude/skills/model-registry-refresh/SKILL.md`](../.claude/skills/model-registry-refresh/SKILL.md) for how the registry gets re-verified and extended.

## CLI usage

```bash
npm run model-policy:report     # print resolved model/reasoning per agent x harness
npm run model-policy:check      # validate the policy and detect drift (also the CI gate)
npm run model-policy:apply      # project the policy into harness files + the assignments index
```

`set` and `import-current` are not wrapped as npm scripts — invoke the engine directly:

```bash
node scripts/model-policy.mjs set \
  --scope provider=frontend \
  --harness codex \
  --model gpt-6-sol \
  --dry-run
```

Drop `--dry-run` to write the rule into `catalog/model-policy.json` and project it into the affected harness files. `--scope` also accepts `all`, `role=<id>`, `agent=<id>`, or `agents=<id-a>,<id-b>` for a batch update. `import-current` bootstraps a policy from whatever values already exist in the tree — useful only when no policy file exists yet.

## The TUI flow

`vfa-tui` exposes model policy as a first-class sidebar section and a global keybinding: press `m` from anywhere to open the Model Policy Builder. Scope is prefilled from context (an agent view targets that agent, a provider view targets that provider, a role view targets that role, otherwise `all`), and the builder walks through harness, model, and reasoning-effort fields before showing the exact command it is about to run and streaming its output. The TUI does not re-implement policy resolution — it shells out to `scripts/model-policy.mjs`, the same engine the CLI and CI gate use.

## CI gate

`validate:model-policy` runs `node scripts/model-policy.mjs check` inside `npm run validate`, immediately after `validate:agent-schema`. It fails the build on:

- structural or referential errors in `catalog/model-policy.json` (unknown provider/role/agent, unsafe or malformed model values, unsupported harness/field combinations),
- unresolved conflicts (two same-tier rules disagreeing without an agent-level override), and
- **drift** — any harness file whose current `model`/`model_reasoning_effort` value does not match what the policy resolves to, or a `catalog/model-assignments.json` that is missing or stale relative to the policy.

Drift means someone edited a harness file's model line directly instead of going through the policy, or edited the policy without re-running `apply`. The fix is always to reconcile the policy and harness files with `npm run model-policy:apply`, never to hand-patch the harness file to make the gate pass.

## Integrity coupling

Applying the policy changes tracked files (`agents/**/harnesses/*.toml`, `agents/**/harnesses/*.agent.md`, `catalog/model-policy.json`, `catalog/model-assignments.json`), which means `catalog/asset-integrity.json` goes stale the moment a non-dry-run apply runs. Run `npm run asset-integrity:write` immediately afterward — the `vfa-tui` Model Policy Builder does this automatically via its **Refresh Integrity** toggle (on by default); CLI users must run it by hand before committing.
