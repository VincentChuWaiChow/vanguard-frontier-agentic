# Offline audit

Use only authorized local code/templates and sanitized supplied metadata. No Azure
MCP resource discovery, CLI provider calls, SDK requests, credential/environment
dumps, or automatic external lookup. User-requested current research is separate
and must contain no private resource/item values.

Check selected feature invariants against actual artifacts: partition/id routing,
tenant parameterization and ETags; index/throughput/consistency assumptions; data
vs control roles; encryption/key dependencies; backup mode and restore plan;
vector dimensions/model/policy and metrics. A desired-state template does not
prove deployment, effective permissions, network connectivity or runtime behavior.

Return file/line evidence and proposed tests. Label metadata snapshots stale when
their date/context is unknown. Do not read customer items to fill an evidence gap.
