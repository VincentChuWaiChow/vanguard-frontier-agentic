#!/usr/bin/env python3
"""Offline command routing checks; never audit AWS or run a load test."""
import ast
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import unittest
from unittest.mock import patch

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/select_mode.py"
spec = importlib.util.spec_from_file_location("modes", SCRIPT)
modes = importlib.util.module_from_spec(spec)
spec.loader.exec_module(modes)


def cli(*args):
    return subprocess.run([sys.executable, str(SCRIPT), *args], text=True,
                          capture_output=True, timeout=5)


class ModeTests(unittest.TestCase):
    def test_real_help_and_json_command_inventory(self):
        result = cli("--help")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("performance-test", result.stdout)
        result = cli("help")
        self.assertEqual(result.returncode, 0, result.stderr)
        data = json.loads(result.stdout)
        self.assertEqual(data["execution"], "none")
        self.assertEqual(data["commands"], ["select", "design", "refactor", "backup", "restore", "audit", "performance-test", "feature-plan"])

    def test_select_is_scoped_deduplicated_and_stateless(self):
        plan = modes.build_plan(["select", "--features", "encryption,iam,encryption"])
        self.assertEqual(plan["selected_features"], ["encryption", "iam"])
        self.assertEqual(plan["read"], ["references/commands/select.md", "references/encryption.md", "references/iam-and-access.md"])
        self.assertEqual(modes.build_plan(["select"])["selected_features"], ["core"])
        self.assertEqual(plan["execution"], "none")

    def test_audit_defaults_to_repo_and_never_claims_findings(self):
        plan = modes.build_plan(["audit", "--features", "iam,tags"])
        self.assertEqual(plan["evidence"], "repo")
        self.assertEqual(plan["read"], ["references/commands/audit.md", "references/commands/audit/offline.md", "references/iam-and-access.md"])
        self.assertNotIn("findings", plan)
        live = modes.build_plan(["audit", "--evidence", "live-metadata"])
        self.assertEqual(live["execution"], "none")
        self.assertTrue(live["live_read_approval_required"])

    def test_performance_produces_bounded_plan_not_measurements(self):
        plan = modes.build_plan(["performance-test", "--features", "vectors",
                                 "--environment", "nonproduction", "--duration-seconds", "45",
                                 "--max-requests", "500", "--max-concurrency", "3"])
        self.assertEqual(plan["limits"], {"duration_seconds": 45, "max_requests": 500,
                                         "max_concurrency": 3})
        self.assertEqual(plan["read"], ["references/commands/performance-test.md", "references/data-types-and-vectors.md"])
        self.assertTrue(plan["run_approval_required"])
        self.assertEqual(plan["execution"], "none")
        self.assertNotIn("latency", plan)

    def test_feature_plan_is_not_activation_and_rejects_incompatible_actions(self):
        plan = modes.build_plan(["feature-plan", "--features", "pitr,streams", "--action", "enable"])
        self.assertEqual(plan["action"], "enable")
        self.assertEqual(plan["execution"], "none")
        self.assertTrue(plan["change_approval_required"])
        self.assertIn("references/commands/feature-plan.md", plan["read"])
        self.assertEqual(modes.build_plan(["feature-plan", "--features", "encryption",
                                          "--action", "configure"])["action"], "configure")
        for args in [["feature-plan", "--features", "encryption", "--action", "disable"],
                     ["feature-plan", "--features", "core", "--action", "enable"],
                     ["feature-plan", "--features", "pitr,encryption", "--action", "enable"]]:
            self.assertEqual(cli(*args).returncode, 2)

    def test_invalid_modes_features_and_unbounded_inputs_fail_closed(self):
        for args in [["execute"], ["select", "--features", "unknown"],
                     ["select", "--features", "pitr,,streams"],
                     ["performance-test", "--environment", "production"],
                     ["performance-test", "--max-requests", "0"],
                     ["performance-test", "--duration-seconds", "-1"],
                     ["performance-test", "--max-concurrency", "infinite"],
                     ["audit", "--execute"], ["feature-plan", "--action", "enable"]]:
            with self.subTest(args=args):
                result = cli(*args)
                self.assertEqual(result.returncode, 2)
                self.assertFalse(result.stdout)

    def test_router_does_not_open_references_write_or_run_external_work(self):
        with patch("builtins.open", side_effect=AssertionError("No file reads/writes")), \
             patch.object(Path, "read_text", side_effect=AssertionError("No eager loading")), \
             patch("subprocess.run", side_effect=AssertionError("No subprocess")):
            plan = modes.build_plan(["audit", "--features", "vectors,encryption"])
        self.assertEqual(plan["execution"], "none")
        imports = {node.names[0].name for node in ast.walk(ast.parse(SCRIPT.read_text()))
                   if isinstance(node, ast.Import)}
        self.assertEqual(imports, {"argparse", "json", "sys"})

    def test_every_feature_routes_to_existing_lazy_resources(self):
        for feature in modes.FEATURES:
            plan = modes.build_plan(["select", "--features", feature])
            for relative in plan["read"]:
                self.assertTrue(relative.startswith("references/"))
                self.assertTrue((ROOT / relative).is_file(), relative)
        entry = (ROOT / "SKILL.md").read_text()
        self.assertIn("references/commands/audit.md", entry)
        self.assertIn("Do not recursively load", entry)
        self.assertNotIn("SuccessfulRequestLatency", entry)

    def test_slash_invocation_matches_real_cli_and_bare_prefix_shows_help(self):
        args = ["audit", "--mode", "live", "--features", "iam"]
        self.assertEqual(modes.build_plan(["/aws-dynamodb-patterns", *args]), modes.build_plan(args))
        result = cli("/aws-dynamodb-patterns", *args)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout)["audit_mode"], "live")
        self.assertEqual(modes.build_plan(["/aws-dynamodb-patterns"])["read"], ["references/commands/help.md"])

    def test_offline_and_live_routes_are_mutually_exclusive_and_legacy_flags_work(self):
        for mode, legacy in [("offline", "repo"), ("live", "live-metadata")]:
            with self.subTest(mode=mode):
                plan = modes.build_plan(["audit", "--mode", mode, "--features", "encryption"])
                self.assertEqual(plan, modes.build_plan(["audit", "--evidence", legacy, "--features", "encryption"]))
                self.assertIn(f"references/commands/audit/{mode}.md", plan["read"])
                other = "live" if mode == "offline" else "offline"
                self.assertNotIn(f"references/commands/audit/{other}.md", plan["read"])
                self.assertEqual(plan["live_read_approval_required"], mode == "live")
        self.assertEqual(modes.build_plan(["audit"])["audit_mode"], "offline")

    def test_conflicting_audit_flags_and_wrong_prefix_fail_closed(self):
        for args in [["audit", "--mode", "live", "--evidence", "repo"],
                     ["audit", "--mode", "automatic"], ["/other-skill", "audit"]]:
            result = cli(*args)
            self.assertEqual(result.returncode, 2)
            self.assertFalse(result.stdout)

    def test_commands_are_separate_files_not_a_flat_or_combined_workflow(self):
        folder = ROOT / "references/commands"
        for name in ["README", "help", "select", "audit", "performance-test", "feature-plan", "aws-cli", "embedding"]:
            self.assertTrue((folder / f"{name}.md").is_file(), name)
        for name in ["modes", "audit", "performance-testing", "feature-plans", "commands", "embedding-commands"]:
            self.assertFalse((ROOT / "references" / f"{name}.md").exists(), name)
        self.assertLess(len((folder / "README.md").read_text().splitlines()), 50)

    def test_design_and_refactor_have_separate_lazy_routes_and_core_defaults(self):
        for command in ["design", "refactor"]:
            with self.subTest(command=command):
                expected = [f"references/commands/{command}.md", "references/modeling.md", "references/runtime-contract.md"]
                plan = modes.build_plan([command])
                self.assertEqual(plan["read"], expected)
                self.assertEqual(plan["selected_features"], ["core"])
                self.assertEqual(plan["execution"], "none")
                result = cli("/aws-dynamodb-patterns", command, "--features", "indexes,encryption")
                self.assertEqual(result.returncode, 0, result.stderr)
                routed = json.loads(result.stdout)
                self.assertEqual(routed["read"], [f"references/commands/{command}.md",
                                                  "references/lsi-vs-gsi.md", "references/encryption.md"])
                self.assertNotIn("references/workflow.md", routed["read"])
                for flag in ["--execute", "--apply", "--mode"]:
                    self.assertEqual(cli(command, flag).returncode, 2)

    def test_design_and_refactor_are_documented_and_do_not_alias_activation(self):
        entry = (ROOT / "SKILL.md").read_text()
        readme = (ROOT / "README.md").read_text()
        help_doc = (ROOT / "references/commands/help.md").read_text()
        for command in ["design", "refactor"]:
            self.assertIn(f"references/commands/{command}.md", entry)
            self.assertIn(f"/aws-dynamodb-patterns {command} --features", readme)
            self.assertIn(command, help_doc)
            self.assertTrue((ROOT / f"references/commands/{command}.md").is_file())

    def test_backup_restore_default_to_pitr_and_support_explicit_on_demand(self):
        for command in ["backup", "restore"]:
            with self.subTest(command=command):
                plan = modes.build_plan([command])
                self.assertEqual(plan["method"], "pitr")
                self.assertEqual(plan["selected_features"], ["pitr"])
                self.assertEqual(plan["read"], [f"references/commands/{command}.md",
                                                "references/commands/recovery-target.md",
                                                f"references/commands/{command}/pitr.md"])
                self.assertTrue(plan["change_approval_required"])
                self.assertEqual(plan["execution"], "none")
                result = cli("/aws-dynamodb-patterns", command, "--method", "on-demand")
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(json.loads(result.stdout)["method"], "on-demand")
                self.assertNotIn(f"references/commands/{command}/pitr.md", json.loads(result.stdout)["read"])
                self.assertNotIn("references/recovery-and-replicas.md", json.loads(result.stdout)["read"])
                self.assertNotIn("references/recovery-and-replicas.md", plan["read"])
                for flags in [["--execute"], ["--method", "unknown"]]:
                    self.assertEqual(cli(command, *flags).returncode, 2)
        self.assertEqual(modes.build_plan(["restore"])["target_policy"], "new-table-only")
        self.assertIn("references/recovery-and-replicas.md", modes.build_plan(["select", "--features", "pitr"])["read"])


if __name__ == "__main__":
    unittest.main(verbosity=2)
