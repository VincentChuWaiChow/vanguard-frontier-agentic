---
layout: default
title: "Documentation"
permalink: /docs/
---

# 📊 Vanguard Frontier Agentic Documentation

Enterprise AI agent ecosystem providing **{{ site.data.catalog.skills }} skills**, **{{ site.data.catalog.agents }} agents**, and **{{ site.data.catalog.providers }} providers** with first-class multi-harness support.

Package: `@raishin/vanguard-frontier-agentic` v{{ site.data.catalog.version }} (Apache-2.0)

---

## 🛰️ What This Is

A curated, validated catalog of AI agent skills and agents designed for enterprise cloud operations, developer platforms, and business functions. Every asset passes {{ site.data.catalog.validation_gates }} automated validation gates before merge. Every release ships with npm provenance, SLSA Build L3 attestations, and an SPDX SBOM.

Supported harnesses: Claude Code, Codex, GitHub Copilot, Cursor, Gemini CLI, Kiro.

---

## 🗂️ Documentation Map

### 🚀 Getting Started

| Page | Description |
|------|-------------|
| [Getting Started](getting-started/) | Installation, first use, CLI tools |
| [Usage Examples](usage-examples/) | Maestro routing, live-guard agents, least-privilege access |
| [Configuration Reference](configuration/) | All settings, schemas, npm scripts |
| [FAQ](faq/) | Common questions answered with evidence |

### 🏗️ Architecture and Design

| Page | Description |
|------|-------------|
| [Architecture](architecture/) | Three-layer system, Mermaid diagrams, design rationale |
| [Language and Stack Boards](language-stack-boards/) | Topical boards, role bundles, execution-tier contract |
| [The Databricks Board](databricks-board/) | Cloud-neutral lakehouse and AI board, routing, roles, safety posture |
| [Governance](governance/) | ADR process, maintainer responsibilities, quality gates |
| [ADR-0001: Initial Architecture](adr/0001-initial-architecture/) | Three-layer Maestro decision record |
| [ADR-0002: Jekyll + GitHub Pages](adr/0002-documentation-site-with-jekyll-github-pages/) | Documentation platform choice |
| [ADR-0003: Agentic AI Board](adr/0003-agentic-ai-board/) | Vendor-neutral agentic board (proposed) |

### 📋 Operations

| Page | Description |
|------|-------------|
| [Deployment](deployment/) | npm publishing, OIDC, provenance, SLSA, SBOM |
| [Security](security/) | Supply chain, scanning, badges, threat model |
| [Testing](testing/) | All {{ site.data.catalog.validation_gates }} validation gates, fuzz testing, routing scenarios |
| [Operations Runbook](operations-runbook/) | Release process, recovery, checklists |
| [Troubleshooting](troubleshooting/) | Common failures and their fixes |

### 🤝 Contributing

| Page | Description |
|------|-------------|
| [Contributing](contributing/) | How to add docs, conventions, local preview |
| [Agentic Delegation](agentic-delegation/) | How work is split across models, and the executable workflow that encodes it |
| [GitHub Pages Setup](github-pages/) | How the docs site is built and deployed |
| [Roadmap](roadmap/) | Current state, planned work, how to propose changes |

---

## ⚡ Quick Reference

```bash
# Install the package
npm install @raishin/vanguard-frontier-agentic

# Export agents for a specific platform
npx vfa-export-agents --platform claude --role cloud-security-engineer

# Run all validation gates
npm run validate

# Run fuzz tests
npm run test:fuzz
```

---

## 📊 Catalog Stats

| Metric | Count |
|--------|-------|
| Skills | {{ site.data.catalog.skills }} |
| Agents | {{ site.data.catalog.agents }} |
| Providers | {{ site.data.catalog.providers }} |
| MCP References | {{ site.data.catalog.mcp_references }} |
| Rules | {{ site.data.catalog.rules }} |
| Install Roles | {{ site.data.catalog.install_roles }} |
| Validation Gates | {{ site.data.catalog.validation_gates }} |
| Maestro Routing Scenarios | {{ site.data.catalog.maestro_scenarios }} |

---

## 🌐 Provider Taxonomy

Agents are organized across {{ site.data.catalog.providers }} providers grouped into categories:

{% for group in site.data.catalog.provider_taxonomy %}
**{{ group.category }}**

| Provider | Agents |
|----------|-------:|{% for p in group.providers %}
| `{{ p.name }}` | {{ p.agents }} |{% endfor %}

{% endfor %}

---

## 🏛️ Enterprise Reviewer Notes

- Every claim in these docs is backed by a file path or command you can verify yourself
- Items marked `[NEEDS OWNER INPUT]` require maintainer clarification
- Security posture is documented in [security.md](security/) with adversarial framing
- Supply chain integrity is verifiable: `npm audit signatures` confirms provenance
