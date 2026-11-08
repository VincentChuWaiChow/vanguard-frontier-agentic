#!/usr/bin/env python3
"""Offline numeric and injected-client example tests; no SDK or AWS calls."""
import importlib.util
import io
import json
from pathlib import Path
import re
import struct
import sys
import unittest

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("flow", ROOT / "assets/embedding-flow.py")
flow = importlib.util.module_from_spec(spec)
spec.loader.exec_module(flow)


class Bedrock:
    def __init__(self, values=None):
        self.values = values if values is not None else [0.1] * 1024
        self.calls = []

    def invoke_model(self, **request):
        self.calls.append(request)
        return {"body": io.BytesIO(json.dumps({"embedding": self.values}).encode())}


class DynamoDB:
    def __init__(self):
        self.puts, self.searches = [], []

    def put_item(self, **request):
        self.puts.append(request)
        return {"ok": True}

    def search_vectors(self, **request):
        self.searches.append(request)
        return {"SearchResults": [{"Item": {"PK": {"S": "example"}}, "Score": 0.1}]}


class EmbeddingFlowTests(unittest.TestCase):
    def test_s3_vectors_put_and_query_have_different_wire_fields(self):
        guide = (ROOT / "references/embedding-integrations.md").read_text()
        requests = [json.loads(block) for block in
                    re.findall(r"```json\n(.*?)\n```", guide, re.S)]
        put, query = requests
        self.assertEqual(put["vectorBucketName"], query["vectorBucketName"])
        self.assertEqual(put["indexName"], query["indexName"])
        self.assertEqual(put["vectors"][0]["data"]["float32"], query["queryVector"]["float32"])
        self.assertEqual(len(query["queryVector"]["float32"]), 3)
        self.assertNotIn("data", query)
        self.assertNotIn("queryVector", put)
        self.assertTrue(all(type(value) is float for value in query["queryVector"]["float32"]))
        self.assertEqual(put["vectors"][0]["metadata"]["sourceVersion"], "v17")

    def test_float32_rounding_not_fixed_decimal_places(self):
        values = flow.number_values([0.1, 1.00000001, -0.25], 3)
        for before, wire in zip([0.1, 1.00000001, -0.25], values):
            rounded = struct.unpack("!f", struct.pack("!f", before))[0]
            self.assertEqual(struct.pack("!f", float(wire["N"])), struct.pack("!f", rounded))
            self.assertLessEqual(len(wire["N"].replace("-", "").split("e")[0].replace(".", "")), 10)
        self.assertEqual(values[1], {"N": "1"})

    def test_finite_normal_boundaries(self):
        largest = struct.unpack("!f", bytes.fromhex("7f7fffff"))[0]
        smallest_normal = 2.0 ** -126
        result = flow.number_values([largest, -largest, smallest_normal, 0.0], 4)
        self.assertEqual(len(result), 4)
        self.assertEqual(struct.pack("!f", float(result[0]["N"])), bytes.fromhex("7f7fffff"))

    def test_rejects_bad_values_and_conservative_underflow_policy(self):
        for values in [[True], ["0.1"], [None], [float("nan")], [float("inf")],
                       [-float("inf")], [1e39], [-1e39], [1e-50], [2.0 ** -149],
                       [0.0], [-0.0], {0.1}, [[0.1]]]:
            with self.subTest(values=values), self.assertRaises(ValueError):
                flow.number_values(values, 1)
        for dimension in [0, 4097, True, 1.5]:
            with self.subTest(dimension=dimension), self.assertRaises(ValueError):
                flow.number_values([0.1], dimension)
        for values in [[], [0.1], [0.1] * 4]:
            with self.subTest(length=len(values)), self.assertRaises(ValueError):
                flow.number_values(values, 3)

    def test_bedrock_request_is_explicit_and_fixed(self):
        client = Bedrock()
        result = flow.embed_text(client, "Synthetic source")
        self.assertEqual(len(result), 1024)
        request = client.calls[0]
        self.assertEqual(request["modelId"], "amazon.titan-embed-text-v2:0")
        self.assertEqual(request["contentType"], "application/json")
        self.assertEqual(request["accept"], "application/json")
        self.assertEqual(json.loads(request["body"]), {
            "inputText": "Synthetic source", "dimensions": 1024,
            "normalize": True, "embeddingTypes": ["float"]})

    def test_item_stores_explicit_source_and_model_not_a_vector_pointer(self):
        bedrock, ddb = Bedrock(), DynamoDB()
        source = {"bucket": "example-source", "key": "manual.txt", "version_id": "v17"}
        flow.index_chunk(bedrock, ddb, "ApprovedTable", "tenant-a", "immutable-chunk-v17",
                         "Synthetic source", source)
        request = ddb.puts[0]
        self.assertEqual(request["TableName"], "ApprovedTable")
        self.assertEqual(request["ConditionExpression"], "attribute_not_exists(PK)")
        item = request["Item"]
        self.assertEqual(item["Source"]["M"], {
            "bucket": {"S": "example-source"}, "key": {"S": "manual.txt"},
            "version_id": {"S": "v17"}})
        self.assertEqual(item["EmbeddingModel"], {"S": flow.MODEL_ID})
        self.assertEqual(item["EmbeddingDimensions"], {"N": "1024"})
        self.assertEqual(item["Tenant"], {"S": "tenant-a"})
        self.assertEqual(item["ChunkText"], {"S": "Synthetic source"})
        self.assertEqual(set(item["Embedding"]), {"L"})
        self.assertEqual(len(item["Embedding"]["L"]), 1024)

    def test_search_matches_model_filters_and_wire_shape(self):
        bedrock, ddb = Bedrock(), DynamoDB()
        result = flow.search_chunks(bedrock, ddb, "ApprovedTable", "ApprovedIndex",
                                    "tenant-a", "Synthetic question", top_k=7)
        self.assertEqual(len(result["SearchResults"]), 1)
        request = ddb.searches[0]
        self.assertEqual(request["IndexName"], "ApprovedIndex")
        self.assertEqual(request["TopK"], 7)
        self.assertIsInstance(request["SearchVector"], list)
        self.assertEqual(len(request["SearchVector"]), 1024)
        self.assertEqual(request["ExpressionAttributeValues"], {
            ":tenant": {"S": "tenant-a"}, ":status": {"S": "PUBLISHED"}})
        self.assertNotIn("ConsistentRead", request)
        self.assertEqual(json.loads(bedrock.calls[0]["body"])["dimensions"], 1024)

    def test_bad_embeddings_stop_before_database_calls(self):
        source = {"bucket": "example", "key": "manual.txt", "version_id": "v1"}
        for values in [[0.1] * 3, [0.0] * 1024, [1e39] * 1024]:
            with self.subTest(values=values[:1]):
                bedrock, ddb = Bedrock(values), DynamoDB()
                with self.assertRaises(ValueError):
                    flow.index_chunk(bedrock, ddb, "Table", "tenant", "chunk", "Text", source)
                with self.assertRaises(ValueError):
                    flow.search_chunks(bedrock, ddb, "Table", "Index", "tenant", "Question")
                self.assertFalse(ddb.puts or ddb.searches)

    def test_invalid_targets_source_and_bounds_stop_before_model_cost(self):
        bedrock, ddb = Bedrock(), DynamoDB()
        for source in [{}, {"bucket": "x", "key": "x", "version_id": "null"}]:
            with self.assertRaises(ValueError):
                flow.index_chunk(bedrock, ddb, "Table", "tenant", "chunk", "Text", source)
        for top_k in [0, 101, True]:
            with self.assertRaises(ValueError):
                flow.search_chunks(bedrock, ddb, "Table", "Index", "tenant", "Question", top_k)
        with self.assertRaises(ValueError):
            flow.search_chunks(bedrock, ddb, "", "Index", "tenant", "Question")
        with self.assertRaises(ValueError):
            flow.embed_text(bedrock, " ")
        self.assertFalse(bedrock.calls or ddb.puts or ddb.searches)

    def test_model_errors_are_not_empty_search_results(self):
        class Broken(Bedrock):
            def invoke_model(self, **request):
                raise RuntimeError("synthetic failure")
        ddb = DynamoDB()
        with self.assertRaisesRegex(RuntimeError, "synthetic failure"):
            flow.search_chunks(Broken(), ddb, "Table", "Index", "tenant", "Question")
        self.assertFalse(ddb.searches)


if __name__ == "__main__":
    unittest.main(verbosity=2)
