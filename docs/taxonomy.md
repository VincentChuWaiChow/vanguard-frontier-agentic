# Cloud Marketplace Taxonomy

## Providers

- `aws`
- `azure`
- `oci`
- `gcp`
- `alibaba`
- `huawei`
- `ovhcloud`
- `ionos`
- `scaleway`
- `hetzner`
- `contabo`
- `kubernetes`
- `argocd`
- `fluxcd`
- `istio`
- `cilium`
- `falco`
- `kyverno`
- `sigstore`
- `cert-manager`
- `opentelemetry`
- `prometheus`
- `nvidia`
- `backstage`
- `terraform`
- `multi-cloud`
- `generic`
- `frontend`
- `marketing`
- `dotnet`
- `java`
- `kotlin`
- `hr`
- `legal`
- `salesforce`
- `netsuite`
- `atlassian`
- `accounting`
- `finance`
- `sap`
- `microsoft`
- `databricks`
- `snowflake`
- `php`
- `python`
- `typescript`

## Language and stack boards

`provider` is a faceting axis. Cloud and platform boards (`aws`, `azure`,
`kubernetes`, ...) are the original members, but the axis also carries
non-cloud **topical and language/stack boards**: `frontend`, `marketing`, `dotnet`, `hr`,
`legal`, `salesforce`, `netsuite`, `atlassian`, `accounting`, `finance`, `sap`,
`microsoft`, `databricks`, `snowflake`, `php`, and `python` each have a dedicated `provider`
enum value, a shared ID prefix (`dotnet-*`, `hr-*`, `legal-*`, `salesforce-*`,
`netsuite-*`, `sap-*`, `microsoft-*`, `databricks-*`, `snowflake-*`, etc.),
and a dedicated topical directory under `agents/` and `skills/`. `microsoft`
covers the Microsoft 365 and Dynamics 365 estate; `databricks` ships a
cloud-neutral lakehouse and AI board alongside three Azure-specific `-at-azure`
assets, and `snowflake` covers the Snowflake data platform on Azure.

A topical board earns its own `provider` value once it ships a coherent
agent/skill set; until then a board uses `provider: generic` (the `qa` board
is the current example). Promoting a board from `generic` to a dedicated
provider keeps the ID prefix unchanged — only the `provider` field, the
schema/validator enums, and the catalog entries move.

See `docs/language-stack-boards.md` for detailed guidance on language/stack boards,
including how to add new ones, discovery via install roles, and the trust posture
of static-review agents.

## Asset types

- `skill`: workflow instructions for a recurring task.
- `agent`: role/persona definition with responsibilities and review behavior.
- `rule`: harness-specific operating guidance.
- `mcp-reference`: catalog entry for an MCP server or MCP setup path.

## Harnesses

- `codex`
- `copilot`
- `claude-code`
- `cursor`
- `gemini`
- `kiro`
- `other`

## Skill categories

Each `SKILL.md` may declare `metadata.category` for marketplace filtering. Categories are coarse and intentionally non-exhaustive — assign the single best fit, not multiple. New categories require a documented rationale and a schema update.

| Category | Scope |
|----------|-------|
| `security` | IAM, posture, secrets, KMS, identity, policy, RBAC, runtime threat, supply chain |
| `networking` | Service mesh, network policy, ingress, segmentation, DNS, private endpoints |
| `platform` | Cluster ops, compute, container platforms, storage, lifecycle automation |
| `data` | Databases, data modeling, query performance, replication, migration |
| `finops` | Cost, anomaly detection, budget, chargeback, optimization |
| `ai` | Generative AI, agents, model platforms, knowledge bases, guardrails |
| `delivery` | CI/CD, release, GitOps, progressive delivery, deployment guards |
| `observability` | Metrics, logs, traces, alerting, SLO, telemetry pipelines |
| `compliance` | Audit, evidence, governance, regulatory mapping |
| `resilience` | Backup, DR, BCDR, restore validation, recovery posture |

## Skill lifecycle

`metadata.lifecycle` declares stability:

- `experimental` — interface unstable, expect breaking changes
- `beta` — externally usable, breaking changes signalled
- `stable` — default for shipped skills; backwards-compatible changes only
- `deprecated` — scheduled for removal; replacement documented

Absence implies `stable`.

## Skill updated date

`metadata.updated` is an ISO 8601 date (`YYYY-MM-DD`) capturing the last meaningful change. "Meaningful" means substantive content, behavior, or contract changes — not whitespace, badge bumps, or transitive metadata churn. Refresh on each substantive edit.

## Trust levels

- **Official**: published by the cloud/provider or official project owner.
- **Community**: public third-party source with clear maintainer and license.
- **Original**: created for this repository.
- **Adapted**: derived from another source and license-reviewed.

Do not blur these categories. A community MCP server that targets AWS is not an official AWS MCP server.
