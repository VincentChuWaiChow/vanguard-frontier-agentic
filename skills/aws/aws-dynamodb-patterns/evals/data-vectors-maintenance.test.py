#!/usr/bin/env python3
"""Offline reminder/vector fixture tests; no live search or general IAM evaluation."""
import copy
from datetime import date
from decimal import Decimal
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import unittest

import yaml

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("review_due", ROOT / "scripts/review_due.py")
review = importlib.util.module_from_spec(spec)
spec.loader.exec_module(review)


def load(name):
    return json.loads((ROOT / "assets" / name).read_text())


def check_vectors(create, item, search):
    """Narrow contract for this example; not an AWS schema/authorization validator."""
    assert create["BillingMode"] == "PAY_PER_REQUEST"
    index = create["VectorIndexes"][0]
    assert len(create["VectorIndexes"]) == 1
    assert index["IndexName"] == search["IndexName"] == "EmbeddingIndex"
    dimension = index["Dimensions"]
    assert type(dimension) is int and 1 <= dimension <= 4096
    declared = {a["AttributeName"] for a in create["AttributeDefinitions"]}
    expected = {k["AttributeName"] for k in create["KeySchema"]}
    expected.update(s["AttributeName"] for s in index["SearchSchema"])
    assert declared == expected == {"PK", "Tenant", "Status"}
    assert len(create["AttributeDefinitions"]) == len(declared)
    assert all(a["AttributeType"] == "S" for a in create["AttributeDefinitions"])
    assert index["VectorAttribute"] == {"AttributeName": "Embedding"}
    assert {k["SearchSchemaElementType"] for k in index["SearchSchema"]} == {"HASH", "INLINE_FILTER"}
    assert set(item["Embedding"]) == {"L"}  # ordered list, not NS or an invented type
    for vector in [item["Embedding"]["L"], search["SearchVector"]]:
        assert isinstance(vector, list) and len(vector) == dimension
        numbers = []
        for value in vector:
            assert set(value) == {"N"} and isinstance(value["N"], str)
            number = Decimal(value["N"])
            assert number.is_finite() and abs(number) <= Decimal("3.4028234663852886E38")
            numbers.append(number)
        assert any(number != 0 for number in numbers), "Example avoids undefined cosine zero vectors"
    assert type(search["TopK"]) is int and 1 <= search["TopK"] <= 100
    assert search["SearchConditionExpression"] == "#tenant = :tenant AND #status = :status"
    assert search["ExpressionAttributeNames"] == {"#tenant": "Tenant", "#status": "Status"}
    assert search["ExpressionAttributeValues"][":tenant"] == item["Tenant"]
    assert search["ExpressionAttributeValues"][":status"] == item["Status"]
    assert not ({"ConsistentRead", "ExclusiveStartKey", "FilterExpression"} & set(search))


class ReminderTests(unittest.TestCase):
    def test_initial_calendar_window_and_missed_reviews(self):
        schedule = load("content-review.json")
        self.assertEqual(schedule["next_review_due"], "2026-12-01")
        self.assertIsNone(schedule["last_full_review"])
        for when, expected in [("2026-11-08", "not_due"), ("2026-11-30", "not_due"),
                               ("2026-12-01", "due"), ("2026-12-31", "due"),
                               ("2027-01-01", "overdue"), ("2027-06-01", "overdue")]:
            with self.subTest(date=when):
                result = review.review_status(schedule, date.fromisoformat(when))
                self.assertEqual(result["status"], expected)
                self.assertEqual(result["next_review_due"], "2026-12-01")

    def test_completed_review_moves_to_next_calendar_window(self):
        schedule = load("content-review.json")
        schedule.update(last_full_review="2026-12-15", next_review_due="2027-06-01")
        for when, expected in [("2027-05-31", "not_due"), ("2027-06-01", "due"),
                               ("2027-06-30", "due"), ("2027-07-01", "overdue")]:
            self.assertEqual(review.review_status(schedule, date.fromisoformat(when))["status"], expected)
        self.assertEqual(review.next_window_after(date(2027, 6, 30)), date(2027, 12, 1))
        self.assertEqual(review.next_window_after(date(2027, 1, 15)), date(2027, 6, 1))
        self.assertEqual(review.next_window_after(date(2026, 12, 1)), date(2027, 6, 1))

    def test_bad_schedule_is_rejected_not_silently_reset(self):
        for updates in [{"review_months": [5, 11]}, {"next_review_due": "2026-12-15"},
                        {"last_full_review": "2026-12-15"}, {"next_review_due": "bad-date"}]:
            schedule = load("content-review.json")
            schedule.update(updates)
            with self.assertRaises(ValueError):
                review.review_status(schedule, date(2026, 11, 8))

    def test_cli_reminder_is_read_only_and_not_an_external_notification(self):
        path = ROOT / "assets/content-review.json"
        before = path.read_bytes()
        result = subprocess.run([sys.executable, str(ROOT / "scripts/review_due.py"),
                                 "--as-of", "2026-12-01"], capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout)["status"], "due")
        self.assertFalse(json.loads(result.stdout)["background_notification"])
        self.assertEqual(path.read_bytes(), before)


class VectorTests(unittest.TestCase):
    def test_typed_item_and_request_contract(self):
        item = load("typed-vector-item.json")
        self.assertEqual({tag for value in item.values() for tag in value},
                         {"S", "N", "B", "BOOL", "NULL", "L", "M", "SS", "NS", "BS"})
        check_vectors(load("vector-create-table.json"), item, load("vector-search.json"))
        policy = load("vector-search-policy.json")
        statement = policy["Statement"][0]
        self.assertEqual(statement["Action"], "dynamodb:SearchVectors")
        self.assertEqual(statement["Resource"],
                         "arn:aws:dynamodb:us-east-1:111122223333:table/VectorDocuments/index/EmbeddingIndex")
        self.assertNotIn("Condition", statement)  # no fabricated LeadingKeys protection

    def test_bad_vector_type_dimensions_schema_and_search_fixtures(self):
        mutations = [
            lambda c, i, s: c.update(BillingMode="PROVISIONED"),
            lambda c, i, s: c["VectorIndexes"][0].update(Dimensions=4),
            lambda c, i, s: c["AttributeDefinitions"].pop(),
            lambda c, i, s: c["AttributeDefinitions"].append({"AttributeName": "Embedding", "AttributeType": "L"}),
            lambda c, i, s: i.update(Embedding={"NS": ["0.1", "0.2", "0.3"]}),
            lambda c, i, s: i.update(Embedding={"V": [0.1, 0.2, 0.3]}),
            lambda c, i, s: s.update(SearchVector={"L": s["SearchVector"]}),
            lambda c, i, s: s["SearchVector"].__setitem__(0, {"N": "NaN"}),
            lambda c, i, s: s["SearchVector"].__setitem__(0, {"S": "0.1"}),
            lambda c, i, s: s["SearchVector"].__setitem__(0, {"N": "1e39"}),
            lambda c, i, s: s.update(TopK=101),
            lambda c, i, s: s.update(SearchConditionExpression="#status = :status"),
            lambda c, i, s: s.update(ConsistentRead=True),
        ]
        for number, mutate in enumerate(mutations):
            create, item, search = [copy.deepcopy(load(name)) for name in
                                    ["vector-create-table.json", "typed-vector-item.json", "vector-search.json"]]
            mutate(create, item, search)
            with self.subTest(fixture=number):
                with self.assertRaises(AssertionError):
                    check_vectors(create, item, search)

    def test_cfn_cli_equivalence_and_explicit_lint_compatibility_finding(self):
        class Loader(yaml.SafeLoader):
            pass
        Loader.add_constructor("!Ref", lambda loader, node: loader.construct_scalar(node))
        template = yaml.load((ROOT / "assets/vector-table.cfn.yaml").read_text(), Loader=Loader)
        resource = template["Resources"]["VectorDocuments"]
        props = copy.deepcopy(resource["Properties"])
        props.pop("TableName")
        self.assertEqual(props, load("vector-create-table.json"))
        self.assertEqual(resource["DeletionPolicy"], "Retain")
        self.assertEqual(resource["UpdateReplacePolicy"], "Retain")
        path = str(ROOT / "assets/vector-table.cfn.yaml")
        strict = subprocess.run(["cfn-lint", "--format", "json", "--template", path],
                                text=True, capture_output=True, timeout=30)
        findings = json.loads(strict.stdout or "[]")
        if strict.returncode:
            self.assertTrue(findings, strict.stderr)
            self.assertEqual({f["Rule"]["Id"] for f in findings}, {"E3039"})
            print("NOTE: strict vector lint E3039 compatibility MISS, not a strict pass")
        else:
            self.assertEqual(findings, [])
        # Exact declared-set semantics were independently checked above. No global
        # suppression, inline ignore or removal of AWS-required attributes.
        result = subprocess.run(["cfn-lint", "--ignore-checks", "E3039", "--template", path],
                                text=True, capture_output=True, timeout=30)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertNotIn("Metadata", template)

    def test_actual_vector_commands_against_fake_cli(self):
        text = (ROOT / "references/data-types-and-vectors.md").read_text()
        code = "\n".join(re.findall(r"```bash\n(.*?)\n```", text, re.S))
        with tempfile.TemporaryDirectory(prefix="ddb-vector-cli-") as directory:
            root = Path(directory)
            (root / "aws").write_text('#!/usr/bin/env python3\nimport json,os,sys\nwith open(os.environ["LOG"],"a") as f: f.write(json.dumps(sys.argv[1:])+"\\n")\nprint("{}")\n')
            (root / "aws").chmod(0o700)
            (root / "assets").symlink_to(ROOT / "assets", target_is_directory=True)
            env = {"PATH": f"{root}:{os.environ['PATH']}", "AWS_PROFILE": "offline-only",
                   "AWS_REGION": "us-east-1", "VECTOR_TABLE_NAME": "VectorDocuments", "LOG": str(root / "calls")}
            result = subprocess.run(["bash", "-c", code], cwd=root, env=env,
                                    text=True, capture_output=True, timeout=15)
            self.assertEqual(result.returncode, 0, result.stderr)
            calls = [json.loads(line) for line in (root / "calls").read_text().splitlines()]
            actions = []
            for args in calls:
                service = "dynamodb" if "dynamodb" in args else "sts"
                action = args[args.index(service) + 1]
                actions.append(action)
                self.assertEqual(args[args.index("--profile") + 1], "offline-only")
                self.assertEqual(args[args.index("--region") + 1], "us-east-1")
                if action == "search-vectors":
                    self.assertIn("file://assets/vector-search.json", args)
                    self.assertFalse({"--consistent-read", "--exclusive-start-key", "--limit"} & set(args))
                if action == "put-item":
                    self.assertIn("--condition-expression", args)
                    self.assertEqual(args[args.index("--cli-binary-format") + 1], "base64")
            self.assertEqual(actions, ["get-caller-identity", "create-table", "wait", "describe-table", "put-item", "search-vectors"])


if __name__ == "__main__":
    unittest.main(verbosity=2)
