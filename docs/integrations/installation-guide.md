# Installation Guide — Vanguard Frontier Agentic

**Audience:** engineers setting up Vanguard Frontier Agentic in any supported AI coding harness.
**Scope:** all eight install paths with prerequisites, step-by-step commands, verification, advanced configuration, pinning, and troubleshooting.

---

## Contents

1. [Quick comparison](#1-quick-comparison)
2. [Claude Code (Anthropic)](#2-claude-code-anthropic)
3. [GitHub Copilot CLI](#3-github-copilot-cli)
4. [Cursor](#4-cursor)
5. [Kiro (Powers)](#5-kiro-powers)
6. [Gemini CLI / Google Antigravity](#6-gemini-cli--google-antigravity)
7. [OpenAI Codex](#7-openai-codex)
8. [npm package + vfa-export-agents (universal)](#8-npm-package--vfa-export-agents-universal)
9. [Skills CLI (third-party, exploration only)](#9-skills-cli-third-party-exploration-only)
10. [Pinning and reproducibility](#10-pinning-and-reproducibility)
11. [Verification checklist](#11-verification-checklist)
12. [Troubleshooting](#12-troubleshooting)

---

## 1. Quick comparison

| Harness | Install method | Single command? | Versioned? | Scope |
|---------|---------------|-----------------|------------|-------|
| **Claude Code** | Plugin marketplace | Yes | Yes (tag pin) | {{ site.data.catalog.agents }} agents |
| **Copilot CLI** | Plugin marketplace | Yes | HEAD / tag | {{ site.data.catalog.agents }} agents |
| **Cursor** | Plugin directory | Clone + UI | Yes (git tag) | {{ site.data.catalog.agents }} agents |
| **Kiro** | Powers panel (per Power) | No (UI per Power) | Yes (git tag) | {{ site.data.catalog.providers }} Powers + agents via npm |
| **Gemini / Antigravity** | npm export | Yes (npm + CLI) | Yes (semver) | Skills + adapters |
| **Codex** | Plugin marketplace | Yes | Yes (sha / tag) | {{ site.data.catalog.agents }} agents + 2 plugins |
| **npm + vfa-export-agents** | npm package | Yes | Yes (semver) | Everything |
| **Skills CLI** | Third-party CLI | Yes | **No** (HEAD) | Skills only |

For the trust model (signed artifacts vs. raw HEAD vs. third-party) see
[`docs/integrations/skills-cli.md`](./skills-cli.md).

---

## 2. Claude Code (Anthropic)

### Prerequisites

- Claude Code CLI installed (`claude --version`)
- Internet access (downloads from GitHub at install time)
- Optional: `~/.claude/settings.json` writable for team-wide config

### Option A — one-command install (recommended)

```bash
# Step 1: register the marketplace
/plugin marketplace add VincentChuWaiChow/vanguard-frontier-agentic

# Step 2: enable the bundled plugin
/plugin install vanguard-frontier-agentic@vanguard-frontier-agentic
```

`/plugin marketplace add` writes an entry under `extraKnownMarketplaces` in
`~/.claude/settings.json`. The second command fetches the plugin manifest
from `.claude-plugin/plugin.json` and enables it.

### Option B — settings.json (team-wide / CI)

Add to `~/.claude/settings.json` (user-level) or `<repo>/.claude/settings.json`
(project-level, committed to source control):

```json
{
  "extraKnownMarketplaces": {
    "vanguard-frontier-agentic": {
      "source": {
        "source": "github",
        "repo": "VincentChuWaiChow/vanguard-frontier-agentic"
      }
    }
  },
  "enabledPlugins": {
    "vanguard-frontier-agentic@vanguard-frontier-agentic": true
  }
}
```

**Pinning to a specific release:**

```json
{
  "extraKnownMarketplaces": {
    "vanguard-frontier-agentic": {
      "source": {
        "source": "github",
        "repo": "VincentChuWaiChow/vanguard-frontier-agentic",
        "ref": "v1.8.0"
      }
    }
  }
}
```

Replace `v1.8.0` with any tag from the
[releases page](https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/releases).

### What gets installed

- **{{ site.data.catalog.agents }} agent adapters** — resolved via the `agents[]` array in
  `.claude-plugin/plugin.json`. Each adapter is a Claude Code–compatible
  markdown file at `agents/<provider>/<agent>/harnesses/claude-code.agent.md`.
- **Not installed automatically:** skills (`skills/`), rules (`rules/`), MCP
  references (`mcp/`). Use the [npm path](#8-npm-package--vfa-export-agents-universal)
  to export those.

### Verify

```bash
# List enabled plugins
/plugin list

# Invoke any agent by ID
"Use vanguard-aws-cost-optimization-agent to find idle EC2 instances."
```

---

## 3. GitHub Copilot CLI

### Prerequisites

- GitHub Copilot CLI installed (`copilot --version`)
- Authenticated with `copilot auth login`

### Option A — one-command install

```bash
# Step 1: register the marketplace
copilot plugin marketplace add VincentChuWaiChow/vanguard-frontier-agentic

# Step 2: enable the plugin
/plugin install vanguard-frontier-agentic
```

`copilot plugin marketplace add` writes the marketplace URL under
`extraKnownMarketplaces` in `~/.copilot/settings.json`.

### Option B — settings.json (repo-wide)

Create or update `.github/copilot/settings.json` in your repo:

```json
{
  "extraKnownMarketplaces": [
    "https://raw.githubusercontent.com/VincentChuWaiChow/vanguard-frontier-agentic/master/.github/plugin/marketplace.json"
  ]
}
```

> **Note:** Copilot CLI's `extraKnownMarketplaces` is an **array of URLs**,
> not an object. This differs from Claude Code's object-keyed format.

**Pinning to a specific release:**

Replace `master` in the URL with a tag or SHA:

```json
{
  "extraKnownMarketplaces": [
    "https://raw.githubusercontent.com/VincentChuWaiChow/vanguard-frontier-agentic/v1.8.0/.github/plugin/marketplace.json"
  ]
}
```

### What gets installed

- **Single plugin** declared in `.github/plugin/marketplace.json` with `source: "./"` — the repo root is the plugin root.
- **{{ site.data.catalog.agents }} Copilot agent adapters** at `agents/<provider>/<agent>/harnesses/copilot.agent.md`.
- **Plugin instructions file** is `AGENTS.md` at repo root (Copilot reads `AGENTS.md` or `.github/copilot-instructions.md`).

### Verify

```bash
/plugin list
# Should show: vanguard-frontier-agentic (enabled)
```

---

## 4. Cursor

### Prerequisites

- Cursor IDE installed (1.x or later)
- Git available on PATH

### Step-by-step

**Step 1 — clone the repo:**

```bash
git clone https://github.com/VincentChuWaiChow/vanguard-frontier-agentic
# For a pinned version:
git clone --branch v1.8.0 https://github.com/VincentChuWaiChow/vanguard-frontier-agentic
```

**Step 2 — register the plugin directory in Cursor:**

Open Cursor → `Settings` → `Plugins` → `Add Plugin Directory` → select the cloned directory.

Or via the Cursor Extension API (e.g., from a VS Code extension or workspace script):

```ts
vscode.cursor.plugins.registerPath("/absolute/path/to/vanguard-frontier-agentic");
```

**Step 3 — verify in Cursor:**

Open the Agent panel. You should see agents from `vanguard-aws`, `vanguard-azure`, etc.

### How the plugin manifest works

Cursor reads `.cursor-plugin/plugin.json` from the registered directory. The
manifest enumerates all **{{ site.data.catalog.agents }} Cursor agent adapters** explicitly via the
`agents[]` array:

```json
{
  "name": "vanguard-frontier-agentic",
  "version": "1.8.0",
  "agents": [
    "agents/aws/aws-cost-optimization/harnesses/cursor.agent.md",
    "agents/aws/aws-security-posture/harnesses/cursor.agent.md",
    "..."
  ]
}
```

Cursor also auto-discovers `rules/` at the repo root.

### Updating

```bash
cd vanguard-frontier-agentic
git fetch origin
git checkout v1.8.0   # or `git pull origin master` for latest
```

Cursor picks up changes automatically after the directory is re-scanned.

---

## 5. Kiro (Powers)

### Prerequisites

- Kiro IDE installed
- Git available on PATH

### What "Powers" are

A Power is a per-provider steering document (`POWER.md`) in the Kiro strict-5
frontmatter format (`name`, `displayName`, `description`, `keywords`, `author`
— exactly these five fields). Powers route Kiro's agent toward provider-specific
discipline: live-mutation guards, MLPS 2.0 compliance, EU sovereignty invariants,
and so on.

This repo ships **{{ site.data.catalog.providers }} Powers** under `powers/`, one per provider/board:

| Power directory | Provider |
|----------------|---------|
| `powers/vanguard-aws` | AWS |
| `powers/vanguard-azure` | Azure |
| `powers/vanguard-gcp` | Google Cloud |
| `powers/vanguard-oci` | Oracle Cloud |
| `powers/vanguard-alibaba` | Alibaba Cloud |
| `powers/vanguard-huawei` | Huawei Cloud |
| `powers/vanguard-ovhcloud` | OVHcloud |
| `powers/vanguard-scaleway` | Scaleway |
| `powers/vanguard-hetzner` | Hetzner |
| `powers/vanguard-contabo` | Contabo |
| `powers/vanguard-ionos` | IONOS |
| `powers/vanguard-kubernetes` | Kubernetes (cross-cloud) |
| `powers/vanguard-terraform` | Terraform (cross-cloud) |
| `powers/vanguard-nvidia` | NVIDIA / GPU compute |
| `powers/vanguard-argocd` | Argo CD (GitOps) |
| `powers/vanguard-fluxcd` | Flux CD (GitOps) |
| `powers/vanguard-istio` | Istio (service mesh) |
| `powers/vanguard-cilium` | Cilium (eBPF networking) |
| `powers/vanguard-kyverno` | Kyverno (policy) |
| `powers/vanguard-falco` | Falco (runtime security) |
| `powers/vanguard-sigstore` | Sigstore (supply chain) |
| `powers/vanguard-cert-manager` | cert-manager (PKI) |
| `powers/vanguard-opentelemetry` | OpenTelemetry (observability) |
| `powers/vanguard-prometheus` | Prometheus (observability) |
| `powers/vanguard-backstage` | Backstage (developer portal) |
| `powers/vanguard-dotnet` | .NET (language/runtime) |
| `powers/vanguard-java` | Java (language/runtime) |
| `powers/vanguard-kotlin` | Kotlin (JVM / Android / Multiplatform) |
| `powers/vanguard-frontend` | Frontend (web platform) |
| `powers/vanguard-php` | PHP (language/runtime) |
| `powers/vanguard-python` | Python (language/runtime + governed live control plane) |
| `powers/vanguard-typescript` | TypeScript (program + published package) |
| `powers/vanguard-generic` | Generic (cross-platform) |
| `powers/vanguard-multi-cloud` | Multi-cloud |
| `powers/vanguard-hr` | HR (professional function) |
| `powers/vanguard-legal` | Legal (professional function) |
| `powers/vanguard-marketing` | Marketing (professional function) |
| `powers/vanguard-salesforce` | Salesforce (CRM) |
| `powers/vanguard-netsuite` | Oracle NetSuite (ERP) |
| `powers/vanguard-atlassian` | Atlassian (Jira, Jira Service Management, Confluence) |
| `powers/vanguard-accounting` | Accounting (professional function) |
| `powers/vanguard-finance` | Finance (professional function) |
| `powers/vanguard-sap` | SAP (S/4HANA, BTP, enterprise) |
| `powers/vanguard-microsoft` | Microsoft 365 & Dynamics 365 |
| `powers/vanguard-databricks` | Databricks (lakehouse, governance, streaming, SQL, AI/BI, MLOps, GenAI, FinOps; + Azure-specific `-at-azure` assets) |
| `powers/vanguard-snowflake` | Snowflake (cloud-neutral data platform; review board + gated live guards) |

### Step-by-step

**Step 1 — clone the repo:**

```bash
git clone https://github.com/VincentChuWaiChow/vanguard-frontier-agentic
cd vanguard-frontier-agentic
```

**Step 2 — add Powers in Kiro:**

In Kiro, open the **Powers panel** → `Add Custom Power` → `Local Directory`.

Paste the absolute path for each Power you need, one at a time. For example:

```
/home/you/vanguard-frontier-agentic/powers/vanguard-aws
/home/you/vanguard-frontier-agentic/powers/vanguard-kubernetes
/home/you/vanguard-frontier-agentic/powers/vanguard-terraform
```

There is **no single-command install** for Kiro Powers — the UI requires a
separate add for each Power directory.

**Step 3 — add agent adapters (optional):**

Kiro agent adapters are exported via the npm path. After cloning:

```bash
npm install @raishin/vanguard-frontier-agentic@latest

# Export kiro-ide agent adapters (writes .kiro/agents/*.md)
npx vfa-export-agents --platform kiro-ide --all --repo .

# Or kiro-cli adapters (writes .kiro/agents/*.json)
npx vfa-export-agents --platform kiro-cli --all --repo .
```

### Docs reference

- [github.com/kirodotdev/powers](https://github.com/kirodotdev/powers) — Kiro Powers specification and strict-5 frontmatter contract

---

## 6. Gemini CLI / Google Antigravity

### Prerequisites

- Node.js 18+ (`node --version`)
- Gemini CLI installed (`gemini --version`)
- npm available on PATH

### How Antigravity reads skills

Antigravity's skills framework reads from two locations:

| Scope | Path |
|-------|------|
| Workspace (project-level) | `.agent/skills/<name>/SKILL.md` |
| Global (user-level) | `~/.gemini/antigravity/skills/<name>/` |

There is no first-party marketplace install command. Use the npm exporter.

### Step-by-step

```bash
# Step 1: install the package
npm install @raishin/vanguard-frontier-agentic@latest

# Step 2: export all agents + companion skills for Gemini Antigravity
npx vfa-export-agents --platform gemini --all --repo .
```

For a specific provider only:

```bash
npx vfa-export-agents --platform gemini --provider aws --repo .
```

### What gets exported

- **Workspace skills:** `.agent/skills/<name>/SKILL.md`
- **Gemini rules file:** `~/.gemini/GEMINI.md` (global scope)
- **Agent adapters:** `agents/<provider>/<agent>/harnesses/gemini.agent.md`
- **MCP:** configure MCP servers via Antigravity's MCP Store UI, which writes `mcp_config.json`

### Docs reference

- [antigravity.google](https://antigravity.google) — Antigravity skills framework
- [github.com/google-gemini/gemini-cli](https://github.com/google-gemini/gemini-cli) — Gemini CLI

---

## 7. OpenAI Codex

### Prerequisites

- Codex CLI installed (`codex --version`)
- Authenticated with `codex auth login`

### Option A — one-command install (recommended)

```bash
# Step 1: register the marketplace
codex plugin marketplace add VincentChuWaiChow/vanguard-frontier-agentic

# Step 2: enable the bundled plugin
/plugin install vanguard-frontier-agentic@vanguard-frontier-agentic
```

`codex plugin marketplace add` clones the marketplace registry from the canonical
path (`.agents/plugins/marketplace.json`) and writes the following block to
`~/.codex/config.toml`:

```toml
[marketplaces.vanguard-frontier-agentic]
last_updated = "2026-05-11T06:46:00Z"
last_revision = "<sha>"
source_type = "git"
source = "https://github.com/VincentChuWaiChow/vanguard-frontier-agentic.git"

[plugins."vanguard-frontier-agentic@vanguard-frontier-agentic"]
enabled = true
```

### Option B — manual config.toml (reproducible environments)

Create or update `~/.codex/config.toml`:

```toml
[marketplaces.vanguard-frontier-agentic]
source_type = "git"
source = "https://github.com/VincentChuWaiChow/vanguard-frontier-agentic.git"

[plugins."vanguard-frontier-agentic@vanguard-frontier-agentic"]
enabled = true
```

**Pinning to a specific tag:**

```toml
[marketplaces.vanguard-frontier-agentic]
source_type = "git"
source = "https://github.com/VincentChuWaiChow/vanguard-frontier-agentic.git"
ref = "v1.8.0"
```

### What gets installed

The marketplace registry (`/.agents/plugins/marketplace.json`) declares two plugins:

| Plugin | Manifest path | Description |
|--------|--------------|-------------|
| `vanguard-frontier-agentic` | `plugins/vanguard-frontier-agentic/.codex-plugin/plugin.json` | Main plugin — {{ site.data.catalog.agents }} agents |
| `cross-platform-agent-template` | `plugins/cross-platform-agent-template/.codex-plugin/plugin.json` | Scaffold for new cross-platform agents |

For the {{ site.data.catalog.agents }} Codex agent adapter files (`.codex/agents/*.toml`):

```bash
npx vfa-export-agents --platform codex --all --repo .
```

### Marketplace management commands

```bash
# Update to latest
codex plugin marketplace upgrade vanguard-frontier-agentic

# Remove
codex plugin marketplace remove vanguard-frontier-agentic

# List installed plugins
codex plugin list
```

### Docs reference

- [github.com/openai/codex](https://github.com/openai/codex) — Codex CLI
- [Codex plugin-json-spec](https://github.com/openai/codex/blob/main/codex-rs/skills/src/assets/samples/plugin-creator/references/plugin-json-spec.md) — plugin manifest schema

---

## 8. npm package + vfa-export-agents (universal)

Use this path for any harness not listed above, or when you need deterministic
versioned exports with full control over what gets written to your repo.

### Prerequisites

- Node.js 18+ (`node --version`)
- npm available on PATH

### Step-by-step

**Step 1 — install the package:**

```bash
# Latest release
npm install @raishin/vanguard-frontier-agentic@latest

# Pinned release (recommended for production)
npm install @raishin/vanguard-frontier-agentic@1.8.0
```

**Step 2 — export agents for your platform and role:**

```bash
# Export all agents for Claude Code
npx vfa-export-agents --platform claude-code --all --repo .

# Export by role (e.g., cloud-security-engineer)
npx vfa-export-agents --platform claude-code --role cloud-security-engineer --repo .

# Export by provider
npx vfa-export-agents --platform claude-code --provider aws --repo .

# Export specific agents by ID
npx vfa-export-agents --platform claude-code --agents aws-cost-optimization-agent,aws-iam-access-analyzer --repo .
```

### Platform values

| `--platform` value | AI harness |
|--------------------|------------|
| `claude-code` | Claude Code (Anthropic) |
| `codex` | OpenAI Codex CLI |
| `copilot` | GitHub Copilot CLI |
| `cursor` | Cursor IDE |
| `gemini` | Gemini CLI / Antigravity |
| `kiro` | Kiro (auto-selects `kiro-ide`) |
| `kiro-ide` | Kiro IDE adapter (`.kiro/agents/*.md`) |
| `kiro-cli` | Kiro CLI adapter (`.kiro/agents/*.json`) |

### All flags

| Flag | Values | Required | Purpose |
|------|--------|----------|---------|
| `--platform` | see table above | Yes (except `--list`) | Target AI harness |
| `--role` | string | No | Install curated role-based agent set |
| `--provider` | string | No | Narrow to one cloud provider |
| `--agents` | comma-separated IDs | No | Install specific agents by ID |
| `--all` | — | No | Install all agents for the platform |
| `--repo` | path | No | Target repo root (default: current directory) |
| `--force` | — | No | Overwrite existing files |
| `--list` | — | No | List available agents; do not write files |
| `--list-roles` | — | No | List available roles; do not write files |
| `--no-skills` | — | No | Skip companion skill files |

### Role catalogue

| Role | Agents installed |
|------|-----------------|
| `cloud-architect` | IAM, networking, multi-cloud, cost, Kubernetes |
| `cloud-security-engineer` | Security posture, zero-trust, compliance, secrets |
| `devops-engineer` | CI/CD, Terraform, Kubernetes, monitoring |
| `finops-engineer` | Cost optimization, rightsizing, reservation |
| `platform-engineer` | Infrastructure, cluster operations, observability |

Use `npx vfa-export-agents --list-roles` for the current list.

### What gets exported

- **Agent adapter files** for the selected platform (e.g.,
  `agents/<provider>/<agent>/harnesses/claude-code.agent.md` copied into your repo)
- **Companion skills** (SKILL.md files) alongside agents, unless `--no-skills`
- **Not exported automatically:** repo-level guidance files (`CLAUDE.md`,
  `AGENTS.md`, `.github/copilot-instructions.md`). Copy those manually if needed;
  see [`docs/normalized-platform-matrix.md`](../normalized-platform-matrix.md).

---

## 9. Skills CLI (third-party, exploration only)

> **Trust caveat:** The `skills` CLI pulls raw GitHub content from HEAD with no
> version pinning or integrity verification. Not recommended for shared or CI
> environments. See [`docs/integrations/skills-cli.md`](./skills-cli.md) for
> the full trust comparison.

### Install

```bash
npx skills add VincentChuWaiChow/vanguard-frontier-agentic
```

This fetches **skills only** (no agents, no rules, no MCP references) from the
default branch at the time of the command.

### Limitation

The `skills` CLI does not support pinning to a tag or SHA. If repeatability
matters, use the [npm path](#8-npm-package--vfa-export-agents-universal).

---

## 10. Pinning and reproducibility

### Why pin?

The default `master` branch receives continuous updates. Unpinned installs will
silently pull in new agents and behavior changes. Pin when:

- Running in CI/CD
- Sharing a setup across a team
- Auditing which agents are in scope

### Pinning by harness

| Harness | How to pin |
|---------|-----------|
| Claude Code | Set `"ref": "v1.8.0"` in `extraKnownMarketplaces` source |
| Copilot CLI | Replace `master` with tag in the raw URL |
| Cursor | `git checkout v1.8.0` in the cloned repo |
| Kiro | `git checkout v1.8.0` in the cloned repo |
| Gemini / npm | `npm install @raishin/vanguard-frontier-agentic@1.8.0` |
| Codex | Set `ref = "v1.8.0"` in `config.toml` |
| vfa-export-agents | Pin the npm package version |
| Skills CLI | **Not supported** — avoid for reproducible environments |

### Release cadence

Releases follow semantic versioning. Changelogs are auto-generated from
conventional commits via the release pipeline. See
[`docs/release-versioning.md`](../release-versioning.md) for the full policy.

---

## 11. Verification checklist

After any install, run through the following:

### Claude Code

```bash
/plugin list
# Expect: vanguard-frontier-agentic (enabled)

# Test an agent
"Use vanguard-aws-cost-optimization-agent to summarize idle EC2 in us-east-1."
```

### Copilot CLI

```bash
/plugin list
# Expect: vanguard-frontier-agentic (enabled)
```

### Cursor

Open the Agent panel. Search for `vanguard` — you should see provider-namespaced
agents from `aws`, `azure`, `gcp`, etc.

### Kiro

Open the Powers panel. Each added Power should appear with its `displayName`.
Ask Kiro: `"Which Powers are active?"` — it should list the ones you added.

### Gemini / Antigravity

```bash
ls .agent/skills/
# Expect: directories named after exported skills
```

### Codex

```bash
codex plugin list
# Expect: vanguard-frontier-agentic (enabled)
```

### npm export

```bash
npx vfa-export-agents --platform claude-code --list
# Expect: {{ site.data.catalog.agents }} agents listed, no file writes
```

---

## 12. Troubleshooting

### `plugin not found` (Claude Code)

The marketplace was added but the plugin ID is wrong. The ID is the key in the
`plugins[]` array of `.claude-plugin/plugin.json`, which is always
`vanguard-frontier-agentic@vanguard-frontier-agentic`. Run:

```bash
/plugin marketplace add VincentChuWaiChow/vanguard-frontier-agentic
/plugin install vanguard-frontier-agentic@vanguard-frontier-agentic
```

### `extraKnownMarketplaces` parse error (Copilot CLI)

Copilot CLI expects an **array**, not an object. Incorrect:

```json
{ "extraKnownMarketplaces": { "vanguard-frontier-agentic": { ... } } }
```

Correct:

```json
{ "extraKnownMarketplaces": ["https://raw.githubusercontent.com/.../marketplace.json"] }
```

### Cursor shows 0 agents

Check that you registered the directory containing `.cursor-plugin/plugin.json`
(the repo root), not a subdirectory. Cursor validates the manifest location.

### Kiro Power add fails — "invalid frontmatter"

Only the strict-5 fields are allowed: `name`, `displayName`, `description`,
`keywords`, `author`. Any other field (e.g., `version`, `license`, `tags`)
causes a parse failure. The Powers in this repo are pre-validated by
`npm run validate:kiro-powers`.

### Codex `config.toml` not updated

`codex plugin marketplace add` requires an authenticated session. Run
`codex auth login` first, then retry.

### npm export writes no files

Ensure you pass `--repo .` (or an explicit path). Without it, the exporter
defaults to a dry run. Add `--force` if files already exist and you want to
overwrite.

### Agents show outdated behavior after update

The harness may have cached the old manifest. In Claude Code: `/plugin
uninstall vanguard-frontier-agentic@vanguard-frontier-agentic` then reinstall.
In Cursor: restart the IDE after pulling the updated repo.

---

## See also

- [`docs/integrations/skills-cli.md`](./skills-cli.md) — trust matrix, pinning guidance, and pre-install inspection for all three npm/skills/exporter paths
- [`docs/compatibility.md`](../compatibility.md) — which harnesses are supported and what each adapter ships
- [`docs/normalized-platform-matrix.md`](../normalized-platform-matrix.md) — repo-level guidance files per harness (`CLAUDE.md`, `AGENTS.md`, `GEMINI.md`, etc.)
- [`docs/marketplace-model.md`](../marketplace-model.md) — how the plugin marketplace pattern works across harnesses
- [`README.md`](../../README.md#install-reference) — Install Reference section with the full `vfa-export-agents` argument matrix and role catalogue
