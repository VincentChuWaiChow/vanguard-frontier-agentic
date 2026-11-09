"""Offline contract/fake-command tests. Never invokes real Azure CLI or SDK."""
import ast
import copy
from datetime import date
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]


def load(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / (name + ".py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


SELECT = load("select_mode")
DUE = load("review_due")
GUARD = load("recovery_guard")
ASSETS = load("validate_assets")


def selector(*args):
    return subprocess.run([sys.executable, "-B", str(ROOT / "scripts/select_mode.py"), *args],
                          capture_output=True, text=True, timeout=5)


class Package(unittest.TestCase):
    def test_metadata_size_and_exact_name(self):
        skill = (ROOT / "SKILL.md").read_text()
        self.assertIn("name: azure-cosmosdb-pattern\n", skill)
        self.assertIn('version: "1.0.0"', skill)
        self.assertLessEqual(len(skill.splitlines()), 90)
        self.assertLessEqual(len(skill.split()), 900)
        self.assertGreater(skill.count("|---"), 2)

    def test_all_local_markdown_links_exist(self):
        for file in ROOT.rglob("*.md"):
            for link in re.findall(r"\[[^\]]*\]\(([^)]+)\)", file.read_text()):
                if re.match(r"https?://|#", link):
                    continue
                with self.subTest(file=file.name, link=link):
                    self.assertTrue((file.parent / link.split("#")[0]).is_file())

    def test_json_and_all_source_routes(self):
        for path in ROOT.rglob("*.json"):
            json.loads(path.read_text())
        evidence = json.loads((ROOT / "evals/source-evidence.json").read_text())
        self.assertEqual(evidence["checked_on"], "2026-11-08")
        self.assertIn("mcp__context7__query_docs", evidence["context7"]["query_tool"])
        self.assertTrue(any(x.get("command") == "microsoft_code_sample_search" for x in evidence["provider_tools"]))

    def test_selector_no_reference_read_write_or_cloud_capability(self):
        tree = ast.parse((ROOT / "scripts/select_mode.py").read_text())
        imports = {node.names[0].name for node in ast.walk(tree) if isinstance(node, ast.Import)}
        self.assertEqual(imports, {"argparse", "json", "sys"})
        with patch("builtins.open", side_effect=AssertionError("selector read file")):
            self.assertEqual(SELECT.build_plan(["design"])["execution"], "none")

    def test_provisioning_asset_invariants(self):
        t = json.loads((ROOT / "assets/account.arm.json").read_text())
        idx = json.loads((ROOT / "assets/indexing-policy.json").read_text())
        self.assertEqual(ASSETS.validate(t, idx)["status"], "PASS")
        self.assertTrue(all(r.get("dependsOn") for r in t["resources"][1:]))

    def test_negative_provisioning_fixtures(self):
        base = json.loads((ROOT / "assets/account.arm.json").read_text())
        idx = json.loads((ROOT / "assets/indexing-policy.json").read_text())
        changes = [
            lambda t: t["resources"][0].update(kind="MongoDB"),
            lambda t: t["resources"][0]["properties"].update(disableLocalAuth=False),
            lambda t: t["resources"][0]["properties"].update(publicNetworkAccess="Enabled"),
            lambda t: t["resources"][0]["properties"].update(keyVaultKeyUri="https://unreviewed"),
            lambda t: t["resources"][0]["properties"]["backupPolicy"].update(type="Periodic"),
            lambda t: t["resources"][1]["properties"].update(options={"throughput": 400}),
            lambda t: t["resources"][2]["properties"]["options"].update(autoscaleSettings={"maxThroughput": 1000}),
            lambda t: t["resources"][2]["properties"]["resource"]["partitionKey"].update(paths=["/wrong"]),
            lambda t: t["resources"][2]["properties"]["resource"].update(defaultTtl=3600),
        ]
        for change in changes:
            t = copy.deepcopy(base); change(t)
            with self.subTest(change=changes.index(change)), self.assertRaises(ValueError):
                ASSETS.validate(t, idx)

    def test_vector_policy_matches_index(self):
        policy = json.loads((ROOT / "assets/vector-policy.json").read_text())["vectorEmbeddings"][0]
        idx = json.loads((ROOT / "assets/vector-indexing-policy.json").read_text())
        self.assertEqual(policy, {"path": "/embedding", "dataType": "float32", "dimensions": 1536, "distanceFunction": "cosine"})
        self.assertEqual(idx["vectorIndexes"], [{"path": policy["path"], "type": "diskANN"}])
        self.assertIn({"path": "/embedding/*"}, idx["excludedPaths"])


class Selector(unittest.TestCase):
    def test_every_command_and_slash_namespace(self):
        for cmd in SELECT.COMMANDS:
            argv = [cmd] + (["--features", "iam", "--action", "configure"] if cmd == "feature-plan" else [])
            with self.subTest(command=cmd):
                normal, prefixed = selector(*argv), selector("/azure-cosmosdb-pattern", *argv)
                self.assertEqual(normal.returncode, 0, normal.stderr)
                self.assertEqual(json.loads(normal.stdout), json.loads(prefixed.stdout))
                for path in json.loads(normal.stdout)["read"]:
                    self.assertTrue((ROOT / path).is_file(), path)

    def test_help_stops_and_audit_defaults_offline(self):
        self.assertEqual(SELECT.build_plan([])["read"], ["references/commands/help.md"])
        offline = SELECT.build_plan(["audit"])
        self.assertEqual(offline["audit_mode"], "offline")
        self.assertFalse(offline["live_read_approval_required"])
        live = SELECT.build_plan(["audit", "--mode", "live", "--features", "monitoring"])
        self.assertTrue(live["live_read_approval_required"])
        self.assertFalse(any("offline.md" in p for p in live["read"]))

    def test_lazy_features_no_indirect_recovery_sibling_load(self):
        for feature in SELECT.FEATURES:
            paths = SELECT.build_plan(["select", "--features", feature])["read"]
            self.assertEqual(paths, ["references/commands/select.md"] + SELECT.FEATURES[feature])
            self.assertFalse(any("commands/restore/" in path or "commands/backup/" in path for path in paths))
        self.assertNotIn("references/encryption.md", SELECT.build_plan(["design"])["read"])

    def test_recovery_methods_are_isolated_and_native(self):
        for cmd, methods in [("backup", ["continuous", "periodic"]),
                             ("restore", ["continuous-new-account", "continuous-deleted-resource", "periodic-support"])]:
            for method in methods:
                plan = SELECT.build_plan([cmd, "--method", method])
                selected = [p for p in plan["read"] if f"commands/{cmd}/" in p]
                self.assertEqual(selected, [f"references/commands/{cmd}/{method}.md"])
                self.assertEqual(plan["execution"], "none")

    def test_safe_positive_performance_defaults_and_caps(self):
        p = SELECT.build_plan(["performance-test"])
        self.assertEqual(p["environment"], "local")
        self.assertTrue(all(v > 0 for v in p["limits"].values()))
        self.assertTrue(p["run_approval_required"])

    def test_unsupported_inputs_fail_closed(self):
        bad = [["combo"], ["select", "--features", ""], ["select", "--features", "core,unknown"],
               ["select", "--features", "core,"], ["select", "--features", "dax"],
               ["backup", "--method", "on-demand"], ["restore", "--method", "pitr"],
               ["backup", "--features", "core"], ["audit", "--mode", "execute"],
               ["design", "--mode", "live"], ["performance-test", "--environment", "production"],
               ["feature-plan", "--features", "encryption", "--action", "disable"],
               ["feature-plan", "--features", "backup", "--action", "disable"],
               ["feature-plan", "--features", "vectors", "--action", "disable"],
               ["select", "--feat", "iam"], ["/azure-cosmosdb-patterns", "help"]]
        for flag in ["--duration-seconds", "--max-requests", "--max-concurrency", "--max-ru"]:
            for value in ["0", "-1", "infinity", "1.5"]:
                bad.append(["performance-test", flag, value])
        for argv in bad:
            with self.subTest(argv=argv):
                result = selector(*argv)
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse(result.stdout.strip())


class Reminder(unittest.TestCase):
    def test_dates_never_advance_or_clear_on_partial_update(self):
        path = ROOT / "assets/content-review.json"
        before = path.read_bytes()
        ledger = json.loads(before)
        for day, status in [("2026-11-08", "not-due"), ("2026-12-01", "due"),
                            ("2026-12-02", "overdue"), ("2027-07-02", "overdue")]:
            with self.subTest(day=day):
                actual = DUE.due_status(ledger, date.fromisoformat(day))
                self.assertEqual(actual["status"], status)
                self.assertEqual(actual["due"], "2026-12-01")
                self.assertIsNone(actual["last_full_review"])
        self.assertEqual(before, path.read_bytes())

    def test_cli_requires_valid_explicit_date(self):
        for args in [[], ["--as-of", "2026-99-01"]]:
            p = subprocess.run([sys.executable, "-B", str(ROOT / "scripts/review_due.py"), *args],
                               capture_output=True, text=True, timeout=5)
            self.assertNotEqual(p.returncode, 0)


class Recovery(unittest.TestCase):
    def evidence(self):
        return json.loads((ROOT / "evals/fixtures/recovery-continuous.json").read_text())

    def test_new_account_plan_and_no_execution(self):
        plan = GUARD.plan(self.evidence())
        self.assertEqual(plan["argv_example"][:3], ["az", "cosmosdb", "restore"])
        self.assertEqual(plan["execution"], "none")
        self.assertIn("--disable-ttl", plan["argv_example"])
        self.assertIn("--subscription", plan["argv_example"])
        self.assertIn("--resource-group", plan["argv_example"])

    def test_recovery_negative_evidence(self):
        base = self.evidence()
        mutations = [
            lambda e: e["target"].update(lookup="Forbidden"),
            lambda e: e["target"].update(lookup="timeout"),
            lambda e: e["target"].update(lookup="exists"),
            lambda e: e["target"].update(account=e["source"]["account"]),
            lambda e: e["target"].update(subscription="wrong"),
            lambda e: e["source"].update(api="MongoDB"),
            lambda e: e["source"].update(backup_mode="Periodic"),
            lambda e: e["source"].update(resource_id="/subscriptions/wrong/accounts/other"),
            lambda e: e.update(observed_source_instance_id="different"),
            lambda e: e.update(source_name_unambiguous=False),
            lambda e: e.update(restore_timestamp="2026-10-01T00:00:00Z"),
            lambda e: e.update(restore_timestamp="2026-11-09T00:00:00Z"),
            lambda e: e.update(restore_timestamp="2026-11-08T08:30:00"),
            lambda e: e.update(disable_ttl=False),
            lambda e: e.update(key_dependencies_verified=False),
            lambda e: e.update(method="on-demand"),
            lambda e: e.update(restore_scope="container"),
            lambda e: e.update(approved_restore_scope="unknown"),
            lambda e: e.update(database="ordersdb", container="orders"),
            lambda e: e.update(databases_to_restore=["ordersdb"]),
        ]
        for i, mutate in enumerate(mutations):
            e = copy.deepcopy(base); mutate(e)
            with self.subTest(case=i), self.assertRaises(ValueError):
                GUARD.plan(e)

    def test_deleted_resource_requires_delete_and_absent_scope(self):
        e = self.evidence(); e["method"] = "continuous-deleted-resource"
        e["target"]["account"] = e["source"]["account"]
        e.update(deletion_event="Delete", deleted_at="2026-11-08T08:40:00Z",
                 resource_kind="container", database="ordersdb", container="orders")
        p = GUARD.plan(e)
        self.assertEqual(p["argv_example"][:5], ["az", "cosmosdb", "sql", "container", "restore"])
        for change in [{"deletion_event": "Update"}, {"deleted_at": "2026-11-08T08:20:00Z"}, {"resource_kind": "item"}]:
            bad = copy.deepcopy(e); bad.update(change)
            with self.assertRaises(ValueError): GUARD.plan(bad)

    def test_periodic_is_support_only_no_fictional_cli(self):
        e = self.evidence(); e["method"] = "periodic-support"; e["source"]["backup_mode"] = "Periodic"
        p = GUARD.plan(e)
        self.assertIsNone(p["argv_example"])
        self.assertTrue(p["periodic_support_required"])

    def test_same_account_new_lifetime_recovery_contract(self):
        restore = (ROOT / "references/commands/restore/continuous-deleted-resource.md").read_text()
        feed = (ROOT / "references/change-feed-and-integrations.md").read_text()
        for text in (restore, feed):
            self.assertIn("pre-deletion", text)
            self.assertIn("continuation tokens", text)
            self.assertIn("lifetime", text)
            self.assertIn("restore-in-account-continuous-backup-introduction", text)


FAKE_AZ = '''#!/usr/bin/env python3
import json, os, sys
a = sys.argv[1:]
with open(os.environ["FAKE_LOG"], "a") as f: f.write(json.dumps(a) + "\\n")
if not a or a[0] != "cosmosdb":
    print("command group must precede --subscription", file=sys.stderr); sys.exit(11)
for flag in ("--subscription", "--resource-group"):
    if flag not in a: sys.exit(12)
if any(word in a for word in ("delete", "list-keys", "update", "account")): sys.exit(13)
verbs = a[:next((i for i, x in enumerate(a) if x.startswith("--")), len(a))]
if verbs == ["cosmosdb", "show"]:
    mode = os.environ.get("FAKE_LOOKUP", "absent")
    if mode == "exists": print('{"id":"already-present"}'); sys.exit(0)
    print("ERROR: (ResourceNotFound) missing" if mode == "absent" else "ERROR: (Forbidden) denied", file=sys.stderr)
    sys.exit(3)
if os.environ.get("FAKE_FAIL") == " ".join(verbs): sys.exit(4)
if verbs not in (["cosmosdb", "create"], ["cosmosdb", "sql", "database", "create"],
                 ["cosmosdb", "sql", "container", "create"], ["cosmosdb", "restore"],
                 ["cosmosdb", "sql", "container", "restore"], ["cosmosdb", "sql", "database", "restore"]): sys.exit(14)
print('{"provisioningState":"Succeeded"}')
'''


class FakeCLI(unittest.TestCase):
    def run_provision(self, lookup="absent", fail="", approval=True):
        with tempfile.TemporaryDirectory() as d:
            directory = Path(d); fake = directory / "az"; fake.write_text(FAKE_AZ); fake.chmod(0o700)
            log = directory / "log"
            env = os.environ.copy(); env.update(PATH=d + os.pathsep + env["PATH"], FAKE_LOG=str(log),
                AZ_SUBSCRIPTION="sub-test", RESOURCE_GROUP="rg-test", ACCOUNT_NAME="cosmos-test",
                LOCATION="eastus", FAKE_LOOKUP=lookup, FAKE_FAIL=fail,
                APPROVED_TARGET_ID="/subscriptions/sub-test/resourceGroups/rg-test/providers/Microsoft.DocumentDB/databaseAccounts/" + ("cosmos-test" if approval else "wrong"))
            result = subprocess.run(["bash", str(ROOT / "assets/provision.sh")], env=env,
                                    capture_output=True, text=True, timeout=5, cwd=d)
            calls = [json.loads(line) for line in log.read_text().splitlines()] if log.exists() else []
            return result, calls

    def test_actual_script_explicit_scope_and_arm_cli_equivalence(self):
        result, calls = self.run_provision()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(calls), 4)
        for call in calls:
            self.assertEqual(call[0], "cosmosdb")
            self.assertEqual(call[call.index("--subscription") + 1], "sub-test")
            self.assertEqual(call[call.index("--resource-group") + 1], "rg-test")
        account, container = calls[1], calls[3]
        template = json.loads((ROOT / "assets/account.arm.json").read_text())
        props = template["resources"][0]["properties"]
        for flag, wanted in [("--kind", template["resources"][0]["kind"]),
                             ("--default-consistency-level", props["consistencyPolicy"]["defaultConsistencyLevel"]),
                             ("--backup-policy-type", props["backupPolicy"]["type"]),
                             ("--continuous-tier", props["backupPolicy"]["continuousModeProperties"]["tier"]),
                             ("--disable-local-auth", "true"), ("--public-network-access", "DISABLED"),
                             ("--minimal-tls-version", props["minimalTlsVersion"])]:
            self.assertEqual(account[account.index(flag) + 1], wanted)
        resource = template["resources"][2]["properties"]
        self.assertEqual(container[container.index("--partition-key-path") + 1], resource["resource"]["partitionKey"]["paths"][0])
        self.assertEqual(int(container[container.index("--throughput") + 1]), resource["options"]["throughput"])
        self.assertEqual(int(container[container.index("--ttl") + 1]), resource["resource"]["defaultTtl"])
        idx = Path(container[container.index("--idx") + 1][1:])
        self.assertEqual(json.loads(idx.read_text()), resource["resource"]["indexingPolicy"])
        self.assertNotIn("--key-uri", account)
        self.assertNotIn("--max-throughput", container)

    def test_create_never_adopts_or_treats_forbidden_as_absence(self):
        for lookup in ["exists", "forbidden", "timeout"]:
            result, calls = self.run_provision(lookup=lookup)
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(len(calls), 1)

    def test_approval_full_target_mismatch_no_cli(self):
        result, calls = self.run_provision(approval=False)
        self.assertNotEqual(result.returncode, 0); self.assertEqual(calls, [])

    def test_errors_abort_no_delete_fallback(self):
        result, calls = self.run_provision(fail="cosmosdb create")
        self.assertNotEqual(result.returncode, 0); self.assertEqual(len(calls), 2)
        self.assertFalse(any("delete" in call for call in calls))

    def test_actual_restore_snippet_with_fake_cli(self):
        doc = (ROOT / "references/commands/restore/continuous-new-account.md").read_text()
        snippet = re.search(r"```bash\n(.*?)```", doc, re.S).group(1)
        with tempfile.TemporaryDirectory() as d:
            directory = Path(d); fake = directory / "az"; fake.write_text(FAKE_AZ); fake.chmod(0o700)
            env = os.environ.copy(); env.update(PATH=d + os.pathsep + env["PATH"], FAKE_LOG=str(directory / "log"),
                AZ_SUBSCRIPTION="sub-test", TARGET_RESOURCE_GROUP="recovery-rg", SOURCE_ACCOUNT="source-cosmos",
                TARGET_ACCOUNT="recovery-cosmos", LOCATION="eastus", RESTORE_TIMESTAMP="2026-11-08T08:30:00Z")
            result = subprocess.run(["bash", "-euc", snippet], env=env, capture_output=True, text=True, timeout=5)
            self.assertEqual(result.returncode, 0, result.stderr)
            calls = [json.loads(line) for line in (directory / "log").read_text().splitlines()]
            self.assertEqual(calls[0][:2], ["cosmosdb", "restore"])
            self.assertEqual(calls[0][calls[0].index("--target-database-account-name") + 1], "recovery-cosmos")
            self.assertEqual(calls[0][calls[0].index("--disable-ttl") + 1], "true")


if __name__ == "__main__":
    unittest.main()
