# Embeddings, source references and Azure integrations

Cosmos stores/searches coordinates; a Blob URI, source text or model/deployment
name isn't an embedding. Generate embeddings through a separately authorized
Azure OpenAI/Foundry model or reviewed application model. No model call runs on
skill load and there is no claim Cosmos automatically embeds a stored source URI.

| Stage | Explicit contract |
|---|---|
| Source | Blob account/container/name AND immutable versionId or equivalent immutable content hash; tenant/ACL/classification, MIME and provenance. Never persist a SAS token as source identity. |
| Chunk | Stable source-version/chunk ID, offsets, tokenizer/chunking version; minimal text required for retrieval, not an accidental source-data dump |
| Model | Provider/model/deployment version, dimensions, normalization, dataType/quantization and metric; same contract for document AND query embeddings |
| Index write | Numeric array, tenantId, id, source ref, model/version and generation; conditional/idempotent write after vector validation |
| Retrieval | Server-derived tenant, partition routing AND SQL predicate, bounded top-k/RUs, validated result tenant/ACL; authorize source access again before fetching Blob |
| Update/delete | Source-version idempotency, tombstones/deletion reconciliation, TTL and source ACL/model migration; no mixing incomparable vector spaces |

For Azure OpenAI `text-embedding-3-small`/`text-embedding-3-large`, dimensions and
supported reduction depend on deployed model/API; don't assume 1536 for every model.
The optional policy is **only** a 1536-dimensional generation. Validate actual
output count/finiteness/float32 representability before writes. Quantization and
float16/int8 conversion require measured quality/recall, not an unreviewed cast.

Illustrative generation with an **already configured, authorized** Azure OpenAI
v1-compatible client (`openai`) and approved embedding deployment; no client is
constructed or request executed by this document:

```javascript
const generated = await openai.embeddings.create({
  model: approvedDeployment, input: chunkText,
  dimensions: 1536, encoding_format: "float"
});
// For a model/deployment supporting this output size; validate before indexing.
const coordinates = vector(generated.data[0].embedding, {dimensions: 1536});
// buildChunk's injected embedding port can use this adapter; it is not automatic.
```

The model-call timeout/attempt/token budget, credential, source authorization and
output handling are caller-owned. Record actual model version, not deployment
name alone; changing deployment backing model can invalidate an existing index.

| Integration | Coherent use | Avoid |
|---|---|---|
| Azure OpenAI / Foundry | App generates versioned coordinates with separate model token/RBAC/network controls | Treating Cosmos RU as embedding token cost or invoking a model automatically |
| Blob Storage | Immutable originals; Cosmos metadata/chunks/source refs | Base64 blobs in item hot paths or persisted SAS secrets |
| Azure Functions/change feed | Idempotent enrichment/projection, leases/checkpoints and retry/dead-letter design | Assuming exactly-once or latest-version feed contains deletes |
| Azure AI Search | Alternative dedicated search index with explicit sync/vectorization/ACL/freshness design | Claiming native Cosmos vector query automatically calls AI Search or that either service enforces tenant ACL from a URI |
| Azure Monitor/App Insights | Charge/latency/error/activity IDs and quality/freshness metrics | Logging raw sensitive text/vectors/tokens |

End-to-end quality needs representative labeled queries, recall/nDCG, tenant
denials, stale/deleted source behavior and budget. A model+store API success is not
semantic relevance or production RAG readiness. Use current first-party docs and
existing SDK for actual model requests; no SDK install is implied.

Sources: [Cosmos vector search](https://learn.microsoft.com/azure/cosmos-db/vector-search),
[Azure OpenAI embeddings](https://learn.microsoft.com/azure/ai-foundry/openai/how-to/embeddings),
[Azure AI Search vectorization](https://learn.microsoft.com/azure/search/vector-search-integrated-vectorization).
