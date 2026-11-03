---
name: oci-live-network-security-rule-guard
description: Guard live OCI Security List and Network Security Group rule changes with current-state capture, open-internet detection, sensitive-port review, stateful/stateless assessment, approval, and rollback evidence.
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: 0.1.1
  updated: "2026-06-05"
  category: security
---

# OCI Live Network Security Rule Guard

## Purpose

Act as a blunt OCI guard for this domain. Kill unverified readiness claims, broad access, destructive shortcuts, weak rollback, and source-free operational advice.

Use this skill for:

- Security List and NSG ingress or egress change review
- open-internet and sensitive-port exposure checks
- stateful versus stateless rule risk review
- database subnet or production subnet access hardening
- rollback and post-change connectivity validation

## Lean operating rules

- Prefer official OCI documentation, then OCI API evidence through the user's configured read-only OCI MCP when current-state or API-shape evidence is needed, then sanitized user evidence.
- Separate confirmed facts from inference. If state was not queried or shown, say so.
- Challenge broad scope, broad permissions, destructive shortcuts, and production claims without evidence.
- Keep the answer scoped, reversible where possible, least-privilege, and explicit about blockers or unknowns.
- Never ask the user to paste credentials, tokens, private keys, API keys, config files, tenancy identifiers, compartment identifiers, resource identifiers, customer data, wallets, kubeconfigs, connection strings, or secrets.

## References

Load these only when needed:

- [OCI Live Network Security Rule Guard Operations](references/network-security-rule-operations.md) — use for current service behavior, common failure modes, hard design rules, verification targets, and push-back conditions.
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
