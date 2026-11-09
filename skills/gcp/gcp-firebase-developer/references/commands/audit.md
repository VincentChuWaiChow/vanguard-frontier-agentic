# audit

Load only for an audit request. Offline is the default. After reading this file,
load **only** the selected variant: [offline](audit/offline.md) or
[live](audit/live.md). Do not read both.

## Scope

- This skill covers all of Firebase, but the audit covers **Firestore only**:
  Native mode ([Native operations](../firestore-operations.md)) and Firestore with
  MongoDB compatibility ([MongoDB operations](../firestore-mongodb-operations.md)).
  Authentication, Hosting, Functions, Realtime Database and Storage are out of scope.
- Name the project, database ID, location, the mode (Native or MongoDB compatibility)
  and the selected sections of the matching operations reference. The two modes are
  separate: do not carry a Native-mode expectation into a MongoDB-compatible database.
  Unselected sections are out of scope.
- An audit is a technical risk review, not a compliance attestation.

## Findings

For each finding give: severity, section, mode, evidence (file:line, or sanitized
command output with its retrieval time), the expectation from the operations
reference with its evidence ID, impact, the smallest proposed fix and how to verify it.

- PASS only for a check actually performed. Missing evidence is UNVERIFIED.
- An absent repository setting does not prove the live setting is off.
- Propose fixes; never apply them as part of the audit.
