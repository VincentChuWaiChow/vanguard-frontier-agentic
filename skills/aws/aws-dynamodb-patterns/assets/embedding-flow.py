"""Injected Boto3-style clients; importing this module makes no service calls.

Example only: caller owns authorization, chunking, versioned keys, retries, item
size, and a dedicated COSINE/1024 index for this exact model configuration.
"""
import json
import math
import struct

MODEL_ID = "amazon.titan-embed-text-v2:0"
DIMENSIONS = 1024


def required(*values):
    if any(not isinstance(value, str) or not value.strip() for value in values):
        raise ValueError("Explicit nonempty targets, authenticated tenant and text required")


def number_values(values, dimensions):
    """Quantize once, then emit low-level DynamoDB Number tags.

    This conservative example rejects subnormal values and nonzero underflow;
    that policy is not an assertion that AWS rejects every subnormal value.
    """
    if type(dimensions) is not int or not 1 <= dimensions <= 4096:
        raise ValueError("Invalid dimensions")
    if not isinstance(values, list) or len(values) != dimensions:
        raise ValueError("Embedding length must match the dedicated index")
    result = []
    for value in values:
        if type(value) not in (int, float):
            raise ValueError("Numeric coordinates required; no bool/string/binary")
        try:
            original = float(value)
            rounded = struct.unpack("!f", struct.pack("!f", original))[0]
        except (OverflowError, struct.error) as error:
            raise ValueError("Coordinate overflows float32") from error
        if not math.isfinite(rounded):
            raise ValueError("NaN/Infinity are not embedding coordinates")
        if (original != 0 and rounded == 0) or 0 < abs(rounded) < 2.0 ** -126:
            raise ValueError("Example policy rejects underflow/subnormal coordinates")
        # Nine significant digits preserve f32 round-trips, not nine decimal places.
        result.append({"N": format(rounded, ".9g")})
    if not any(float(value["N"]) != 0 for value in result):
        raise ValueError("Zero vector is unsuitable for this COSINE example")
    return result


def embed_text(bedrock, text):
    required(text)
    response = bedrock.invoke_model(
        modelId=MODEL_ID, contentType="application/json", accept="application/json",
        body=json.dumps({"inputText": text, "dimensions": DIMENSIONS,
                         "normalize": True, "embeddingTypes": ["float"]}))
    payload = json.loads(response["body"].read())
    return number_values(payload["embedding"], DIMENSIONS)


def index_chunk(bedrock, dynamodb, table, tenant, chunk_key, text, source):
    """Billable model call and item write when explicitly invoked by the caller."""
    required(table, tenant, chunk_key, text)
    if not isinstance(source, dict) or set(source) != {"bucket", "key", "version_id"}:
        raise ValueError("Explicit versioned source reference required")
    required(*source.values())
    if source["version_id"] == "null":
        raise ValueError("This example requires a pinned, non-null S3 object version")
    numbers = embed_text(bedrock, text)
    return dynamodb.put_item(
        TableName=table, ConditionExpression="attribute_not_exists(PK)", Item={
            "PK": {"S": chunk_key}, "Tenant": {"S": tenant},
            "Status": {"S": "PUBLISHED"}, "ChunkText": {"S": text},
            "Source": {"M": {key: {"S": value} for key, value in source.items()}},
            "EmbeddingModel": {"S": MODEL_ID},
            "EmbeddingDimensions": {"N": str(DIMENSIONS)},
            "EmbeddingNormalized": {"BOOL": True}, "Embedding": {"L": numbers}})


def search_chunks(bedrock, dynamodb, table, index, tenant, question, top_k=10):
    """Caller derives tenant/index from trusted configuration, never request input."""
    required(table, index, tenant, question)
    if type(top_k) is not int or not 1 <= top_k <= 100:
        raise ValueError("TopK must be an integer in 1..100")
    return dynamodb.search_vectors(
        TableName=table, IndexName=index, TopK=top_k,
        SearchVector=embed_text(bedrock, question),
        SearchConditionExpression="#tenant = :tenant AND #status = :status",
        ExpressionAttributeNames={"#tenant": "Tenant", "#status": "Status"},
        ExpressionAttributeValues={":tenant": {"S": tenant},
                                   ":status": {"S": "PUBLISHED"}})
