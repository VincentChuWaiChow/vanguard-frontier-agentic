# Offline audit

Use only supplied/local sanitized code, DDL, Terraform, plans, policy/metric/export
metadata fixtures. No OCI command, data query, credentials or automatic external
lookup. Check query predicates/tenant binding, same-shard transaction assumptions,
version conflicts/retries, cursor binding, capacity/TTL, selected key/recovery
settings and IaC ownership. Source verification is a separately requested lookup.
Report file/line evidence; effective permissions/state/runtime stay UNVERIFIED.
