#!/usr/bin/env python3
"""Gate: run the offline evals shipped with skills/aws/aws-dynamodb-patterns.

The skill ships executable helpers (route selector, review reminder) and example
assets (shell, Python, Node). Its own evals check them with fake AWS binaries and
never contact AWS. Shipping code without running its tests would let a broken
selector or asset reach every harness, so this gate runs them all.

Fails closed when a prerequisite is missing: two eval files lint CloudFormation
with ``cfn-lint`` (pinned in CI), and the Node eval needs ``node``. Install with
``python3 -m pip install cfn-lint==1.60.0``.
"""
from __future__ import annotations

import shutil
import subprocess
import sys
from pathlib import Path

SKILL = Path(__file__).resolve().parents[1] / "skills/aws/aws-dynamodb-patterns"
TIMEOUT = 300


def main() -> int:
    missing = [tool for tool in ("cfn-lint", "node", "bash") if shutil.which(tool) is None]
    if missing:
        print(f"FAIL: missing prerequisite(s): {', '.join(missing)}. "
              "Install cfn-lint with: python3 -m pip install cfn-lint==1.60.0")
        return 1
    commands = [[sys.executable, str(path)] for path in sorted((SKILL / "evals").glob("*.py"))]
    commands += [["node", "--test", str(path)] for path in sorted((SKILL / "evals").glob("*.mjs"))]
    if not commands:
        print(f"FAIL: no evals found under {SKILL / 'evals'}")
        return 1
    failures = 0
    for command in commands:
        name = Path(command[-1]).name
        try:
            result = subprocess.run(command, cwd=SKILL, capture_output=True, text=True, timeout=TIMEOUT)
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
    print(f"{len(commands) - failures}/{len(commands)} aws-dynamodb-patterns eval files passed")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
