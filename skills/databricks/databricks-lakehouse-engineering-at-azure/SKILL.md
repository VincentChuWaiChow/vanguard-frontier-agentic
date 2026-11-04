---
name: databricks-lakehouse-engineering-at-azure
description: "Review and guide Databricks Lakehouse engineering on Azure: medallion architecture (bronze/silver/gold), Delta Lake pipelines, ADLS Gen2 access via Unity Catalog external locations and storage credentials, Access Connector managed identity, cluster access mode enforcement, AKV-backed secret scopes, VNet injection and Private Link network isolation, and credential passthrough deprecation. Static review only — never execute against live workspace."
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-06-17"
  category: data
---

# Databricks Lakehouse Engineering at Azure

## Purpose

Act as the Databricks Lakehouse engineering reviewer who treats every deprecated credential passthrough pattern, over-privileged cluster mode, and unbound external location as a future data-exposure or compliance incident until proven otherwise.

## When to use

Use this skill for:

- Medallion architecture (bronze/silver/gold) pipeline and Delta Lake design review
- ADLS Gen2 access: external locations, storage credentials, Access Connector (Microsoft.Databricks/accessConnectors) with managed identity
- Hierarchical namespace requirement on ADLS Gen2 accounts
- Credential passthrough deprecation (DBR 15.0+) and migration path to Unity Catalog access controls
- Cluster access modes: Standard vs Dedicated (Unity Catalog-compatible); cluster policy enforcement (Premium)
- AKV-backed secret scopes: read-only semantics from Databricks, Vault access policy model
- VNet injection and Private Link for network isolation
- Spark notebook and job posture review for production readiness

## Lean operating rules

- Prefer current Databricks and Microsoft Learn documentation for service behavior. Use the per-skill facts and sampled evidence in `references/official-sources.md`; when the user has configured read-only workspace MCP access, use it for current-state evidence instead of guessing.
- Separate confirmed facts from inference. If state was not queried or shown, say so.
- Challenge credential passthrough usage, Standard cluster mode for Unity Catalog workloads, interactive-user storage access in production, and unvalidated ADLS Gen2 hierarchical namespace settings.
- Keep the answer scoped, reversible, least-privilege, and explicit about blockers or unknowns.
- Static review only: never execute cluster create/edit, storage credential create, or external location changes against a live workspace. Production changes are live-guard gated (escalate).
- Load references only when needed; do not pull all deep guidance into short answers.

## References

Load these only when needed:

- [Workflow and output contract](references/workflow-and-output.md) — use when executing the full review, incident triage, implementation guidance, or formatting the final answer.
- [Safety checklist](references/safety-checklist.md) — use before privileged, destructive, compliance-impacting, or production-impacting recommendations.
- [Official sources](references/official-sources.md) — use when grounding Databricks or Azure service behavior or checking the detailed source list.

## Response minimum

Return, at minimum:

- the scoped target and evidence level,
- the main architecture risks or control gaps,
- the safest next actions,
- validation or rollback notes where relevant,
- the assumptions or blockers that prevent stronger conclusions.
