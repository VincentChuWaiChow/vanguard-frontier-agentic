#!/usr/bin/env python3
"""Gate: run the Atlassian case-capsule fixtures through the skill's own validator.

The schema and validator live inside the skill
(skills/cross-functional/atlassian-case-capsule/{assets,scripts}) so an exported skill is
self-contained; this gate only supplies the repository's fixtures. See the validator's module
docstring for the rule set and the fixture format.
"""
from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
VALIDATOR = REPO / "skills" / "cross-functional" / "atlassian-case-capsule" / "scripts" / "validate_capsule.py"
FIXTURES = REPO / "tests" / "fixtures" / "atlassian-case-capsule"

spec = importlib.util.spec_from_file_location("atlassian_validate_capsule", VALIDATOR)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

if __name__ == "__main__":
    sys.exit(module.main(sys.argv[1:]) if len(sys.argv) > 1 else module.run_suite(FIXTURES))
