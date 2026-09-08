# Model Policy Matrix

The verified model-name and reasoning-effort capability matrix behind `catalog/model-registry.json` (schema: `schemas/model-registry.schema.json`) — every model and effort value `scripts/model-policy.mjs` will accept when projecting `catalog/model-policy.json` into harness files.

## Why this exists

A model policy that pins a name the provider does not recognize does not fail at edit time — it fails at request time, on every single agent invocation that resolves to it. OpenAI returns HTTP 404 with `error.code: "model_not_found"` and `error.type: "invalid_request_error"`; OpenRouter returns HTTP 404 on an unrecognized slug. Those are paid-for failures: the harness has already spent a turn constructing the request before the provider rejects it, and in a repo with hundreds of agents across three harnesses, a single mistyped or invented slug in a `provider:` or `all` scope rule can break every agent that inherits it.

`catalog/model-registry.json` is the fail-closed guard against that. `scripts/model-policy.mjs` validates every `model` and `reasoning_effort` value in `catalog/model-policy.json` against the registry before projecting anything into a harness file — a policy can never write a value the registry has not verified against official documentation. This page is the human-readable companion to that machine-readable registry: read it to understand what is currently verified, how the namespace/membership model works, and how the engine turns a registry-valid model into the exact harness config line it projects.

## Registry structure at a glance

- **`manifest_version`**, **`last_refreshed`** — registry-wide staleness marker. See [`.claude/skills/model-registry-refresh/SKILL.md`](../.claude/skills/model-registry-refresh/SKILL.md) for the re-verification cadence.
- **`harnesses.<codex|claude-code|cursor>`** — one block per harness, each with `reasoning_key` (the config field name), `reasoning_efforts` (complete harness vocabulary), and an ordered `namespaces` array.
- **Namespaces** are evaluated in array order; the first whose `match` regex accepts a model value classifies it.
  - **`closed` membership** — the value must appear in that namespace's `models` array, or the check fails. Used where the provider's catalog is small, stable, and worth enumerating exactly (OpenAI, Anthropic, Cursor named models).
  - **`open` membership** — only the `match` shape is validated; the value is not checked against an enumerated list. Used where the catalog is too large or too volatile to enumerate (Ollama's local model library, OpenRouter's aggregated catalog).

## codex

### Field mapping

| Field | Config key | Vocabulary |
|---|---|---|
| Model | `model` in `codex.toml` | namespace-dependent (see below) |
| Reasoning effort | `model_reasoning_effort` in `codex.toml` | `none` \| `minimal` \| `low` \| `medium` \| `high` \| `xhigh` \| `max` (harness-wide vocabulary; narrowed per model/namespace — `max` is advertised only by the gpt-6 and gpt-5.6 families) |
| Provider route | `model_provider` in `codex.toml` | derived automatically from the model's namespace — never set by hand |

Codex validates effort at runtime against each model's advertised `supportedReasoningEfforts` (`model/list`); the registry narrows the harness-wide vocabulary per model family so the policy engine catches an unsupported pairing before it reaches the provider.

### Namespace table

| Namespace | Shape / pattern | Membership | `model_provider` projected | Reasoning support | Examples |
|---|---|---|---|---|---|
| `openai` | `^(gpt-\|o[0-9])[a-z0-9.-]*$` | closed (enumerated below) | *(none — default provider)* | Per-model, see table below | `gpt-5.5`, `o3` |
| `ollama` | `^[a-z0-9][a-z0-9._-]*:[a-z0-9][a-z0-9._-]*$` (explicit `name:tag`; a bare floating-`:latest` name is rejected by shape) | open (shape only) | `ollama` | None — pinning `reasoning_effort` fails check. Ollama documents the field on `/v1/chat/completions`, but this route is `wire_api responses` and `reasoning_effort` is absent from the fields Ollama lists for `/v1/responses` | `deepseek-r1:14b`, `qwen3:32b`, `glm-5.3:cloud`, `gpt-oss:120b`, `llama3.3:70b` |
| `openrouter` | `^[a-z0-9][a-z0-9.-]*/[a-z0-9][a-z0-9._-]*(:(free\|extended\|nitro\|thinking))?$` (`author/model`, optional variant suffix) | open (shape only) | `openrouter` | None — pinning `reasoning_effort` fails check. OpenRouter documents the field on its chat-completions surface; its Responses route, which this namespace uses, is unverified | `anthropic/claude-sonnet-4.5`, `openai/gpt-4o`, `google/gemini-2.5-pro` |

Namespaces are matched in this order, so a value must clear the `openai` pattern before falling through to `ollama` or `openrouter`.

The `ollama` and `openrouter` routes require a matching `[model_providers.<id>]` table in the operator's `codex.toml` (`requires_provider_table: true` in the registry) — the policy engine projects the `model_provider` line, but it does not create or validate that table's `base_url`/`wire_api`/`env_key` contents. See [Enforcement boundaries](#enforcement-boundaries).

### Verified OpenAI models (`openai` namespace)

| Model | Supported reasoning efforts | Notes |
|---|---|---|
| `gpt-6-astra` | low, medium, high, xhigh, max | flagship end-to-end coding model; does not advertise none |
| `gpt-5.6` | none, low, medium, high, xhigh, max | alias for gpt-5.6-sol; no minimal |
| `gpt-5.6-sol` | none, low, medium, high, xhigh, max | no minimal |
| `gpt-5.6-terra` | none, low, medium, high, xhigh, max | no minimal |
| `gpt-5.6-luna` | none, low, medium, high, xhigh, max | no minimal |
| `gpt-5.5` | none, minimal, low, medium, high, xhigh | |
| `gpt-5.5-pro` | none, minimal, low, medium, high, xhigh | |
| `gpt-5.4` | none, minimal, low, medium, high, xhigh | |
| `gpt-5.4-mini` | none, minimal, low, medium, high, xhigh | |
| `gpt-5.4-nano` | none, minimal, low, medium, high, xhigh | |
| `gpt-5.3-codex` | none, minimal, low, medium, high, xhigh | |
| `gpt-5.1-codex-mini` | none, minimal, low, medium, high, xhigh | |
| `gpt-4.1-mini` | *(none)* | non-reasoning text model |
| `gpt-4.1-nano` | *(none)* | non-reasoning text model |
| `o1` | low, medium, high | o-series predates none/minimal/xhigh |
| `o3` | low, medium, high | o-series predates none/minimal/xhigh |
| `o4-mini` | low, medium, high | o-series predates none/minimal/xhigh |
| `gpt-5-2025-08-07` | minimal, low, medium, high | retiring 2026-12-11 → gpt-5.5 |
| `gpt-5-mini-2025-08-07` | minimal, low, medium, high | retiring 2026-12-11 → gpt-5.4-mini |
| `gpt-5-nano-2025-08-07` | minimal, low, medium, high | retiring 2026-12-11 → gpt-5.4-nano |

The GPT-6 and GPT-5.6 families advertise a `max` reasoning effort on the OpenAI models API, so `max` is now part of the codex harness vocabulary and appears in the rows for those models. Earlier revisions of this table withheld `max` pending confirmation that the Codex CLI's `ReasoningEffort` enum accepted it; the enum in `codex-rs/protocol/src/openai_models.rs` carries `None`, `Minimal`, `Low`, `Medium`, `High`, `XHigh`, `Max`, `Ultra`, `Persistent` and a `Custom(String)` fallback, and `config.schema.json` types the key as a free-form non-empty string, so that exclusion is lifted. Note the consequence: the harness itself will not reject an effort value, which is precisely why the per-model narrowing in this table is what turns an unsupported pairing into a check-time failure rather than a provider-side one. `gpt-6-astra` is the exception within that group: it does not advertise `none`, so that value is omitted from its row.

`gpt-5` and `gpt-5-codex` are **not valid slugs** — do not add them from memory; they do not exist in the current OpenAI model map. Image, audio, and video model slugs are deliberately excluded from this registry — they are not valid agent models for this repo's harness files.

Sources: Context7 `/openai/codex` `references/latest-model.md` (current model map); [developers.openai.com/codex](https://developers.openai.com/codex); [developers.openai.com/api/docs/models](https://developers.openai.com/api/docs/models) (GPT-6 + GPT-5.6 families, verified 2026-09-05).

## Model lifecycle (retirement and fallback)

Registry model entries carry three optional lifecycle fields: `status` (`available` | `retiring` | `retired`), `retirement_date`, and `successor`. Lifecycle behavior is driven **only by the committed `status` field, never the wall clock** — `scripts/model-policy.mjs` never consults `Date.now()`, so a build run today produces the same result as the same build run a year from now until someone actually commits the `retiring` → `retired` flip (via the `model-registry-refresh` workflow). This keeps builds reproducible: passing a `retirement_date` does not itself change behavior.

- **`available`** — the default; the model projects normally with no warning.
- **`retiring`** — projection is unchanged (the pinned model still projects as-is), but `check`, `apply`, `report`, and `set` print an aggregated warning naming the documented successor (`WARNING: ... [affects N assignment(s)]`), and every affected assignment in `catalog/model-assignments.json` carries a `model_warning` field. The exit code is unaffected — this is a heads-up, not a failure.
- **`retired`** — projection automatically falls back to the documented `successor` (successor chains are followed through further `retired` links and are validated to terminate, with no cycles); the assignment records `model_fallback_from` so the substitution is traceable, and the warning persists until the policy rule is migrated to pin the successor directly. A `retired` model with no documented `successor` is a hard validation error — there is no silent limp-mode fallback.

| `status` | Projection | Warning | Assignment field |
|---|---|---|---|
| `available` | pinned model, unchanged | none | — |
| `retiring` | pinned model, unchanged | yes (aggregated CLI warning) | `model_warning` |
| `retired` | documented successor (chain-followed) | yes, until migrated | `model_fallback_from` + `model_warning` |
| `retired`, no successor | *(rejected)* | hard error | — |

Warnings surface in three places: the `model-policy.mjs` CLI (`check`/`apply`/`report`/`set`) prints one aggregated `WARNING: ...` line per distinct message with an affected-assignment count; the `model_warning` (and, for `retired`, `model_fallback_from`) field is written into every affected entry of `catalog/model-assignments.json`; and the `vfa-tui` agent detail view renders a styled `warning` row directly under the affected harness row.

The `vfa-tui` Model Policy Builder reads `catalog/model-registry.json` directly to populate its model picker and to narrow the reasoning-effort cycle per (harness, model) — previously the builder used a free-text model field and a hardcoded effort union that was gated to the codex harness only. It resolves a `retired` entry's efforts through the `successor` chain so the picker narrows against the model the engine will actually project, and always offers `auto` where the harness has a reasoning field, since clearing an inherited effort is the documented remedy when a rule moves onto a model that supports none.

## claude-code

### Field mapping

| Field | Config key | Vocabulary |
|---|---|---|
| Model | `model:` in subagent frontmatter | `sonnet` \| `opus` \| `haiku` \| `fable` \| `inherit`, or a pinned `claude-*` ID. The alias namespace is enumerated rather than shape-only, so `haiku` carries the same empty `reasoning_efforts` as the pinned Claude Haiku 4.5 ids instead of inheriting the full vocabulary |
| Reasoning effort | `effort:` in subagent frontmatter | `low` \| `medium` \| `high` \| `xhigh` \| `max` (not supported for Claude Haiku 4.5 — see below) |

### Namespace table

| Namespace | Shape / pattern | Membership | Examples |
|---|---|---|---|
| `alias` | `^(sonnet\|opus\|haiku\|fable\|inherit)$` | open (version-floating aliases Claude Code resolves to a maintained default; always valid) | `sonnet`, `opus` |
| `anthropic` | `^claude-[a-z0-9.-]+$` | closed (enumerated below) | `claude-sonnet-5` |

### Verified Anthropic model IDs (`anthropic` namespace)

| Model ID | Supported reasoning efforts | Note |
|---|---|---|
| `claude-fable-5-1` | low, medium, high, xhigh, max | current Fable lineup; default effort high |
| `claude-opus-5` | low, medium, high, xhigh, max | current Opus lineup; default effort high |
| `claude-opus-4-8` | low, medium, high, xhigh, max | |
| `claude-opus-4-7` | low, medium, high, xhigh, max | |
| `claude-opus-4-6` | low, medium, high, xhigh, max | |
| `claude-opus-4-5-20251101` | low, medium, high, xhigh, max | dated snapshot ID |
| `claude-sonnet-5` | low, medium, high, xhigh, max | |
| `claude-sonnet-4-6` | low, medium, high, xhigh, max | |
| `claude-sonnet-4-5-20250929` | low, medium, high, xhigh, max | dated snapshot ID |
| `claude-haiku-4-5` | *(none)* | effort not supported |
| `claude-haiku-4-5-20251001` | *(none)* | dated snapshot ID; effort not supported |
| `claude-fable-5` | low, medium, high, xhigh, max | |

### Effort vocabulary and fallback

`effort` is a subagent frontmatter field (registry `reasoning_key: effort`) with harness vocabulary `low`, `medium`, `high`, `xhigh`, `max`. [code.claude.com/docs/en/sub-agents](https://code.claude.com/docs/en/sub-agents) states available levels depend on the model, and the [models overview](https://platform.claude.com/docs/en/about-claude/models/overview) lists Claude Haiku 4.5 as not supporting effort at all — so `claude-haiku-4-5`, `claude-haiku-4-5-20251001` and the floating `haiku` alias all carry an empty `reasoning_efforts` list in the registry and fail closed on any `effort` value. `auto` remains valid for them — it clears the managed field, which is how a rule moves onto such a model. Every other model in the `anthropic` namespace accepts the full harness vocabulary; Claude Code falls back gracefully to the highest supported level when a specific level isn't available.

An invalid `model` value is not caught at startup — it surfaces as an HTTP 404 at request time, which is exactly the class of failure this registry exists to prevent before it reaches the provider.

Sources: [code.claude.com/docs/en/sub-agents](https://code.claude.com/docs/en/sub-agents), [code.claude.com/docs/en/model-config](https://code.claude.com/docs/en/model-config), [platform.claude.com/docs/en/about-claude/models/overview](https://platform.claude.com/docs/en/about-claude/models/overview), [platform.claude.com/docs/en/about-claude/model-deprecations](https://platform.claude.com/docs/en/about-claude/model-deprecations).

## cursor

### Field mapping

| Field | Config key | Vocabulary |
|---|---|---|
| Model | `model:` in subagent frontmatter | `inherit` \| `auto`, or a named model |
| Reasoning effort | *(not projected)* | Cursor has no frontmatter effort field — per-model parameters live in Cursor's own SDK model discovery (`Cursor.models.list()`), not in agent files |

### Namespace table

| Namespace | Shape / pattern | Membership | Examples |
|---|---|---|---|
| `alias` | `^(auto\|inherit)$` | open (`auto` = Cursor picks; `inherit` = parent agent's model; always valid) | `auto`, `inherit` |
| `named` | `^[a-z0-9][a-z0-9.-]*$` | closed (enumerated below) | `gpt-5.5`, `composer-2.5` |

### Verified named models (`named` namespace)

| Model | Note |
|---|---|
| `gpt-5.6-sol` | |
| `gpt-5.5` | |
| `gpt-5` | |
| `claude-opus-5` | |
| `composer-2` | |
| `composer-2.5` | |

Cursor's model picker evolves quickly and availability is plan/admin-dependent; this list is deliberately narrow and should be extended only through the refresh workflow, not from memory. An unknown model name raises `ConfigurationError` in Cursor.

Source: [cursor.com/docs/subagents](https://cursor.com/docs/subagents).

## Failure modes

| Harness / provider | Bad-model error observed | Reasoning-unsupported behavior |
|---|---|---|
| codex → OpenAI | HTTP 404, `error.code: "model_not_found"`, `error.type: "invalid_request_error"` | Codex validates effort against the model's advertised `supportedReasoningEfforts` at runtime; a mismatched pairing is rejected |
| codex → Ollama | Request fails against the local Ollama server (no such model pulled) — not an OpenAI-shaped 404 | The field is documented for `/v1/chat/completions` but not for `/v1/responses`, which is the route this namespace configures; Codex would send it and the route would drop it silently, so the registry gives the namespace an empty `reasoning_efforts` list and fails closed |
| codex → OpenRouter | HTTP 404 on an unrecognized slug | Same fail-closed treatment as Ollama, for the same reason: the documented effort field belongs to the chat-completions surface, and the Responses route this namespace configures is unverified |
| claude-code | HTTP 404 at request time (not caught at subagent startup) | `effort` degrades gracefully to the highest level the resolved model supports on every model except Claude Haiku 4.5, which the registry gates to an empty `reasoning_efforts` list and fails closed at policy-check time |
| cursor | `ConfigurationError` | N/A — no reasoning field is projected for Cursor |

## Enforcement boundaries

What `scripts/model-policy.mjs` guarantees, given this registry:

- Every `model` value in `catalog/model-policy.json` matches a known namespace shape; values in `closed` namespaces are additionally checked against the enumerated `models` list.
- Every `reasoning_effort` value is checked against the resolved model's actual supported vocabulary (harness vocabulary narrowed by namespace, narrowed again by per-model `reasoning_efforts` where the registry declares them) — a resolution-time model × reasoning compatibility check, so pinning `reasoning_effort` onto an Ollama- or OpenRouter-routed codex model is rejected before it is written anywhere.
- The codex `model_provider` line is derived automatically from the model's namespace: `openai`-namespace models project no line (default provider); `ollama`-namespace (`name:tag`) and `openrouter`-namespace (`author/model`) values project `model_provider = "ollama"` / `"openrouter"` respectively.

What remains the operator's responsibility — the registry and engine cannot verify these:

- The operator's `codex.toml` must define the matching `[model_providers.ollama]` / `[model_providers.openrouter]` table (`base_url`, `wire_api`, and — for OpenRouter — `env_key`) for a projected `model_provider` route to actually work at runtime.
- Ollama models must already be pulled locally (`ollama pull <name:tag>`) — the registry validates the slug's *shape*, not that the model exists on the operator's machine.
- Cursor named-model availability is plan/admin-dependent; a model listed here as registry-valid may still be unavailable to a given Cursor workspace.
- The registry records what official documentation verified as of `last_refreshed`; it does not poll providers live, so a model retired or renamed after that date will not be caught until the next refresh.

## How to extend

Add or update models through [`.claude/skills/model-registry-refresh/SKILL.md`](../.claude/skills/model-registry-refresh/SKILL.md) — never by editing `catalog/model-registry.json` from memory. After any registry change, run `npm run model-policy:check` to confirm the policy still resolves cleanly against the updated registry, then `npm run validate`, then `npm run asset-integrity:write` last.
