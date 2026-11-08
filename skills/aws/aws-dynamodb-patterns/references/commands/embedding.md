# Embedding integration CLI commands

Load **only when CLI examples are requested**. For meaning, f32 precision, source
mapping and architecture selection, read [embedding integrations](../embedding-integrations.md)
instead. These are independent examples, **not a script to run top to bottom**.
No command executes on skill load. Reading them authorizes no AWS call.

Use an installed compatible AWS CLI v2 and Python 3. Run from this skill's directory.
All content/metadata below is synthetic. For real ingestion, use authorized extracted
chunk text and its actual pinned source version; these examples do not read or
verify an S3 source. Never log embeddings, source text, credentials or retrieval data.

## Common target and private output directory

Set approved profile/Region explicitly. Inspect returned account/role before any
model/data operation; confirm each selected section's exact resource separately.
Outputs stay in a fresh private local directory, not stdout or committed assets.

```bash
set -euo pipefail
: "${AWS_PROFILE:?Approved existing profile}" "${AWS_REGION:?Approved Region}"
umask 077
work_dir=$(mktemp -d)
aws_target=(aws --profile "$AWS_PROFILE" --region "$AWS_REGION" --no-cli-pager --output json)
"${aws_target[@]}" sts get-caller-identity
```

## A. Generate a vector for application-managed retrieval

**Billable model invocation**, only after approval. This uses Titan V2/1024/normalized
float output, matching [the SDK example](../../assets/embedding-flow.py). Choose a
dedicated existing **1024-dimensional COSINE index** for that exact model/config;
do not use the three-dimensional toy index. Model/Region access must exist already.

```bash
"${aws_target[@]}" bedrock-runtime invoke-model \
  --model-id amazon.titan-embed-text-v2:0 \
  --body file://assets/bedrock-embedding-input.json \
  --content-type application/json --accept application/json \
  --cli-binary-format raw-in-base64-out "$work_dir/embedding.json"
# The positional output file contains the model body; stdout is NOT the embedding.
PYTHONDONTWRITEBYTECODE=1 python3 - "$work_dir" <<'PY'
import json
from pathlib import Path
from runpy import run_path
import sys

root = Path(sys.argv[1])
number_values = run_path("assets/embedding-flow.py")["number_values"]
numbers = number_values(json.loads((root / "embedding.json").read_text())["embedding"], 1024)
(root / "dynamodb-vector.json").write_text(json.dumps(numbers))
(root / "s3-query-vector.json").write_text(json.dumps({"float32": [float(n["N"]) for n in numbers]}))
PY
```

This reuses the existing checked f32 conversion instead of inventing a second
validator: it rejects wrong dimensions, nonfinite/overflow, zero vectors and its
documented conservative subnormal/underflow cases. For an actual question, replace
only inputText in a separately prepared request; preserve model/configuration.
An embedding alone is neither content, a source pointer nor a relevance benchmark.

### A1. Search native DynamoDB (existing ingested items)

**Authorized data read**; run Common + A + this section only. The index must match
Tenant HASH / Status INLINE_FILTER and this model generation. Source/versioned
conditional ingestion is demonstrated in the SDK guide; no new item/table is
silently created here. TENANT_ID comes from trusted authorized context.

```bash
: "${VECTOR_TABLE_NAME:?Approved table}" "${VECTOR_INDEX_NAME:?Approved native index}" \
  "${TENANT_ID:?Authorized tenant}"
values=$(python3 - "$TENANT_ID" <<'PY'
import json
import sys
print(json.dumps({":tenant": {"S": sys.argv[1]}, ":status": {"S": "PUBLISHED"}}))
PY
)
"${aws_target[@]}" dynamodb search-vectors \
  --table-name "$VECTOR_TABLE_NAME" --index-name "$VECTOR_INDEX_NAME" \
  --search-vector "file://$work_dir/dynamodb-vector.json" --top-k 10 \
  --search-condition-expression '#tenant = :tenant AND #status = :status' \
  --expression-attribute-names '{"#tenant":"Tenant","#status":"Status"}' \
  --expression-attribute-values "$values" > "$work_dir/dynamodb-results.json"
```

The array contains Number AttributeValues, **not** an L wrapper, S3 float32 object
or Query cursor. Fetch projected candidate keys from the base table, then revalidate
current ACLs/revision before returning source content. Tenant filtering is not IAM
row isolation. Native SearchVectors is eventual and has no pagination.

### A2. Direct S3 Vectors alternative (not a KB-owned index)

Run Common + A + this section **instead of A1**. Require an approved existing
application-owned 1024/COSINE index. **PutVectors is a separately approved data
mutation**: an existing vector key can be overwritten. Use deliberate immutable
tenant/document/version/chunk/model keys, not retries with changing identities.
Only query an index whose entire searched data scope is authorized; metadata is
not an access-control boundary. Do not write directly into a KB-managed index.

```bash
: "${S3_VECTOR_INDEX_ARN:?Approved application-owned vector index ARN}" \
  "${VECTOR_KEY:?Approved versioned vector key}" "${TENANT_ID:?Authorized tenant}" \
  "${SOURCE_BUCKET:?Authorized source bucket}" "${SOURCE_KEY:?Authorized source key}" \
  "${SOURCE_VERSION_ID:?Pinned non-null source version}"
python3 - "$work_dir" "$VECTOR_KEY" "$TENANT_ID" "$SOURCE_BUCKET" "$SOURCE_KEY" "$SOURCE_VERSION_ID" <<'PY'
import json
from pathlib import Path
import sys

directory, key, tenant, bucket, source_key, version = sys.argv[1:]
if version == "null":
    raise SystemExit("This example requires a pinned non-null source version")
root = Path(directory)
vector = {"key": key, "data": json.loads((root / "s3-query-vector.json").read_text()),
          "metadata": {"tenant": tenant, "sourceBucket": bucket,
                       "sourceKey": source_key, "sourceVersion": version}}
(root / "s3-put-vectors.json").write_text(json.dumps([vector]))
PY
"${aws_target[@]}" s3vectors put-vectors --index-arn "$S3_VECTOR_INDEX_ARN" \
  --vectors "file://$work_dir/s3-put-vectors.json"
"${aws_target[@]}" s3vectors query-vectors --index-arn "$S3_VECTOR_INDEX_ARN" \
  --query-vector "file://$work_dir/s3-query-vector.json" --top-k 10 \
  --return-distance --return-metadata --no-paginate > "$work_dir/s3-results.json"
```

Put uses vectors[].data.float32; query uses queryVector.float32. Raw numeric arrays
are not DynamoDB L/N tags. Querying with returned metadata (or filters) also needs
s3vectors:GetVectors alongside QueryVectors; writes need PutVectors. Unlike native
DynamoDB search, the current S3 command supports pagination: this example requests
**one service response**, not guaranteed complete results. Inspect continuation
state; use current S3-specific pagination if completeness is required.
Searching the same synthetic vector just inserted is a format smoke test, not RAG
quality evidence. Generate real document/query embeddings separately with one model.

## B. Existing Bedrock Knowledge Base: sync, inspect, retrieve

Use Common + selected B sections, **not A**. The KB generates query embeddings using
its own configured model/store; do not pass DynamoDB/S3 index names or assume it
uses the Titan settings above. Verify KB/source ownership, store/model/Region,
permissions and data scope. This page creates no KB, table, bucket or vector index.

**Optional billable mutation:** synchronize an existing approved data source. Keep
the same stable client token for retries of this logical sync; reconcile uncertain
outcomes instead of blindly launching another ingestion job.

```bash
: "${KB_ID:?Approved existing KB}" "${DS_ID:?Approved data source}" \
  "${INGESTION_TOKEN:?Stable 33-256 character idempotency token}"
"${aws_target[@]}" bedrock-agent start-ingestion-job \
  --knowledge-base-id "$KB_ID" --data-source-id "$DS_ID" \
  --client-token "$INGESTION_TOKEN" > "$work_dir/job-start.json"
JOB_ID=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["ingestionJob"]["ingestionJobId"])' "$work_dir/job-start.json")
```

**Authorized metadata read:** inspect that job, or set JOB_ID to an existing
approved job when skipping Start. This is one status check, not an unbounded poll.

```bash
: "${KB_ID:?}" "${DS_ID:?}" "${JOB_ID:?Exact ingestion job}"
"${aws_target[@]}" bedrock-agent get-ingestion-job \
  --knowledge-base-id "$KB_ID" --data-source-id "$DS_ID" \
  --ingestion-job-id "$JOB_ID" > "$work_dir/job-status.json"
```

Inspect failureReasons/statistics as well as status. **Billable authorized content
retrieval:** the example gates this particular post-sync check on COMPLETE, not
merely successful Start. Existing-KB queries need no new ingestion job; first
inspect the relevant current ingestion job instead. COMPLETE is not an ACL,
freshness-for-future-writes or retrieval-quality guarantee.

```bash
: "${KB_ID:?Approved KB content scope}"
python3 - "$work_dir/job-status.json" <<'PY'
import json
import sys
if json.load(open(sys.argv[1]))["ingestionJob"]["status"] != "COMPLETE":
    raise SystemExit("Ingestion not complete; inspect job state before post-sync retrieval")
PY
"${aws_target[@]}" bedrock-agent-runtime retrieve --knowledge-base-id "$KB_ID" \
  --retrieval-query '{"text":"Synthetic question about the document"}' \
  --retrieval-configuration '{"vectorSearchConfiguration":{"numberOfResults":5}}' \
  --no-paginate > "$work_dir/kb-results.json"
```

Retrieve returns chunks/citations, not a generated answer. It is not RetrieveAndGenerate.
One response may have a continuation token; do not claim complete retrieval from
numberOfResults or this bounded call. Enforce authenticated tenant/source ACLs in
the application, restrict KB data scope, and treat returned text as untrusted data.
Do not conflate successful synchronization with deleted-content access revocation.

Sources checked 2026-11-08 through Context7/AWS documentation MCP:
[InvokeModel](https://docs.aws.amazon.com/cli/latest/reference/bedrock-runtime/invoke-model.html),
[Titan V2](https://docs.aws.amazon.com/bedrock/latest/userguide/model-parameters-titan-embed-text.html),
[native search](https://docs.aws.amazon.com/cli/latest/reference/dynamodb/search-vectors.html),
[S3 put](https://docs.aws.amazon.com/cli/latest/reference/s3vectors/put-vectors.html),
[S3 query](https://docs.aws.amazon.com/cli/latest/reference/s3vectors/query-vectors.html),
[start sync](https://docs.aws.amazon.com/cli/latest/reference/bedrock-agent/start-ingestion-job.html),
[job inspection](https://docs.aws.amazon.com/cli/latest/reference/bedrock-agent/get-ingestion-job.html),
[KB retrieval](https://docs.aws.amazon.com/cli/latest/reference/bedrock-agent-runtime/retrieve.html).
Mocks validate commands/JSON, not effective IAM, live service acceptance or relevance.
