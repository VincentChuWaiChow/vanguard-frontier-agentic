#!/usr/bin/env python3
"""Gate: every NoSQL behavioral scenario is answerable from the file the skill loads.

Reads tests/fixtures/nosql-scenarios/scenarios.json. For each scenario it checks,
deterministically (no model, no network, no clock):

* the target skill exists and its SKILL.md links the scenario's reference file
  (``SKILL.md`` itself is allowed for entry-point rules), so progressive
  disclosure actually reaches the guidance;
* every required evidence ID is a row in that skill's evidence ledger AND is
  cited in the reference file, so the answer is traceable;
* every ``must_mention`` phrase occurs in the reference file (case-insensitive),
  so the decisive fact or refusal rule has not been edited away.

This does not prove a model answers correctly. ``pass_criteria`` is the rubric
for live model runs, which stay outside CI because they are not reproducible.

Usage: python3 tests/validate-nosql-scenarios.py
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
SCENARIOS = REPO / "tests/fixtures/nosql-scenarios/scenarios.json"
REQUIRED_KEYS = {"id", "skill", "reference", "prompt", "required_evidence", "must_mention", "pass_criteria"}


def ledger_ids(skill_dir: Path) -> set[str]:
    ledger = skill_dir / "references/evidence-ledger.md"
    if not ledger.exists():
        return set()
    return set(re.findall(r"^\|\s*([A-Z][A-Z0-9-]*-E\d{2,})\s*\|", ledger.read_text(encoding="utf-8"), re.M))


def check(scenario: dict, errors: list[str], seen: set[str]) -> None:
    sid = scenario.get("id", "<missing id>")
    missing = REQUIRED_KEYS - scenario.keys()
    if missing:
        errors.append(f"{sid}: missing keys {sorted(missing)}")
        return
    if sid in seen:
        errors.append(f"{sid}: duplicate scenario id")
    seen.add(sid)
    if len(scenario["pass_criteria"]) < 20 or len(scenario["prompt"]) < 20:
        errors.append(f"{sid}: prompt and pass_criteria must be concrete (>= 20 chars)")

    skill_dir = REPO / scenario["skill"]
    skill_md = skill_dir / "SKILL.md"
    if not skill_md.exists():
        errors.append(f"{sid}: skill {scenario['skill']} has no SKILL.md")
        return
    reference = skill_dir / scenario["reference"]
    if not reference.exists():
        errors.append(f"{sid}: reference {scenario['reference']} does not exist")
        return
    if scenario["reference"] != "SKILL.md" and f"]({scenario['reference']})" not in skill_md.read_text(encoding="utf-8"):
        errors.append(f"{sid}: SKILL.md does not link {scenario['reference']}; the skill would never load it")

    text = reference.read_text(encoding="utf-8")
    ids = ledger_ids(skill_dir)
    for evidence in scenario["required_evidence"]:
        if evidence not in ids:
            errors.append(f"{sid}: {evidence} is not a row in the skill's evidence ledger")
        elif f"[{evidence}]" not in text:
            errors.append(f"{sid}: {evidence} is not cited in {scenario['reference']}")
    for phrase in scenario["must_mention"]:
        if phrase.lower() not in text.lower():
            errors.append(f"{sid}: '{phrase}' not found in {scenario['reference']}")


def main() -> int:
    data = json.loads(SCENARIOS.read_text(encoding="utf-8"))
    scenarios = data.get("scenarios", [])
    errors: list[str] = []
    seen: set[str] = set()
    if not scenarios:
        errors.append("no scenarios defined")
    for scenario in scenarios:
        check(scenario, errors, seen)
    if errors:
        print("validate-nosql-scenarios: FAIL", file=sys.stderr)
        for error in errors:
            print(f"  - {error}", file=sys.stderr)
        return 1
    print(f"validate-nosql-scenarios: OK ({len(scenarios)} scenarios grounded in their loaded references)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
