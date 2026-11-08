#!/usr/bin/env python3
"""Run real recovery command snippets with fake AWS only; never touch a table."""
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
DOCS = ROOT / "references/commands"
TABLE_ARN = "arn:aws:dynamodb:us-east-1:111122223333:table/Orders"
BACKUP_ARN = TABLE_ARN + "/backup/01794110400000-example"
PRINCIPAL = "arn:aws:sts::111122223333:assumed-role/RecoveryRole/example"
FAKE = r'''#!/usr/bin/env python3
import json, os, sys
a = sys.argv[1:]
with open(os.environ["CALL_LOG"], "a") as log: log.write(json.dumps(a) + "\n")
if "sts" in a:
    print(json.dumps({"Account": os.environ.get("FAKE_ACCOUNT", "111122223333"),
                      "Arn": os.environ.get("FAKE_PRINCIPAL", os.environ["EXPECTED_PRINCIPAL_ARN"])}))
else:
    op = a[a.index("dynamodb") + 1]
    if op == "describe-table":
        print(json.dumps({"Table": {"TableArn": os.environ["SOURCE_ARN"], "TableStatus": "ACTIVE"}}))
    elif op == "describe-continuous-backups":
        print(json.dumps({"ContinuousBackupsDescription": {"PointInTimeRecoveryDescription": {
            "PointInTimeRecoveryStatus": os.environ.get("PITR_STATUS", "ENABLED"), "RecoveryPeriodInDays": 35,
            "EarliestRestorableDateTime": "2026-11-01T00:00:00Z", "LatestRestorableDateTime": "2026-11-08T02:00:00Z"}}}))
    elif op == "create-backup":
        print(json.dumps({"BackupDetails": {"BackupArn": os.environ["BACKUP_ARN"]}}))
    elif op == "describe-backup":
        print(json.dumps({"BackupDescription": {"BackupDetails": {"BackupStatus": os.environ.get("BACKUP_STATUS", "AVAILABLE")},
            "SourceTableDetails": {"TableArn": os.environ.get("BACKUP_SOURCE", os.environ["SOURCE_ARN"])}}}))
    elif op in {"restore-table-to-point-in-time", "restore-table-from-backup"}:
        if os.environ.get("RESTORE_FAIL") == "1": raise SystemExit("Existing target or unknown restore failure")
        print("{}")
    elif op in {"update-continuous-backups", "wait"}: print("{}")
    else: raise SystemExit("Forbidden/unexpected recovery operation")
'''


def blocks(name):
    return re.findall(r"```bash\n(.*?)\n```", (DOCS / name).read_text(), re.S)


def run(command, method="pitr", missing=None, **updates):
    with tempfile.TemporaryDirectory(prefix="fake-recovery-") as directory:
        root = Path(directory)
        (root / "aws").write_text(FAKE)
        (root / "aws").chmod(0o700)
        env = {"PATH": f"{root}:{os.environ['PATH']}", "TMPDIR": directory,
               "AWS_PROFILE": "offline-only", "AWS_REGION": "us-east-1", "TABLE_NAME": "Orders",
               "EXPECTED_ACCOUNT_ID": "111122223333", "EXPECTED_PRINCIPAL_ARN": PRINCIPAL,
               "SOURCE_ARN": TABLE_ARN, "BACKUP_ARN": BACKUP_ARN, "BACKUP_NAME": "ApprovedBackup",
               "RECOVERY_DAYS": "35", "RESTORE_TABLE_NAME": "OrdersRecovered",
               "RESTORE_AT": "2026-11-08T01:00:00Z", "CALL_LOG": str(root / "calls.jsonl"), **updates}
        if missing:
            env.pop(missing)
        script = blocks("recovery-target.md")
        if command == "restore":
            before, after = blocks("restore.md")
            script += [before, *blocks(f"restore/{method}.md"), after]
        else:
            script += blocks(f"backup/{method}.md")
        result = subprocess.run(["bash", "-c", "\n".join(script)], cwd=ROOT, env=env,
                                text=True, capture_output=True, timeout=15)
        log = root / "calls.jsonl"
        calls = [json.loads(line) for line in log.read_text().splitlines()] if log.exists() else []
        return result, calls


def operations(calls):
    return [args[args.index("dynamodb") + 1] for args in calls if "dynamodb" in args]


class RecoveryTests(unittest.TestCase):
    def test_all_actual_method_snippets_parse_and_target_expected_resources(self):
        for name in ["recovery-target.md", "restore.md", "backup/pitr.md", "backup/on-demand.md",
                     "restore/pitr.md", "restore/on-demand.md"]:
            syntax = subprocess.run(["bash", "-n"], input="\n".join(blocks(name)), text=True, capture_output=True)
            self.assertEqual(syntax.returncode, 0, syntax.stderr)
        for command in ["backup", "restore"]:
            for method in ["pitr", "on-demand"]:
                with self.subTest(command=command, method=method):
                    result, calls = run(command, method)
                    self.assertEqual(result.returncode, 0, result.stderr)
                    for args in calls:
                        self.assertEqual(args[args.index("--profile") + 1], "offline-only")
                        self.assertEqual(args[args.index("--region") + 1], "us-east-1")
                        if "--target-table-name" in args:
                            self.assertEqual(args[args.index("--target-table-name") + 1], "OrdersRecovered")
                        if "--backup-arn" in args:
                            self.assertEqual(args[args.index("--backup-arn") + 1], BACKUP_ARN)
                        if "--restore-date-time" in args:
                            self.assertEqual(args[args.index("--restore-date-time") + 1], "2026-11-08T01:00:00Z")
                            self.assertNotIn("--use-latest-restorable-time", args)
                        if "--point-in-time-recovery-specification" in args:
                            spec = json.loads(args[args.index("--point-in-time-recovery-specification") + 1])
                            self.assertEqual(spec, {"PointInTimeRecoveryEnabled": True, "RecoveryPeriodInDays": 35})
                    self.assertFalse(any(op.startswith("delete-") for op in operations(calls)))
                    if command == "restore":
                        expected = "restore-table-to-point-in-time" if method == "pitr" else "restore-table-from-backup"
                        self.assertIn(expected, operations(calls))
                        self.assertEqual(operations(calls)[-2:], ["wait", "describe-table"])

    def test_pitr_requires_enabled_history_in_window_and_timezone(self):
        cases = [{"PITR_STATUS": "DISABLED"}, {"RESTORE_AT": "2026-10-01T00:00:00Z"},
                 {"RESTORE_AT": "2026-11-09T00:00:00Z"}, {"RESTORE_AT": "2026-11-08T01:00:00"}]
        for case in cases:
            result, calls = run("restore", **case)
            self.assertNotEqual(result.returncode, 0)
            self.assertNotIn("restore-table-to-point-in-time", operations(calls))
            self.assertNotIn("wait", operations(calls))

    def test_on_demand_requires_available_exact_source_before_restore(self):
        for case in [{"BACKUP_STATUS": "CREATING"}, {"BACKUP_SOURCE": TABLE_ARN + "Wrong"}]:
            result, calls = run("restore", "on-demand", **case)
            self.assertNotEqual(result.returncode, 0)
            self.assertNotIn("restore-table-from-backup", operations(calls))
            self.assertNotIn("wait", operations(calls))

    def test_same_source_target_or_failed_restore_never_adopts_existing_table(self):
        for case in [{"RESTORE_TABLE_NAME": "Orders"}, {"RESTORE_FAIL": "1"}]:
            result, calls = run("restore", **case)
            self.assertNotEqual(result.returncode, 0)
            self.assertNotIn("wait", operations(calls))
            self.assertFalse(any(op.startswith("delete-") for op in operations(calls)))

    def test_bad_retention_never_updates_pitr(self):
        for value in ["0", "36", "invalid"]:
            result, calls = run("backup", RECOVERY_DAYS=value)
            self.assertNotEqual(result.returncode, 0)
            self.assertNotIn("update-continuous-backups", operations(calls))

    def test_missing_target_or_identity_drift_stops_before_discovery(self):
        for name in ["AWS_PROFILE", "AWS_REGION", "TABLE_NAME", "EXPECTED_ACCOUNT_ID", "EXPECTED_PRINCIPAL_ARN"]:
            result, calls = run("backup", missing=name)
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(calls, [])
        for case in [{"FAKE_ACCOUNT": "999900001111"}, {"FAKE_PRINCIPAL": "WrongRole"}]:
            result, calls = run("backup", **case)
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(operations(calls), [])

    def test_async_backup_not_ready_never_recreates_it_to_poll(self):
        result, calls = run("backup", "on-demand", BACKUP_STATUS="CREATING")
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(operations(calls).count("create-backup"), 1)
        self.assertEqual(operations(calls).count("describe-backup"), 1)
        self.assertNotIn("restore-table-from-backup", operations(calls))


if __name__ == "__main__":
    unittest.main(verbosity=2)
