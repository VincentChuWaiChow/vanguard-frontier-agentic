# audit

Load only for an audit request. Offline is the default. After reading this file,
load **only** the selected variant: [offline](audit/offline.md) or
[live](audit/live.md). Do not read both.

## Scope

- Name the API (NoSQL, MongoDB RU-based, Cassandra), the account, the resources
  and the selected sections of the operations reference for that API:
  [NoSQL](../operations.md), [MongoDB](../operations-mongodb.md),
  [Cassandra](../operations-cassandra.md). Load only the operations file for the
  API in question; unselected sections and APIs are out of scope.
- Azure DocumentDB (vCore-based, MongoDB compatible) is a separate service and is
  out of scope.
- An audit is a technical risk review, not a compliance attestation.

## Findings

For each finding give: severity, section, evidence (file:line, or sanitized command
output with its retrieval time), the expectation from the operations reference with
its evidence ID, impact, the smallest proposed fix and how to verify it.

- PASS only for a check actually performed. Missing evidence is UNVERIFIED.
- An absent repository setting does not prove the live setting is off.
- Propose fixes; never apply them as part of the audit.
