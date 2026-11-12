#!/usr/bin/env python3
"""Static packaging regressions, not a behavioral evaluation of agent compliance."""

from __future__ import annotations

import json
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RULE_PATH = "rules/codex/rust-safety-rule.md"
RULE_ID = "rust-safety-rule"
SECTION_IDS = [f"RUST-{number}" for number in range(1, 9)]


def check_contract(body: str, metadata: dict, entry: dict, guidance: str) -> None:
    """Check the declared static contract only; prose correctness needs review."""
    assert metadata == entry, "metadata must exactly mirror the catalog"
    assert metadata["id"] == RULE_ID and metadata["path"] == RULE_PATH
    assert metadata["type"] == "rule" and metadata["provider"] == "generic"
    assert metadata["harnesses"] == ["codex"]
    assert metadata["author"] == "github: Raishin"
    assert f"  author: {json.dumps(metadata['author'])}\n" in body
    assert "When Codex" in body and "Rust" in body
    assert re.findall(r"^## (RUST-\d+):", body, re.MULTILINE) == SECTION_IDS
    assert len(body.split()) <= 700, "keep the on-demand rule compact"
    assert not re.search(r"/(?:home|Users|tmp)/|[A-Za-z]:\\", body), "no fixed host paths"
    assert "rust-safety-rule.evals.md" in body
    conditional = rf"- Rust work in Codex (?:→|->) `{re.escape(RULE_PATH)}`"
    assert any(re.fullmatch(conditional, line) for line in guidance.splitlines())
    assert metadata["official_docs"]
    for source in metadata["official_docs"]:
        assert source.startswith(("https://doc.rust-lang.org/", "https://github.com/rust-lang/"))
        assert source in body, "declared primary sources must be visible in the rule"


class RustSafetyRuleTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.body = (ROOT / RULE_PATH).read_text(encoding="utf-8")
        cls.metadata = json.loads((ROOT / RULE_PATH).with_suffix(".metadata.json").read_text())
        entries = [item for item in json.loads((ROOT / "catalog/rules.json").read_text())
                   if item["id"] == RULE_ID]
        assert len(entries) == 1, "rule must be registered exactly once"
        cls.entry = entries[0]
        cls.guidance = (ROOT / "AGENTS.md").read_text(encoding="utf-8")

    def check(self, *, body=None, metadata=None, guidance=None) -> None:
        check_contract(self.body if body is None else body,
                       self.metadata if metadata is None else metadata,
                       self.entry, self.guidance if guidance is None else guidance)

    def test_live_contract(self) -> None:
        self.check()

    def test_rejects_missing_or_duplicate_sections(self) -> None:
        for section in SECTION_IDS:
            with self.subTest(section=section), self.assertRaises(AssertionError):
                self.check(body=self.body.replace(f"## {section}:", "## Removed:"))
        with self.assertRaises(AssertionError):
            self.check(body=self.body.replace("## RUST-2:", "## RUST-1:"))

    def test_rejects_metadata_drift(self) -> None:
        for key, value in [("provider", "aws"), ("harnesses", ["other"]),
                           ("author", "unverified"), ("summary", "drift")]:
            with self.subTest(key=key), self.assertRaises(AssertionError):
                self.check(metadata={**self.metadata, key: value})

    def test_rejects_unquoted_author_frontmatter(self) -> None:
        body = self.body.replace('  author: "github: Raishin"', '  author: github: Raishin')
        self.assertNotEqual(body, self.body)
        with self.assertRaises(AssertionError):
            self.check(body=body)

    def test_rejects_fixed_host_paths(self) -> None:
        for value in ["/home/example/project", "/Users/example/project", "/tmp/project",
                      r"C:\example\project"]:
            with self.subTest(path=value), self.assertRaises(AssertionError):
                self.check(body=self.body + "\nUse " + value)

    def test_rejects_missing_conditional_load_hint(self) -> None:
        guidance = "\n".join(line for line in self.guidance.splitlines() if RULE_PATH not in line)
        with self.assertRaises(AssertionError):
            self.check(guidance=guidance)

    def test_rejects_unconditional_load_hint(self) -> None:
        guidance = self.guidance.replace(
            f"- Rust work in Codex → `{RULE_PATH}`",
            f"- Always load Rust `{RULE_PATH}`",
        )
        self.assertNotEqual(guidance, self.guidance)
        with self.assertRaises(AssertionError):
            self.check(guidance=guidance)

    def test_rejects_unscoped_or_oversized_rule(self) -> None:
        for body in [self.body.replace("When Codex", "Always"), self.body + " word" * 701]:
            with self.subTest(), self.assertRaises(AssertionError):
                self.check(body=body)

    def test_rejects_missing_source_or_lazy_eval_link(self) -> None:
        for value in [*self.metadata["official_docs"], "rust-safety-rule.evals.md"]:
            with self.subTest(value=value), self.assertRaises(AssertionError):
                self.check(body=self.body.replace(value, "removed"))

    def test_adversarial_scenarios_are_present(self) -> None:
        cases = (ROOT / RULE_PATH).with_suffix(".evals.md").read_text(encoding="utf-8")
        self.assertEqual(re.findall(r"^\| (S\d+) \|", cases, re.MULTILINE),
                         [f"S{number}" for number in range(1, 9)])


if __name__ == "__main__":
    unittest.main(verbosity=2)
