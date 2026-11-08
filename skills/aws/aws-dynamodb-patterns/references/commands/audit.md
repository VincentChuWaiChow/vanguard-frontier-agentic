# audit

Load only for audit mode. Use selected feature references, not the entire skill.
Default to offline repository/sanitized evidence. This is a technical risk review,
not a compliance attestation. After this shared review procedure load **only** the
selected transport: [offline](audit/offline.md) or [live metadata](audit/live.md).
Do not read both. The selector returns the chosen variant explicitly.

```text
/aws-dynamodb-patterns audit --mode offline --features iam,encryption
/aws-dynamodb-patterns audit --mode live --features pitr,monitoring
```

The real local equivalent is `python3 scripts/select_mode.py audit --mode live`.
Offline is the default. Old --evidence repo/live-metadata spellings remain aliases;
mixing --mode and --evidence is rejected. Selecting live requests an authorized
evidence workflow; it does not perform or authorize AWS calls itself.

## Evidence and scope

Record selected features, application/access patterns, IaC owner and inspected
paths/revisions. For live metadata, first confirm approved account/role, Region,
table/index and time window; capture retrieval time and distinguish observed
settings from repository intent. The selector does not gather evidence.

Do not read credentials/customer items, scan tables, enable Contributor Insights,
change capacity, create alarms, run load or repair findings as part of audit.
Existing CLI guides provide bounded metadata commands when separately authorized.
Missing evidence is UNVERIFIED; an absent repository setting does not prove a live
feature is disabled. Unselected features remain explicitly out of scope.

## Review selected domains only

- **Core/indexes:** match each access pattern to a complete key/request; examine
  hot-path scans, filtering, pagination/cursor ownership, consistency, hot-key/skew
  assumptions, sparse-index transitions and conditional/idempotent writes.
- **IAM/tags/resource policy/encryption:** compare actual action/resource scopes,
  trust/tenant boundaries and KMS dependencies; separate runtime from administration.
  Tag filters or vector partition filters do not independently prove isolation.
- **TTL/PITR/replicas:** check expiry authorization, observed recovery windows,
  backup/restore evidence, consistency/topology and recovery runbooks. Enabled
  settings or replica ACTIVE state do not prove a successful restore/failover.
- **Streams/DAX:** inspect ordering/retry/checkpoint/idempotency, lag/failure paths,
  cache freshness, client/network constraints and separate execution/service roles.
- **Vectors/embeddings:** check same-model/configuration and dimensions, precision,
  source/version lineage, authorization, update/delete freshness and retrieval
  evaluation. Matching dimensions or a self-match test is insufficient evidence.
- **Monitoring:** inspect dimensions/statistics, known throttles/retries, client
  versus service latency, alarms/notification ownership and missing-data behavior.

If a selected feature has a material dependency outside scope, name the gap and
request the narrow extension needed; do not silently load or audit every domain.

## Return findings, not automatic fixes

For each finding give severity, selected feature, exact file/line or sanitized
metadata evidence, violated invariant, concrete impact, smallest proposed fix and
verification. Separate confirmed defects from assumptions and missing evidence.
Use PASS only for a criterion actually checked; do not label the entire system
healthy from one template or a keyword checklist.

Finish with selected scope, inspected evidence/time, findings, PASS/MISS/UNVERIFIED
criteria, unselected scope and next approval needed. Propose remediation separately
through feature-plan or existing implementation workflow. No remediation executes
merely because audit discovered a defect.
