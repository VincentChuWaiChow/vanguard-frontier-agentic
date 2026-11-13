# `powers/` — Kiro Powers

This directory holds **46 Kiro Powers** for `vanguard-frontier-agentic`, one
per cloud/platform/IaC provider. Each Power is a directory containing a
`POWER.md` file with strict-5 frontmatter and steering content.

## What's in here

```
powers/
├── vanguard-accounting/POWER.md
├── vanguard-alibaba/POWER.md
├── vanguard-argocd/POWER.md
├── vanguard-atlassian/POWER.md
├── vanguard-aws/POWER.md
├── vanguard-azure/POWER.md
├── vanguard-backstage/POWER.md
├── vanguard-cert-manager/POWER.md
├── vanguard-cilium/POWER.md
├── vanguard-contabo/POWER.md
├── vanguard-databricks/POWER.md
├── vanguard-dotnet/POWER.md
├── vanguard-falco/POWER.md
├── vanguard-finance/POWER.md
├── vanguard-fluxcd/POWER.md
├── vanguard-frontend/POWER.md
├── vanguard-gcp/POWER.md
├── vanguard-generic/POWER.md
├── vanguard-hetzner/POWER.md
├── vanguard-hr/POWER.md
├── vanguard-huawei/POWER.md
├── vanguard-ionos/POWER.md
├── vanguard-istio/POWER.md
├── vanguard-java/POWER.md
├── vanguard-kotlin/POWER.md
├── vanguard-kubernetes/POWER.md
├── vanguard-kyverno/POWER.md
├── vanguard-legal/POWER.md
├── vanguard-marketing/POWER.md
├── vanguard-microsoft/POWER.md
├── vanguard-multi-cloud/POWER.md
├── vanguard-netsuite/POWER.md
├── vanguard-nvidia/POWER.md
├── vanguard-oci/POWER.md
├── vanguard-opentelemetry/POWER.md
├── vanguard-ovhcloud/POWER.md
├── vanguard-php/POWER.md
├── vanguard-prometheus/POWER.md
├── vanguard-python/POWER.md
├── vanguard-salesforce/POWER.md
├── vanguard-sap/POWER.md
├── vanguard-scaleway/POWER.md
├── vanguard-sigstore/POWER.md
├── vanguard-snowflake/POWER.md
├── vanguard-terraform/POWER.md
└── vanguard-typescript/POWER.md
```

Each `POWER.md` declares:

- **Frontmatter (strict-5):** `name`, `displayName`, `description` (≤ 3
  sentences), `keywords` (specific, non-broad), `author`. **No other fields
  permitted** by Kiro spec.
- **Body steering:** when to engage, routing pattern (`<provider>-maestro`),
  live-mutation discipline, provider-specific invariants (e.g. MLPS 2.0 for
  Alibaba/Huawei, Enterprise Project vs IAM scope for Huawei, account-ID
  /region confirmation for AWS).

## How users install

Kiro Powers don't have a one-command marketplace install — the Powers panel
is per-Power directory add. Users clone the repo and add each Power they
need via the Kiro UI:

```bash
# 1. Clone the repo
git clone https://github.com/VincentChuWaiChow/vanguard-frontier-agentic
cd vanguard-frontier-agentic
```

```text
2. In Kiro:
   Open the Powers panel → "Add Custom Power" → "Local Directory"
   Paste the absolute path to the Power(s) you need:
      /absolute/path/to/vanguard-frontier-agentic/powers/vanguard-aws
      /absolute/path/to/vanguard-frontier-agentic/powers/vanguard-kubernetes
   Repeat for each provider you work with.
```

## How to update

```bash
# Regenerate the 46 Powers from catalog/agents.json + per-provider config:
npm run kiro-powers:write

# Then verify everything is in sync:
npm run validate:kiro-powers
```

The `validate` chain runs `validate:kiro-powers` automatically. The
validator enforces:

- strict-5 frontmatter (any extra field fails)
- lowercase kebab-case names
- name matches directory name
- description ≤ 3 sentences (decimal-aware — "MLPS 2.0" doesn't count as a
  sentence break)
- non-empty keywords list, no broad terms (`cloud`, `devops`, `code`,
  `agent`, `ml`, etc.) per Kiro's anti-false-activation guidance
- generator in sync (`--check`)

## Schema references (official Kiro docs)

- **Kiro Powers repo:** <https://github.com/kirodotdev/powers>
- **POWER.md frontmatter spec:**
  <https://github.com/kirodotdev/powers/blob/main/power-builder/POWER.md>
- **Interactive power builder:**
  <https://github.com/kirodotdev/powers/blob/main/power-builder/steering/interactive.md>
- **Testing a power locally:**
  <https://github.com/kirodotdev/powers/blob/main/power-builder/steering/testing.md>
- **Kiro IDE:** <https://kiro.dev/>

## Design notes

- **One Power per provider, not one mega-Power** — Kiro docs warn that
  broad keywords trigger false activations across unrelated tasks. One
  narrowly-scoped Power per provider keeps activation precise:
  `vanguard-alibaba` activates on Alibaba Cloud work only; `vanguard-aws`
  never activates on Azure questions.
- **Hetzner and Contabo Powers exist** even though their agents don't yet
  ship Kiro adapter files (their `harnesses: [codex, claude-code]`). Powers
  are steering-first — the steering content stands alone. When their Kiro
  adapter files land, the Powers will gain agent-routing as well.
- **No `version`, `repository`, `license`, or `tags`** — Kiro spec
  explicitly forbids these fields in frontmatter. The validator fails on
  any extra field.
