---
name: analytics-data-minimization-review
description: "Use this skill when reviewing analytics platform configuration — GA4 property settings, BigQuery export schema, custom event-parameter definitions, and user-property declarations — for data-minimization violations, excessive collection, and storage-period over-retention. Trigger when a user provides a GA4 property configuration export, a BigQuery raw-event export schema, a custom event or user-property inventory, data-retention settings, or asks whether their analytics setup collects more personal data than necessary, retains data longer than required, or converts an analytics platform into a personal-data processor. Distinct from marketing-pixel-data-leakage-review: this skill reviews what analytics platforms collect and retain internally, not outbound pixel payloads to ad networks."
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-05-17"
  category: data
  lifecycle: experimental
---

# Analytics Data-Minimization Review

## Purpose
This skill reviews analytics platform configuration — GA4 property settings, BigQuery export schema, custom event-parameter definitions, and user-property declarations — for data-minimization violations, excessive collection, and storage-period over-retention. Analytics platforms are a primary regulatory surface for GDPR enforcement: European DPAs (Austrian DSB, French CNIL, Italian Garante) have found that user_pseudo_id, IP address, and precise geo combined with a BigQuery export constitute transfers of personal data requiring a lawful basis, a valid transfer mechanism, and compliance with the storage-limitation principle under GDPR Article 5(1)(e). This skill is distinct from `marketing-pixel-data-leakage-review` — it reviews what analytics platforms collect and retain internally (schema, user properties, retention periods), not outbound pixel payloads transmitted to ad networks. The review works from sanitized configuration exports only; never request live analytics data or real user identifiers.

## Lean operating rules
- Treat a GA4 user-scoped custom dimension populated with a persistent first-party user ID linked to a CRM contact record as HIGH — it converts GA4 into a personal-data processor for identified individuals, triggering DPA obligations and requiring a separate documented lawful basis beyond the analytics purpose.
- Treat a BigQuery raw-event export retaining user_pseudo_id and geo.city at full precision with no anonymization transform or partitioned deletion job as HIGH — the combination of fields constitutes personal data under GDPR, and uncontrolled raw export creates an unmanaged data store with no retention ceiling.
- Treat a data-retention period set to the maximum (14 months in GA4) with no documented justification tied to a specific, time-bound analytical purpose as HIGH — GDPR Article 5(1)(e) requires retention only as long as necessary; the maximum is not a default entitlement.
- Treat user properties collecting device fingerprint components, precise IP, or persistent advertising identifiers (GCLID, FBCLID passed as user properties) in a property lacking a valid transfer mechanism for non-EEA exports as HIGH — these fields individually or in combination constitute personal data with cross-border transfer obligations.
- Treat event parameters collecting free-text field values from search queries, form inputs, or support chats as HIGH — free-text fields frequently contain names, emails, or health information that exceed the analytics collection purpose.
- Treat session-scoped custom dimensions collecting full URL paths that include query parameters with PII (e.g., `/reset?email=user@example.com`) as HIGH — URL-embedded PII is personal data regardless of whether it was intentionally collected.
- Flag custom event schemas that duplicate standard GA4 automatically collected events with additional parameters adding no documented analytical value as MEDIUM — redundant collection without justification violates data minimization under GDPR Article 5(1)(c).
- Flag BigQuery export schemas that retain raw event data beyond the property's configured retention period because no partition-expiry or scheduled query enforces deletion as MEDIUM — the property setting does not automatically govern the export.
- Flag user-property schemas with no documented owner, purpose, or review date as MEDIUM — absence of governance documentation is a proxy indicator of speculative or abandoned collection.
- Do not recommend disabling an event or parameter without naming the analytical purpose it serves and the impact of its removal on measurement continuity.
- Label every finding with evidence basis: configuration export provided, schema provided, documentation-based, or inference from missing element.

## References
Load these only when needed:
- [Workflow and output contract](references/workflow-and-output.md) — use when executing the full review or formatting the final answer.

## Response minimum
Return, at minimum:
- User-scoped custom dimension assessment (CRM linkage, persistent identifiers)
- BigQuery export schema assessment (field precision, anonymization, partitioned deletion)
- Data-retention period assessment (documented justification vs. maximum default)
- User-property and event-parameter PII assessment (free-text, URL-embedded PII, fingerprint components)
- Cross-border transfer assessment (user_pseudo_id + geo fields in non-EEA export)
- Schema governance assessment (owner, purpose, review date)
- Severity-labelled finding list (critical / high / medium / low)
- Safe next actions
