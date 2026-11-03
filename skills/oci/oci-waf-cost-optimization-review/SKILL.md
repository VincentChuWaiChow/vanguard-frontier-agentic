---
name: oci-waf-cost-optimization-review
description: Review OCI Well-Architected cost optimization posture across usage, budgets, tags, idle resources, rightsizing, quotas, forecasts, and Cloud Advisor recommendations.
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: 0.1.1
  updated: "2026-06-05"
  category: finops
---

# OCI WAF Cost Optimization Review

## Purpose

Review OCI Well-Architected cost posture with documented Cost Analysis, Budgets, Cloud Advisor, usage API, tagging, ownership, forecast caveats, and safe-change approval gates.

## Use When

- Reviewing this OCI domain for stale, missing, risky, vague, over-permissive, or under-specified guidance.
- Comparing current official OCI documentation against existing operational assumptions.
- Producing a source-grounded advisory that separates documented behavior, sampled configured-environment evidence, inference, and unknowns.

## Operating Rules

- Keep primary responses lean; put detailed service behavior and caveats in references.
- Use official OCI documentation for documented service behavior.
- Use OCI API evidence through the user’s configured read-only OCI MCP only for command shape or sanitized sampled current-state observations.
- Do not mention local connector names, internal tool names, profile names, account-specific identifiers, or environment-specific paths.
- Never ask for credentials, tokens, tenancy details, compartment or resource identifiers, customer data, private keys, wallets, or config contents.
- Require explicit approval before any mutation or external support-channel action.

## Reference Pack

- `references/oci-waf-cost-optimization-review-operations.md` — service shape, wrong assumptions, operating rules, and pushback triggers.
- `references/safety-checklist.md` — risk gates, mutation boundaries, credential boundaries, and evidence labels.
- `references/mcp-and-evidence.md` — official docs versus sampled OCI API evidence discipline.
- `references/workflow-and-output.md` — execution flow and final response contract.
- `references/official-sources.md` — official OCI source URLs and current evidence notes.

## Response Contract

Return `verdict`, `evidence_level`, `blockers`, `safe_next_actions`, and `open_questions`. Be blunt when evidence is insufficient.
