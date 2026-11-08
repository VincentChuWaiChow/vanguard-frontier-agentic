#!/usr/bin/env python3
"""Execute documentation snippets with a fake AWS binary; lint actual variants.

Requires PyYAML and cfn-lint. No network, credentials or real AWS CLI calls.
These tests establish syntax/request structure, not service acceptance or semantics.
"""
import copy
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import unittest

import yaml

ROOT = Path(__file__).resolve().parents[1]
KEY_ARN = "arn:aws:kms:us-east-1:111122223333:key/00000000-0000-4000-8000-000000000000"
FAKE_AWS = '''#!/usr/bin/env python3
import json,os,sys
args=sys.argv[1:]
with open(os.environ["CALL_LOG"],"a") as f: f.write(json.dumps(args)+"\\n")
service=next(v for v in args if v in ("sts","dynamodb","kms"))
action=args[args.index(service)+1]
if service=="dynamodb" and action=="query":
    print(json.dumps({"Items":[],"LastEvaluatedKey":{"PK":{"S":"TENANT#acme#CUSTOMER#c42"},"SK":{"S":"ORDER#next"}}}))
else: print("{}")
'''


class CfnLoader(yaml.SafeLoader):
    pass


CfnLoader.add_constructor("!Ref", lambda loader, node: {"Ref": loader.construct_scalar(node)})
CfnLoader.add_constructor("!GetAtt", lambda loader, node: {"Fn::GetAtt": loader.construct_scalar(node).split(".")})


def blocks(name, language):
    text = (ROOT / "references" / name).read_text()
    return re.findall(r"```" + language + r"\n(.*?)\n```", text, re.S)


def flag(args, name):
    return args[args.index(name) + 1]


def service_action(args):
    service = next(v for v in args if v in ("sts", "dynamodb", "kms"))
    return service, args[args.index(service) + 1]


def run_reference(name, missing=None):
    with tempfile.TemporaryDirectory(prefix="dynamodb-docs-") as directory:
        root = Path(directory)
        (root / "aws").write_text(FAKE_AWS)
        (root / "aws").chmod(0o700)
        (root / "assets").symlink_to(ROOT / "assets", target_is_directory=True)
        env = {"PATH": f"{root}:{os.environ['PATH']}", "AWS_PROFILE": "offline-only",
               "AWS_REGION": "us-east-1", "TABLE_NAME": "OfflineOrders",
               "LSI_TABLE_NAME": "OfflineOrdersWithLSI", "KMS_KEY_ARN": KEY_ARN,
               "CALL_LOG": str(root / "calls.jsonl")}
        if missing:
            env.pop(missing)
        # All alternatives are executed ONLY against mocks to inspect argv.
        result = subprocess.run(["bash", "-c", "\n".join(blocks(name, "bash"))],
                                cwd=root, env=env, text=True, capture_output=True, timeout=15)
        log = root / "calls.jsonl"
        calls = [json.loads(s) for s in log.read_text().splitlines()] if log.exists() else []
        return result, calls


class CommandsTests(unittest.TestCase):
    def test_all_bash_snippets_parse_and_are_targeted(self):
        for document in ("commands/aws-cli.md", "encryption.md"):
            with self.subTest(document=document):
                syntax = subprocess.run(["bash", "-n"], input="\n".join(blocks(document, "bash")),
                                        text=True, capture_output=True)
                self.assertEqual(syntax.returncode, 0, syntax.stderr)
                result, calls = run_reference(document)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertTrue(calls)
                for args in calls:
                    self.assertEqual(flag(args, "--profile"), "offline-only")
                    self.assertEqual(flag(args, "--region"), "us-east-1")
                    self.assertIn("--no-cli-pager", args)
                    self.assertNotIn("--no-verify-ssl", args)
                    if service_action(args)[0] == "dynamodb":
                        self.assertIn(flag(args, "--table-name"),
                                      {"OfflineOrders", "OfflineOrdersWithLSI"})
                    for option in ("--key", "--item", "--expression-attribute-values",
                                   "--expression-attribute-names", "--global-secondary-index-updates",
                                   "--exclusive-start-key"):
                        if option in args:
                            json.loads(flag(args, option))

    def test_missing_target_stops_before_any_aws_call(self):
        for document in ("commands/aws-cli.md", "encryption.md"):
            for name in ("AWS_PROFILE", "AWS_REGION", "TABLE_NAME"):
                with self.subTest(document=document, missing=name):
                    result, calls = run_reference(document, missing=name)
                    self.assertNotEqual(result.returncode, 0)
                    self.assertEqual(calls, [])

    def test_crud_and_pagination_request_invariants(self):
        result, calls = run_reference("commands/aws-cli.md")
        self.assertEqual(result.returncode, 0, result.stderr)
        queries = [a for a in calls if service_action(a) == ("dynamodb", "query")]
        self.assertEqual(len(queries), 4)  # base, base continuation, GSI, LSI
        for args in queries:
            self.assertIn("--no-paginate", args)
        self.assertIn("--exclusive-start-key", queries[1])
        self.assertEqual(json.loads(flag(queries[1], "--exclusive-start-key"))["SK"], {"S": "ORDER#next"})
        for args in queries:
            if "--index-name" in args:
                index = flag(args, "--index-name")
                self.assertIn("--consistent-read" if index == "LSI1" else "--no-consistent-read", args)
        for args in calls:
            if service_action(args)[1] in {"put-item", "update-item", "delete-item"}:
                self.assertIn("--condition-expression", args)
                if "--key" in args:
                    self.assertEqual(set(json.loads(flag(args, "--key"))), {"PK", "SK"})
            if service_action(args)[1] == "update-item":
                self.assertIn("REMOVE GSI1PK, GSI1SK", flag(args, "--update-expression"))

    def test_lsi_is_new_table_only_and_retains_default_schema(self):
        default = json.loads((ROOT / "assets/create-table.json").read_text())
        alternative = json.loads((ROOT / "assets/create-table-with-lsi.json").read_text())
        stripped = copy.deepcopy(alternative)
        lsi = stripped.pop("LocalSecondaryIndexes")
        self.assertEqual(stripped["AttributeDefinitions"].pop(),
                         {"AttributeName": "LSI1SK", "AttributeType": "S"})
        self.assertEqual(stripped, default)
        self.assertEqual(lsi[0]["KeySchema"], [
            {"AttributeName": "PK", "KeyType": "HASH"},
            {"AttributeName": "LSI1SK", "KeyType": "RANGE"}])
        _, calls = run_reference("commands/aws-cli.md")
        for args in calls:
            if service_action(args)[1] == "update-table":
                self.assertNotIn("--local-secondary-indexes", args)
                updates = json.loads(flag(args, "--global-secondary-index-updates"))
                self.assertEqual(updates[0]["Create"]["IndexName"], "GSI2")

    def test_encryption_cli_variants_and_no_dangerous_key_lifecycle(self):
        result, calls = run_reference("encryption.md")
        self.assertEqual(result.returncode, 0, result.stderr)
        creates = [a for a in calls if service_action(a)[1] == "create-table"]
        self.assertEqual(len(creates), 3)
        self.assertNotIn("--sse-specification", creates[0])
        self.assertEqual(flag(creates[1], "--sse-specification"), "Enabled=true,SSEType=KMS")
        self.assertEqual(flag(creates[2], "--sse-specification"),
                         "Enabled=true,SSEType=KMS,KMSMasterKeyId=" + KEY_ARN)
        forbidden = {"disable-key", "schedule-key-deletion", "create-key", "put-key-policy"}
        self.assertFalse(forbidden & {service_action(a)[1] for a in calls})

    def test_actual_cfn_and_cli_variants_lint(self):
        template = yaml.load((ROOT / "assets/table.cfn.yaml").read_text(), Loader=CfnLoader)
        variants = []
        encryption = [yaml.load(b, Loader=CfnLoader) for b in blocks("encryption.md", "yaml")]
        specifications = [b["SSESpecification"] for b in encryption if "SSESpecification" in b]
        self.assertEqual(specifications, [
            {"SSEEnabled": False}, {"SSEEnabled": True, "SSEType": "KMS"},
            {"SSEEnabled": True, "SSEType": "KMS", "KMSMasterKeyId": {"Ref": "KmsKeyArn"}}])
        for spec in specifications:
            variant = copy.deepcopy(template)
            variant["Resources"]["OrdersTable"]["Properties"]["SSESpecification"] = spec
            if "KMSMasterKeyId" in spec:
                for item in encryption:
                    variant.setdefault("Parameters", {}).update(item.get("Parameters", {}))
            variants.append(variant)
        lsi = json.loads((ROOT / "assets/create-table-with-lsi.json").read_text())
        variants.append({"Resources": {"Orders": {"Type": "AWS::DynamoDB::Table", "Properties": lsi}}})
        fragment = yaml.safe_load(blocks("lsi-vs-gsi.md", "yaml")[0])
        self.assertEqual(fragment["LocalSecondaryIndexes"], lsi["LocalSecondaryIndexes"])
        with tempfile.TemporaryDirectory(prefix="dynamodb-cfn-") as directory:
            for number, variant in enumerate(variants):
                path = Path(directory) / f"variant-{number}.json"
                path.write_text(json.dumps(variant))
                result = subprocess.run(["cfn-lint", str(path)], capture_output=True, text=True, timeout=30)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main(verbosity=2)
