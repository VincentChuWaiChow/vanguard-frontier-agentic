# audit

Load only for an audit request. Offline is the default. After reading this file,
load **only** the selected variant: [offline](audit/offline.md) or
[live](audit/live.md). Do not read both.

## Scope

- Name the service (Scaleway Managed MongoDB®, `mongodb/v1` API), the Database
  Instances and the selected sections of [the operations reference](../operations.md).
  Unselected sections are out of scope.
- An audit is a technical risk review, not a compliance attestation.
- Live mode reads control-plane metadata only. It never opens a MongoDB connection
  and never reads customer data.

## Findings

For each finding give: severity, section, evidence (file:line, or sanitized command
output with its retrieval time), the expectation from the operations reference with
its evidence ID, impact, the smallest proposed fix and how to verify it.

- PASS only for a check actually performed. Missing evidence is UNVERIFIED.
- An absent repository setting does not prove the live setting is off.
- Propose fixes; never apply them as part of the audit.
