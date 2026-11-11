#!/usr/bin/env python3
"""Human-trigger examples and compact lazy entrypoint; no cloud or file writes."""
import importlib.util
from pathlib import Path
import re
import shlex
import sys
import unittest

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("modes", ROOT / "scripts/select_mode.py")
modes = importlib.util.module_from_spec(spec)
spec.loader.exec_module(modes)


class ReadmeLoadingTests(unittest.TestCase):
    def test_human_tables_cover_commands_and_parse_every_literal_trigger(self):
        readme = (ROOT / "README.md").read_text()
        self.assertIn("## Command triggers", readme)
        self.assertIn("## Feature combinations", readme)
        self.assertIn("## Workflow combinations", readme)
        examples = re.findall(r"`(/aws-dynamodb-patterns[^`\n]+)`", readme)
        self.assertGreaterEqual(len(examples), 12)
        commands = set()
        for example in examples:
            plan = modes.build_plan(shlex.split(example))
            commands.add(plan.get("mode", "help"))
            self.assertEqual(plan["execution"], "none")
        self.assertEqual(commands, {"help", *modes.MODE_GUIDES})

    def test_feature_combinations_use_supported_ids_and_do_not_imply_activation(self):
        readme = (ROOT / "README.md").read_text()
        section = readme.split("## Feature combinations\n", 1)[1].split("\n## ", 1)[0]
        bundles = re.findall(r"`([a-z-]+(?:,[a-z-]+)+)`", section)
        self.assertGreaterEqual(len(bundles), 4)
        for bundle in bundles:
            self.assertTrue(set(bundle.split(",")).issubset(modes.FEATURES))
        self.assertIn("one command at a time", readme)
        self.assertIn("not an automatic pipeline", readme)

    def test_entrypoint_is_compact_and_deep_details_are_only_referenced(self):
        entry = (ROOT / "SKILL.md").read_text()
        self.assertLessEqual(len(entry.splitlines()), 90)
        self.assertLessEqual(len(entry.split()), 900)
        self.assertIn("references/workflow.md", entry)
        self.assertIn("Do not recursively load", entry)
        self.assertNotIn("## 1. Establish the workload", entry)
        self.assertNotIn("LastEvaluatedKey", entry)
        self.assertIn("June", entry)
        self.assertIn("December", entry)
        self.assertIn("scripts/review_due.py", entry)
        self.assertNotIn("references/workflow.md", modes.build_plan(["help"])["read"])

    def test_lazy_workflow_preserves_full_design_and_correctness_sequence(self):
        workflow = (ROOT / "references/workflow.md").read_text()
        self.assertTrue(workflow.endswith("\n"))
        self.assertFalse(workflow.endswith("\n\n"), "No extra blank line at EOF")
        headings = ["## Scope and safety", "## 1. Establish the workload", "## 2. Choose",
                    "## 3. Implement", "## 4. Prove", "## Response contract"]
        positions = [workflow.index(heading) for heading in headings]
        self.assertEqual(positions, sorted(positions))
        for invariant in ["LastEvaluatedKey", "timeout-after-commit", "wrong-tenant cursor",
                          "Do not reset data", "remaining match count", "Use Context7"]:
            self.assertIn(invariant, workflow)


if __name__ == "__main__":
    unittest.main(verbosity=2)
