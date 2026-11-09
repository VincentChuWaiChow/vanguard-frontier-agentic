# Primary-source ledger

Retrieved **2026-11-08**; scope NoSQL API. Source retrieval is documentation
evidence, not Azure account/SDK/deployment evidence. See machine-readable
[tool evidence](../evals/source-evidence.json). No customer/account data was sent.

Actual tools used: `mcp__context7__resolve_library_id` then `query_docs` against
`/websites/learn_microsoft_en-us_azure_cosmos-db`; MicrosoftDocs
`microsoft_docs_search`/`microsoft_docs_fetch`; Azure `documentation` router
`learn=true` and `microsoft_code_sample_search`. No Cosmos resource-discovery tool
was invoked. Azure extension/codegen-bestpractice helpers mentioned by its shared
description weren't exposed in this tool catalog; no installation/fake tool
fallback. Official docs retrieval plus offline checks were the available path.

| Subject | Primary reference | Applied observation |
|---|---|---|
| API/SDK scope | [NoSQL overview](https://learn.microsoft.com/azure/cosmos-db/overview), [Rust quickstart](https://learn.microsoft.com/azure/cosmos-db/quickstart-rust) | JSON/NoSQL scope; Rust SDK is first-party public preview, not GA |
| Routing | [Partitioning](https://learn.microsoft.com/azure/cosmos-db/partitioning-overview) | Logical/physical distribution and hot-key limits |
| Transactions | [Transactional batch](https://learn.microsoft.com/azure/cosmos-db/transactional-batch) | Same logical partition/container; 100 ops/2 MB/5 seconds |
| Consistency | [Levels](https://learn.microsoft.com/azure/cosmos-db/consistency-levels), [management/strategy](https://learn.microsoft.com/azure/cosmos-db/how-to-manage-consistency) | Classic override vs newer preview stronger strategies; session tokens |
| Pagination | [Query pagination](https://learn.microsoft.com/cosmos-db/query/pagination) | Empty pages/opaque tokens and updated Python streamable support |
| TTL | [TTL](https://learn.microsoft.com/azure/cosmos-db/time-to-live) | Container/item overrides and modification-based expiry |
| Native vectors | [Vector search](https://learn.microsoft.com/azure/cosmos-db/vector-search), [index policy](https://learn.microsoft.com/azure/cosmos-db/index-policy) | float16 now documented; flat505 vs quantized/diskANN4096; population/lifecycle limits |
| Embeddings | [Azure OpenAI](https://learn.microsoft.com/azure/ai-foundry/openai/how-to/embeddings) | Generation is separate from storing numeric vectors/source refs |
| Data/control RBAC | [Entra/RBAC](https://learn.microsoft.com/azure/cosmos-db/how-to-setup-rbac) | Separate Azure management and native Cosmos data roles |
| Key lifecycle | [CMK](https://learn.microsoft.com/azure/cosmos-db/how-to-setup-cmk) | Identity/Key Vault and periodic/continuous recovery dependencies |
| Feed | [Change-feed modes](https://learn.microsoft.com/azure/cosmos-db/change-feed-modes) | Latest version misses deletes; all versions/deletes requires continuous backup |
| Metrics | [Monitor reference](https://learn.microsoft.com/azure/cosmos-db/monitor-reference) | Deprecated generic server latency vs Direct/Gateway current metrics |
| Backup | [Modes](https://learn.microsoft.com/azure/cosmos-db/online-backup-and-restore), [continuous](https://learn.microsoft.com/azure/cosmos-db/continuous-backup-restore-introduction), [periodic](https://learn.microsoft.com/azure/cosmos-db/periodic-backup-restore-introduction) | Service now documents 7/30/35-day continuous tiers; periodic is support-assisted |
| Snapshot policy | [Periodic settings](https://learn.microsoft.com/azure/cosmos-db/periodic-backup-modify-interval-retention) | Bounds/defaults/redundancy change recovery risks |
| Recovery | [New-account](https://learn.microsoft.com/azure/cosmos-db/restore-account-continuous-backup), [deleted-resource](https://learn.microsoft.com/azure/cosmos-db/how-to-restore-in-account-continuous-backup), [latest timestamp](https://learn.microsoft.com/azure/cosmos-db/get-latest-restore-timestamp), [Support](https://learn.microsoft.com/azure/cosmos-db/periodic-backup-request-data-restore) | Distinct destinations, UTC/window/async/settings/TTL checks |
| Provisioning | [Account ARM 2025-04-15](https://learn.microsoft.com/azure/templates/microsoft.documentdb/2025-04-15/databaseaccounts), [container ARM](https://learn.microsoft.com/azure/templates/microsoft.documentdb/2025-04-15/databaseaccounts/sqldatabases/containers), [CLI](https://learn.microsoft.com/cli/azure/cosmosdb), [container CLI](https://learn.microsoft.com/cli/azure/cosmosdb/sql/container) | Complete pinned core example; current schema/version subset |
| JS SDK | [Query examples](https://learn.microsoft.com/javascript/api/overview/azure/cosmos-readme), [RequestOptions](https://learn.microsoft.com/javascript/api/@azure/cosmos/requestoptions) | fetchNext/partition/session/IfMatch signatures; no compilation claim |
| REST/Java | [REST query](https://learn.microsoft.com/rest/api/cosmos-db/query-documents), [REST auth](https://learn.microsoft.com/rest/api/cosmos-db/access-control-on-cosmosdb-resources), [Java container](https://learn.microsoft.com/java/api/com.azure.cosmos.cosmosasynccontainer) | Portable request/SDK contracts, not invented PHP/Kotlin-native SDKs |

## Observed source conflicts / limits

- [Same-account restore introduction](https://learn.microsoft.com/azure/cosmos-db/restore-in-account-continuous-backup-introduction)
  explicitly invalidates old tokens/consumer checkpoints and doesn't restore
  pre-deletion feed history. This is an exception to normal deployment checkpoint
  preservation, not a replay promise. Retrieved through MicrosoftDocs full fetch.

- New service backup docs include 35 days; core CLI and pinned ARM 2025-04-15
  enumerate 7/30, while newer/extension docs can expose 35. Supplied assets use
  supported seven-day subset; don't force a newer enum into older tools.
- Vector page includes float16; pinned older ARM enum lacks it. Supplied optional
  vector policy stays float32. No float16 SDK/service test was performed.
- NoSQL vector/index pages and unified Azure/Fabric index guidance disagree on
  policy mutability/drop/re-add. Conservative guidance uses new generation/container
  and requires exact version confirmation, not “can never change” as universal law.
- Current Python continuation docs permit streamable cross-partition queries,
  unlike older single-partition-only advice. Aggregates still have restrictions.
- New ReadConsistencyStrategy is preview (Java >=4.69/.NET >=3.46, direct mode);
  stronger per-read options don't mean every SDK/topology supports them.
- Some old `/azure/cosmos-db/query/...`, how-to-Java and data-RBAC URLs returned
  unavailable pages. They were replaced with current unified query/Java API/RBAC
  references; no claim that an unavailable retrieval proves feature absence.

Refresh mutable claims at use. No runtime SDK/CLI installation/version/service
acceptance, effective RBAC/key/network posture, performance/recall or actual restore
was verified. Offline package tests don't close those evidence gaps.
