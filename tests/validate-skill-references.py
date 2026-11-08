#!/usr/bin/env python3
"""Gate: skill/agent reference links resolve, and evidence ledgers are traceable.

Two checks, both deterministic (no network, no wall clock):

1. Every relative Markdown link in a ``SKILL.md``, ``AGENT.md`` or skill
   ``references/*.md`` file resolves to a file on disk. Progressive disclosure
   depends on these links: a broken one silently drops guidance the entry point
   promised to load.

2. Any skill or agent directory that ships ``references/evidence-ledger.md``
   opts into claim-to-source traceability:

   * every ledger row has a unique evidence ID, an ``https`` URL on an
     allow-listed provider documentation host, a ``YYYY-MM-DD`` retrieval date
     and a status of ``VERIFIED``, ``UNVERIFIED`` or ``CONFLICT``;
   * every ``[EVIDENCE-ID]`` marker cited in that directory's Markdown exists in
     its ledger (no citation to nothing);
   * every ledger row is cited at least once outside the ledger (no orphan
     evidence that nothing depends on).

The allow-list is deliberately narrow: a ledger proves provider behaviour, so
its sources must be the provider's own documentation or SDK repositories.
Extend ``OFFICIAL_SOURCES`` only with a provider-owned host or org.

Usage: python3 tests/validate-skill-references.py
"""
from __future__ import annotations

import re
import sys
from pathlib import Path
from urllib.parse import urlparse

REPO = Path(__file__).resolve().parent.parent

# host -> allowed path prefixes ("" = any path on that host)
OFFICIAL_SOURCES: dict[str, tuple[str, ...]] = {
    "docs.oracle.com": ("",),
    "docs.aws.amazon.com": ("",),
    "learn.microsoft.com": ("",),
    "github.com": ("/oracle/", "/aws/", "/awslabs/", "/Azure/"),
}

LINK_RE = re.compile(r"\]\(([^)\s]+)\)")
EVIDENCE_ID = r"[A-Z][A-Z0-9]*(?:-[A-Z0-9]+)*-E\d{2,}"
MARKER_RE = re.compile(r"\[(" + EVIDENCE_ID + r")\]")
ROW_RE = re.compile(r"^\|\s*(" + EVIDENCE_ID + r")\s*\|(.*)\|\s*$")
DATE_RE = re.compile(r"^\d{4}-\d{2}-\d{2}$")
STATUSES = {"VERIFIED", "UNVERIFIED", "CONFLICT"}
LEDGER_COLUMNS = 5  # claim | source URL | section | retrieved | status


def markdown_files() -> list[Path]:
    files: list[Path] = []
    for root in ("skills", "agents"):
        base = REPO / root
        files += base.rglob("SKILL.md")
        files += base.rglob("AGENT.md")
        files += (p for p in base.rglob("references/*.md"))
    return sorted(set(files))


def strip_code(text: str) -> str:
    """Drop fenced and inline code so example syntax is never read as a link."""
    text = re.sub(r"```.*?```", "", text, flags=re.S)
    return re.sub(r"`[^`\n]*`", "", text)


def check_links(errors: list[str]) -> int:
    checked = 0
    for path in markdown_files():
        for match in LINK_RE.finditer(strip_code(path.read_text(encoding="utf-8"))):
            target = match.group(1).split("#", 1)[0]
            if not target or re.match(r"^[a-z][a-z0-9+.-]*:", target):
                continue  # pure anchor, or an absolute URL / mailto
            checked += 1
            if not (path.parent / target).exists():
                errors.append(f"{path.relative_to(REPO)}: broken relative link '{target}'")
    return checked


def official(url: str) -> bool:
    parsed = urlparse(url)
    if parsed.scheme != "https" or parsed.hostname not in OFFICIAL_SOURCES:
        return False
    return any(parsed.path.startswith(prefix) for prefix in OFFICIAL_SOURCES[parsed.hostname])


def check_ledger(ledger: Path, errors: list[str]) -> int:
    rel = ledger.relative_to(REPO)
    ids: dict[str, int] = {}
    for lineno, line in enumerate(ledger.read_text(encoding="utf-8").splitlines(), 1):
        row = ROW_RE.match(line.strip())
        if not row:
            continue
        evidence_id = row.group(1)
        cells = [c.strip() for c in row.group(2).split("|")]
        where = f"{rel}:{lineno} {evidence_id}"
        if evidence_id in ids:
            errors.append(f"{where}: duplicate evidence ID (first at line {ids[evidence_id]})")
        ids[evidence_id] = lineno
        if len(cells) != LEDGER_COLUMNS:
            errors.append(f"{where}: expected {LEDGER_COLUMNS} columns after the ID, found {len(cells)}")
            continue
        claim, source, _section, retrieved, status = cells
        url = re.sub(r"^<|>$", "", source)
        if len(claim) < 10:
            errors.append(f"{where}: claim text is missing or too short")
        if not official(url):
            errors.append(f"{where}: source '{url}' is not an https URL on an allow-listed provider host")
        if not DATE_RE.match(retrieved):
            errors.append(f"{where}: retrieved date '{retrieved}' is not YYYY-MM-DD")
        if status not in STATUSES:
            errors.append(f"{where}: status '{status}' not in {sorted(STATUSES)}")
    if not ids:
        errors.append(f"{rel}: ledger has no evidence rows")
        return 0

    cited: set[str] = set()
    for md in ledger.parent.parent.rglob("*.md"):
        if md == ledger:
            continue
        for marker in MARKER_RE.findall(md.read_text(encoding="utf-8")):
            cited.add(marker)
            if marker not in ids:
                errors.append(f"{md.relative_to(REPO)}: cites [{marker}] which is not in {rel}")
    for orphan in sorted(set(ids) - cited):
        errors.append(f"{rel}: {orphan} is never cited outside the ledger")
    return len(ids)


def main() -> int:
    errors: list[str] = []
    links = check_links(errors)
    ledgers = sorted((REPO / "skills").rglob("references/evidence-ledger.md"))
    ledgers += sorted((REPO / "agents").rglob("references/evidence-ledger.md"))
    rows = sum(check_ledger(ledger, errors) for ledger in ledgers)
    if errors:
        print("validate-skill-references: FAIL", file=sys.stderr)
        for error in errors:
            print(f"  - {error}", file=sys.stderr)
        return 1
    print(
        f"validate-skill-references: OK ({links} relative links resolved; "
        f"{len(ledgers)} evidence ledgers, {rows} rows traced)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
