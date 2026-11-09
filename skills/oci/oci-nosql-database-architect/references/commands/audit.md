# audit

Load only for an audit request. Offline is the default. After reading this file,
load **only** the selected variant: [offline](audit/offline.md) or
[live](audit/live.md). Do not read both.

## Scope

- Name the service (Oracle NoSQL Database Cloud Service, not Autonomous Database),
  the tenancy, region, compartment, tables and the selected sections of
  [the operations reference](../operations.md). Unselected sections are out of scope.
- An audit is a technical risk review, not a compliance attestation.
- Offline reads the repository (Terraform, DDL files, policy statements, client
  code) and sanitized exports. Live runs read-only OCI CLI commands by an
  authorized operator; selecting live mode grants no permission by itself.

## Findings

For each finding give: severity, section, evidence (file:line, or sanitized command
output with its retrieval time), the expectation from the operations reference with
its evidence ID, impact, the smallest proposed fix and how to verify it.

- PASS only for a check actually performed. Missing evidence is UNVERIFIED.
- An absent repository setting does not prove the live setting is off.
- Several sections of the operations reference are PARTIAL or UNVERIFIED (no native
  backup, no documented private endpoint). Report those as documented gaps, not as
  findings against the deployment.
- Propose fixes; never apply them as part of the audit.
