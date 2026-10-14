#!/usr/bin/env python3
"""Validate the trust_matrix block on every MCP reference, source and catalog.

The trust_matrix field is optional in the schema today (graceful rollout),
but this validator enforces it on every file currently committed under
mcp/. New entries that lack trust_matrix fail this check, which prevents
regressions and steers contributors to declare the security-relevant
posture explicitly.

Promotes to a required field in the schema once the corpus and
contribution guide have been updated. Until then, this validator is the
de-facto contract.

It also enforces that catalog/mcp-references.json carries the SAME
trust_matrix as the mcp/ source metadata. That index is hand-maintained and
had lost the field entirely: the Rust console deserialises it with
`Option<TrustMatrix>` and treats `None` as "no boundary to enforce", so a
missing trust_matrix in the catalog silently made every MCP pass every trust
check no matter what its source metadata declared. Source and index must
agree, or the enforcement path is decorative.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

REQUIRED_TM_FIELDS = {
    "mutation_capable",
    "requires_egress",
    "requires_credentials",
    "signed_release",
    "pin_strategy",
}
ALLOWED_SIGNED = {"cosign", "gh-attestation", "unsigned", "unknown"}
ALLOWED_PIN = {"digest", "tag", "version", "none"}


def metadata_files() -> list[Path]:
    paths: list[Path] = []
    for path in (ROOT / "mcp").rglob("*.metadata.json"):
        paths.append(path)
    for path in (ROOT / "mcp").rglob("metadata.json"):
        paths.append(path)
    return sorted(set(paths))


CATALOG_INDEX = ROOT / "catalog" / "mcp-references.json"


def catalog_entries() -> dict[str, dict]:
    """Map id -> entry from the hand-maintained catalog index."""
    if not CATALOG_INDEX.exists():
        return {}
    data = json.loads(CATALOG_INDEX.read_text(encoding="utf-8"))
    items = data if isinstance(data, list) else data.get("mcp_references", [])
    return {e["id"]: e for e in items if isinstance(e, dict) and e.get("id")}


def main() -> int:
    files = metadata_files()
    if not files:
        print("OK: no MCP metadata files present")
        return 0

    errors: list[str] = []
    for path in files:
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except Exception as exc:  # noqa: BLE001
            errors.append(f"{path.relative_to(ROOT)}: invalid JSON: {exc}")
            continue

        tm = data.get("trust_matrix")
        rel = path.relative_to(ROOT)
        if not isinstance(tm, dict):
            errors.append(f"{rel}: missing trust_matrix block")
            continue

        missing = REQUIRED_TM_FIELDS - tm.keys()
        if missing:
            errors.append(f"{rel}: trust_matrix missing fields {sorted(missing)}")

        for boolean_field in ("mutation_capable", "requires_egress", "requires_credentials"):
            if boolean_field in tm and not isinstance(tm[boolean_field], bool):
                errors.append(f"{rel}: trust_matrix.{boolean_field} must be boolean")

        if tm.get("signed_release") not in ALLOWED_SIGNED:
            errors.append(
                f"{rel}: trust_matrix.signed_release must be one of {sorted(ALLOWED_SIGNED)}"
            )
        if tm.get("pin_strategy") not in ALLOWED_PIN:
            errors.append(
                f"{rel}: trust_matrix.pin_strategy must be one of {sorted(ALLOWED_PIN)}"
            )

    # Source <-> catalog parity. The console reads only the catalog index, so a
    # trust_matrix that exists in mcp/ but not here is not enforced anywhere.
    catalog = catalog_entries()
    checked_parity = 0
    for path in files:
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except Exception:  # noqa: BLE001
            continue  # already reported above
        mcp_id = data.get("id")
        source_tm = data.get("trust_matrix")
        if not mcp_id or not isinstance(source_tm, dict):
            continue
        entry = catalog.get(mcp_id)
        if entry is None:
            errors.append(
                f"catalog/mcp-references.json: no entry for MCP id {mcp_id!r} "
                f"declared in {path.relative_to(ROOT)}"
            )
            continue
        catalog_tm = entry.get("trust_matrix")
        if not isinstance(catalog_tm, dict):
            errors.append(
                f"catalog/mcp-references.json: entry {mcp_id!r} is missing trust_matrix. "
                f"The console treats a missing matrix as 'nothing to enforce', so this "
                f"MCP would pass every trust-boundary check. Copy it from "
                f"{path.relative_to(ROOT)}."
            )
            continue
        if catalog_tm != source_tm:
            errors.append(
                f"catalog/mcp-references.json: trust_matrix for {mcp_id!r} does not match "
                f"{path.relative_to(ROOT)} (source={source_tm}, catalog={catalog_tm})"
            )
            continue
        checked_parity += 1

    if errors:
        print("ERROR: MCP trust_matrix validation failed", file=sys.stderr)
        for err in errors:
            print(f"  - {err}", file=sys.stderr)
        return 1

    print(
        f"OK: validated trust_matrix on {len(files)} MCP reference files "
        f"({checked_parity} in source/catalog parity)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
