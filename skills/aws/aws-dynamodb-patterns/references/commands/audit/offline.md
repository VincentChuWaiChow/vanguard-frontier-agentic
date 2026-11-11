# audit --mode offline

Read this transport only for offline audit; do not load live instructions or probe
profiles, credentials, endpoints or AWS metadata. Follow the shared [audit](../audit.md)
criteria using only the selected feature references.

```bash
python3 scripts/select_mode.py audit --mode offline --features iam,encryption,pitr
```

1. Establish repository paths/revision, selected features, intended targets and
   existing IaC/SDK ownership from local evidence. Do not assume repo intent equals
   deployed state. User-provided sanitized exports are snapshots, not live reads.
2. Inspect only relevant code/templates/request fixtures and supplied exports.
   Run authorized existing offline validators/tests without installing packages,
   contacting AWS, refreshing schemas online or executing deployment snippets.
3. Keep offline truly offline: do not fetch external documentation automatically.
   Use bundled/provided sources, label freshness gaps, or request a separate
   documentation lookup if current behavior is necessary for correctness.
4. Apply the shared selected-domain checks; cite exact paths/lines and snapshot
   provenance/time. Do not invent live index readiness, encryption access, restore,
   IAM enforcement, throughput or retrieval results.
5. Return confirmed local findings, proposed fixes, criterion-level evidence and
   UNVERIFIED live/runtime gaps. Excluded features stay out of scope. Do not
   remediate, benchmark or enable anything as part of this read-only audit.

An offline audit is a real review of local evidence, not a simulated live audit.
If required files/exports are missing, report that gap instead of switching to live.
