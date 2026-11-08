#!/usr/bin/env python3
"""Execute the lazy command guide against fake AWS; never use credentials/network."""
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
GUIDE = ROOT / "references/commands/embedding.md"
FAKE_AWS = r'''#!/usr/bin/env python3
import json, os, pathlib, sys
args = sys.argv[1:]
services = {"sts", "bedrock-runtime", "dynamodb", "s3vectors", "bedrock-agent", "bedrock-agent-runtime"}
service = next(x for x in args if x in services)
action = args[args.index(service) + 1]
def flag(name): return args[args.index(name) + 1]
files = {name: json.loads(pathlib.Path(flag(name)[7:]).read_text())
         for name in ("--body", "--search-vector", "--query-vector", "--vectors") if name in args}
with open(os.environ["CALL_LOG"], "a") as log:
    log.write(json.dumps({"service": service, "action": action, "args": args, "files": files}) + "\n")
if (service, action) == ("bedrock-runtime", "invoke-model"):
    mode = os.environ.get("EMBEDDING_CASE", "valid")
    values = [0.1] * (3 if mode == "dimension" else 1024)
    if mode == "overflow": values[0] = 1e39
    pathlib.Path(args[-1]).write_text(json.dumps({"embedding": values}))
    print("{}")
elif (service, action) == ("bedrock-agent", "start-ingestion-job"):
    print(json.dumps({"ingestionJob": {"ingestionJobId": "JOB1234567"}}))
elif (service, action) == ("bedrock-agent", "get-ingestion-job"):
    print(json.dumps({"ingestionJob": {"status": os.environ.get("JOB_STATUS", "COMPLETE")}}))
elif (service, action) in {("sts", "get-caller-identity"), ("dynamodb", "search-vectors"),
                          ("s3vectors", "put-vectors"), ("s3vectors", "query-vectors"),
                          ("bedrock-agent-runtime", "retrieve")}:
    print("{}")
else:
    raise SystemExit("Unexpected fake AWS operation")
'''


def blocks():
    return re.findall(r"```bash\n(.*?)\n```", GUIDE.read_text(), re.S)


def run_sections(selected, missing=None, **updates):
    with tempfile.TemporaryDirectory(prefix="embedding-cli-") as directory:
        root = Path(directory)
        (root / "aws").write_text(FAKE_AWS)
        (root / "aws").chmod(0o700)
        (root / "assets").symlink_to(ROOT / "assets", target_is_directory=True)
        env = {"PATH": f"{root}:{os.environ['PATH']}", "TMPDIR": directory,
               "AWS_PROFILE": "offline-only", "AWS_REGION": "us-east-1",
               "VECTOR_TABLE_NAME": "OfflineVectorTable", "VECTOR_INDEX_NAME": "EmbeddingIndex",
               "TENANT_ID": "tenant-\"quoted", "S3_VECTOR_INDEX_ARN": "arn:aws:s3vectors:us-east-1:111122223333:bucket/example/index/example",
               "VECTOR_KEY": "tenant-doc-v17-chunk-1", "SOURCE_BUCKET": "example-source",
               "SOURCE_KEY": "folder/quoted-\"manual.txt", "SOURCE_VERSION_ID": "v17",
               "KB_ID": "KB12345678", "DS_ID": "DS12345678", "JOB_ID": "JOB1234567",
               "INGESTION_TOKEN": "12345678-1234-1234-1234-123456789abc",
               "CALL_LOG": str(root / "calls.jsonl"), **updates}
        if missing:
            env.pop(missing)
        result = subprocess.run(["bash", "-c", "\n".join(blocks()[i] for i in selected)],
                                cwd=root, env=env, text=True, capture_output=True, timeout=15)
        log = root / "calls.jsonl"
        calls = [json.loads(line) for line in log.read_text().splitlines()] if log.exists() else []
        return result, calls


class EmbeddingCommandsTests(unittest.TestCase):
    def test_routes_are_selective_not_eager(self):
        entry = (ROOT / "SKILL.md").read_text()
        self.assertIn("references/commands/embedding.md", entry)
        self.assertIn("Do not recursively load", entry)
        self.assertNotIn("--cli-binary-format", entry)
        self.assertNotIn("run_path(", entry)
        for name in ["commands/aws-cli.md", "embedding-integrations.md"]:
            self.assertRegex((ROOT / "references" / name).read_text(), r"\]\((?:commands/)?embedding\.md\)")

    def test_all_sections_parse_and_use_explicit_targets(self):
        syntax = subprocess.run(["bash", "-n"], input="\n".join(blocks()), text=True, capture_output=True)
        self.assertEqual(syntax.returncode, 0, syntax.stderr)
        result, calls = run_sections(range(7))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(calls), 8)
        for call in calls:
            args = call["args"]
            self.assertEqual(args[args.index("--profile") + 1], "offline-only")
            self.assertEqual(args[args.index("--region") + 1], "us-east-1")
            self.assertIn("--no-cli-pager", args)
            self.assertNotIn("--no-verify-ssl", args)
            self.assertNotIn(call["action"], {"create-table", "create-index", "create-vector-bucket", "delete-vectors"})

    def test_model_output_becomes_distinct_wire_formats(self):
        result, calls = run_sections([0, 1, 2, 3])
        self.assertEqual(result.returncode, 0, result.stderr)
        model, native, put, query = calls[1:]
        body = model["files"]["--body"]
        self.assertEqual(body["dimensions"], 1024)
        self.assertEqual(body["embeddingTypes"], ["float"])
        self.assertIs(body["normalize"], True)
        self.assertIn("amazon.titan-embed-text-v2:0", model["args"])
        self.assertIn("raw-in-base64-out", model["args"])
        numbers = native["files"]["--search-vector"]
        self.assertEqual(len(numbers), 1024)
        self.assertEqual(set(numbers[0]), {"N"})
        values_arg = native["args"][native["args"].index("--expression-attribute-values") + 1]
        self.assertEqual(json.loads(values_arg)[":tenant"], {"S": 'tenant-"quoted'})
        vector = put["files"]["--vectors"][0]
        self.assertEqual(vector["data"], query["files"]["--query-vector"])
        self.assertEqual(len(vector["data"]["float32"]), 1024)
        self.assertEqual(vector["metadata"]["sourceKey"], 'folder/quoted-"manual.txt')
        self.assertEqual(vector["metadata"]["sourceVersion"], "v17")
        for call in (put, query):
            index = call["args"][call["args"].index("--index-arn") + 1]
            self.assertEqual(index, "arn:aws:s3vectors:us-east-1:111122223333:bucket/example/index/example")
        self.assertIn("--no-paginate", query["args"])

    def test_missing_targets_fail_before_affected_service(self):
        for missing in ["AWS_PROFILE", "AWS_REGION"]:
            result, calls = run_sections([0], missing=missing)
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(calls, [])
        groups = [([0, 1, 2], "dynamodb", ["VECTOR_TABLE_NAME", "VECTOR_INDEX_NAME", "TENANT_ID"]),
                  ([0, 1, 3], "s3vectors", ["S3_VECTOR_INDEX_ARN", "VECTOR_KEY", "TENANT_ID",
                                            "SOURCE_BUCKET", "SOURCE_KEY", "SOURCE_VERSION_ID"]),
                  ([0, 4], "bedrock-agent", ["KB_ID", "DS_ID", "INGESTION_TOKEN"]),
                  ([0, 5], "bedrock-agent", ["KB_ID", "DS_ID", "JOB_ID"])]
        for selected, service, names in groups:
            for missing in names:
                with self.subTest(missing=missing, selected=selected):
                    result, calls = run_sections(selected, missing=missing)
                    self.assertNotEqual(result.returncode, 0)
                    self.assertFalse(any(call["service"] == service for call in calls))
        result, calls = run_sections([0, 1, 3], SOURCE_VERSION_ID="null")
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(any(call["service"] == "s3vectors" for call in calls))

    def test_bad_vectors_stop_before_database_operations(self):
        for mode in ["dimension", "overflow"]:
            with self.subTest(mode=mode):
                result, calls = run_sections([0, 1, 2, 3], EMBEDDING_CASE=mode)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual([call["service"] for call in calls], ["sts", "bedrock-runtime"])

    def test_kb_job_is_inspected_and_incomplete_job_does_not_retrieve(self):
        result, calls = run_sections([0, 4, 5, 6])
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual([call["action"] for call in calls], [
            "get-caller-identity", "start-ingestion-job", "get-ingestion-job", "retrieve"])
        self.assertIn("--client-token", calls[1]["args"])
        self.assertIn("JOB1234567", calls[2]["args"])
        config = calls[3]["args"][calls[3]["args"].index("--retrieval-configuration") + 1]
        self.assertEqual(json.loads(config), {"vectorSearchConfiguration": {"numberOfResults": 5}})
        result, calls = run_sections([0, 5, 6], JOB_STATUS="IN_PROGRESS")
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(any(call["action"] == "retrieve" for call in calls))


if __name__ == "__main__":
    unittest.main(verbosity=2)
