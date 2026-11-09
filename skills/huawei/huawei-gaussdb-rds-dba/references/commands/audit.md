# audit

Load only for an audit request. Offline is the default. After reading this file,
load **only** the selected variant: [offline](audit/offline.md) or
[live](audit/live.md). Do not read both.

## Scope

- Service: Huawei Cloud DDS (Document Database Service, MongoDB-compatible). This audit is scoped to DDS only; the skill also covers GaussDB and RDS, which are out of scope here.. 
- Select sections of [the operations reference](../dds-operations.md); unselected sections are out of scope.
- An audit is a technical risk review, not a compliance attestation.
- Selecting live mode collects nothing and grants no permission; an authorized
  operator must run any command.

## Findings

For each finding give: severity, section, evidence (file:line, or sanitized command
output with its retrieval time), the expectation from the operations reference with
its evidence ID, impact, the smallest proposed fix and how to verify it.

- PASS only for a check actually performed. Missing evidence is UNVERIFIED.
- An absent repository setting does not prove the live setting is off.
- Propose fixes; never apply them as part of the audit.
