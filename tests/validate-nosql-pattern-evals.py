#!/usr/bin/env python3
"""Gate: run the offline evals shipped with the ported NoSQL pattern skills.

The pattern skills (ported from the canonical agentic-vantage copies) ship
executable helpers (route selectors, review reminders, recovery checks) and example
assets (shell, Python, Node). Their own evals check them with fake cloud binaries
and never contact a cloud. Shipping code without running its tests would let a
broken selector or asset reach every harness, so this gate runs them all.

Fails closed when a prerequisite is missing: two eval files lint CloudFormation
with ``cfn-lint`` (pinned in CI), and the Node eval needs ``node``. Install with
``python3 -m pip install cfn-lint==1.60.0``.
"""
from __future__ import annotations

import os
import shutil
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
SKILLS = [
    REPO / "skills/aws/aws-dynamodb-patterns",
    REPO / "skills/oci/oci-nosql-pattern",
    REPO / "skills/azure/azure-cosmosdb-pattern",
]
TIMEOUT = 300


def main() -> int:
    missing = [tool for tool in ("cfn-lint", "node", "bash") if shutil.which(tool) is None]
    if missing:
        print(f"FAIL: missing prerequisite(s): {', '.join(missing)}. "
              "Install cfn-lint with: python3 -m pip install cfn-lint==1.60.0")
        return 1
    commands = []
    for skill in SKILLS:
        found = [(skill, [sys.executable, str(path)]) for path in sorted((skill / "evals").glob("*.py"))]
        found += [(skill, ["node", "--test", str(path)]) for path in sorted((skill / "evals").glob("*.mjs"))]
        if not found:
            print(f"FAIL: no evals found under {skill / 'evals'}")
            return 1
        commands += found
    failures = 0
    for skill, command in commands:
        name = f"{skill.name}/{Path(command[-1]).name}"
        try:
            result = subprocess.run(command, cwd=skill, capture_output=True, text=True, timeout=TIMEOUT,
                                    env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1"})
        except subprocess.TimeoutExpired:
            print(f"FAIL {name}: timed out after {TIMEOUT}s")
            failures += 1
            continue
        if result.returncode == 0:
            print(f"OK   {name}")
        else:
            failures += 1
            print(f"FAIL {name} (exit {result.returncode})")
            print((result.stdout + result.stderr)[-4000:])
    print(f"{len(commands) - failures}/{len(commands)} NoSQL pattern eval files passed")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
