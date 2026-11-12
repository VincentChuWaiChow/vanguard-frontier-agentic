# AGENTS.md

## Purpose
- Cloud + zero-trust agentic marketplace for skills, agents, rules, MCP references, compliance-aware docs, and npm distribution.
- Optimize for evidence-backed security workflows: least privilege, source grounding, manifests, validation, and safe automation.

## Stack Map
- `.code-review-graph/` → generated local graph cache; do not edit.
- `.git/` → Git internals; do not edit.
- `agents/` → Markdown/JSON agent definitions; provider/domain layout.
- `assets/` → curated logos and visual assets.
- `catalog/` → JSON marketplace indexes, skill integrity manifest, and role taxonomy.
- `docs/` → Jekyll documentation site (GitHub Pages). Uses `_data/catalog.yml` for computed values. Never hardcode counts.
- `docs/_data/` → Auto-generated data files for Jekyll. Single source of truth for all metrics displayed on the site.
- `mcp/` → Markdown/JSON MCP references.
- `rules/` → Markdown/JSON harness rules.
- `schemas/` → JSON Schema metadata contracts.
- `scripts/` → Generation and automation scripts (readme-counts, docs-data, plugin manifests, export CLI).
- `skills/` → Markdown/JSON skill packages with reference files.
- `templates/` → starter Markdown/JSON asset templates.
- `tests/` → Python/Node.js validation scripts.
- `tools/` → Standalone tooling (e.g., `tools/vfa-tui/` Rust TUI for catalog browsing).
- `package.json` → npm package metadata and validation scripts.

## Workflows
- `npm run validate` → 19+ gates: catalog, AWS quality, skill manifest, `allowed-tools`, skill schema, agent schema, links (offline), asset integrity, MCP trust matrix, no-lifecycle-scripts, promotion gatekeeper, install coverage, maestro routing (357 scenarios), Claude Code plugin manifest, Kiro Powers, multi-harness marketplace (Cursor + Copilot), Codex marketplace, finops fixtures, readme counts, QA cluster.
- `npm run lint:docs` → advisory markdownlint + codespell (runs as `Docs Quality` workflow in CI).
- `npm run manifest:write` → refresh `catalog/skill-manifest.json` after intentional skill edits.
- `npm run readme-counts:write` → regenerate inline count markers in README.md from live catalog data. Validated by `validate:readme-counts`.
- `npm run docs-data:write` → regenerate `docs/_data/catalog.yml` for the Jekyll documentation site. All counts, provider taxonomy, and role lists are computed from the live catalog — the Jekyll site uses Liquid template variables, never hardcoded numbers.
- `npm run plugin-manifest:write` → regenerate `.claude-plugin/plugin.json` from `catalog/agents.json` after intentional agent additions or removals. The repo is a Claude Code plugin marketplace (`/plugin marketplace add Raishin/vanguard-frontier-agentic`) in addition to an npm package.
- `npm run kiro-powers:write` → regenerate the 35 Kiro Powers under `powers/vanguard-*` from `catalog/agents.json` plus the per-provider steering config baked into `scripts/generate-kiro-powers.mjs`. Kiro frontmatter is **strictly** limited to five fields (`name`, `displayName`, `description`, `keywords`, `author`) — adding any other field will fail `validate:kiro-powers`.
- `npm run cursor-plugin:write` → regenerate `.cursor-plugin/plugin.json` from `catalog/agents.json`. The repo is also a Cursor plugin (`.cursor-plugin/plugin.json`) — enumerates cursor agent adapter paths explicitly per Cursor's plugin spec (cursor.com/docs/reference/plugins).
- `npm run manifest:write:all` → runs ALL regeneration scripts in parallel (manifest, plugin-manifest, cursor-plugin, kiro-powers, asset-integrity, readme-counts).
- GitHub Copilot CLI marketplace lives at `.github/plugin/marketplace.json`. Single-plugin marketplace with source `./` — the repo root is the plugin root. Install via `copilot plugin marketplace add Raishin/vanguard-frontier-agentic`. Both Cursor and Copilot manifests are validated together by `validate:multi-harness-marketplace`.
- Codex marketplace lives at `.agents/plugins/marketplace.json` (canonical Codex location per [codex-rs plugin-json-spec](https://github.com/openai/codex/blob/main/codex-rs/skills/src/assets/samples/plugin-creator/references/plugin-json-spec.md)). Declares two plugins: `vanguard-frontier-agentic` (main) and `cross-platform-agent-template` (scaffold). Install via `codex plugin marketplace add Raishin/vanguard-frontier-agentic`. Validated by `validate:codex-marketplace`: marketplace shape, plugin name = folder name rule, kebab-case names, `policy.{installation, authentication}` and `category` required on every entry, plugin.json version parity with package.json.
- `python3 tests/validate-links.py` → online link validation before release.
- `npm pack --dry-run` → inspect npm package contents before publish.
- `vfa-export-agents --list-roles` → list available role IDs with agent counts.
- `vfa-export-agents --platform claude-code --all --repo <path>` → install all agents AND auto-bundle their companion skills (default for `claude-code`).
- `vfa-export-agents --platform claude-code --all --no-skills --repo <path>` → opt out of skill bundling (agents only).
- `vfa-export-agents --platform <p> --role <role-id> --repo <path>` → install agents (and companion skills on `claude-code`) for a role.
- `vfa-export-agents --platform <p> --role <role-id> --provider <provider> --repo <path>` → install role agents for one provider.

## Change Rules
- Update catalog JSON when adding, moving, or removing cataloged assets.
- **After any change to `.github/workflows/release.yml`, root files (package.json, CLAUDE.md, AGENTS.md, etc.), or any asset file, regenerate asset integrity:** `python3 tests/validate-asset-integrity.py --write && git add catalog/asset-integrity.json && git commit -m "chore: regenerate asset integrity after <description>"` — this blocks the validation gate and release workflow if stale.
- Regenerate skill manifest after any intentional change under cataloged `skills/**`.
- For every `skills/**/SKILL.md`, keep skill frontmatter metadata under `metadata`, including `metadata.version` and `metadata.author`; use the GitHub-style author value (for example `github: Raishin`) and do not use top-level `version` or `author` keys in skill frontmatter.
- For cross-platform agent work, keep `author` and `version` truth in the canonical contract plus adjacent `metadata.json` unless a harness's official docs explicitly verify executable metadata support.
- Do not create partial metadata truth across harnesses: if `author` and `version` are not both doc-verified in an executable adapter, keep both out of that adapter.
- Keep README human-friendly; keep this file agent-focused and compressed.
- For provider-specific `README.md` files, add the matching cloud-provider logo near the top using a repo-local asset from `assets/logos/cloud/<provider>/`.
- Do not add a provider logo to neutral or multi-cloud READMEs unless one provider is clearly the primary subject.
- Do not add secrets, credentials, tokens, wallets, tenant IDs, or customer data.
- Prefer official docs and live evidence over memory for cloud/compliance claims.
- Treat broad permissions, destructive automation, and MCP mutation paths as high-risk.
- When adding new agents, update `catalog/install-roles.json` if the agent belongs to one or more roles. An agent may appear in multiple roles. List current roles with `vfa-export-agents --list-roles`.
- All live-guard and review agents must produce the five required evidence fields defined in `docs/evidence-output-spec.md`: `verdict`, `evidence_level`, `blockers`, `safe_next_actions`, `open_questions`.

## Documentation Maintenance (DRY / Single Responsibility)

**Principle: Counts, lists, and taxonomy are NEVER hardcoded in documentation.** They are computed from the catalog and injected via templating.

### Single Source of Truth chain:
1. `catalog/*.json` → canonical data (agents, skills, roles, providers, MCP refs, rules)
2. `scripts/generate-readme-counts.mjs` → updates `README.md` inline `<!-- count:KEY -->` markers
3. `scripts/generate-docs-data.mjs` → generates `docs/_data/catalog.yml` for Jekyll site
4. `docs/*.md` → uses `{{ site.data.catalog.X }}` Liquid variables, never raw numbers

### When to regenerate:
- **After adding/removing agents or skills:** `npm run readme-counts:write && npm run docs-data:write`
- **After adding/removing roles:** same as above (roles are in `catalog/install-roles.json`)
- **After adding a validation gate:** `npm run docs-data:write` (counts `validate:*` scripts in package.json)
- **After bumping package.json version:** `npm run readme-counts:write && npm run docs-data:write`
- **After any catalog change:** `npm run manifest:write:all` (runs everything including above)

### Jekyll docs rules:
- `docs/_data/catalog.yml` is auto-generated — do NOT edit by hand.
- All docs pages use Liquid templates: `{{ site.data.catalog.agents }}`, `{{ site.data.catalog.validation_gates }}`, `{% for role in site.data.catalog.role_list %}`, `{% for group in site.data.catalog.provider_taxonomy %}`.
- The provider taxonomy groups providers into 9 categories (Cloud Hyperscalers, European Cloud, Container & Orchestration, Security & Supply Chain, Observability, IaC, AI & Compute, Developer Platforms, Business Functions). When adding a new provider, add it to the taxonomy in `scripts/generate-docs-data.mjs`.
- When creating or editing README.md, also run `npm run docs-data:write` to keep the Jekyll site in sync.
- Do not duplicate role lists, provider lists, or count tables — reference the data file.
- The `_config.yml` excludes non-documentation directories from Jekyll processing.

### What NOT to do:
- ❌ Hardcode "426 agents" or "17 validation gates" in any docs page
- ❌ Manually list all 21 roles in a markdown file (use a Liquid loop)
- ❌ Copy-paste provider counts from catalog into prose
- ❌ Edit `docs/_data/catalog.yml` by hand
- ❌ Forget to run `docs-data:write` after catalog changes

## Release & Versioning (semantic-release)

This project uses **semantic-release** for fully automated versioning and npm publishing. No manual version bumps.

### How it works:
1. PRs merge to `master` with Conventional Commit messages (`feat:`, `fix:`, `chore:`, `docs:`)
2. semantic-release analyzes commits: `feat:` → minor bump, `fix:` → patch bump, `BREAKING CHANGE` → major
3. On release: version bumped in `package.json`, Git tag created, published to npm via OIDC, chore(release) commit pushed
4. The chore(release) commit includes `[skip ci]` to prevent recursive triggers

### Version parity rules:
- `package.json` is the source of truth for the current version
- `.claude-plugin/plugin.json`, `.claude-plugin/marketplace.json`, `.cursor-plugin/plugin.json` MUST match `package.json` version
- The plugin manifest generators (`plugin-manifest:write`, `cursor-plugin:write`) auto-read the version from `package.json`
- After merging a branch that was behind master's latest release, ALWAYS run: `npm run plugin-manifest:write && npm run cursor-plugin:write` to sync plugin versions
- The Codex marketplace (`.agents/plugins/marketplace.json`) auto-computes version from `package.json` via `generate-codex-marketplace.mjs`

### After merging from master (catching up to latest release):
```bash
npm run plugin-manifest:write    # Sync .claude-plugin version
npm run cursor-plugin:write      # Sync .cursor-plugin version
npm run kiro-powers:write        # Regenerate all Kiro Powers
npm run readme-counts:write      # Update README inline counts
npm run docs-data:write          # Update Jekyll docs data
python3 tests/validate-asset-integrity.py --write  # Refresh hashes
```

Or use the all-in-one: `npm run manifest:write:all`

### What triggers a release:
- `feat:` commit → next minor (e.g., 2.8.0 → 2.9.0)
- `fix:` commit → next patch (e.g., 2.8.0 → 2.8.1)
- `chore:` / `docs:` → no release (maintenance only)
- `BREAKING CHANGE` footer → next major

### What NOT to do with versions:
- ❌ Manually edit `"version"` in package.json (semantic-release owns this)
- ❌ Push a branch with a stale version without regenerating plugin manifests
- ❌ Assume `package.json` version matches plugin manifests after a branch merge — always regenerate
- ❌ Create Git tags manually (semantic-release creates `vX.Y.Z` tags)

## Role-Based Pattern

`catalog/install-roles.json` defines cross-provider roles (currently 21). Each role is a curated list of agent (and skill) IDs that practitioners in that function need, across all supported cloud providers. Run `vfa-export-agents --list-roles` to see the current full list.

Core cloud roles:

| Role ID | Who uses it |
|---------|----------|
| `cloud-security-engineer` | IAM reviewers, security posture teams, compliance engineers |
| `cloud-platform-engineer` | Infrastructure/SRE, IaC owners, Kubernetes platform teams |
| `cloud-dba` | Database administrators, data platform engineers |
| `cloud-finops-analyst` | FinOps leads, cost governance teams |
| `cloud-solutions-architect` | Cloud architects, migration leads, AI/generative engineers |
| `cloud-devops-engineer` | CI/CD engineers, release managers, SRE ops |
| `cloud-ai-platform-engineer` | AI/ML platform teams, Bedrock/Vertex/OCI AI engineers |

Kubernetes specialist roles:

| Role ID | Who uses it |
|---------|----------|
| `kubernetes-admission-security-engineer` | Admission policy, Kyverno, OPA engineers |
| `kubernetes-network-engineer` | CNI, service mesh, network policy engineers |
| `kubernetes-pki-engineer` | cert-manager, CA, mTLS operators |
| `kubernetes-observability-engineer` | OTEL, Prometheus, tracing operators |
| `kubernetes-supply-chain-security-engineer` | Sigstore, SBOM, image provenance |
| `kubernetes-runtime-security-engineer` | Falco, runtime detection, forensics |
| `kubernetes-application-platform-engineer` | ArgoCD, app delivery, GitOps |
| `kubernetes-developer-platform-engineer` | Backstage, FluxCD, developer experience |
| `kubernetes-disaster-recovery-engineer` | Velero, backup, failover |

Business function roles:

| Role ID | Who uses it |
|---------|----------|
| `legal-hr-risk-reviewer` | Employment law, investigations, HR compliance |
| `salesforce-portfolio-architect` | CRM platform review, Apex, integration |
| `netsuite-platform-advisor` | NetSuite ERP review, SuiteScript, SuiteCloud, integrations |
| `microsoft-365-d365-platform-advisor` | Microsoft 365, Dynamics 365, Power Platform & Copilot governance review |
| `microsoft-security-compliance-engineer` | Entra Zero Trust, Purview data security & compliance, Defender XDR SecOps, Copilot data-exposure readiness |
| `microsoft-collaboration-endpoint-admin` | Intune endpoints, Teams collaboration, Exchange/SharePoint information governance |
| `microsoft-data-analytics-engineer` | Microsoft Fabric data engineering & analytics engineering, Power BI semantic-model governance |
| `dotnet-application-review-engineer` | .NET code review, architecture, security |
| `marketing-governance-reviewer` | Brand compliance, campaign review |
| `qa-test-quality-engineer` | Test strategy, CI quality gates, coverage |

Roles overlap intentionally — an agent useful to both a security engineer and a platform engineer appears in both lists.

The `--role` flag in `vfa-export-agents` resolves the role's agent list and exports them in one command. Use `--provider` to further filter by cloud provider (e.g., `--provider azure`).

Pipeline enforcement — pushing role installs into CI/CD so guardrails run without developer opt-in — is documented in `docs/ci-cd-enforcement-pattern.md`.

Evidence output — how the structured verdict response from every live-guard and review agent satisfies SOC 2, PCI DSS, NIS2, NIST CSF, and ISO 27001 controls — is documented in `docs/evidence-output-spec.md`.

## Load When
- Rust work → `rules/rust/rust-safety-rule.md`
- editing `agents/` → `agents/AGENTS.md`
- editing `catalog/` → `catalog/AGENTS.md`
- editing `docs/` → `docs/AGENTS.md`
- editing `mcp/` → `mcp/AGENTS.md`
- editing `rules/` → `rules/AGENTS.md`
- editing `schemas/` → `schemas/AGENTS.md`
- editing `skills/` → `skills/AGENTS.md`
- editing `templates/` → `templates/AGENTS.md`
- editing `tests/` or validation scripts → `tests/AGENTS.md`
