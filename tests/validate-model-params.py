#!/usr/bin/env python3
"""Gate: Cursor per-model parameter groups are accepted or rejected correctly.

`scripts/model-policy.mjs` accepts Cursor's documented per-model parameter
group appended to a model id in square brackets, e.g.

    claude-opus-5[effort=high,context=300k]

The registry namespace pattern only shapes that group (a nested-quantifier
regex is rejected by the registry's own ReDoS guard), so the structure — the
documented keys, the key=value shape, and no duplicates — is enforced
semantically inside the script. `validate:model-policy` only runs `check`
against the committed policy, which exercises the accepting path alone; this
gate pins the rejecting paths too, so the semantic validation cannot quietly
stop working.

Every case runs through `set --dry-run`, which validates the resulting policy
without writing anything.
"""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
ENGINE = REPO / "scripts" / "model-policy.mjs"

# (model value, should_be_accepted, why)
CASES: list[tuple[str, bool, str]] = [
    ("composer-2.5", True, "bare registered id, no parameter group"),
    ("claude-opus-5[effort=high]", True, "single documented key"),
    ("gpt-5.6-sol[effort=high,context=300k]", True, "two documented keys"),
    ("composer-2[fast=false]", True, "the fast key, boolean-ish value"),
    ("claude-opus-5[bogus=1]", False, "undocumented key"),
    ("claude-opus-5[effort=high,bogus=1]", False, "one good key, one undocumented"),
    ("claude-opus-5[effort]", False, "missing =value"),
    ("claude-opus-5[=high]", False, "missing key"),
    ("claude-opus-5[effort=]", False, "empty value"),
    ("claude-opus-5[]", False, "empty parameter group"),
    ("claude-opus-5[effort=high,effort=low]", False, "duplicate key"),
    ("not-a-registered-model[effort=high]", False, "base id absent from the allowlist"),
    ("claude-opus-5[a=1][b=2]", False, "two groups"),
    ("claude-opus-5[effort=high]trailing", False, "group not at the end"),
]


def pick_agent_id() -> str:
    """An agent id that exists in the catalog, so the scope resolves."""
    agents = json.loads((REPO / "catalog" / "agents.json").read_text(encoding="utf-8"))
    if not agents:
        raise SystemExit("ERROR: catalog/agents.json is empty; cannot scope the probe")
    return sorted(a["id"] for a in agents)[0]


def accepted(agent_id: str, model: str) -> tuple[bool, str]:
    proc = subprocess.run(
        [
            "node",
            str(ENGINE),
            "set",
            "--scope",
            f"agent={agent_id}",
            "--harness",
            "cursor",
            "--model",
            model,
            "--dry-run",
        ],
        cwd=REPO,
        capture_output=True,
        text=True,
    )
    return proc.returncode == 0, (proc.stdout + proc.stderr).strip()


def main() -> int:
    agent_id = pick_agent_id()
    failures: list[str] = []

    for model, want_ok, why in CASES:
        got_ok, output = accepted(agent_id, model)
        if got_ok == want_ok:
            verdict = "accepted" if got_ok else "rejected"
            print(f"OK   {verdict:8} {model}  ({why})")
            continue
        wanted = "accepted" if want_ok else "rejected"
        got = "accepted" if got_ok else "rejected"
        failures.append(
            f"{model!r} should have been {wanted} ({why}) but was {got}\n"
            f"      engine output: {output.splitlines()[-1] if output else '(none)'}"
        )

    # A rejection must say why, not just fail: an operator needs to know which
    # key was wrong.
    _, msg = accepted(agent_id, "claude-opus-5[bogus=1]")
    if "bogus" not in msg or "parameter" not in msg:
        failures.append(
            "the rejection message for an undocumented key must name the key and "
            f"mention the parameter group; got: {msg!r}"
        )

    if failures:
        print("\nFAIL: model parameter-group validation drifted:", file=sys.stderr)
        for f in failures:
            print(f"  - {f}", file=sys.stderr)
        return 1

    print(f"\nOK: {len(CASES)} model parameter-group cases behave as documented")
    return 0


if __name__ == "__main__":
    sys.exit(main())
