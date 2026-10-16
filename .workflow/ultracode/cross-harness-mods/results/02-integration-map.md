# 02 — Repository integration map

Status: complete (orchestrator, read-only). Date: 2026-10-14.

- `package.json`:
  - `validate` is a `&&` chain of `validate:*` scripts, so a new `validate:mods` must be added to that chain.
  - `test:fuzz` runs `node tests/fuzz-properties.test.mjs`.
  - fast-check `^4.7.0` is already a devDependency.
- `catalog/index.json`: the `catalogs` map lists skills, agents, rules, mcp_references and workflows. A `"mods": "catalog/mods.json"` entry would go here.
- `tests/validate-asset-integrity.py`: `TREES` does not include `mods`. Add it, or the mod scripts sit outside the integrity manifest. That would be a supply-chain blind spot, for the same reason `tests/` is covered.
- `.plugin-scanner.toml`: the header policy keeps every rule active on executable code. Only data, docs and fixtures may be ignored. Ignoring `mods/**/*.sh` would contradict that policy.
- `catalog/agents.json`: 735 agents. Ids that match `*-live-*-guard-agent` exist across providers (alibaba, aws, azure, contabo, d365, …). Note that `aws-live-deployment-guarded-operator-agent` does not match that glob.
- `catalog/rules.json` exists, so it is a usable pattern for a flat catalog array with `security_notes`.
- `tests/fuzz-properties.test.mjs`: logic is currently "reproduced verbatim" from the scripts. The eval contract requires new mods tests to import production exports instead.
- Python 3.12.3 is available, so `tomllib` is in the stdlib.
- `mods/` does not exist yet. The `.kiro/kiro-agents/` directory is untracked and pre-existing, and is not touched.
