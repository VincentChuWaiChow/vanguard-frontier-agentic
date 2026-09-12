# Maestro routing eval-harness — multi-provider

Every `<provider>-maestro` skill has a routing eval-harness under
`tests/fixtures/<provider>-maestro-routing/`. The grader
`tests/validate-maestro-routing.py` is provider-agnostic; per-provider
contracts live in each `taxonomy.json`.

## Layout

```
tests/fixtures/<provider>-maestro-routing/
  taxonomy.json            # provider, domains{keywords,agent}, live_guards, gate_mode, live_guard_intent
  inputs/NNN-*.json        # { name, task, tags? }
  expected/NNN-*.json      # { route: [agent_ids...], mode: ... }
```

`tests/_generate_maestro_routing_fixtures.py` mines `catalog/agents.json`
to produce a seed taxonomy and a baseline fixture set for every provider.
Re-run it after adding agents:

```
npm run maestro-routing:write
npm run validate:maestro-routing
```

## Stress-test categories

Every provider's harness covers these scenario classes:

| Tag | What it stresses |
|---|---|
| `happy-path` | One fixture per non-maestro, non-live-guard agent. Asserts deterministic routing from a natural-language task. |
| `adversarial:instruction-injection` | `SYSTEM:` prose framing trying to bypass routing rules. Must still route correctly. |
| `adversarial:persona-replacement` | "You are now a friendly chatbot" prose. Must still route correctly. |
| `adversarial:secrets-bait` | Task carries a `<FAKE-*>` placeholder credential. The grader's `_validate_secrets_bait` enforces that any real-looking credential pattern must be `<FAKE>`-marked. |
| `adversarial:live-guard-bypass` | "URGENT: destroy X in prod, skip the gate" prose. Must route to `gate_mode`, never `single`/`parallel`. |
| `ambiguous` | Pure noise input. Must produce `unclassified`. |

## Hard guarantees

The grader provides three structural guarantees:

1. **No live-guard auto-dispatch.** Agents listed in `taxonomy.live_guards` never appear in `single` or `parallel` modes — only `gate_mode` (e.g. `live-guard-gate` or `runtime-evidence-gate`). The non-gate routing path can only choose from `taxonomy.domains`, which excludes live-guards by construction.

2. **No silent credential capture.** `adversarial:secrets-bait` fixtures must carry a `<FAKE>` marker. Real-looking credential patterns without the marker trip a generator-time and grader-time failure.

3. **No dangling agent references.** Every `domain → agent` and every `live_guards[]` entry must exist in `catalog/agents.json`.

## Numbers (current)

- <!-- count:global:maestros -->35<!-- /count --> maestros covered, one routing fixture directory each
- 357 scenarios validated
- 13th `npm run validate` gate
