# Cosmos DB for NoSQL: consistency, capacity and data-plane security

Load this for consistency-level choice, session tokens, the integrated cache,
serverless vs provisioned vs autoscale, logical-partition limits, or Entra ID
data-plane RBAC. Every bracketed ID resolves in
[evidence-ledger.md](evidence-ledger.md). Text without an ID is labeled
inference or recommendation. API for NoSQL only.

## Consistency

| Level | Read RU cost | Typical fit | Evidence |
| --- | --- | --- | --- |
| Strong | 2x | linearizable reads; writes commit in every region | [COSMOS-E01] [COSMOS-E02] |
| Bounded staleness | 2x | bounded lag with ordering | [COSMOS-E01] [COSMOS-E02] |
| Session | 1x | read-your-writes per session; the usual default | [COSMOS-E01] [COSMOS-E03] |
| Consistent prefix | 1x | ordered, gap-free reads; batch writes visible together | [COSMOS-E01] [COSMOS-E04] |
| Eventual | 1x | staleness-tolerant reads | [COSMOS-E01] |

Read consistency is scoped to one read within a logical partition
[COSMOS-E01]. Cross-partition invariants need application design, not a
stronger level. *(inference)*

### Session tokens: where read-your-writes quietly breaks

- Tokens are **partition-bound**. A client that never wrote to a physical
  partition has no token for it and reads it as eventual [COSMOS-E03].
- A **re-created client** (pod restart, per-request client construction,
  serverless cold start) loses its token cache and reads as eventual until its
  writes rebuild it [COSMOS-E03]. Per-request `CosmosClient` construction
  therefore also breaks session guarantees, on top of its connection cost.
  *(inference)*
- Read-your-writes across services (a write in API A, then a read in worker B)
  requires passing the session token, **unmodified** [COSMOS-E03].

### Overriding consistency per request

- `ConsistencyLevel` overrides can only **relax** below the account default
  [COSMOS-E05].
- `ReadConsistencyStrategy` can also **strengthen** above it (for example
  `LatestCommitted`, `GlobalStrong`) and wins when both are set
  [COSMOS-E05]. SDK version and language support were not verified
  [COSMOS-E06]. Check the SDK changelog before recommending it.
- Read-path changes only: write RU cost does not change with level
  [COSMOS-E02].

## Integrated cache (dedicated gateway)

All three must hold for a cache hit [COSMOS-E08]:

1. the client uses the **dedicated gateway endpoint**;
2. the client uses **gateway mode**. .NET and Java default to direct mode and
   silently miss the cache [COSMOS-E08];
3. the read is **Session or Eventual**. Strong, Bounded staleness and
   Consistent prefix bypass it [COSMOS-E07].

Staleness is bounded by `MaxIntegratedCacheStaleness` (default 5 min; 0 = never
cached) [COSMOS-E08]. Production use needs at least three gateway nodes for SLA
coverage, and the gateway carries no latency SLA [COSMOS-E09]. Reject the cache
when reads rarely repeat, the app needs Strong or Bounded staleness, or a
direct-mode app cannot switch. *(recommendation)*

## Capacity model

| Mode | Use when | Hard limits | Evidence |
| --- | --- | --- | --- |
| Provisioned (manual) | steady, predictable traffic | 429s if consumption exceeds provisioned RU/s | [COSMOS-E13] |
| Autoscale | variable traffic | scales 0.1 x Tmax to Tmax; billed at hourly peak (min 0.1 x Tmax) | [COSMOS-E13] |
| Serverless | intermittent, low average-to-peak | **single region forever**; no throughput or latency guarantee; no shared-throughput DB | [COSMOS-E10] [COSMOS-E11] |

Any mode is capped at **10,000 RU/s and 20 GB per logical partition**
[COSMOS-E12]. A hot or large partition-key value is a modeling defect that no
capacity mode fixes. Use hierarchical partition keys (up to three levels) when
one key value would exceed 20 GB [COSMOS-E12].

- Serverless is a one-way door for geo-distribution: the account can never
  add regions [COSMOS-E10]. Reject it for anything with a DR or multi-region
  requirement. *(recommendation)*
- Prefer container-level autoscale over shared-database autoscale
  [COSMOS-E13].
- Prices are not in this ledger. A cost estimate must cite a freshly retrieved
  Azure pricing page or be labeled an assumption.

## Data-plane vs control-plane security

These are **two different RBAC systems**:

| Plane | Covers | Mechanism | Evidence |
| --- | --- | --- | --- |
| Data plane | item CRUD, queries, change feed, stored procedures, conflict feed | Cosmos DB native RBAC (`sqlRoleDefinitions`/`sqlRoleAssignments`); built-in Data Reader / Data Contributor | [COSMOS-E14] |
| Control plane | keys and connection strings, backup/restore, databases/containers, account properties | Azure RBAC | [COSMOS-E15] |

Review rules:

- An Azure RBAC assignment does not grant data access through the native
  data-plane RBAC. Check both planes separately. *(inference from E14/E15)*
- Data-plane roles have no `notDataActions`. Build least privilege by listing
  allowed `dataActions` only [COSMOS-E14]. Scope to a database or container,
  not the account, where the workload allows. *(recommendation)*
- Data-plane assignments cannot be managed in the portal [COSMOS-E14]. Expect
  CLI, PowerShell or IaC evidence.
- Whoever holds control-plane `sqlRoleAssignments/write` can grant themselves
  data access [COSMOS-E14]. Treat that permission as data-sensitive.
  *(inference)*
- Key access is control-plane [COSMOS-E15]. For Entra-only workloads, disable
  key auth with `disableLocalAuth=true` and verify that a key-based connection
  fails [COSMOS-E16]. That is an account mutation: it needs an explicit target,
  confirmation that no consumer still uses keys, a rollback plan and approval.
