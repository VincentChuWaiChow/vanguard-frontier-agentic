#!/usr/bin/env python3
"""Offline package/asset contract checks. Requires PyYAML; never contacts AWS.

Semantic guidance is tested separately by adversarial responses/domain review.
Negative fixtures ensure the asset checker rejects common broken table examples.
"""
from __future__ import annotations

import copy
import json
from pathlib import Path
import re
import unittest

import yaml

ROOT = Path(__file__).resolve().parents[1]


class CfnLoader(yaml.SafeLoader):
    pass


for tag in ("!Ref", "!GetAtt"):
    CfnLoader.add_constructor(tag, lambda loader, node: loader.construct_scalar(node))


def assets():
    cfn = yaml.load((ROOT / "assets/table.cfn.yaml").read_text(), Loader=CfnLoader)
    cli = json.loads((ROOT / "assets/create-table.json").read_text())
    return cfn, cli


def check_assets(cfn, cli):
    resource = cfn["Resources"]["OrdersTable"]
    props = resource["Properties"]
    assert resource["Type"] == "AWS::DynamoDB::Table"
    assert resource["DeletionPolicy"] == resource["UpdateReplacePolicy"] == "Retain"
    for field in ("BillingMode", "DeletionProtectionEnabled", "AttributeDefinitions",
                  "KeySchema", "GlobalSecondaryIndexes"):
        assert props[field] == cli[field], f"CFN/CLI differ: {field}"
    assert "TableName" not in cli, "CLI target must come from explicit argument"
    assert props["BillingMode"] == "PAY_PER_REQUEST"
    assert props["DeletionProtectionEnabled"] is True
    for shape in (props, cli, *props["GlobalSecondaryIndexes"],
                  *cli["GlobalSecondaryIndexes"]):
        assert "ProvisionedThroughput" not in shape
    definitions = props["AttributeDefinitions"]
    declared = {value["AttributeName"] for value in definitions}
    used = {value["AttributeName"] for value in props["KeySchema"]}
    for index in props["GlobalSecondaryIndexes"]:
        used.update(value["AttributeName"] for value in index["KeySchema"])
        projection = index["Projection"]
        assert projection["ProjectionType"] == "INCLUDE"
        assert set(projection["NonKeyAttributes"]) == {
            "status", "orderId", "totalMinor", "currency"}
        assert not (set(projection["NonKeyAttributes"]) & used)
    assert declared == used == {"PK", "SK", "GSI1PK", "GSI1SK"}
    assert len(definitions) == len(declared)
    assert all(value["AttributeType"] == "S" for value in definitions)
    assert props["KeySchema"] == [
        {"AttributeName": "PK", "KeyType": "HASH"},
        {"AttributeName": "SK", "KeyType": "RANGE"}]
    assert len(props["GlobalSecondaryIndexes"]) == 1
    assert props["GlobalSecondaryIndexes"][0]["IndexName"] == "GSI1"
    assert props["TimeToLiveSpecification"] == {
        "AttributeName": "expiresAt", "Enabled": True}
    assert props["PointInTimeRecoverySpecification"]["PointInTimeRecoveryEnabled"] is True


class PackageTests(unittest.TestCase):
    def test_asset_contract(self):
        check_assets(*assets())

    def test_negative_asset_fixtures(self):
        mutations = [
            lambda c, a: a.update(BillingMode="PROVISIONED"),
            lambda c, a: a["AttributeDefinitions"].append(
                {"AttributeName": "status", "AttributeType": "S"}),
            lambda c, a: a["AttributeDefinitions"].pop(),
            lambda c, a: a.update(ProvisionedThroughput={}),
            lambda c, a: a["GlobalSecondaryIndexes"][0].update(ProvisionedThroughput={}),
            lambda c, a: a["GlobalSecondaryIndexes"][0].update(IndexName="Wrong"),
            lambda c, a: c["Resources"]["OrdersTable"].update(DeletionPolicy="Delete"),
            lambda c, a: c["Resources"]["OrdersTable"]["Properties"].update(
                TimeToLiveSpecification={"AttributeName": "ttl", "Enabled": True}),
            lambda c, a: c["Resources"]["OrdersTable"]["Properties"].update(
                PointInTimeRecoverySpecification={"PointInTimeRecoveryEnabled": False}),
        ]
        for number, mutate in enumerate(mutations):
            with self.subTest(mutation=number):
                cfn, cli = copy.deepcopy(assets())
                mutate(cfn, cli)
                with self.assertRaises(AssertionError):
                    check_assets(cfn, cli)

    def test_common_mode_errors_do_not_pass_on_equivalence_alone(self):
        for mutation in ("extra-attribute", "wrong-type", "provisioned", "bad-key"):
            with self.subTest(mutation=mutation):
                cfn, cli = assets()
                props = cfn["Resources"]["OrdersTable"]["Properties"]
                for shape in (props, cli):
                    if mutation == "extra-attribute":
                        shape["AttributeDefinitions"].append(
                            {"AttributeName": "status", "AttributeType": "S"})
                    elif mutation == "wrong-type":
                        shape["AttributeDefinitions"][0]["AttributeType"] = "N"
                    elif mutation == "provisioned":
                        shape["ProvisionedThroughput"] = {"ReadCapacityUnits": 1,
                                                          "WriteCapacityUnits": 1}
                    else:
                        shape["KeySchema"][0]["KeyType"] = "RANGE"
                with self.assertRaises(AssertionError):
                    check_assets(cfn, cli)

    def test_metadata_resources_and_whitespace(self):
        skill = (ROOT / "SKILL.md").read_text()
        meta = yaml.safe_load(skill.split("---", 2)[1])
        self.assertEqual(meta["name"], ROOT.name)
        self.assertEqual(meta["metadata"]["version"], "1.8.0")
        self.assertLess(len(skill.splitlines()), 250)
        for file in sorted(ROOT.rglob("*")):
            if not file.is_file() or "__pycache__" in file.parts:
                continue
            text = file.read_text()
            self.assertTrue(text.endswith("\n"), str(file))
            for number, line in enumerate(text.splitlines(), 1):
                self.assertEqual(line, line.rstrip(), f"{file}:{number}: trailing space")
            if file.suffix == ".md":
                self.assertNotRegex(text, r"/home/|/Users/|[A-Z]:\\Users\\")
                for link in re.findall(r"\]\(([^)]+)\)", text):
                    if "://" in link or link.startswith("#"):
                        continue
                    target = (file.parent / link.split("#")[0]).resolve()
                    self.assertTrue(target.is_relative_to(ROOT.resolve()), link)
                    self.assertTrue(target.exists(), f"{file}: {link}")

    def test_json_assets_and_fences(self):
        for file in ROOT.rglob("*.json"):
            json.loads(file.read_text())
        for file in ROOT.rglob("*.md"):
            for block in re.findall(r"```json\n(.*?)\n```", file.read_text(), re.S):
                json.loads(block)
        request = json.loads((ROOT / "assets/query-input.json").read_text())
        self.assertNotIn("IndexName", request)  # strong base-table example
        self.assertTrue(request["ConsistentRead"])
        self.assertNotIn("FilterExpression", request)
        self.assertEqual(request["ExpressionAttributeValues"][":prefix"], {"S": "ORDER#"})

    def test_eval_manifest_and_adapter_presence(self):
        manifest = json.loads((ROOT / "evals/evals.json").read_text())
        self.assertEqual(manifest["skill_name"], ROOT.name)
        self.assertEqual([case["id"] for case in manifest["evals"]], list(range(1, 13)))
        for case in manifest["evals"]:
            self.assertEqual(len(case["expectations"]), 6)
            self.assertTrue(case["prompt"] and case["expected_output"])
        text = (ROOT / "references/language-adapters.md").read_text()
        for language in ("JavaScript", "TypeScript", "Java", "Kotlin", "Rust", "PHP"):
            self.assertIn(f"## {language} (", text)
        # Presence checks are not compilation or semantic validation.
        for fence in ("javascript", "typescript", "java", "kotlin", "rust", "php"):
            self.assertIn(f"```{fence}\n", text)


if __name__ == "__main__":
    unittest.main(verbosity=2)
