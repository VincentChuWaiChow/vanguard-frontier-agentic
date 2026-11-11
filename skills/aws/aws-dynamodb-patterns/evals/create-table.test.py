#!/usr/bin/env python3
"""Exercise the shell example against a fake aws executable; never calls AWS."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / "assets/create-table.sh"
FAKE_AWS = '''#!/usr/bin/env python3
import json, os, sys
args = sys.argv[1:]
with open(os.environ["CALL_LOG"], "a") as log:
    log.write(json.dumps(args) + "\\n")
service = next(word for word in args if word in ("sts", "dynamodb"))
action = args[args.index(service) + 1]
if action == os.environ.get("FAIL_ACTION"):
    sys.exit(42)
if service == "sts":
    print("000000000000" if "--query" in args else '{"Account":"000000000000","Arn":"offline-role"}')
else:
    print("{}")
'''


class ShellExampleTests(unittest.TestCase):
    def run_script(self, *, account="000000000000", confirmation="CREATE OfflineTable",
                   fail="", missing=""):
        with tempfile.TemporaryDirectory(prefix="dynamodb-example-") as temp:
            root = Path(temp)
            fake = root / "aws"
            fake.write_text(FAKE_AWS)
            fake.chmod(0o700)
            env = {
                "PATH": f"{temp}:{os.environ['PATH']}",
                "AWS_PROFILE": "offline-only", "AWS_REGION": "us-east-1",
                "TABLE_NAME": "OfflineTable", "EXPECTED_ACCOUNT_ID": account,
                "CALL_LOG": str(root / "calls.jsonl"), "FAIL_ACTION": fail,
            }
            if missing:
                env.pop(missing)
            result = subprocess.run(["bash", str(SCRIPT)], input=confirmation + "\n",
                                    text=True, capture_output=True, env=env, timeout=10)
            log = root / "calls.jsonl"
            calls = [json.loads(line) for line in log.read_text().splitlines()] if log.exists() else []
            for call in calls:
                self.assertIn("--profile", call)
                self.assertIn("--region", call)
            actions = []
            for call in calls:
                service = next(word for word in call if word in ("sts", "dynamodb"))
                actions.append(call[call.index(service) + 1])
            return result, calls, actions

    def test_missing_explicit_target_stops_before_any_call(self):
        for variable in ("AWS_PROFILE", "AWS_REGION", "TABLE_NAME", "EXPECTED_ACCOUNT_ID"):
            with self.subTest(variable=variable):
                result, _, actions = self.run_script(missing=variable)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(actions, [])

    def test_wrong_account_stops_before_create(self):
        result, _, actions = self.run_script(account="111111111111")
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(actions, ["get-caller-identity", "get-caller-identity"])
        self.assertIn("Account mismatch", result.stderr)

    def test_refused_confirmation_stops_before_create(self):
        result, _, actions = self.run_script(confirmation="no")
        self.assertNotEqual(result.returncode, 0)
        self.assertNotIn("create-table", actions)

    def test_success_orders_create_wait_config_and_readback(self):
        result, calls, actions = self.run_script()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(actions, [
            "get-caller-identity", "get-caller-identity", "create-table", "wait",
            "update-time-to-live", "update-continuous-backups", "describe-table",
            "describe-time-to-live", "describe-continuous-backups"])
        self.assertIn("table-exists", calls[3])
        self.assertTrue(any(arg.endswith("create-table.json") for arg in calls[2]))

    def test_existing_or_failed_create_is_not_adopted(self):
        result, _, actions = self.run_script(fail="create-table")
        self.assertEqual(result.returncode, 42)
        self.assertEqual(actions[-1], "create-table")
        self.assertNotIn("update-time-to-live", actions)

    def test_partial_configuration_failure_does_not_delete_or_continue(self):
        result, _, actions = self.run_script(fail="update-time-to-live")
        self.assertEqual(result.returncode, 42)
        self.assertEqual(actions[-1], "update-time-to-live")
        self.assertNotIn("delete-table", actions)
        self.assertNotIn("update-continuous-backups", actions)


if __name__ == "__main__":
    unittest.main(verbosity=2)
