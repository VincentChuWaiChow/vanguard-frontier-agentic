---
layout: default
title: "Governance"
permalink: /docs/governance/
---

# 🏛️ Governance

How decisions are made, who has authority, and what quality gates must pass.

---

## 📐 Decision-Making: Architecture Decision Records (ADRs)

Significant technical decisions are documented as ADRs in `docs/adr/`.

### ADR Process

1. **Propose** - Create a new ADR file with status "Proposed"
2. **Discuss** - Open a PR, get feedback
3. **Accept/Reject** - Maintainer makes final call
4. **Implement** - Status moves to "Accepted" once the decision is enacted

### Current ADRs

| # | Title | Status |
|---|-------|--------|
| 0001 | [Initial Architecture (Three-Layer Maestro)](adr/0001-initial-architecture/) | Accepted |
| 0002 | [Documentation Site with Jekyll + GitHub Pages](adr/0002-documentation-site-with-jekyll-github-pages/) | Accepted |
| 0003 | [Vendor-Neutral Agentic AI Board](adr/0003-agentic-ai-board/) | Proposed |

### When to Write an ADR

Write an ADR when:
- Adding a new architectural layer or subsystem
- Changing the build/release pipeline
- Adopting or removing a tool/framework
- Changing security posture (new trust boundaries)
- Any decision that is hard to reverse

---

## ⚖️ Maintainer Responsibilities

### Code Ownership

File: `.github/CODEOWNERS`

All paths default to `@VincentChuWaiChow`. Critical infrastructure paths have explicit ownership:

| Path | Owner | Scope |
|------|-------|-------|
| `/.github/` | @VincentChuWaiChow | CI/CD, workflows, templates |
| `/scripts/` | @VincentChuWaiChow | Build and generation scripts |
| `/schemas/` | @VincentChuWaiChow | JSON Schema contracts |
| `/catalog/` | @VincentChuWaiChow | Machine-readable indexes |
| `/tests/` | @VincentChuWaiChow | Validation scripts |
| `/CLAUDE.md` | @VincentChuWaiChow | Steering file |

### Review Requirements

Every PR requires:
- At least one approving review from a CODEOWNERS match
- All CI checks passing ({{ site.data.catalog.validation_gates }} validation gates + fuzz tests)
- No unresolved review comments

---

## 🚀 Release Authority

Releases are fully automated. No human manually triggers a release under normal operation.

### How Releases Happen

1. A PR with `fix:` or `feat:` commits merges to master
2. semantic-release analyzes commits
3. If releasable commits exist, a new version is published

### Who Can Release

- Anyone who can merge to master (requires CODEOWNERS approval)
- The release itself is automated (no manual npm publish)
- Recovery actions (republish) require Actions workflow_dispatch access

### What Prevents Bad Releases

- Branch protection: no direct pushes to master
- CODEOWNERS: review required from designated owners
- {{ site.data.catalog.validation_gates }} validation gates must pass
- semantic-release only processes conventional commits
- OIDC: no stored credentials to steal

---

## 🔒 Security Response

Full policy: `SECURITY.md`

### SLA

| Action | Timeline |
|--------|----------|
| Acknowledge report | 48 hours |
| Assess severity | 7 days |
| Patch (high/critical) | Current + previous minor |
| Patch (medium/low) | Current minor only |
| Versions < 2.5.0 | No patches |

### Reporting Channel

Private reporting via GitHub Security Advisories only:

> [https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/security/advisories/new](https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/security/advisories/new)

Public disclosure of vulnerabilities before a fix is available is not acceptable.

---

## 🧪 Quality Gates

Every PR must pass these gates before merge:

### CI Validation (`.github/workflows/ci.yml`)

- Catalog validation
- Skill manifest check
- Asset integrity
- MCP trust matrix
- No lifecycle scripts
- Link validation (offline)

### Fuzz Tests (separate CI job)

- Property-based testing via fast-check

### Additional Workflows

| Workflow | Purpose |
|---------|---------|
| `install-paths-smoke.yml` | npm install works end-to-end |
| `packed-artifact-smoke.yml` | npm pack produces valid tarball |
| `provider-scope-regression.yml` | New providers do not break existing ones |
| `codeql.yml` | Static analysis for vulnerabilities |
| `scorecard.yml` | OpenSSF Scorecard compliance |
| `docs-quality.yml` | Markdown lint + spell check |

### Gate Enforcement

Gates are configured as **required** status checks in branch protection. A PR cannot merge if any required check fails.

---

## 📏 Code Review Standards

### What reviewers check

- [ ] Conventional commit messages (`feat:`, `fix:`, `chore:`, `docs:`)
- [ ] Schema compliance for new skills/agents
- [ ] No lifecycle scripts introduced
- [ ] No secrets or credentials in content
- [ ] Routing scenarios added for new providers
- [ ] Manifests regenerated if structure changed
- [ ] Asset integrity regenerated if critical files changed

### What automated checks catch

- Schema violations (validate:skill-schema, validate:agent-schema)
- Stale manifests (manifest:check, validate:plugin-manifest)
- Broken links (validate:links)
- Integrity violations (validate:asset-integrity)

---

## ✅ How to Verify This Works

```bash
# Confirm branch protection is enforced (check via GitHub API or UI)
# Settings > Branches > master > Protection rules

# Verify CODEOWNERS
cat .github/CODEOWNERS

# Run all quality gates locally
npm run validate && npm run test:fuzz

# Check security policy
cat SECURITY.md
```

---

## 🏛️ Enterprise Reviewer Notes

- Single maintainer (@VincentChuWaiChow) is the current governance model; this is appropriate for the project's maturity stage
- No manual release capability exists outside of workflow_dispatch (which is logged and auditable)
- CODEOWNERS + branch protection provide a hard gate; advisory reviews are insufficient for this project
- The ADR process is lightweight (Markdown files in `docs/adr/`) rather than requiring a formal RFC process
