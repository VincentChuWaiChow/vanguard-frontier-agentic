# Authorized live metadata audit

Before any read, confirm tenant/subscription/principal, resource IDs, account API,
database/container/region, permitted fields and explicit live-read authorization.
Prefer existing safe MCP metadata tools if offered; discover their schema first.
Never infer permission from MCP availability or login state.

Use a scoped `az cosmosdb show`, container show/throughput show and approved role/
metric/backup-window metadata only. Explicit `--subscription`, resource group and
name on every request; never `list-keys`, broad subscription enumeration, item
queries/exports or credential output. If not authorized or tools unavailable,
return UNVERIFIED and offer offline evidence, not a login/install workaround.

Check returned resource ID/API/region and timestamps before interpreting fields.
Stop on target drift, unexpected authorization failure, partial errors or any
proposed mutation. Readback proves settings only, not fresh workload access,
performance, tenant isolation, key continuity or successful recovery.
