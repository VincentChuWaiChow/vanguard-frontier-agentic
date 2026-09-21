#!/usr/bin/env python3
"""Block credential-shaped strings anywhere in tracked content.

Findings SEC1 and SEC2. The HOL plugin scanner runs twice: a blocking listing
gate scoped to `plugins/vanguard-frontier-agentic` (three files), and a
repository-root scan that is explicitly advisory (`min_score: 0`,
`fail_on_severity: none`) and additionally suppresses roughly 6,500 tracked
files through `.plugin-scanner.toml`. There is no gitleaks, trufflehog,
detect-secrets, or pre-commit hook in the repository. The practical effect is
that a credential committed almost anywhere would not fail a build.

This gate closes that hole from the other direction. Rather than widening the
external scanner's exclusions -- which were each added against a confirmed
false positive, and which cover fixtures that deliberately contain
secret-shaped bait -- it scans every tracked file with a small set of
high-precision, vendor-prefixed patterns that do not fire on prose.

Two escape hatches, both narrow and both deliberate:

  * A line carrying the `<FAKE>` marker is ignored. The repository already uses
    this convention for credential bait in routing fixtures, and
    `validate-maestro-routing.py` enforces that secrets-bait tasks carry it.
  * `ALLOWLIST` names individual paths, each with a reason. Entries are paths,
    never directories, so adding a whole content class is not possible by
    accident.

Deterministic by construction: it reads committed files only and never
consults the clock, the network, or the environment.
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# Vendor-prefixed, fixed-shape credentials. Every pattern here identifies its
# issuer, so a match is a credential rather than a word that looks like one.
PATTERNS: list[tuple[str, re.Pattern[str]]] = [
    ("AWS access key id", re.compile(r"\bAKIA[0-9A-Z]{16}\b")),
    ("GitHub token", re.compile(r"\bgh[pousr]_[A-Za-z0-9]{36,}\b")),
    ("Google API key", re.compile(r"\bAIza[0-9A-Za-z_\-]{35}\b")),
    ("Slack token", re.compile(r"\bxox[abprs]-[A-Za-z0-9-]{10,}")),
    ("Stripe live secret key", re.compile(r"\bsk_live_[0-9a-zA-Z]{20,}\b")),
    ("OpenAI-style secret key", re.compile(r"\bsk-[A-Za-z0-9]{32,}\b")),
    ("Private key block", re.compile(r"-----BEGIN (?:[A-Z ]+ )?PRIVATE KEY-----")),
    ("Azure storage account key", re.compile(r"\bAccountKey=[A-Za-z0-9+/]{60,}={0,2}")),
    ("Generic bearer JWT", re.compile(r"\beyJ[A-Za-z0-9_\-]{10,}\.eyJ[A-Za-z0-9_\-]{10,}\.")),
]

# Line-level opt-out already used by routing fixtures for credential bait.
FAKE_MARKER = "<FAKE>"

# Per-path exemptions. Paths only -- never directories -- each with a reason.
ALLOWLIST: dict[str, str] = {
    # This file carries the patterns themselves.
    "tests/validate-tracked-secrets.py": "defines the detection patterns",
}

# Binary and generated blobs that are not meaningfully greppable.
SKIP_SUFFIXES = {".png", ".jpg", ".jpeg", ".gif", ".ico", ".webp", ".pdf", ".zip", ".gz", ".woff", ".woff2"}


def tracked_files() -> list[str]:
    out = subprocess.run(
        ["git", "ls-files", "-z"], cwd=ROOT, capture_output=True, text=True, check=True
    ).stdout
    return [p for p in out.split("\0") if p]


def main() -> int:
    findings: list[str] = []
    scanned = 0
    skipped_fake = 0

    for rel in tracked_files():
        if rel in ALLOWLIST:
            continue
        if Path(rel).suffix.lower() in SKIP_SUFFIXES:
            continue
        path = ROOT / rel
        try:
            text = path.read_text(encoding="utf-8")
        except (UnicodeDecodeError, FileNotFoundError, IsADirectoryError):
            continue
        scanned += 1

        for lineno, line in enumerate(text.splitlines(), start=1):
            for label, pattern in PATTERNS:
                if not pattern.search(line):
                    continue
                if FAKE_MARKER in line:
                    skipped_fake += 1
                    continue
                # Report the location and the kind only. The value itself is
                # never echoed, so a real leak is not copied into CI logs.
                findings.append(f"{rel}:{lineno}: {label}")

    if findings:
        print("ERROR: credential-shaped strings found in tracked content", file=sys.stderr)
        for f in findings:
            print(f"  - {f}", file=sys.stderr)
        print(
            f"\n{len(findings)} finding(s). If a match is deliberate bait, mark the line "
            f"with {FAKE_MARKER}; if a file is a genuine exception, add it to ALLOWLIST "
            f"with a reason. Never silence a real credential -- rotate it.",
            file=sys.stderr,
        )
        return 1

    print(
        f"OK: no credential-shaped strings in {scanned} tracked text files "
        f"({len(PATTERNS)} patterns, {skipped_fake} {FAKE_MARKER}-marked line(s) ignored)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
