#!/usr/bin/env python3
"""Offline tests for domain command argv and the shipped illustrative policies.

The policy checker encodes this package's narrow example contracts, NOT arbitrary
IAM evaluation. Passing it is not Access Analyzer, simulation or runtime proof.
Fake CLI execution never uses the real AWS binary, credentials or network.
"""
import copy
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
DOCS = ["iam-and-access.md", "streams-and-dax.md", "recovery-and-replicas.md", "monitoring.md"]
TABLE = "arn:aws:dynamodb:us-east-1:111122223333:table/Orders"
INDEX = TABLE + "/index/GSI1"
STREAM = TABLE + "/stream/2026-11-08T00:00:00.000"
PRINCIPAL = "arn:aws:iam::444455556666:role/DynamoReader"
SERVICES = {"sts", "iam", "accessanalyzer", "dynamodb", "dynamodbstreams", "dax", "lambda", "cloudwatch"}
FAKE = '''#!/usr/bin/env python3
import json,os,sys
with open(os.environ["CALL_LOG"],"a") as f: f.write(json.dumps(sys.argv[1:])+"\\n")
if "Table.GlobalSecondaryIndexes" in sys.argv:
    print(os.environ.get("FAKE_INDEXES",'[{"IndexName":"OrdersByStatus","IndexStatus":"ACTIVE"}]'))
else: print("{}")
'''


def blocks(name):
    return re.findall(r"```bash\n(.*?)\n```", (ROOT / "references" / name).read_text(), re.S)


def flag(args, key):
    return args[args.index(key) + 1]


def values(args, key):
    result = []
    for value in args[args.index(key) + 1:]:
        if value.startswith("--"):
            break
        result.append(value)
    return result


def operation(args):
    service = next(value for value in args if value in SERVICES)
    return service, args[args.index(service) + 1]


def run_doc(name, missing=None, overrides=None):
    with tempfile.TemporaryDirectory(prefix="ddb-operations-") as directory:
        root = Path(directory)
        (root / "aws").write_text(FAKE)
        (root / "aws").chmod(0o700)
        (root / "assets").symlink_to(ROOT / "assets", target_is_directory=True)
        env = {
            "PATH": f"{root}:{os.environ['PATH']}", "AWS_PROFILE": "offline-only",
            "AWS_REGION": "us-east-1", "TABLE_NAME": "Orders", "TABLE_ARN": TABLE,
            "STREAM_ARN": STREAM, "SHARD_ID": "shardId-offline", "SHARD_ITERATOR": "offline-iterator",
            "FUNCTION_NAME": "offline-consumer", "DAX_CLUSTER_NAME": "offline-dax",
            "DAX_NODE_TYPE": "dax.r5.large", "DAX_SUBNET_GROUP": "offline-subnets",
            "DAX_SECURITY_GROUP_ID": "sg-00000000", "DAX_ROLE_ARN": "arn:aws:iam::111122223333:role/DaxService",
            "RESTORE_TABLE_NAME": "OrdersRecovered", "REPLICA_REGION": "eu-west-1",
            "IDENTITY_POLICY_FILE": "assets/table-index-read-policy.json",
            "RESOURCE_POLICY_FILE": "assets/table-resource-policy.json", "EXPECTED_REVISION_ID": "observed-r1",
            "METRIC_START_UTC": "2026-11-08T00:00:00Z", "METRIC_END_UTC": "2026-11-08T01:00:00Z",
            "ALARM_NAME": "offline-gsi-throttles", "SNS_TOPIC_ARN": "arn:aws:sns:us-east-1:111122223333:OfflineAlerts",
            "GSI_NAME": "OrdersByStatus",
            "CALL_LOG": str(root / "calls.jsonl"),
        }
        env.update(overrides or {})
        if missing:
            env.pop(missing)
        result = subprocess.run(["bash", "-c", "\n".join(blocks(name))],
                                cwd=root, env=env, text=True, capture_output=True, timeout=20)
        log = root / "calls.jsonl"
        calls = [json.loads(line) for line in log.read_text().splitlines()] if log.exists() else []
        return result, calls


def as_list(value):
    return value if isinstance(value, list) else [value]


def check_policy(policy, kind):
    """The three deliberately read-only example shapes, not a general IAM validator."""
    assert policy["Version"] == "2012-10-17"
    found_index = False
    for statement in policy["Statement"]:
        assert statement["Effect"] == "Allow"
        actions = set(as_list(statement["Action"]))
        resources = set(as_list(statement["Resource"]))
        assert all("*" not in a for a in actions)
        assert all("*" not in r for r in resources)
        if kind == "stream":
            assert actions <= {"dynamodb:DescribeStream", "dynamodb:GetShardIterator", "dynamodb:GetRecords"}
            assert resources == {STREAM}
            assert "Principal" not in statement
        else:
            assert actions <= {"dynamodb:GetItem", "dynamodb:BatchGetItem", "dynamodb:Query"}
            assert resources <= {TABLE, INDEX} and resources
            if INDEX in resources:
                assert actions == {"dynamodb:Query"} and TABLE in resources
                found_index = True
            if kind == "resource":
                assert statement["Principal"] == {"AWS": PRINCIPAL}
                assert "dynamodb:BatchGetItem" not in actions
            else:
                assert "Principal" not in statement
        assert actions
    if kind != "stream":
        assert found_index, "The shipped example promises Query on the named index"


class OperationsTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.runs = {name: run_doc(name) for name in DOCS}

    def test_domain_snippets_syntax_and_explicit_targets(self):
        for name, (result, calls) in self.runs.items():
            with self.subTest(document=name):
                syntax = subprocess.run(["bash", "-n"], input="\n".join(blocks(name)),
                                        text=True, capture_output=True)
                self.assertEqual(syntax.returncode, 0, syntax.stderr)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertTrue(calls)
                for args in calls:
                    self.assertEqual(flag(args, "--profile"), "offline-only")
                    self.assertIn(flag(args, "--region"), {"us-east-1", "eu-west-1"})
                    self.assertIn("--no-cli-pager", args)
                    self.assertNotIn("--no-verify-ssl", args)
                    self.assertNotIn("--confirm-remove-self-resource-access", args)

    def test_missing_core_target_fails_before_service_call(self):
        for name in DOCS:
            for variable in ("AWS_PROFILE", "AWS_REGION", "TABLE_NAME"):
                with self.subTest(document=name, missing=variable):
                    result, calls = run_doc(name, missing=variable)
                    self.assertNotEqual(result.returncode, 0)
                    self.assertEqual(calls, [])

    def test_iam_tags_and_resource_policy_group_arguments(self):
        calls = self.runs["iam-and-access.md"][1]
        actions = {operation(args) for args in calls}
        self.assertIn(("accessanalyzer", "validate-policy"), actions)
        self.assertIn(("iam", "simulate-custom-policy"), actions)
        tagged = set()
        for args in calls:
            service, action = operation(args)
            if action in {"tag-resource", "untag-resource", "list-tags-of-resource"}:
                self.assertIn(flag(args, "--resource-arn"), {TABLE, STREAM})
            if action == "tag-resource":
                tagged.add(flag(args, "--resource-arn"))
            if action == "put-resource-policy":
                self.assertEqual(flag(args, "--resource-arn"), TABLE)
                self.assertEqual(flag(args, "--expected-revision-id"), "observed-r1")
                self.assertEqual(flag(args, "--policy"), "file://assets/table-resource-policy.json")
            if service == "iam":
                json.loads(flag(args, "--policy-input-list"))
                self.assertEqual(set(values(args, "--resource-arns")), {TABLE, INDEX})
        self.assertEqual(tagged, {TABLE, STREAM})

    def test_stream_and_dax_groups_preserve_distinct_protocol_and_role(self):
        calls = self.runs["streams-and-dax.md"][1]
        for args in calls:
            service, action = operation(args)
            if service == "dynamodbstreams" and action != "get-records":
                self.assertEqual(flag(args, "--stream-arn"), STREAM)
            if action == "get-shard-iterator":
                self.assertEqual(flag(args, "--shard-iterator-type"), "TRIM_HORIZON")
            if action == "get-records":
                self.assertEqual(flag(args, "--limit"), "25")
            if service == "dax" and action == "create-cluster":
                self.assertEqual(flag(args, "--replication-factor"), "3")
                self.assertEqual(flag(args, "--cluster-endpoint-encryption-type"), "TLS")
                self.assertEqual(flag(args, "--sse-specification"), "Enabled=true")
                self.assertIn("--iam-role-arn", args)
                self.assertIn("--security-group-ids", args)
        self.assertIn(("lambda", "list-event-source-mappings"), {operation(a) for a in calls})

    def test_pitr_and_replica_requests_have_safe_targets(self):
        calls = self.runs["recovery-and-replicas.md"][1]
        actions = {operation(a)[1] for a in calls}
        self.assertNotIn("create-global-table", actions)
        self.assertNotIn("update-global-table", actions)
        for args in calls:
            action = operation(args)[1]
            if action == "restore-table-to-point-in-time":
                self.assertNotEqual(flag(args, "--source-table-name"), flag(args, "--target-table-name"))
                self.assertIn("--use-latest-restorable-time", args)
                self.assertNotIn("--restore-date-time", args)
            if action == "update-table":
                self.assertEqual(flag(args, "--region"), "us-east-1")
                self.assertEqual(json.loads(flag(args, "--replica-updates")),
                                 [{"Create": {"RegionName": "eu-west-1"}}])
        self.assertTrue(any(flag(a, "--region") == "eu-west-1" for a in calls))

    def test_unsafe_restore_or_replica_target_is_rejected(self):
        result, calls = run_doc("recovery-and-replicas.md", overrides={"RESTORE_TABLE_NAME": "Orders"})
        self.assertEqual(result.returncode, 64)
        self.assertNotIn("restore-table-to-point-in-time", {operation(a)[1] for a in calls})
        result, calls = run_doc("recovery-and-replicas.md", overrides={"REPLICA_REGION": "us-east-1"})
        self.assertEqual(result.returncode, 64)
        self.assertFalse(any("--replica-updates" in a for a in calls))

    def test_metrics_and_alarm_match_dimension_and_statistic_contract(self):
        calls = self.runs["monitoring.md"][1]
        metric_names = set()
        for args in calls:
            if "--metric-name" not in args:
                continue
            metric = flag(args, "--metric-name")
            metric_names.add(metric)
            dimensions = {v.split(",")[0].split("=", 1)[1]: v.split("Value=", 1)[1]
                          for v in values(args, "--dimensions")}
            if metric in {"SuccessfulRequestLatency", "SystemErrors"}:
                self.assertEqual(set(dimensions), {"TableName", "Operation"})
            if metric == "WriteThrottleEvents":
                self.assertEqual(set(dimensions), {"TableName", "GlobalSecondaryIndexName"})
                self.assertEqual(dimensions["GlobalSecondaryIndexName"], "OrdersByStatus")
            if metric == "ReplicationLatency":
                self.assertEqual(set(dimensions), {"TableName", "ReceivingRegion"})
            if metric == "IteratorAge":
                self.assertEqual(flag(args, "--namespace"), "AWS/Lambda")
                self.assertEqual(flag(args, "--statistics"), "Maximum")
            if metric == "ItemCacheHits":
                self.assertEqual(flag(args, "--namespace"), "AWS/DAX")
                self.assertEqual(set(dimensions), {"ClusterId"})
            if operation(args)[1] == "put-metric-alarm":
                self.assertIn("--alarm-actions", args)
                self.assertIn("--actions-enabled", args)
                self.assertEqual(flag(args, "--treat-missing-data"), "ignore")
                self.assertEqual(flag(args, "--statistic"), "Sum")
        self.assertTrue({"ReadThrottleEvents", "WriteThrottleEvents", "SuccessfulRequestLatency",
                         "SystemErrors", "ReplicationLatency", "IteratorAge", "ItemCacheHits"} <= metric_names)

    def test_wrong_or_inactive_index_stops_before_metric_or_alarm_calls(self):
        for response in ["[]", "null", '[{"IndexName":"Other","IndexStatus":"ACTIVE"}]',
                         '[{"IndexName":"OrdersByStatus","IndexStatus":"CREATING"}]']:
            with self.subTest(response=response):
                result, calls = run_doc("monitoring.md", overrides={"FAKE_INDEXES": response})
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual({operation(args)[0] for args in calls}, {"dynamodb"})
        result, calls = run_doc("monitoring.md", missing="GSI_NAME")
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(calls, [])

    def test_shipped_policy_contracts(self):
        for name, kind in [("table-index-read-policy.json", "identity"),
                           ("stream-read-policy.json", "stream"),
                           ("table-resource-policy.json", "resource")]:
            check_policy(json.loads((ROOT / "assets" / name).read_text()), kind)

    def test_policy_negative_fixtures(self):
        identity = json.loads((ROOT / "assets/table-index-read-policy.json").read_text())
        stream = json.loads((ROOT / "assets/stream-read-policy.json").read_text())
        resource = json.loads((ROOT / "assets/table-resource-policy.json").read_text())
        fixtures = []
        for field, value in [("Resource", "*"), ("Resource", INDEX),
                             ("Action", ["dynamodb:TransactWriteItems"]),
                             ("Action", ["dynamodb:ExecuteStatement"])]:
            bad = copy.deepcopy(identity)
            bad["Statement"][0][field] = value
            fixtures.append((bad, "identity"))
        bad = copy.deepcopy(identity)
        bad["Statement"][1]["Resource"] = [TABLE]
        fixtures.append((bad, "identity"))
        for field, value in [("Resource", TABLE), ("Action", ["dynamodbstreams:GetRecords"])]:
            bad = copy.deepcopy(stream)
            bad["Statement"][0][field] = value
            fixtures.append((bad, "stream"))
        bad = copy.deepcopy(resource)
        bad["Statement"][0]["Principal"] = "*"
        fixtures.append((bad, "resource"))
        for number, (policy, kind) in enumerate(fixtures):
            with self.subTest(fixture=number):
                with self.assertRaises(AssertionError):
                    check_policy(policy, kind)


if __name__ == "__main__":
    unittest.main(verbosity=2)
