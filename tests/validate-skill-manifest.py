#!/usr/bin/env python3
"""Generate or validate the skill integrity manifest.

The manifest is deterministic: it records every file under each cataloged skill
directory, plus an aggregate SHA-256 per skill. Use --write when skill content
changes intentionally. Use default check mode in CI.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT / "catalog" / "skills.json"
MANIFEST = ROOT / "catalog" / "skill-manifest.json"


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def skill_files(skill_path: Path) -> list[dict]:
    files: list[dict] = []
    # Skip interpreter bytecode caches: they are git-ignored, so hashing them makes the
    # manifest depend on whether a skill's scripts happened to run locally (CI never has them).
    for path in sorted(
        p for p in skill_path.rglob("*")
        if p.is_file() and "__pycache__" not in p.parts and p.suffix != ".pyc"
    ):
        rel = path.relative_to(ROOT).as_posix()
        data = path.read_bytes()
        files.append({
            "path": rel,
            "sha256": sha256_bytes(data),
            "bytes": len(data),
        })
    return files


def aggregate_hash(files: list[dict]) -> str:
    h = hashlib.sha256()
    for item in files:
        h.update(item["path"].encode("utf-8"))
        h.update(b"\0")
        h.update(item["sha256"].encode("ascii"))
        h.update(b"\0")
        h.update(str(item["bytes"]).encode("ascii"))
        h.update(b"\n")
    return h.hexdigest()


def build_manifest() -> dict:
    catalog = load_json(CATALOG)
    entries: list[dict] = []
    for item in sorted(catalog, key=lambda entry: entry["id"]):
        if item["type"] != "skill":
            continue
        skill_path = ROOT / item["path"]
        if not skill_path.is_dir():
            raise AssertionError(f"{item['id']}: skill path is not a directory: {item['path']}")
        files = skill_files(skill_path)
        if not files:
            raise AssertionError(f"{item['id']}: skill path has no files: {item['path']}")
        entries.append({
            "id": item["id"],
            "path": item["path"],
            "aggregate_sha256": aggregate_hash(files),
            "files": files,
        })
    return {
        "manifest_version": 1,
        "algorithm": "sha256",
        "root": "skills",
        "entries": entries,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true", help="Rewrite catalog/skill-manifest.json")
    args = parser.parse_args()

    try:
        current = build_manifest()
    except AssertionError as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 1

    rendered = json.dumps(current, indent=2, sort_keys=False) + "\n"
    if args.write:
        MANIFEST.write_text(rendered, encoding="utf-8")
        print(f"OK: wrote {MANIFEST.relative_to(ROOT)} with {len(current['entries'])} skill entries")
        return 0

    if not MANIFEST.exists():
        print(f"ERROR: missing {MANIFEST.relative_to(ROOT)}; run tests/validate-skill-manifest.py --write", file=sys.stderr)
        return 1

    expected = MANIFEST.read_text(encoding="utf-8")
    if expected != rendered:
        print(
            f"ERROR: {MANIFEST.relative_to(ROOT)} is stale or skill contents changed; "
            "run tests/validate-skill-manifest.py --write if intentional",
            file=sys.stderr,
        )
        return 1

    print(f"OK: skill manifest matches {len(current['entries'])} skill entries")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
