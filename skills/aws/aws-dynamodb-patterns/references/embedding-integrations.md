# Embeddings: meaning, precision and AWS integration

Documentation checked 2026-11-08. Read this when someone asks what vector numbers
mean, how many floating-point digits survive, or whether vectors point into S3.
For index schemas, request shapes and limits, use [native vectors](data-types-and-vectors.md).
Load [the CLI companion](commands/embedding.md) only when commands are requested;
do not load it or its request assets merely to explain precision or architecture.

## First correct the mental model

An embedding is an ordered array of coordinates produced by a trained model for
text, images or other supported input. Nearby vectors can represent similar content
under that model and distance function. A coordinate is **not** a document ID,
memory address, S3 object offset, URL or reliably human-named concept. The whole
vector participates in comparison; do not assign a meaning to dimension 17.

Keep three different things:

| Thing | Example | Job |
|---|---|---|
| Content | A paragraph from a manual | Original evidence used to answer |
| Embedding | `[0.12, -0.04, ...]` | Similarity ranking, not a source locator |
| Identity/lineage | Document ID, chunk ID, bucket/key/VersionId, model configuration | Retrieve the correct source and track changes |

Neither DynamoDB nor S3 Vectors discovers a source object from the coordinates.
Store that association explicitly in the item/metadata or a separately controlled
lookup. Content can also live directly in DynamoDB; **S3 is not mandatory**.
An embedding is not encryption, anonymization, proof of relevance or factual truth.
Do not expose vectors/content merely because the original document is elsewhere.

## What does “up to what floating point” mean?

Separate **dimensions**, **precision**, **numeric range** and **wire format**:

| Layer | Actual contract |
|---|---|
| Model output | Model-specific dimensions and representations; choose first |
| DynamoDB base item | Ordered `L` of `N`; Numbers allow up to 38 significant decimal digits, not a native float64 column |
| DynamoDB vector index | **float32 / f32**; extra base-item precision is lost in the index |
| S3 Vectors | `data.float32` coordinates; higher-precision SDK inputs are converted to float32 |
| Runtime | JS/Python/PHP numbers are commonly binary64; that does not turn either index into float64 |

Binary32 has 24 significand bits: roughly **7 significant decimal digits**, not
7 digits after the decimal point. Six decimal significant digits have the usual
decimal-to-f32-to-decimal preservation guarantee; nine digits suffice to serialize
an existing f32 value for round-trip recovery. Printing more digits adds no index
precision. For example, `1.00000001` rounds to `1.0` in f32. Do not truncate every
coordinate to a fixed number of decimal places: that can damage small values.

- Largest finite magnitude is approximately **3.4028235e38**. This is a format
  boundary, not a useful target value or a guarantee of stable distance arithmetic.
- Smallest positive *normal* value is approximately **1.17549435e-38**. IEEE f32
  also represents subnormals down to approximately **1.40129846e-45**, with reduced
  precision. Smaller nonzero numbers can round to zero. The cited AWS docs do not
  specify an end-to-end guarantee for subnormal handling in distance computation.
- Reject NaN, positive/negative Infinity, overflow and wrong dimensions. Define
  underflow/subnormal and all-zero-vector policies; a zero vector has no cosine
  direction. This example conservatively rejects subnormals/nonzero underflow.
- DynamoDB Number's much wider range (positive 1e-130 to about 1e126, plus negative
  counterparts and zero) is **not** the safe vector-index range.
- float16/bfloat16 inputs converted to f32 cannot regain lost information. Packed
  binary embeddings are a different representation: do not reinterpret them as
  DynamoDB numeric-list coordinates or S3 Vectors float32 data. Neither service
  exposes a float64 vector-index selection in the contracts cited here.

Normalizing a nonzero vector makes its length one; it does not change its
dimensions, create source links, align different models, or increase precision.
Do not assume all models always output values in `[-1, 1]`. Persist the model ID,
revision/deployment, dimension count, normalization and preprocessing/chunking
version. Same dimensions alone do **not** make two models' spaces comparable.

## Pick one retrieval architecture, not every AWS service

| Requirement | Smallest candidate | Ownership/caveat |
|---|---|---|
| Operational items and embeddings already belong together | Bedrock model + DynamoDB native index | Application owns ingestion, authorization and retrieval orchestration |
| Managed document RAG from S3 files | Bedrock Knowledge Bases + S3 Vectors | KB manages parsing/chunking/embedding/ingestion; verify features/Region/quotas |
| Direct vector storage without a KB | Bedrock or another model + S3 Vectors API | Application owns chunk/source mapping, synchronization and query generation |
| Workload needs additional lexical/hybrid capabilities | Evaluate OpenSearch or another supported store | Prove feature needs and retrieval quality before adding an engine |

These are design candidates, not measured performance/cost recommendations.
An **ordinary S3 object bucket** holds documents (or serialized vector files).
Uploading `embedding.json` there does not create semantic search. An **S3 vector
bucket** holds vector indexes and uses dedicated `s3vectors` APIs. A **DynamoDB
vector index** belongs to a DynamoDB table and uses `SearchVectors`. It does not
need or imply an S3 vector bucket, nor automatically mirror into one.

### A. Application-managed DynamoDB retrieval

```text
Source text (in DynamoDB, or extracted from an authorized versioned S3 object)
  -> deterministic chunking -> Bedrock embedding model -> validate float32/dimensions
  -> DynamoDB item: chunk + source reference + model lineage + L-of-N embedding
  -> native vector index (asynchronous)

User question -> SAME embedding configuration -> SearchVectors
  -> candidate PKs -> authorized GetItem/BatchGetItem -> verify current access/revision
  -> chunk text or exact authorized S3 object version -> optional Bedrock generation
```

[embedding-flow.py](../assets/embedding-flow.py) is an injected-client, low-level
Boto3-style example; import does nothing and no dependencies are installed. Its
`index_chunk`/`search_chunks` functions incur service calls **only when invoked**.
It uses Titan Text Embeddings V2, `amazon.titan-embed-text-v2:0`, **1024** dimensions,
`normalize: true`, `embeddingTypes: ["float"]`, with COSINE. Titan V2 also supports
512/256 dimensions; it cannot produce the existing three-dimensional toy fixture.
**Do not run this example against that fixture unchanged.**

For a separately approved new table, clone the optional vector table asset and set
the vector index Dimensions to 1024, retaining Tenant HASH / Status INLINE_FILTER
and COSINE. Review schema/IAM/costs first; dimensions of an existing index are not
mutable. The current narrow projection returns PK, not all source/chunk fields;
fetch base items explicitly. Confirm trusted deployment configuration binds that
index to this exact model/preprocessing generation, not just dimension 1024.

Example caller fragment (not a standalone deployment or authorization layer):

```python
import boto3
# Import functions from the bundled embedding-flow.py, e.g. after copying it to
# your application's embedding_flow module. Import does not create clients.
from embedding_flow import index_chunk, search_chunks

# Fill only from reviewed deployment configuration and authenticated server context.
session = boto3.Session(profile_name=approved_profile, region_name=approved_region)
bedrock = session.client("bedrock-runtime")
ddb = session.client("dynamodb")  # SDK version must expose search_vectors
index_chunk(bedrock, ddb, approved_table, authenticated_tenant,
            immutable_tenant_document_version_chunk_key, extracted_chunk_text,
            {"bucket": approved_source_bucket, "key": approved_source_key,
             "version_id": approved_non_null_version_id})
results = search_chunks(bedrock, ddb, approved_table, approved_index,
                       authenticated_tenant, user_question, top_k=10)
# Results contain candidate keys/scores; authorize and fetch source content next.
```

The example quantizes to f32 deliberately and serializes with nine significant
digits as low-level `{"N":"..."}` values. Storage uses `{"L":[...]}`; search uses
the bare array. Other languages follow the same contract: explicit f32 conversion,
finite/length checks, ordered numeric serialization and the same model settings.
For JS use a checked `Math.fround` conversion, not binary bytes from Float32Array;
Java/Kotlin/Rust use their f32-compatible type/conversion. A document client and a
low-level AttributeValue client need different marshaling; see [SDK adapters](language-adapters.md).

### B. Managed Knowledge Bases with S3 Vectors

```text
Ordinary S3 source bucket -> KB synchronization/ingestion
  -> parsing/chunking -> configured Bedrock embedding model
  -> separate S3 vector bucket + vector index
User question -> KB Retrieve (chunks/citations) or RetrieveAndGenerate (also answer)
```

S3 Vectors is a documented KB backing store. Current Bedrock StorageConfiguration
does **not** list DynamoDB as a backing store; do not invent direct managed KB to
native DynamoDB-vector support. Calling Bedrock yourself is not the same as using
a Knowledge Base. Recheck support at use time rather than claiming never.

With **direct S3 Vectors**, PutVectors uses `vectors[].data.float32`, while
QueryVectors uses **`queryVector.float32`**. Do not substitute DynamoDB L/N or
ordinary S3 GetObject APIs. These illustrative requests assume a separately created
**three-dimensional** S3 vector index, not the 1024-dimensional Titan example:

```json
{
  "vectorBucketName": "example-vector-bucket",
  "indexName": "example-three-dimensional-index",
  "vectors": [{
    "key": "manual-v17-chunk-1",
    "data": {"float32": [0.1, 0.2, 0.3]},
    "metadata": {"sourceBucket": "example-source", "sourceKey": "manual.txt", "sourceVersion": "v17"}
  }]
}
```

```json
{
  "vectorBucketName": "example-vector-bucket",
  "indexName": "example-three-dimensional-index",
  "queryVector": {"float32": [0.1, 0.2, 0.3]},
  "topK": 3,
  "returnMetadata": true
}
```

A vector key identifies a vector *within its index*; it need not be an S3 object key.
KB owns its managed metadata/citations; do not assume custom raw-vector writes will
become retrievable KB chunks. S3 Vectors' strongly consistent writes do not make
KB ingestion synchronous or DynamoDB vector indexes strongly consistent.

## Failure cases that must survive review

- **“S3 upload means searchable now.”** No. An application ingestion job or KB sync
  must succeed. Test create/update/delete, duplicate/out-of-order events and lag;
  use bounded retries, a failure destination and observable ingestion state.
- **“One chunk key can be overwritten on every retry.”** Stale workers can replace
  newer content. Use tenant/document/version/chunk/model-generation identity and
  conditional/idempotent writes. The example condition prevents overwrites; it is
  not a complete event ledger or source-version arbitration implementation. Publish
  the current revision only after ingestion validation; retire obsolete vectors.
- **“Changing model only requires changing a string.”** New coordinate space needs
  re-embedding/backfill and a separate index/generation, then measured cutover.
  Do not mix models even when they emit identical dimensions.
- **“A tenant filter authorizes access.”** Derive it server-side; DynamoDB search
  filters are not IAM row isolation. Revalidate content ACLs/revocation before
  returning chunks or sending them to generation. A source URI grants no access.
- **“One permission covers everything.”** Separate Bedrock InvokeModel, DynamoDB
  writes/index search/base reads, S3 GetObjectVersion for pinned source versions,
  S3 Vectors actions and applicable KMS permissions. Encryption configuration and
  key access are service-specific; see [encryption](encryption.md).
- **“More decimals/dimensions/services guarantee better answers.”** Compare labeled
  retrieval recall@k/relevance, filtered/tenant-negative results, source freshness,
  latency and total ingestion/query/storage/generation cost. Measure item bytes;
  base Number encoding is not simply `dimensions * 4`. Respect model input and
  DynamoDB 400 KB item limits. Protect logs and treat retrieved text as untrusted
  data, not instructions; citations and similarity scores are not truth guarantees.

## Primary sources

- [DynamoDB precision/constraints](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/VectorSearch.Requirements.html),
  [base Number types](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/HowItWorks.NamingRulesDataTypes.html),
  [embedding workflow](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/VectorSearchWorkingWith.html).
- [IEEE binary formats in Python struct](https://docs.python.org/3/library/struct.html),
  [binary32 constants/significand](https://learn.microsoft.com/en-us/cpp/cpp/floating-limits?view=msvc-170).
  Subnormal floor is derived as 2^-149; this is format math, not a tested AWS promise.
- [Titan V2 contract](https://docs.aws.amazon.com/bedrock/latest/userguide/model-parameters-titan-embed-text.html),
  [DynamoDB/Bedrock tutorial](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/VectorSearchTutorial.html).
- [S3 bucket/API distinction](https://aws.amazon.com/s3/faqs/),
  [S3 PutVectors float32](https://docs.aws.amazon.com/AmazonS3/latest/API/API_S3VectorBuckets_PutVectors.html),
  [S3 QueryVectors shape](https://docs.aws.amazon.com/aws-sdk-php/v3/api/api-s3vectors-2025-07-15.html),
  [KB integration](https://docs.aws.amazon.com/AmazonS3/latest/userguide/s3-vectors-bedrock-kb.html),
  [KB storage configuration](https://docs.aws.amazon.com/bedrock/latest/APIReference/API_agent_StorageConfiguration.html),
  [KB permissions](https://docs.aws.amazon.com/bedrock/latest/userguide/kb-permissions.html).

These are documentation-backed designs plus locally mocked examples. Live IAM,
SDK/service acceptance, ingestion/retrieval freshness, relevance, latency and cost
remain **UNVERIFIED**. Do not deploy or claim production readiness from this guide.
