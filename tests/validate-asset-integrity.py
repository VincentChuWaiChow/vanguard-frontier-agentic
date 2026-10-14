#!/usr/bin/env python3
"""Generate or validate the cross-asset integrity manifest.

The skill-manifest covers SKILL.md trees only. This manifest covers every
asset surface a downstream consumer trusts at runtime: agents/, rules/,
mcp/, catalog/, schemas/, plus root-level governance files
(README, SECURITY, LICENSE, CONTRIBUTING, CODE_OF_CONDUCT, CLAUDE,
AGENTS, GEMINI). Each file is hashed individually (sha256) and rolled
into a per-tree aggregate hash, plus a single top-level aggregate.

The manifest itself is the artifact attested at release time
(see .github/workflows/release.yml). A consumer who trusts the
attestation can then verify any asset by recomputing its sha256 and
comparing against the manifest entry, without having to trust the
git history or the npm tarball file list separately.

Use --write when asset content changes intentionally. Default mode is
verification, suitable for CI gates.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "catalog" / "asset-integrity.json"

# Trees walked recursively. Order is stable (preserved in manifest).
TREES = [
    "agents",
    "rules",
    "mcp",
    "mods",
    "schemas",
    "catalog",
    # The shipped CLI and generator scripts (npm bin + manifest generators)
    # are runtime-critical: a malicious change here would not change any
    # other tracked path but would compromise downstream installs.
    "scripts",
    # Marketplace surface — each plugin manifest is consumed verbatim by
    # the corresponding harness at install time. Cover them all.
    "powers",
    "plugins",
    ".claude-plugin",
    ".cursor-plugin",
    ".github/plugin",
    ".agents/plugins",
    # Validator scripts execute with full CI credentials during release
    # (scripts/release-prepare.mjs calls validate-asset-integrity.py --write).
    # A backdoor in tests/ would not trigger manifest drift unless tests/
    # is also covered, giving an attacker a blind spot for supply-chain
    # compromise. Include tests/ so any change here shows up in CI.
    "tests",
]

# Top-level governance files. These are part of the trust surface for
# any downstream consumer evaluating the project (procurement reviews,
# compliance attestation, supply-chain due diligence).
ROOT_FILES = [
    "README.md",
    "SECURITY.md",
    "LICENSE",
    "CONTRIBUTING.md",
    "CODE_OF_CONDUCT.md",
    "CLAUDE.md",
    "AGENTS.md",
    "GEMINI.md",
    "package.json",
    ".releaserc.js",
]

# Files inside the trees that are not part of the trust surface and
# would create false drift on every CI run. Kept narrow on purpose.
EXCLUDED_NAMES = {".DS_Store"}

# Directories whose contents are generated build artifacts, never trust
# surface. Pruned during the walk to keep the manifest stable.
EXCLUDED_DIR_NAMES = {"__pycache__", ".pytest_cache", "node_modules"}

# Bootstrap exception: the manifest cannot hash itself.
EXCLUDED_RELATIVE_PATHS = {"catalog/asset-integrity.json"}


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def file_record(path: Path) -> dict:
    data = path.read_bytes()
    return {
        "path": path.relative_to(ROOT).as_posix(),
        "sha256": sha256_bytes(data),
        "bytes": len(data),
    }


def aggregate_hash(records: list[dict]) -> str:
    h = hashlib.sha256()
    for item in records:
        h.update(item["path"].encode("utf-8"))
        h.update(b"\0")
        h.update(item["sha256"].encode("ascii"))
        h.update(b"\0")
        h.update(str(item["bytes"]).encode("ascii"))
        h.update(b"\n")
    return h.hexdigest()


def walk_tree(tree: str) -> list[dict]:
    base = ROOT / tree
    if not base.is_dir():
        raise AssertionError(f"missing tree: {tree}")
    files: list[dict] = []
    for path in sorted(p for p in base.rglob("*") if p.is_file()):
        if path.is_symlink():
            # Symlinks are followed by is_file() but their target may lie
            # outside the repo, producing a misleading integrity guarantee.
            # Reject any symlink in the trust surface.
            raise AssertionError(f"symlink in trust surface: {path.relative_to(ROOT)}")
        if path.name in EXCLUDED_NAMES:
            continue
        if any(part in EXCLUDED_DIR_NAMES for part in path.parts):
            continue
        if path.relative_to(ROOT).as_posix() in EXCLUDED_RELATIVE_PATHS:
            continue
        files.append(file_record(path))
    return files


def build_manifest() -> dict:
    trees: list[dict] = []
    for tree in TREES:
        records = walk_tree(tree)
        trees.append({
            "tree": tree,
            "aggregate_sha256": aggregate_hash(records),
            "files": records,
        })

    root_records: list[dict] = []
    for name in ROOT_FILES:
        path = ROOT / name
        if not path.is_file():
            raise AssertionError(f"missing root file: {name}")
        root_records.append(file_record(path))

    manifest = {
        "manifest_version": 1,
        "algorithm": "sha256",
        "scope": {
            "trees": TREES,
            "root_files": ROOT_FILES,
        },
        "trees": trees,
        "root_files": root_records,
    }

    # Top-level aggregate covers every per-tree aggregate plus every
    # root-file record. Tampering with any single byte changes this hash.
    h = hashlib.sha256()
    for tree_entry in trees:
        h.update(tree_entry["tree"].encode("utf-8"))
        h.update(b"\0")
        h.update(tree_entry["aggregate_sha256"].encode("ascii"))
        h.update(b"\n")
    for record in root_records:
        h.update(record["path"].encode("utf-8"))
        h.update(b"\0")
        h.update(record["sha256"].encode("ascii"))
        h.update(b"\n")
    manifest["aggregate_sha256"] = h.hexdigest()

    return manifest


def render(manifest: dict) -> str:
    return json.dumps(manifest, indent=2, sort_keys=False) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--write", action="store_true",
        help="Rewrite catalog/asset-integrity.json from disk state.",
    )
    args = parser.parse_args()

    try:
        current = build_manifest()
    except AssertionError as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 1

    rendered = render(current)

    if args.write:
        MANIFEST.write_text(rendered, encoding="utf-8")
        n_files = sum(len(t["files"]) for t in current["trees"]) + len(current["root_files"])
        print(
            f"OK: wrote {MANIFEST.relative_to(ROOT)} "
            f"({n_files} files, top-level sha256 {current['aggregate_sha256'][:12]}...)"
        )
        return 0

    if not MANIFEST.exists():
        print(
            f"ERROR: missing {MANIFEST.relative_to(ROOT)}; "
            "run tests/validate-asset-integrity.py --write",
            file=sys.stderr,
        )
        return 1

    expected = MANIFEST.read_text(encoding="utf-8")
    if expected != rendered:
        print(
            f"ERROR: {MANIFEST.relative_to(ROOT)} is stale; asset content has drifted. "
            "Run tests/validate-asset-integrity.py --write if the change is intentional, "
            "then re-commit.",
            file=sys.stderr,
        )
        return 1

    print(
        f"OK: asset integrity manifest matches "
        f"(top-level sha256 {current['aggregate_sha256'][:12]}...)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
