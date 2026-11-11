#!/usr/bin/env python3
"""Exercise the live-audit example only with fake metadata; never contact AWS."""
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
LIVE = ROOT / "references/commands/audit/live.md"
PRINCIPAL = "arn:aws:sts::111122223333:assumed-role/ApprovedAuditRole/example"
FAKE_AWS = r'''#!/usr/bin/env python3
import json, os, sys
args = sys.argv[1:]
with open(os.environ["CALL_LOG"], "a") as log:
    log.write(json.dumps(args) + "\n")
if "sts" in args and "get-caller-identity" in args:
    print(json.dumps({"Account": os.environ.get("FAKE_ACCOUNT", "111122223333"),
                      "Arn": os.environ.get("FAKE_PRINCIPAL", os.environ["EXPECTED_PRINCIPAL_ARN"])}))
elif "dynamodb" in args and "describe-table" in args:
    print(json.dumps({"Table": {"TableName": args[args.index("--table-name") + 1], "TableStatus": "ACTIVE"}}))
else:
    raise SystemExit("Forbidden non-metadata operation in audit example")
'''


def run_live(missing=None, **updates):
    with tempfile.TemporaryDirectory(prefix="audit-offline-test-") as directory:
        root = Path(directory)
        (root / "aws").write_text(FAKE_AWS)
        (root / "aws").chmod(0o700)
        env = {"PATH": f"{root}:{os.environ['PATH']}", "TMPDIR": directory,
               "AWS_PROFILE": "offline-only", "AWS_REGION": "us-east-1", "TABLE_NAME": "OfflineTable",
               "EXPECTED_ACCOUNT_ID": "111122223333", "EXPECTED_PRINCIPAL_ARN": PRINCIPAL,
               "CALL_LOG": str(root / "calls.jsonl"), **updates}
        if missing:
            env.pop(missing)
        script = "\n".join(re.findall(r"```bash\n(.*?)\n```", LIVE.read_text(), re.S))
        result = subprocess.run(["bash", "-c", script], cwd=ROOT, env=env,
                                text=True, capture_output=True, timeout=10)
        log = root / "calls.jsonl"
        calls = [json.loads(line) for line in log.read_text().splitlines()] if log.exists() else []
        tables = [json.loads(path.read_text()) for path in root.rglob("table.json")]
        return result, calls, tables


class AuditTests(unittest.TestCase):
    def test_actual_live_example_is_bounded_targeted_metadata_only(self):
        blocks = re.findall(r"```bash\n(.*?)\n```", LIVE.read_text(), re.S)
        self.assertEqual(len(blocks), 1)
        syntax = subprocess.run(["bash", "-n"], input=blocks[0], text=True, capture_output=True)
        self.assertEqual(syntax.returncode, 0, syntax.stderr)
        result, calls, tables = run_live()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(calls), 2)
        self.assertIn("get-caller-identity", calls[0])
        self.assertIn("describe-table", calls[1])
        self.assertEqual(calls[1][calls[1].index("--table-name") + 1], "OfflineTable")
        for args in calls:
            self.assertEqual(args[args.index("--profile") + 1], "offline-only")
            self.assertEqual(args[args.index("--region") + 1], "us-east-1")
            self.assertIn("--no-cli-auto-prompt", args)
        self.assertEqual(tables, [{"Table": {"TableName": "OfflineTable", "TableStatus": "ACTIVE"}}])

    def test_missing_approved_target_stops_before_any_aws_call(self):
        for name in ["AWS_PROFILE", "AWS_REGION", "TABLE_NAME", "EXPECTED_ACCOUNT_ID", "EXPECTED_PRINCIPAL_ARN"]:
            with self.subTest(missing=name):
                result, calls, tables = run_live(missing=name)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(calls, [])
                self.assertEqual(tables, [])

    def test_account_or_principal_drift_stops_before_table_discovery(self):
        for updates in [{"FAKE_ACCOUNT": "999900001111"}, {"FAKE_PRINCIPAL": "different-role"}]:
            with self.subTest(updates=updates):
                result, calls, tables = run_live(**updates)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("Identity mismatch", result.stderr)
                self.assertEqual(len(calls), 1)
                self.assertIn("get-caller-identity", calls[0])
                self.assertEqual(tables, [])


if __name__ == "__main__":
    unittest.main(verbosity=2)
