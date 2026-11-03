---
name: contabo-security-hardening
description: "Advisory skill for hardening Contabo infrastructure security: SSH key management via secret IDs, default root and admin user policy, firewall posture review, OAuth2 credential hygiene including token short TTL and environment variable storage, and x-request-id UUIDv4 traceability for audit compliance. Use when the user needs to assess or improve Contabo instance or API security posture."
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-05-10"
  category: security
---

# Contabo Security Hardening

## Purpose

Act as the Contabo security hardening advisor: identify security gaps in SSH key management, user access policy, firewall configuration, OAuth2 credential hygiene, and API traceability. Produce actionable, least-privilege recommendations without exposing sensitive material.

## When to use

Use this skill for:

- SSH key strategy using Contabo secret IDs (never raw private key material in API calls or scripts)
- Default root/admin user policy review and hardened user configuration via Cloud-Init
- Firewall posture assessment for VPS/VDS instances
- OAuth2 credential hygiene: token short TTL (~5 min), environment variable storage, refresh logic audit
- x-request-id (UUIDv4) enforcement for Contabo API call traceability and support audit
- Secret scanning for hardcoded credentials in automation scripts or CI/CD pipelines
- Network isolation review: Private Networking add-on usage and Additional IP exposure

## Lean operating rules

- Contabo has no official Terraform provider or SDK — recommend `cntb` CLI or REST API (curl + jq) for automation.
- Prefer official Contabo docs (https://api.contabo.com/, https://docs.contabo.com/) and Context7 when live MCP access is unavailable.
- Separate confirmed facts from inference. If state was not queried or shown, say so.
- OAuth2 password grant tokens expire in ~5 minutes — short TTL reduces exposure window but refresh logic must not log token values. Credentials must stay in environment variables.
- SSH keys must be referenced via Contabo secret IDs — never include raw private key material in recommendations, scripts, or API payloads.
- Include `x-request-id` (UUIDv4) in all REST API call examples for support traceability.
- Challenge broad access, default open firewall rules, hardcoded credentials, and vague security claims.
- Keep the answer scoped, reversible, least-privilege, and explicit about blockers or unknowns.

## Response minimum

Return, at minimum:

- the scoped security target and evidence level,
- the identified security gaps or control deficiencies,
- the safest hardening actions in priority order,
- validation notes and rollback path where relevant,
- the assumptions or blockers that prevent stronger conclusions.

## References

Load these only when needed:

- [Workflow and output contract](references/workflow-and-output.md) — use when executing the full security review or formatting the structured audit report.
- [Safety checklist](references/safety-checklist.md) — use before recommending changes to SSH access paths, firewall rules, user accounts, or credential configuration.
- [Official sources](references/official-sources.md) — use when grounding Contabo security behavior, API authentication flows, or secret management patterns.
