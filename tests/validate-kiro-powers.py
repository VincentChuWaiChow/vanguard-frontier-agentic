#!/usr/bin/env python3
"""Validate Kiro Powers under powers/ for spec conformance.

Kiro Powers (kirodotdev/powers) frontmatter is strictly limited to FIVE
fields: name, displayName, description, keywords, author. Any extra
field — version, repository, license, tags — is forbidden and will be
rejected by the Kiro Powers UI.

Gates:
  1. Every powers/<name>/ subdirectory has a POWER.md.
  2. POWER.md frontmatter contains exactly the five allowed fields.
  3. name is lowercase kebab-case.
  4. description has at most three sentences (Kiro constraint).
  5. keywords is a non-empty list of specific (non-broad) terms.
  6. Generator (scripts/generate-kiro-powers.mjs) is in sync.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
POWERS = REPO / "powers"
GENERATOR = REPO / "scripts" / "generate-kiro-powers.mjs"

ALLOWED_FIELDS = {"name", "displayName", "description", "keywords", "author"}
NAME_RE = re.compile(r"^[a-z][a-z0-9-]*$")
# Broad terms that Kiro docs warn against — these are too generic to scope
# activation reliably and produce false positives across unrelated tasks.
BROAD_KEYWORDS = {
    "cloud",
    "code",
    "devops",
    "infrastructure",
    "infra",
    "agent",
    "agents",
    "ai",
    "ml",
    "ops",
    "automation",
    "tool",
    "tools",
    "general",
}


def fail(msg: str) -> None:
    print(f"FAIL [kiro-powers] {msg}", file=sys.stderr)


def _unquote(value: str) -> str:
    value = value.strip()
    if len(value) >= 2 and value[0] == value[-1] and value[0] in ("'", '"'):
        # YAML quoting: "..." uses backslash escapes, '...' doubles ''.
        if value[0] == '"':
            try:
                return json.loads(value)
            except ValueError:  # YAML-only escape; keep the legacy strip
                return value[1:-1]
        return value[1:-1].replace("''", "'")
    return value


def _parse_flow_list(value: str) -> list[str] | None:
    value = value.strip()
    if not (value.startswith("[") and value.endswith("]")):
        return None
    inner = value[1:-1].strip()
    if not inner:
        return []
    return [_unquote(item.strip()) for item in inner.split(",") if item.strip()]


def parse_frontmatter(text: str, path: Path) -> dict | None:
    """Parse the strict-5 Kiro Powers frontmatter without a YAML library.

    Accepted shapes:
      key: value
      key: "quoted value"
      key: [item, item]   # flow list
      key:                # block list
        - item
        - item
    Anything outside this shape is rejected with a clear error.
    """
    if not text.startswith("---\n"):
        fail(f"{path.relative_to(REPO)}: POWER.md must start with YAML frontmatter delimiter")
        return None
    end = text.find("\n---\n", 4)
    if end == -1:
        fail(f"{path.relative_to(REPO)}: POWER.md frontmatter is not terminated")
        return None
    block = text[4:end]

    data: dict = {}
    lines = block.split("\n")
    i = 0
    while i < len(lines):
        line = lines[i]
        if not line.strip() or line.lstrip().startswith("#"):
            i += 1
            continue
        if ":" not in line:
            fail(f"{path.relative_to(REPO)}: malformed frontmatter line: {line!r}")
            return None
        key, _, rest = line.partition(":")
        key = key.strip()
        rest = rest.strip()
        if rest == "":
            items: list[str] = []
            i += 1
            while i < len(lines) and lines[i].lstrip().startswith("- "):
                items.append(_unquote(lines[i].lstrip()[2:].strip()))
                i += 1
            data[key] = items
            continue
        flow = _parse_flow_list(rest)
        if flow is not None:
            data[key] = flow
        else:
            data[key] = _unquote(rest)
        i += 1

    if not data:
        fail(f"{path.relative_to(REPO)}: frontmatter must be a mapping")
        return None
    return data


_DECIMAL_RE = re.compile(r"\d\.\d")


def count_sentences(text: str) -> int:
    # Count terminal punctuation only when followed by whitespace or EOL,
    # excluding periods inside decimal numbers (e.g. "MLPS 2.0", "v1.2.3")
    # and abbreviation dots (e.g. "i.e.", "e.g.") that are not at a word
    # boundary. Matches the same algorithm used in test-marketplace-validators.py
    # to keep test expectations and live validation in sync.
    text = text.strip()
    if not text:
        return 0
    masked = _DECIMAL_RE.sub(lambda m: m.group(0).replace(".", "_"), text)
    return len(re.findall(r"[.!?](?:\s|$)", masked))


def validate_power(power_dir: Path) -> list[str]:
    errs: list[str] = []
    md = power_dir / "POWER.md"
    if not md.is_file():
        return [f"{power_dir.relative_to(REPO)}: POWER.md is missing"]

    data = parse_frontmatter(md.read_text(), md)
    if data is None:
        return [f"{md.relative_to(REPO)}: frontmatter could not be parsed"]

    keys = set(data.keys())
    extra = keys - ALLOWED_FIELDS
    missing = ALLOWED_FIELDS - keys
    if extra:
        errs.append(
            f"{md.relative_to(REPO)}: forbidden frontmatter fields (Kiro strict-5 rule): {sorted(extra)}",
        )
    if missing:
        errs.append(
            f"{md.relative_to(REPO)}: missing required frontmatter fields: {sorted(missing)}",
        )
    if extra or missing:
        return errs

    name = data["name"]
    if not isinstance(name, str) or not NAME_RE.match(name):
        errs.append(f"{md.relative_to(REPO)}: name must be lowercase kebab-case ({name!r})")
    if name != power_dir.name:
        errs.append(
            f"{md.relative_to(REPO)}: name {name!r} must match directory name {power_dir.name!r}",
        )

    description = data["description"]
    if not isinstance(description, str) or not description.strip():
        errs.append(f"{md.relative_to(REPO)}: description must be a non-empty string")
    else:
        n = count_sentences(description)
        if n > 3:
            errs.append(
                f"{md.relative_to(REPO)}: description has {n} sentences; Kiro limit is 3",
            )

    keywords = data["keywords"]
    if not isinstance(keywords, list) or not any(
        isinstance(k, str) and k.strip() for k in keywords
    ):
        errs.append(f"{md.relative_to(REPO)}: keywords must be a non-empty list")
    else:
        for k in keywords:
            if not isinstance(k, str):
                errs.append(f"{md.relative_to(REPO)}: keyword {k!r} is not a string")
            elif k.lower() in BROAD_KEYWORDS:
                errs.append(
                    f"{md.relative_to(REPO)}: keyword {k!r} is too broad; "
                    "Kiro docs warn that broad keywords cause false activations",
                )

    if not isinstance(data.get("author"), str) or not data["author"].strip():
        errs.append(f"{md.relative_to(REPO)}: author must be a non-empty string")
    if not isinstance(data.get("displayName"), str) or not data["displayName"].strip():
        errs.append(f"{md.relative_to(REPO)}: displayName must be a non-empty string")

    return errs


def main() -> int:
    if not POWERS.is_dir():
        fail("powers/ directory is missing")
        return 1

    errors: list[str] = []
    power_dirs = sorted(p for p in POWERS.iterdir() if p.is_dir())
    if not power_dirs:
        fail("no powers found under powers/")
        return 1

    for d in power_dirs:
        errors.extend(validate_power(d))

    # Generator drift
    result = subprocess.run(
        ["node", str(GENERATOR), "--check"],
        capture_output=True,
        text=True,
        timeout=60,
    )
    if result.returncode != 0:
        errors.append((result.stderr or result.stdout).strip())

    if errors:
        for err in errors:
            fail(err)
        return 1

    print(f"OK: {len(power_dirs)} Kiro Powers valid (strict-5 frontmatter, kebab-case names, ≤3-sentence descriptions, generator in sync)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
