#!/usr/bin/env python3
"""Validate SKILL.md frontmatter against schemas/skill.frontmatter.schema.json.

Uses jsonschema for validation (consistent with existing jsonschema usage
in validate-catalog.py). Falls back to a minimal hand-rolled validator if
jsonschema is unavailable.

Validation covers:
  - Required fields: name, description, allowed-tools, metadata
  - metadata required sub-fields: author, version
  - name: pattern ^[a-z0-9][a-z0-9-]*$
  - description: minLength 50, maxLength 1500
  - allowed-tools: non-empty string or non-empty list of strings
  - metadata.version: semver pattern ^\\d+\\.\\d+\\.\\d+(-[\\w.-]+)?$
  - metadata.author: minLength 1
  - disable-model-invocation (optional): boolean
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SKILLS_DIR = ROOT / "skills"
SCHEMA_PATH = ROOT / "schemas" / "skill.frontmatter.schema.json"

# ──────────────────────────────────────────────────────────────────────────────
# YAML frontmatter parser (stdlib only, mirrors validate-skill-allowed-tools.py)
# ──────────────────────────────────────────────────────────────────────────────

def parse_frontmatter_raw(text: str) -> dict | None:
    """Return a dict of the raw YAML frontmatter block, or None on failure.

    Supports:
      - scalar values  (key: value)
      - quoted scalars (key: "value")
      - nested mappings (metadata:\\n  sub: val)
      - block sequences (key:\\n  - item)
      - inline sequences (key: [a, b])

    Uses the hand-rolled parser (not PyYAML) to avoid failures on unquoted
    colons in description values — a known pattern in this corpus.
    """
    if not text.startswith("---\n"):
        return None
    end = text.find("\n---", 4)
    if end == -1:
        return None
    block = text[4:end]

    # Use hand-rolled parser to handle unquoted colons in scalar values.
    return _minimal_yaml_parse(block)


def _minimal_yaml_parse(block: str) -> dict:
    """Parse a restricted YAML subset into a Python dict."""
    result: dict = {}
    lines = block.splitlines()
    i = 0

    def parse_scalar(s: str):
        s = s.strip()
        if (s.startswith('"') and s.endswith('"')) or \
           (s.startswith("'") and s.endswith("'")):
            # YAML quoting: "..." uses backslash escapes, '...' doubles ''.
            if s.startswith('"'):
                try:
                    return json.loads(s)
                except ValueError:  # YAML-only escape; keep the legacy strip
                    return s[1:-1]
            return s[1:-1].replace("''", "'")
        if s.lower() == "true":
            return True
        if s.lower() == "false":
            return False
        return s

    while i < len(lines):
        line = lines[i]
        # top-level key
        if line and not line.startswith(" ") and ":" in line:
            key, _, rest = line.partition(":")
            key = key.strip()
            rest = rest.strip()

            if rest == "":
                # look ahead for block sequence or nested mapping
                children_list: list = []
                children_dict: dict = {}
                i += 1
                while i < len(lines) and (lines[i].startswith("  ") or lines[i] == ""):
                    child = lines[i]
                    if child.strip() == "":
                        i += 1
                        continue
                    if child.startswith("  - "):
                        children_list.append(child[4:].strip())
                    elif child.startswith("  ") and ":" in child:
                        ckey, _, crest = child.strip().partition(":")
                        children_dict[ckey.strip()] = parse_scalar(crest)
                    i += 1
                if children_list:
                    result[key] = children_list
                elif children_dict:
                    result[key] = children_dict
                else:
                    result[key] = None
                continue
            elif rest.startswith("[") and rest.endswith("]"):
                inner = rest[1:-1]
                result[key] = [t.strip().strip("'\"") for t in inner.split(",") if t.strip()]
            else:
                result[key] = parse_scalar(rest)
        i += 1

    return result


# ──────────────────────────────────────────────────────────────────────────────
# Schema-based validation
# ──────────────────────────────────────────────────────────────────────────────

NAME_RE = re.compile(r"^[a-z0-9][a-z0-9-]*$")
VERSION_RE = re.compile(r"^\d+\.\d+\.\d+(-[\w.-]+)?$")


def _try_jsonschema_validate(instance: dict, schema: dict) -> list[str]:
    """Validate using jsonschema if available; return list of error messages."""
    try:
        import jsonschema  # type: ignore
        errors = []
        validator_cls = jsonschema.validators.validator_for(schema)
        validator = validator_cls(schema)
        for err in sorted(validator.iter_errors(instance), key=lambda e: list(e.path)):
            path = ".".join(str(p) for p in err.absolute_path) or "<root>"
            errors.append(f"  field '{path}': {err.message}")
        return errors
    except ImportError:
        return []  # fall through to hand-rolled validator


def _hand_rolled_validate(fm: dict) -> list[str]:
    """Minimal validator that mirrors the schema constraints."""
    errors: list[str] = []

    # Required top-level fields
    for req in ("name", "description", "allowed-tools", "metadata"):
        if req not in fm or fm[req] is None:
            errors.append(f"  field '{req}': required field is missing")

    # name pattern
    name = fm.get("name")
    if isinstance(name, str):
        if not NAME_RE.match(name):
            errors.append(
                f"  field 'name': '{name}' does not match pattern ^[a-z0-9][a-z0-9-]*$"
            )
    elif name is not None:
        errors.append(f"  field 'name': must be a string, got {type(name).__name__}")

    # description length
    desc = fm.get("description")
    if isinstance(desc, str):
        if len(desc) < 50:
            errors.append(
                f"  field 'description': length {len(desc)} is below minimum 50"
            )
        if len(desc) > 1500:
            errors.append(
                f"  field 'description': length {len(desc)} exceeds maximum 1500"
            )
    elif desc is not None:
        errors.append(f"  field 'description': must be a string, got {type(desc).__name__}")

    # allowed-tools: string or list of strings, non-empty
    at = fm.get("allowed-tools")
    if at is not None:
        if isinstance(at, str):
            if len(at.strip()) == 0:
                errors.append("  field 'allowed-tools': string value must not be empty")
        elif isinstance(at, list):
            if len(at) == 0:
                errors.append("  field 'allowed-tools': list must contain at least one item")
            for idx, item in enumerate(at):
                if not isinstance(item, str) or len(item.strip()) == 0:
                    errors.append(
                        f"  field 'allowed-tools[{idx}]': each item must be a non-empty string"
                    )
        else:
            errors.append(
                f"  field 'allowed-tools': must be a string or array, got {type(at).__name__}"
            )

    # metadata sub-fields
    meta = fm.get("metadata")
    if isinstance(meta, dict):
        author = meta.get("author")
        if author is None:
            errors.append("  field 'metadata.author': required field is missing")
        elif not isinstance(author, str) or len(author) < 1:
            errors.append("  field 'metadata.author': must be a non-empty string")

        version = meta.get("version")
        if version is None:
            errors.append("  field 'metadata.version': required field is missing")
        elif not isinstance(version, str):
            errors.append(
                f"  field 'metadata.version': must be a string, got {type(version).__name__}"
            )
        elif not VERSION_RE.match(version):
            errors.append(
                f"  field 'metadata.version': '{version}' does not match semver pattern"
            )
    elif meta is not None:
        errors.append(
            f"  field 'metadata': must be a mapping/object, got {type(meta).__name__}"
        )

    # disable-model-invocation (optional): must be boolean if present
    dmi = fm.get("disable-model-invocation")
    if dmi is not None and not isinstance(dmi, bool):
        errors.append(
            f"  field 'disable-model-invocation': must be a boolean, got {type(dmi).__name__}"
        )

    return errors


def validate_skill(skill_md: Path, schema: dict) -> list[str]:
    text = skill_md.read_text(encoding="utf-8")
    fm = parse_frontmatter_raw(text)
    if fm is None or not isinstance(fm, dict):
        return [f"  no valid YAML frontmatter block found"]

    # Prefer jsonschema; fall back to hand-rolled
    errors = _try_jsonschema_validate(fm, schema)
    if not errors and _hand_rolled_validate(fm):
        # jsonschema not available or returned nothing; use hand-rolled
        errors = _hand_rolled_validate(fm)
    elif not errors:
        # double-check with hand-rolled to catch anything jsonschema might miss
        errors = _hand_rolled_validate(fm)

    return errors


# ──────────────────────────────────────────────────────────────────────────────
# Entry point
# ──────────────────────────────────────────────────────────────────────────────

def main() -> int:
    if not SCHEMA_PATH.exists():
        print(f"ERROR: schema not found at {SCHEMA_PATH}", file=sys.stderr)
        return 2

    schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))

    skill_files = sorted(SKILLS_DIR.glob("*/*/SKILL.md"))
    if not skill_files:
        print("ERROR: no SKILL.md files found", file=sys.stderr)
        return 2

    failed: list[tuple[Path, list[str]]] = []

    for skill_md in skill_files:
        errors = validate_skill(skill_md, schema)
        if errors:
            failed.append((skill_md, errors))

    if failed:
        print(
            f"FAIL: {len(failed)} skill(s) failed frontmatter schema validation "
            f"(out of {len(skill_files)} checked):",
            file=sys.stderr,
        )
        for path, errs in failed:
            print(f"\n  {path}", file=sys.stderr)
            for e in errs:
                print(f"    {e}", file=sys.stderr)
        return 1

    print(
        f"OK: all {len(skill_files)} skills passed SKILL.md frontmatter schema validation"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
