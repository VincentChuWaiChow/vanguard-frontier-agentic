#!/usr/bin/env python3
"""Strict-YAML gate for every SKILL.md frontmatter block.

The schema gate (validate-skill-frontmatter-schema.py) uses a deliberately
lenient hand-rolled parser that accepts unquoted ``: `` inside scalar values.
Real YAML parsers do not: the `skills` CLI (`npx skills add`, which feeds the
skills.sh directory) silently skipped 131 skills for exactly that reason. This
gate parses each frontmatter block with PyYAML's safe loader so a skill that
external tooling cannot read fails here instead of disappearing downstream.

Scope: skills/**/SKILL.md and .claude/skills/**/SKILL.md (the paths the
skills CLI discovers in this repo). Requires PyYAML; fails closed without it.
"""
from __future__ import annotations

import sys
from pathlib import Path

try:
    import yaml  # type: ignore
except ImportError:
    print("FAIL: PyYAML is required (python3 -m pip install pyyaml)", file=sys.stderr)
    sys.exit(1)

ROOT = Path(__file__).resolve().parent.parent
ROOTS = [ROOT / "skills", ROOT / ".claude" / "skills"]


def check(path: Path) -> str | None:
    text = path.read_text(encoding="utf-8")
    if not text.startswith("---\n"):
        return "missing opening '---' frontmatter delimiter"
    end = text.find("\n---", 4)
    if end == -1:
        return "missing closing '---' frontmatter delimiter"
    try:
        data = yaml.safe_load(text[4:end])
    except yaml.YAMLError as exc:
        first = str(exc).splitlines()[0]
        return f"invalid YAML ({first}); quote values that contain ': '"
    if not isinstance(data, dict):
        return "frontmatter is not a mapping"
    for key in ("name", "description"):
        if not isinstance(data.get(key), str) or not data[key].strip():
            return f"'{key}' must be a non-empty string"
    return None


def main() -> int:
    files = sorted(p for r in ROOTS if r.is_dir() for p in r.rglob("SKILL.md"))
    failures = [(p, err) for p in files if (err := check(p))]
    for p, err in failures:
        print(f"FAIL {p.relative_to(ROOT)}: {err}")
    if failures:
        print(f"\n{len(failures)} of {len(files)} SKILL.md files fail strict YAML parsing.")
        return 1
    print(f"OK: {len(files)} SKILL.md frontmatter blocks parse as strict YAML.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
