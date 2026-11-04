---
name: sap-integration-suite-review
description: "Review SAP Integration Suite topology and configuration: Cloud Integration iFlows, API Management policies, Event Mesh topics and queues, OAuth/certificate security, error handling, idempotency patterns, and monitoring. Flags security gaps, missing error handling, and idempotency violations. Does not touch live systems."
allowed-tools: Read Grep Glob WebSearch WebFetch
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-06-19"
  category: platform
  lifecycle: experimental
---

# SAP Integration Suite Review

## Purpose

Assess the design, security, and operational posture of SAP Integration Suite components. Review Cloud Integration iFlow design for error handling completeness, idempotency, adapter security, and message processing patterns. Review API Management proxy configuration for OAuth policy correctness, rate limiting, quota enforcement, and threat protection. Review SAP Event Mesh topic and queue design for access control, consumer group isolation, and dead-letter handling. Surface security gaps, missing operational guardrails, and integration anti-patterns. Does not connect to or mutate any live Integration Suite tenant.

## When to use

Use this skill when the user asks to:

- review Cloud Integration (CPI) iFlow design for error handling, retry logic, idempotency, exception subprocess coverage, or message mapping correctness,
- assess API Management proxy configuration including OAuth policy enforcement, rate limiting, quota plans, JWT validation, threat protection, and API product design,
- evaluate SAP Event Mesh topic configuration, namespace design, queue bindings, consumer group isolation, and dead-letter queue handling,
- audit Integration Suite security posture: OAuth client credential flows, certificate-based adapter authentication, credential store usage, and absence of plaintext secrets,
- review monitoring and alerting configuration: Cloud Integration message monitoring, alert rules, retry exhaustion handling, and operational visibility,
- flag integration anti-patterns: synchronous fallback in async iFlows, missing correlation ID propagation, absent receiver determination, or unclear message routing,
- assess OData or REST API exposure patterns from Cloud Integration or API Management against standard OData V4 or OpenAPI 3.0 conventions.

## When not to use

- When the user needs live inspection of a running iFlow, deployed API proxy, or Event Mesh namespace — this skill accepts only user-provided iFlow exports, API proxy descriptors, or configuration descriptions.
- When the request is about BTP account-level governance (entitlements, subaccounts, role collections) — use `sap-btp-governance-review`.
- When the request is about ABAP integration patterns inside S/4HANA rather than Integration Suite middleware — use `sap-clean-core-debt-review`.
- When the request is about end-to-end identity and access management — use `sap-security-iam-grc-sod-review`.

## Does not touch live systems

This skill operates on user-provided iFlow export bundles (.iflw files), API proxy descriptors, Event Mesh configuration exports, architecture descriptions, monitoring screenshots, or configuration summaries. It does not connect to any Cloud Integration tenant, API Management portal, Event Mesh namespace, or SAP BTP service API. All live inspection is out of scope.

## Lean operating rules

- Classify integration issues before recommending. Every finding must be assigned to a component domain (Cloud Integration / API Management / Event Mesh / Security / Monitoring) before remediation is proposed.
- Error handling is non-negotiable. Every iFlow that processes business-critical messages must have an exception subprocess. An iFlow with no exception handling is a `high` severity finding.
- Idempotency is required for all retry-capable integrations. Any iFlow that uses JMS retry, polling adapter, or webhook delivery without idempotency controls (duplicate check, idempotent receiver, or business-key deduplication) is a `high` severity finding.
- OAuth is the required API Management security baseline. API proxies without an OAuth or API key policy enforced at the inbound step are a `critical` finding. Basic auth on inbound proxy endpoints is not acceptable.
- Credentials must use the Credential Store. Plaintext credentials in iFlow configuration properties, message mappings, or API proxy flows are a `critical` finding. All credentials must reference the Integration Suite Secure Store or SAP Credential Store service.
- OData and OpenAPI exposure must match the declared standard. OData V4 services exposed via Cloud Integration or API Management must conform to OData V4 metadata conventions. OpenAPI 3.0 specs exposed via API Management must accurately describe the deployed API behavior.
- Event Mesh access control must use role-based bindings. Event Mesh service instance bindings must use dedicated service keys per consumer; shared service keys across multiple consumers are a governance risk.
- Correlation ID propagation is required for end-to-end traceability. iFlows that do not propagate a correlation ID (SAP-MessageId header or business key) across adapter boundaries impede incident diagnosis.
- Evidence from user-provided artifacts or official SAP Integration Suite documentation takes precedence over inference.
- Context7 CAP documentation applies supplementarily when reviewing OData service exposure patterns from CAP-based services integrated via Integration Suite.

## Evidence rules

Label all claims with one of:

- `documentation-based` — grounded in SAP Integration Suite Help Portal, Cloud Integration docs, API Management docs, or Event Mesh docs
- `user-provided evidence` — iFlow exports, API proxy descriptors, Event Mesh config exports, monitoring screenshots, or descriptions provided by the user
- `context7-supplementary` — grounded in CAP OData/OpenAPI framework docs fetched from Context7 (supplementary; applies only when CAP-based OData or OpenAPI service integration is in scope)
- `inference` — derived reasoning not directly confirmed by official docs or user evidence

## Live-environment rules

**This skill does not touch live systems.** There is no Cloud Integration API call, API Management portal request, Event Mesh management API invocation, or BTP service binding access in this skill's execution path. Users must supply iFlow export bundles, API proxy descriptors, Event Mesh configuration exports, or written descriptions of their Integration Suite topology for this skill to review.

## References

Load only when needed:

- [Workflow and output contract](references/workflow-and-output.md) — integration issue taxonomy, severity classification, output format.
- [Safety checklist](references/safety-checklist.md) — non-negotiables, common mistakes, when to push back.
- [Official sources](references/official-sources.md) — SAP Integration Suite, Cloud Integration, API Management, Event Mesh, and security documentation.
- [Context7 framework docs](references/context7-framework-docs.md) — CAP OData service and event handling patterns (supplementary; use when CAP-based service integration is in scope).

## Response minimum

Return, at minimum:

- **Problem classification**: Integration Suite component(s) in scope and specific finding(s) per component.
- **Evidence used**: documentation-based / user-provided evidence / context7-supplementary / inference.
- **Risk level**: critical (security or data loss risk) / high (operational reliability risk) / medium (governance gap) / low (best practice deviation).
- **Recommended action**: specific remediation per finding (exception subprocess addition, idempotency control, OAuth policy enforcement, credential store migration, etc.).
- **Refusal / escalation triggers**: if live tenant access, iFlow deployment, or API proxy deployment is required to complete the review, state that clearly and do not proceed.
- **Business impact**: integration failure blast radius, data loss risk, security exposure, or compliance gap.
- **Next verification step**: validate recommended changes against the current Integration Suite tenant configuration before deploying.
