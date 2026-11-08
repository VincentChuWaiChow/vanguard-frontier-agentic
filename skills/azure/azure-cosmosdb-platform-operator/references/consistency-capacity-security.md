# Cosmos DB for NoSQL: consistency, capacity and data-plane security

Load this for consistency-level choice, session tokens, the integrated cache,
serverless vs provisioned vs autoscale, logical-partition limits, or Entra ID
data-plane RBAC. Every bracketed ID resolves in
[evidence-ledger.md](evidence-ledger.md). Text without an ID is labeled
inference or recommendation. API for NoSQL only.

## Consistency

| Level | Read RU cost | What it gives | Region-outage RPO (more than one region) | Evidence |
| --- | --- | --- | --- | --- |
| Strong | 2x | linearizable reads; writes commit in every region; **not available with multiple write regions**; spans over 5,000 miles blocked by default | 0 | [COSMOS-E02] [COSMOS-E17] [COSMOS-E18] |
| Bounded staleness | 2x | reads lag by at most K versions or T time | K and T | [COSMOS-E02] [COSMOS-E18] |
| Session | 1x | read-your-writes per session; the most widely used level | under 15 min | [COSMOS-E03] [COSMOS-E18] |
| Consistent prefix | 1x | transactional-batch writes become visible together; single-document writes behave eventually | under 15 min | [COSMOS-E04] [COSMOS-E18] |
| Eventual | 1x | staleness-tolerant reads | under 15 min | [COSMOS-E01] [COSMOS-E18] |

- Read consistency is scoped to one read within a logical partition
  [COSMOS-E01]. Cross-partition invariants need application design, not a
  stronger level. *(inference)*
- Strong across regions costs write latency of about twice the round trip to
  the farthest region plus 10 ms at p99 [COSMOS-E17]. Choosing multi-region
  writes rules Strong out entirely [COSMOS-E17]. Treat "multi-write + strong"
  as a contradiction to resolve, not a configuration. *(recommendation)*
- A single-region account has RPO under 240 minutes at every level
  [COSMOS-E18]. A consistency level is not a DR plan.

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
  [COSMOS-E05]. It is **preview**, **direct mode only** [COSMOS-E05], and
  documented only for Java SDK v4.69+ and .NET SDK v3.46+ [COSMOS-E06]. Do not
  recommend it for Python, JavaScript or Go, or for production paths that
  cannot accept preview behavior. *(recommendation)*
- Because it needs direct mode and the integrated cache needs gateway mode
  [COSMOS-E08], one client cannot use both. *(inference)*
- Read-path changes only: write RU cost does not change with level
  [COSMOS-E02].

## Integrated cache (dedicated gateway)

All three must hold for a cache hit [COSMOS-E08]:

1. the client uses the **dedicated gateway endpoint**;
2. the client uses **gateway mode**. .NET and Java default to direct mode and
   silently miss the cache [COSMOS-E08];
3. the read is **Session or Eventual**. Strong, Bounded staleness and
   Consistent prefix bypass it [COSMOS-E07].

Staleness is bounded by `MaxIntegratedCacheStaleness` (default 5 min). A value
of 0 means reads never use the cache, although writes still populate it
[COSMOS-E07] [COSMOS-E08]. Production use needs at least three gateway nodes
for SLA coverage, and the gateway carries no latency SLA [COSMOS-E09]. Reject
the cache when reads rarely repeat, the app needs Strong or Bounded staleness,
or a direct-mode app cannot switch. *(recommendation)*

## Capacity model

| Mode | Use when | Hard limits | Evidence |
| --- | --- | --- | --- |
| Provisioned (manual) | steady, predictable traffic | 429s if consumption exceeds provisioned RU/s | [COSMOS-E12] |
| Autoscale | variable traffic | scales 0.1 x Tmax to Tmax; billed at hourly peak (min 0.1 x Tmax) | [COSMOS-E13] |
| Serverless | intermittent, low average-to-peak | **cannot add regions after creation** while serverless; no throughput or latency guarantee; 5,000 RU/s per physical partition; no shared-throughput DB | [COSMOS-E10] [COSMOS-E11] [COSMOS-E19] |

Any mode is capped at **10,000 RU/s and 20 GB per logical partition**, and
work beyond a physical partition's share is rate-limited [COSMOS-E12]. A hot or
large partition-key value is a modeling defect that no capacity mode fixes.
Use hierarchical partition keys (up to three levels) when one key value would
exceed 20 GB [COSMOS-E12].

- Autoscale avoids 429s only while total consumption stays within the
  configured range [COSMOS-E13]. A hot partition can still be rate-limited
  [COSMOS-E12], and shared-database autoscale lets one container's burst
  throttle the others [COSMOS-E13]. Prefer container-level autoscale.
  *(recommendation)*
- A serverless account cannot add regions after creation [COSMOS-E10]. The
  only path to multi-region is an **in-place, one-time, irreversible**
  conversion to provisioned throughput, which has no duration SLA and blocks
  management operations while it runs [COSMOS-E20]. "Serverless now, DR
  later" therefore means a planned capacity-mode migration, an approved
  change window, and losing serverless billing for good. If DR is a known
  requirement, start on provisioned or autoscale. *(recommendation)*
- Prices are not in this ledger. A cost estimate must cite a freshly retrieved
  Azure pricing page or be labeled an assumption.

## Data-plane vs control-plane security

These are **two different RBAC systems**:

| Plane | Covers | Mechanism | Evidence |
| --- | --- | --- | --- |
| Data plane | item CRUD, queries, change feed, stored procedures, conflict feed | Cosmos DB native RBAC (`sqlRoleDefinitions`/`sqlRoleAssignments`); built-in Data Reader / Data Contributor | [COSMOS-E14] |
| Control plane | keys and connection strings, backup/restore, databases/containers, account properties | Azure RBAC | [COSMOS-E15] |

Review rules:

- An Azure RBAC assignment does not grant data access through native
  data-plane RBAC. Check both planes separately. *(inference from E14/E15)*
- **But keys bridge the planes.** A control-plane role that can read account
  keys or connection strings [COSMOS-E15] obtains a credential that works for
  data access unless key auth is disabled [COSMOS-E16]. Treat key-listing
  permissions as data access. *(inference)*
- Data-plane roles have no `notDataActions`. Build least privilege by listing
  allowed `dataActions` only [COSMOS-E14]. Scope to a database or container,
  not the account, where the workload allows. *(recommendation)*
- Data-plane assignments cannot be managed in the portal [COSMOS-E14]. Expect
  CLI, PowerShell or IaC evidence.
- Whoever holds control-plane `sqlRoleAssignments/write` can grant themselves
  data access [COSMOS-E14]. Treat that permission as data-sensitive.
  *(inference)*
- For Entra-only workloads, disable key auth with `disableLocalAuth=true` and
  verify that a key-based connection fails [COSMOS-E16]. That is an account
  mutation: it needs an explicit target, confirmation that no consumer still
  uses keys, a rollback plan and approval.
