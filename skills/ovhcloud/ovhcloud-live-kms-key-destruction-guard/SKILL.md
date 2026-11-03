---
name: ovhcloud-live-kms-key-destruction-guard
description: "Gate and audit OVHcloud KMS key version destruction requests by enforcing five mandatory checks: confirmed key ID and KMS service URN, named approving identity, usage audit confirming zero active references within the retention window, documented waiting period, and a rollback or data-recovery plan. Use when a user requests destruction or rotation of an OVHcloud KMS key version. Hard-stop if any gate is absent or ambiguous."
allowed-tools: Read Grep Glob Bash
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-05-10"
  category: security
---

# OVHcloud Live KMS Key Destruction Guard

## Purpose

Act as the approval gate for OVHcloud KMS key version destruction. All five mandatory checks must pass before producing a destruction plan. KMS key destruction is irreversible; encrypted data is permanently unrecoverable if the key is destroyed while still in use.

## When to use

Use this skill when:

- A user requests destruction or scheduled deletion of an OVHcloud KMS key version
- A key rotation workflow requires decommissioning an old key version
- An audit or compliance review identifies a key that should be retired

## Lean operating rules

- Prefer OVHcloud KMS docs and Terraform provider docs; if MCP tooling is unavailable, fall back to https://help.ovhcloud.com/ and Context7.
- **HARD STOP** — refuse to produce a destruction plan if any of the five mandatory gates are absent or ambiguous:
  1. Exact key ID and KMS service URN of the target key version.
  2. Named, authenticated approving identity (not just a role or alias).
  3. Usage audit result confirming zero active references within the retention window.
  4. Documented waiting period (as required by OVHcloud KMS policy or organizational standard).
  5. Documented rollback plan or confirmed data recovery path for any data encrypted under this key.
- Never accept vague intent ("just delete it") as a gate pass.
- Never ask for actual encryption key material, OAuth2 client secrets, or application keys.
- After all gates pass, output the destruction plan for human review — do not execute automatically.

## References

Load these only when needed:

- [Workflow and output contract](references/workflow-and-output.md) — use when processing a destruction request or formatting the gate verdict and destruction plan.
- [Safety checklist](references/safety-checklist.md) — use before every destruction request; contains hard-stops and mandatory posture for this live-guard skill.
- [Official sources](references/official-sources.md) — use when grounding OVHcloud KMS service behavior or checking the source list.

## Response minimum

Return, at minimum:

- the gate-by-gate verdict (pass/fail with evidence for each gate),
- the evidence level for each gate assertion,
- a hard-stop message if any gate fails,
- the full destruction plan only when all five gates pass,
- explicit rollback or recovery steps in the destruction plan.
