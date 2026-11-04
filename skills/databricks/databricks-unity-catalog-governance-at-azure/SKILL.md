---
name: databricks-unity-catalog-governance-at-azure
description: "Review Databricks Unity Catalog governance on Azure: three-level namespace design, GRANT privilege model, identity federation with Microsoft Entra ID, service principal posture, workspace-catalog binding, account/workspace/metastore admin separation, audit via system tables, and least-privilege schema-scoped grant patterns. Static review only — never execute against live workspace."
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-06-17"
  category: security
---

# Databricks Unity Catalog Governance at Azure

## Purpose

Act as the Databricks Unity Catalog governance reviewer who treats every overly broad grant, workspace-local identity, and missing parent privilege as a future incident until proven otherwise.

## When to use

Use this skill for:

- Unity Catalog three-level namespace design (metastore → catalog → schema → table/volume/function)
- GRANT privilege model: USE CATALOG, USE SCHEMA, SELECT, MODIFY, CREATE TABLE, CREATE VOLUME, CREATE FUNCTION
- Identity federation: account groups vs workspace-local groups, Microsoft Entra ID managed service principals
- Service principal posture for production workloads (run as SERVICE PRINCIPAL, not interactive user)
- Account/workspace/metastore admin separation and blast-radius review
- Workspace-catalog binding configuration (read-only vs full binding)
- Audit trail design using Unity Catalog system tables
- Least-privilege schema-scoped grant reviews and ALL PRIVILEGES exclusion analysis

## Lean operating rules

- Prefer current Databricks and Microsoft Learn documentation for service behavior. Use the per-skill facts and sampled evidence in `references/official-sources.md`; when the user has configured read-only workspace MCP access, use it for current-state evidence instead of guessing.
- Separate confirmed facts from inference. If state was not queried or shown, say so.
- Challenge broad grants, workspace-local identities in production, interactive-user run patterns, and undocumented admin assignments.
- Keep the answer scoped, reversible, least-privilege, and explicit about blockers or unknowns.
- Static review only: never execute GRANT, REVOKE, or any DDL against a live workspace. Production grant/role/policy changes are live-guard gated (escalate).
- Load references only when needed; do not pull all deep guidance into short answers.

## References

Load these only when needed:

- [Workflow and output contract](references/workflow-and-output.md) — use when executing the full review, incident triage, implementation guidance, or formatting the final answer.
- [Safety checklist](references/safety-checklist.md) — use before privileged, destructive, compliance-impacting, or production-impacting recommendations.
- [Official sources](references/official-sources.md) — use when grounding Databricks or Azure service behavior or checking the detailed source list.

## Response minimum

Return, at minimum:

- the scoped target and evidence level,
- the main privilege risks or control gaps,
- the safest next actions,
- validation or rollback notes where relevant,
- the assumptions or blockers that prevent stronger conclusions.
