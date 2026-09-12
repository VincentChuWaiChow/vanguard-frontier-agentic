---
layout: default
title: "Configuration Reference"
permalink: /docs/configuration/
---

# 🔧 Configuration Reference

All configurable surfaces in the Vanguard Frontier Agentic ecosystem, with file paths and expected values.

---

## 📋 npm Scripts

The `package.json` defines all build, validation, and generation commands.

### Validation Gates (run via `npm run validate`)

These <!-- count:global:gates -->28<!-- /count --> `validate:*` gates run sequentially in CI and must all pass, in the order `npm run validate` invokes them (the table also lists `manifest:check`, which runs in the same sequence but is not a `validate:*` script):

| # | Script | Command | Purpose |
|---|--------|---------|---------|
| 1 | `validate:catalog` | `python3 tests/validate-catalog.py` | Catalog JSON matches filesystem |
| 2 | `validate:aws` | `python3 tests/validate-aws-skill-quality.py && python3 tests/validate-aws-progressive-disclosure.py` | AWS skill quality + progressive disclosure |
| 3 | `manifest:check` | `python3 tests/validate-skill-manifest.py` | Skill manifest is current |
| 4 | `validate:allowed-tools` | `python3 tests/validate-skill-allowed-tools.py` | Skills only reference allowed tools |
| 5 | `validate:skill-coherence` | `python3 tests/validate-skill-coherence.py` | Shell examples in a SKILL.md are covered by its declared `allowed-tools` |
| 6 | `validate:skill-schema` | `python3 tests/validate-skill-frontmatter-schema.py` | Skill frontmatter matches schema |
| 7 | `validate:agent-schema` | `python3 tests/validate-agent-frontmatter-schema.py` | Agent metadata matches schema |
| 8 | `validate:model-policy` | `node scripts/model-policy.mjs check` | Model/effort assignments resolve against `catalog/model-registry.json` |
| 9 | `validate:links` | `python3 tests/validate-links.py --offline` | Internal links resolve |
| 10 | `validate:asset-integrity` | `python3 tests/validate-asset-integrity.py` | SHA-256 hashes match |
| 11 | `validate:mcp-trust-matrix` | `python3 tests/validate-mcp-trust-matrix.py` | MCP references are trusted |
| 12 | `validate:no-lifecycle-scripts` | `python3 tests/validate-no-lifecycle-scripts.py` | No install/postinstall scripts |
| 13 | `validate:promotion-gatekeeper` | `python3 tests/validate-nvidia-promotion-gatekeeper.py` | NVIDIA promotion rules |
| 14 | `validate:install-coverage` | `node tests/test-vfa-export-coverage.test.mjs` | Export CLI covers all roles |
| 15 | `validate:maestro-routing` | `python3 tests/validate-maestro-routing.py` | Maestro routing scenarios pass |
| 16 | `validate:plugin-manifest` | `python3 tests/validate-plugin-manifest.py` | Plugin manifests current |
| 17 | `validate:kiro-powers` | `python3 tests/validate-kiro-powers.py` | Kiro Powers valid |
| 18 | `validate:multi-harness-marketplace` | `python3 tests/validate-multi-harness-marketplace.py` | Cross-harness consistency |
| 19 | `validate:codex-marketplace` | `python3 tests/validate-codex-marketplace.py` | Codex marketplace valid |
| 20 | `validate:finops-fixtures` | `python3 tests/validate-finops-price-fixtures.py` | FinOps price fixtures |
| 21 | `validate:readme-counts` | `node tests/validate-readme-counts.mjs && node scripts/generate-readme-counts.mjs --check` | README stats accurate |
| 22 | `validate:board-counts` | `node scripts/generate-board-counts.mjs --check` | Inline `<!-- count:… -->` markers in docs match the catalog |
| 23 | `validate:qa-cluster` | `node tests/eval-qa-cluster.mjs` | QA cluster evaluation |
| 24 | `validate:frontend-security-detection` | `python3 tests/validate-frontend-security-detection.py` | Frontend security skills still document their sink keywords and detect the fixture corpus |
| 25 | `validate:agent-tool-tiers` | `python3 tests/validate-agent-tool-tiers.py` | Copilot tool grants match each agent's declared `execution_tier` |
| 26 | `validate:workflow-catalog` | `node scripts/generate-workflow-catalog.mjs --check` | `catalog/workflows.json` matches the `meta` literal in each `.claude/workflows/*.js` |

### Generation Scripts

| Script | Purpose |
|--------|---------|
| `manifest:write` | Regenerate skill manifest |
| `plugin-manifest:write` | Regenerate Claude plugin manifest |
| `cursor-plugin:write` | Regenerate Cursor plugin |
| `kiro-powers:write` | Regenerate Kiro Powers |
| `asset-integrity:write` | Regenerate asset integrity hashes |
| `readme-counts:write` | Update README catalog counts |
| `manifest:write:all` | Run all generation scripts in parallel |
| `maestro-routing:write` | Regenerate Maestro routing fixtures |

### Test Scripts

| Script | Purpose |
|--------|---------|
| `test:fuzz` | Property-based fuzz tests via fast-check |
| `test:marketplace-validators` | Marketplace validator unit tests |
| `test:copilot-bundling` | Copilot skill bundling tests |
| `test:gemini-bundling` | Gemini skill bundling tests |
| `test:cursor-kiro-notices` | Cursor/Kiro notice export tests |
| `test:codex-plugin-marketplace-install` | Codex install path tests |

### Lint Scripts

| Script | Purpose |
|--------|---------|
| `lint:md` | markdownlint-cli2 on all Markdown |
| `lint:spell` | codespell for typos |
| `lint:docs` | Both linters combined |

---

## 🗂️ Catalog Structure

### `catalog/agents.json`

Array of agent entries. Each entry references:
- Agent ID
- Provider
- Install roles
- Skill references
- Harness compatibility

### `catalog/skills.json`

Array of skill entries with frontmatter metadata extracted from each skill's Markdown file.

### `catalog/install-roles.json`

Maps install roles to their agent sets:
- `cloud-security-engineer`
- `cloud-platform-engineer`
- `cloud-dba`
- `cloud-finops-analyst`
- `cloud-solutions-architect`
- `cloud-devops-engineer`

### `catalog/asset-integrity.json`

SHA-256 hashes of critical repository files. Validated by `npm run validate:asset-integrity`. Regenerated by `python3 tests/validate-asset-integrity.py --write`.

---

## 📏 Schema Contracts

Located in `schemas/`:

### `skill.frontmatter.schema.json`

Required fields in skill Markdown frontmatter:
- `title` (string)
- `provider` (string)
- `category` (string)
- `triggers` (array of strings)

### `agent.frontmatter.schema.json`

Required fields in agent `metadata.json`:
- `id` (string)
- `name` (string)
- `provider` (string)
- `roles` (array of install role strings)
- `skills` (array of skill ID references)

### `mcp-reference.schema.json`

Structure for MCP (Model Context Protocol) references defining external tool integrations.

### `rule.schema.json`

Structure for harness rules that constrain agent behavior.

---

## 🧠 Skill File Format

Skills are Markdown files at `skills/<provider>/<skill-id>.md`:

```markdown
---
title: "Descriptive Skill Title"
provider: aws
category: security
triggers:
  - "check S3 bucket policy"
  - "audit IAM permissions"
allowed_tools: []
---

# Skill Content

Step-by-step guidance the agent follows...
```

---

## 🤖 Agent File Format

Agents live at `agents/<provider>/<agent-id>/metadata.json`:

```json
{
  "id": "aws-iam-auditor",
  "name": "AWS IAM Auditor",
  "provider": "aws",
  "description": "Audits IAM policies for least-privilege violations",
  "roles": ["cloud-security-engineer"],
  "skills": ["aws/iam-policy-audit", "aws/iam-access-analyzer"],
  "harnesses": {
    "claude": true,
    "codex": true,
    "copilot": true,
    "cursor": true,
    "gemini": true,
    "kiro": true
  }
}
```

---

## 📄 Steering Files

### `CLAUDE.md` (repo root)

Provides Claude Code with project context, conventions, and constraints.

### `AGENTS.md` (repo root)

Universal agent instructions applicable to all harnesses.

### `.github/copilot-instructions.md`

GitHub Copilot-specific context and instructions.

---

## Jekyll Configuration (`_config.yml`)

```yaml
title: "Vanguard Frontier Agentic"
baseurl: "/vanguard-frontier-agentic"
theme: minima
markdown: kramdown
plugins:
  - jekyll-seo-tag
```

The `exclude` list prevents Jekyll from processing non-documentation directories (agents, skills, node_modules, etc.).

---

## ✅ How to Verify This Works

```bash
# Validate all schemas are enforced
npm run validate:skill-schema
npm run validate:agent-schema

# Confirm catalog matches filesystem
npm run validate:catalog

# Check all generation outputs are current
npm run manifest:check
npm run validate:plugin-manifest
npm run validate:kiro-powers
```
