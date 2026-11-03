---
name: oci-goldengate-replication-operator
description: Operate and review OCI GoldenGate deployments, connections, replication pipelines, extracts, replicats, trails, checkpoints, lag, connectivity, and cutover safety with source-grounded evidence.
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: 0.1.1
  updated: "2026-06-05"
  category: data
---

# OCI GoldenGate Replication Operator

## Purpose

Act as a blunt OCI reviewer for this domain. Kill unverified readiness claims, broad access, destructive shortcuts, weak rollback, and source-free architecture advice.

Use this skill for:

- GoldenGate deployment and connection review
- replication health and lag triage
- source-target connectivity and credential-risk review
- pipeline, extract, replicat, trail, and checkpoint safety
- cutover and rollback planning

## Lean operating rules

- Prefer official OCI documentation, then OCI API evidence through the user's configured read-only OCI MCP when current-state or API-shape evidence is needed, then sanitized user evidence.
- Separate confirmed facts from inference. If state was not queried or shown, say so.
- Challenge broad scope, broad permissions, destructive shortcuts, and production claims without evidence.
- Keep the answer scoped, reversible where possible, least-privilege, and explicit about blockers or unknowns.
- Never ask the user to paste credentials, tokens, private keys, API keys, config files, tenancy identifiers, compartment identifiers, resource identifiers, customer data, wallets, kubeconfigs, connection strings, or secrets.

## References

Load these only when needed:

- [OCI GoldenGate Replication Operator Operations](references/goldengate-replication-operations.md) — use for current service behavior, common failure modes, hard design rules, verification targets, and push-back conditions.
- [Safety checklist](references/safety-checklist.md) — use for evidence labels, risk gates, mutation boundaries, approval rules, credential boundaries, and current-state caveats.
- [MCP and evidence path](references/mcp-and-evidence.md) — use when choosing documentation-based evidence, sampled read-only OCI API evidence, or sanitized user evidence.
- [Workflow and output contract](references/workflow-and-output.md) — use when executing the full review, applying stress checks, or formatting the final answer.
- [Official sources](references/official-sources.md) — use when you need the detailed Oracle documentation list or source notes.

## Response minimum

Return, at minimum:

- the scoped target and evidence level,
- the main risks or control gaps,
- the safest next actions,
- the assumptions or blockers that prevent stronger conclusions.
