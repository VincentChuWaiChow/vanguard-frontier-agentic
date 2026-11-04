---
name: oci-storage-backup-steward
description: Operate as an OCI storage and backup steward for Object Storage, Block Volume, File Storage, backup policies, retention, replication, lifecycle rules, restore readiness, and IAM-scoped storage operations.
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: 0.1.1
  updated: "2026-06-05"
  category: resilience
---

# OCI Storage Backup Steward

## Purpose

Steward OCI storage and backup posture with source-grounded checks for Object Storage, Block Volume, File Storage, retention, lifecycle rules, replication, restore testing, and least-privilege storage access.

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

- `references/oci-storage-backup-steward-operations.md` — service shape, wrong assumptions, operating rules, and pushback triggers.
- `references/safety-checklist.md` — risk gates, mutation boundaries, credential boundaries, and evidence labels.
- `references/mcp-and-evidence.md` — official docs versus sampled OCI API evidence discipline.
- `references/workflow-and-output.md` — execution flow and final response contract.
- `references/official-sources.md` — official OCI source URLs and current evidence notes.

## Response Contract

Return `verdict`, `evidence_level`, `blockers`, `safe_next_actions`, and `open_questions`. Be blunt when evidence is insufficient.
