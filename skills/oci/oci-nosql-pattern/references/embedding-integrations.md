# Embedding/source mapping and separate retrieval service

Application flow: authenticated source→approved embedding model→validate finite
numeric vector/model/dimension→persist NoSQL source/provenance→idempotently update
a separately chosen vector index. NoSQL holds canonical source metadata, not a
native knowledge-base ingestion/search control plane. OCI Generative AI generation
and Oracle AI Database vector search are different services; neither silently
makes NDCS an ANN database. Do not import Bedrock or S3 Vectors commands.

Record `tenantId, sourceId, sourceHash, chunkId, model/version, dimension, precision,
normalization, generatedAt, sourceVersion` with the embedding. A URI is a reference;
materialize authorized content before generation and honor source deletions/TTL.
Never mix query/document embedding spaces or version sets. NoSQL and vector-store
writes are not one transaction: design durable app outbox/reconciliation,
idempotency, ordering and tombstone propagation; do not invent DynamoDB Streams.

For a requested OCI GenAI example, resolve current supported model/region first,
then use its official `EmbedText` operation with approved non-sensitive text;
the model's returned embedding length, not a copied constant, fixes dimensions.
The [validator](../assets/query-pages.mjs) demonstrates only a local contract;
it does not call a model. Evaluate recall@k vs labeled authorized results, stale
source/deletion leakage, precision loss, latency/cost and per-tenant filtering.

[OCI vector-store concepts](https://docs.oracle.com/en-us/iaas/Content/generative-ai/vector-stores.htm),
[separate AI Database embedding workflow](https://docs.oracle.com/en/cloud/paas/autonomous-database/vcapi/concepts/vector-table.html), checked 2026-11-08.
